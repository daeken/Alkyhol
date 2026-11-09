//! Physical GUI capability probe; this is an owned window, not DAO execution.
fn unsupported(message: &str) -> ! {
    eprintln!("Alkyhol native window probe unsupported: {message}");
    std::process::exit(78)
}
type Result<T> = std::result::Result<T, String>;
#[path = "src/window_host32.rs"]
mod host;
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
    window.position(110, 120, outer.size.width as i32, outer.size.height as i32);
    window.show(true);
    app.activate();
    for _ in 0..20 {
        app.pump()?;
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let frame = window.frame();
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
    window.show(false);
    app.pump()?;
    assert!(!window.visible());
    Ok(())
}
