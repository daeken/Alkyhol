//! Physical GUI capability probe; this is an owned window, not DAO execution.
fn unsupported(message: &str) -> ! {
    eprintln!("Alkyhol native window probe unsupported: {message}");
    std::process::exit(78)
}
type Result<T> = std::result::Result<T, String>;
#[path = "src/window_host32.rs"]
mod host;
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventCreateMouseEvent(
        source: host::Id,
        kind: u32,
        point: host::Point,
        button: u32,
    ) -> host::Id;
    fn CGEventPost(tap: u32, event: host::Id);
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFRelease(value: host::Id);
}
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}
fn main() -> Result<()> {
    let app = host::App::new()?;
    let style = 15;
    let outer = app.frame_for_client(
        host::Rect {
            origin: host::Point { x: 0., y: 0. },
            size: host::Size {
                width: 640.,
                height: 480.,
            },
        },
        style,
    )?;
    let window = app.create(
        100,
        100,
        outer.size.width as i32,
        outer.size.height as i32,
        style,
        &"Alkyhol owned window backend test"
            .encode_utf16()
            .collect::<Vec<_>>(),
    )?;
    window.title(
        &"Alkyhol owned window backend test: resized"
            .encode_utf16()
            .collect::<Vec<_>>(),
    )?;
    window.position(110, 120, outer.size.width as i32, outer.size.height as i32)?;
    window.show(true);
    app.activate();
    for _ in 0..20 {
        app.pump()?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let frame = window.frame()?;
    let client = window.client();
    let onscreen = window.onscreen()?;
    println!(
        "visible={} WindowServer_onscreen={} native_window={} frame={frame:?} client={client:?}",
        window.visible(),
        onscreen,
        window.number()
    );
    assert!(window.visible() && onscreen && window.number() > 0);
    assert_eq!((frame.origin.x, frame.origin.y), (110., 120.));
    assert_eq!((client.size.width, client.size.height), (640., 480.));
    if std::env::args().any(|arg| matches!(arg.as_str(), "--mouse" | "--capture")) {
        if !unsafe { AXIsProcessTrusted() } {
            return Err("native mouse probe requires Accessibility access".into());
        }
        let content = window.client_frame()?;
        let point = host::Point {
            x: content.origin.x + 75.,
            y: content.origin.y + 90.,
        };
        app.take_mouse();
        // Post through WindowServer, then consume AppKit's real event queue.
        // No direct call to the conversion function or guest event injection.
        for kind in [5, 1, 2] {
            let event = unsafe { CGEventCreateMouseEvent(std::ptr::null_mut(), kind, point, 0) };
            if event.is_null() {
                return Err("native mouse event creation failed".into());
            }
            unsafe {
                CGEventPost(0, event);
                CFRelease(event);
            }
        }
        let mut events = Vec::new();
        for _ in 0..100 {
            app.pump()?;
            events.extend(app.take_mouse());
            if events
                .iter()
                .any(|e| e.window == window.number() && e.message == 0x202)
            {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let events: Vec<_> = events
            .into_iter()
            .filter(|e| e.window == window.number())
            .collect();
        for (message, buttons) in [(0x200, 0), (0x201, 1), (0x202, 0)] {
            let event = events
                .iter()
                .find(|e| e.message == message)
                .ok_or("posted native mouse event was not delivered")?;
            assert_eq!(event.client, [75, 90]);
            assert_eq!(event.screen, [point.x as i32, point.y as i32]);
            assert_eq!(event.keys & 3, buttons);
            assert!(event.timestamp_ns != 0);
        }
        println!("WindowServer/AppKit mouse move/down/up: {events:?}");
        if std::env::args().any(|arg| arg == "--capture") {
            let outside = host::Point {
                x: content.origin.x - 40.,
                y: content.origin.y - 30.,
            };
            for (kind, at) in [(1, point), (6, outside), (2, outside)] {
                let event = unsafe { CGEventCreateMouseEvent(std::ptr::null_mut(), kind, at, 0) };
                if event.is_null() {
                    return Err("native drag event creation failed".into());
                }
                unsafe {
                    CGEventPost(0, event);
                    CFRelease(event);
                }
                for _ in 0..10 {
                    app.pump()?;
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                if kind == 1 {
                    app.take_mouse();
                    assert_eq!(app.set_mouse_capture(Some(window.number()))?, None);
                    assert_eq!(app.mouse_capture(), Some(window.number()));
                    assert_eq!(
                        app.set_mouse_capture(Some(window.number()))?,
                        Some(window.number())
                    );
                } else {
                    let events = app.take_mouse();
                    let message = if kind == 6 { 0x200 } else { 0x202 };
                    let event = events
                        .iter()
                        .find(|e| e.message == message)
                        .ok_or("native outside-window drag/up was not captured")?;
                    assert_eq!(event.window, window.number());
                    assert_eq!(event.client, [-40, -30]);
                    assert_eq!(event.screen, [outside.x as i32, outside.y as i32]);
                    assert_eq!(event.keys & 1, u32::from(kind == 6));
                    println!("WindowServer/AppKit captured outside event: {event:?}");
                }
            }
            assert_eq!(app.set_mouse_capture(None)?, Some(window.number()));
            assert_eq!(app.mouse_capture(), None);
            assert_eq!(app.set_mouse_capture(None)?, None);
            assert!(app.set_mouse_capture(Some(i64::MAX)).is_err());
        }
    }
    window.show(false);
    app.pump()?;
    assert!(!window.visible());
    Ok(())
}
