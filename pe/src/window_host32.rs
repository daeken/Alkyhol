//! Direct AppKit window ownership for the PE32 main-thread GUI adapter.
//! No guest pointer is used as an Objective-C object or native window handle.
use std::{
    collections::{BTreeSet, VecDeque},
    ffi::{CString, c_void},
    ptr,
    sync::{
        Condvar, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
pub(super) type Id = *mut c_void;
static GUEST_WINDOWS: Mutex<BTreeSet<usize>> = Mutex::new(BTreeSet::new());
// DirectInput reads Quartz key state independently. Forwarding ordinary game
// keys to NSWindow's responder chain also invokes AppKit's unhandled-key beep.
// Native dialogs and Command shortcuts must retain their normal event routing.
fn consume_game_key(owned: bool, kind: u64, modifiers: u64) -> bool {
    owned && matches!(kind, 10 | 11) && modifiers & (1 << 20) == 0
}
#[test]
fn game_key_routing_retains_shortcuts_dialogs_and_nonkeyboard_events() {
    for kind in [10, 11] {
        for modifiers in [0, 1 << 17, 1 << 18, 1 << 19] {
            assert!(consume_game_key(true, kind, modifiers));
            assert!(!consume_game_key(false, kind, modifiers));
            assert!(!consume_game_key(true, kind, modifiers | (1 << 20)));
        }
    }
    for kind in [1, 2, 5, 12, 22, 25, 26, 27] {
        assert!(!consume_game_key(true, kind, 0));
    }
}
type Sel = *const c_void;
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub(super) struct Point {
    pub x: f64,
    pub y: f64,
}
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub(super) struct Size {
    pub width: f64,
    pub height: f64,
}
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub(super) struct Rect {
    pub origin: Point,
    pub size: Size,
}
#[link(name = "objc")]
unsafe extern "C" {
    fn objc_getClass(name: *const std::ffi::c_char) -> Id;
    fn sel_registerName(name: *const std::ffi::c_char) -> Sel;
    fn objc_msgSend();
    #[cfg(target_arch = "x86_64")]
    fn objc_msgSend_stret();
}
#[link(name = "AppKit", kind = "framework")]
unsafe extern "C" {
    static NSDefaultRunLoopMode: Id;
    static NSDeviceRGBColorSpace: Id;
}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGMainDisplayID() -> u32;
    fn CGDisplayCopyAllDisplayModes(display: u32, options: Id) -> Id;
    fn CGDisplayCopyDisplayMode(display: u32) -> Id;
    fn CGDisplayModeGetWidth(mode: Id) -> usize;
    fn CGDisplayModeGetHeight(mode: Id) -> usize;
    fn CGDisplayModeGetRefreshRate(mode: Id) -> f64;
    fn CGDisplayModeIsUsableForDesktopGUI(mode: Id) -> bool;
    fn CGWindowLevelForKey(key: i32) -> i32;
    fn CGDisplayBounds(display: u32) -> Rect;
    fn CGWindowListCopyWindowInfo(options: u32, window: u32) -> Id;
    fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
    static kCGWindowIsOnscreen: Id;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFArrayGetCount(array: Id) -> isize;
    fn CFArrayGetValueAtIndex(array: Id, index: isize) -> Id;
    fn CFDictionaryGetValue(dictionary: Id, key: Id) -> Id;
    fn CFBooleanGetValue(value: Id) -> i8;
    fn CFRelease(value: *const c_void);
}
unsafe extern "C" {
    fn pthread_main_np() -> i32;
}
/// Logical desktop coordinates match this bridge's window/client coordinates.
/// The virtual D3D9 adapter uses an explicitly selected 8-bit RGB display format.
#[derive(Debug)]
pub(super) struct DisplayModes {
    pub display: u32,
    pub current: [u32; 4],
    pub modes: Vec<[u32; 4]>,
}
fn display_mode(width: usize, height: usize, rate: f64) -> crate::Result<[u32; 4]> {
    if width == 0
        || height == 0
        || width > u32::MAX as usize
        || height > u32::MAX as usize
        || !rate.is_finite()
        || rate < 0.0
        || rate.round() > u32::MAX as f64
    {
        return Err("native display mode has invalid dimensions or refresh".into());
    }
    // CoreGraphics reports zero for an unspecified/dynamic refresh. Preserve it;
    // never turn an unknown rate into an invented fixed 60/120 Hz observation.
    Ok([width as u32, height as u32, rate.round() as u32, 22])
}
pub(super) fn display_modes() -> crate::Result<DisplayModes> {
    main_exec(|| {
        struct Owned(Id);
        impl Drop for Owned {
            fn drop(&mut self) {
                unsafe {
                    CFRelease(self.0);
                }
            }
        }
        fn mode(value: Id) -> crate::Result<[u32; 4]> {
            unsafe {
                display_mode(
                    CGDisplayModeGetWidth(value),
                    CGDisplayModeGetHeight(value),
                    CGDisplayModeGetRefreshRate(value),
                )
            }
        }
        let display = unsafe { CGMainDisplayID() };
        let current = unsafe { CGDisplayCopyDisplayMode(display) };
        if current.is_null() {
            return Err("current display mode is unavailable".into());
        }
        let current = Owned(current);
        let current = mode(current.0)?;
        let available = unsafe { CGDisplayCopyAllDisplayModes(display, ptr::null_mut()) };
        if available.is_null() {
            return Err("display mode inventory is unavailable".into());
        }
        let available = Owned(available);
        let count = unsafe { CFArrayGetCount(available.0) };
        if !(1..=4096).contains(&count) {
            return Err("display mode inventory exceeds bounds or is empty".into());
        }
        let mut modes = vec![current];
        for index in 0..count {
            let value = unsafe { CFArrayGetValueAtIndex(available.0, index) };
            if !value.is_null() && unsafe { CGDisplayModeIsUsableForDesktopGUI(value) } {
                modes.push(mode(value)?);
            }
        }
        modes.sort_unstable();
        modes.dedup();
        Ok(DisplayModes {
            display,
            current,
            modes,
        })
    })
}
// A request stays on the submitting thread's stack until main has completed it.
// Fn/T are Send, so only AppKit's opaque tokens cross native thread boundaries.
struct MainTask {
    address: usize,
    invoke: unsafe fn(usize),
}
static MAIN_TASKS: OnceLock<Mutex<VecDeque<MainTask>>> = OnceLock::new();
fn tasks() -> &'static Mutex<VecDeque<MainTask>> {
    MAIN_TASKS.get_or_init(|| Mutex::new(VecDeque::new()))
}
/// Balance one native hide/unhide pair for each active guest queue transition.
pub(super) fn cursor_visibility(visible: bool) -> crate::Result<()> {
    main_exec(move || unsafe {
        send0::<()>(class("NSCursor")?, if visible { "unhide" } else { "hide" });
        Ok(())
    })
}
pub(super) fn is_main_thread() -> bool {
    unsafe { pthread_main_np() == 1 }
}
pub(super) fn pump_tasks() {
    if !is_main_thread() {
        return;
    }
    for _ in 0..128 {
        let task = tasks().lock().unwrap().pop_front();
        let Some(task) = task else { break };
        unsafe {
            (task.invoke)(task.address);
        }
    }
}
fn main_exec<T: Send, F: FnOnce() -> T + Send>(operation: F) -> T {
    if is_main_thread() {
        return operation();
    }
    struct Request<F, T> {
        operation: Mutex<Option<F>>,
        value: Mutex<Option<T>>,
        changed: Condvar,
        started: AtomicBool,
    }
    unsafe fn invoke<F: FnOnce() -> T + Send, T: Send>(address: usize) {
        // SAFETY: main_exec does not return or move its request until value is
        // published. Queue tasks execute once; no guest/CPU RefCell is accessed.
        let request = unsafe { &*(address as *const Request<F, T>) };
        request.started.store(true, Ordering::Release);
        let operation = request
            .operation
            .lock()
            .unwrap()
            .take()
            .expect("one GUI request execution");
        let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation))
            .unwrap_or_else(|_| super::unsupported("host main-thread GUI request panicked"));
        // Hold publication lock through the final notification. The submitting
        // thread can wake from its timed wait without this notification; letting
        // it destroy the stack request first would race Condvar::notify_one.
        let mut slot = request.value.lock().unwrap();
        *slot = Some(value);
        request.changed.notify_one();
    }
    let request = Request {
        operation: Mutex::new(Some(operation)),
        value: Mutex::new(None),
        changed: Condvar::new(),
        started: AtomicBool::new(false),
    };
    tasks().lock().unwrap().push_back(MainTask {
        address: ptr::from_ref(&request) as usize,
        invoke: invoke::<F, T>,
    });
    let start = Instant::now();
    let mut value = request.value.lock().unwrap();
    while value.is_none() {
        let (next, _) = request
            .changed
            .wait_timeout(value, Duration::from_millis(100))
            .unwrap();
        value = next;
        if start.elapsed() > Duration::from_secs(5) && !request.started.load(Ordering::Acquire) {
            super::unsupported("GUI request requires a pumping host main thread");
        }
    }
    value.take().unwrap()
}
unsafe extern "C" {
    fn alkyhol_pe32_capture_display(
        width: usize,
        height: usize,
        bytes: *mut u8,
        length: usize,
    ) -> i32;
    #[cfg(test)]
    fn alkyhol_pe32_capture_color_contract() -> i32;
}
pub(super) fn capture_desktop() -> crate::Result<(u32, u32, Vec<u8>)> {
    main_exec(|| {
        let bounds = unsafe { CGDisplayBounds(CGMainDisplayID()) };
        let width = bounds.size.width as u32;
        let height = bounds.size.height as u32;
        if width == 0 || height == 0 || width > 8192 || height > 8192 {
            return Err("desktop capture dimensions exceed profile".into());
        }
        let mut pixels = vec![0; width as usize * height as usize * 4];
        let status = unsafe {
            alkyhol_pe32_capture_display(
                width as usize,
                height as usize,
                pixels.as_mut_ptr(),
                pixels.len(),
            )
        };
        if status != 0 {
            return Err(format!(
                "real ScreenCaptureKit desktop readback failed with status {status} (5 requires macOS Screen Recording access)"
            ));
        }
        Ok((width, height, pixels))
    })
}
fn selector(name: &str) -> Sel {
    let name = CString::new(name).unwrap();
    unsafe { sel_registerName(name.as_ptr()) }
}
fn class(name: &str) -> crate::Result<Id> {
    let name = CString::new(name).unwrap();
    let object = unsafe { objc_getClass(name.as_ptr()) };
    if object.is_null() {
        Err("required AppKit class is unavailable".into())
    } else {
        Ok(object)
    }
}
unsafe fn send0<R>(object: Id, name: &str) -> R {
    let call: unsafe extern "C" fn(Id, Sel) -> R =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { call(object, selector(name)) }
}
unsafe fn send1<A, R>(object: Id, name: &str, arg: A) -> R {
    let call: unsafe extern "C" fn(Id, Sel, A) -> R =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { call(object, selector(name), arg) }
}
#[cfg(target_arch = "x86_64")]
unsafe fn rectangle(object: Id, name: &str) -> Rect {
    let mut out = Rect::default();
    let call: unsafe extern "C" fn(*mut Rect, Id, Sel) =
        unsafe { std::mem::transmute(objc_msgSend_stret as *const ()) };
    unsafe { call(&mut out, object, selector(name)) };
    out
}
#[cfg(target_arch = "x86_64")]
unsafe fn convert(rect: Rect, style: u64, method: &str) -> crate::Result<Rect> {
    let mut out = Rect::default();
    let call: unsafe extern "C" fn(*mut Rect, Id, Sel, Rect, u64) =
        unsafe { std::mem::transmute(objc_msgSend_stret as *const ()) };
    unsafe { call(&mut out, class("NSWindow")?, selector(method), rect, style) };
    Ok(out)
}
#[cfg(target_arch = "aarch64")]
unsafe fn rectangle(object: Id, name: &str) -> Rect {
    let call: unsafe extern "C" fn(Id, Sel) -> Rect =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    unsafe { call(object, selector(name)) }
}
#[cfg(target_arch = "aarch64")]
unsafe fn convert(rect: Rect, style: u64, method: &str) -> crate::Result<Rect> {
    let call: unsafe extern "C" fn(Id, Sel, Rect, u64) -> Rect =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    Ok(unsafe { call(class("NSWindow")?, selector(method), rect, style) })
}
fn string(text: &[u16]) -> crate::Result<Id> {
    let call: unsafe extern "C" fn(Id, Sel, *const u16, usize) -> Id =
        unsafe { std::mem::transmute(objc_msgSend as *const ()) };
    let out = unsafe {
        call(
            class("NSString")?,
            selector("stringWithCharacters:length:"),
            text.as_ptr(),
            text.len(),
        )
    };
    if out.is_null() {
        Err("AppKit rejected the UTF-16 string".into())
    } else {
        Ok(out)
    }
}
// AppKit uses a bottom-left origin; Win32 and WindowServer use top-left.
// Display modes can change while the process is alive. Never cache this height
// in App/Window: a stale value shifts MSG.pt and every window/screen conversion.
fn flipped_y(desktop_height: f64, y: f64, extent: f64) -> f64 {
    desktop_height - y - extent
}
fn current_desktop() -> crate::Result<Rect> {
    checked_desktop(unsafe { CGDisplayBounds(CGMainDisplayID()) })
}
fn checked_desktop(rect: Rect) -> crate::Result<Rect> {
    if !rect.origin.x.is_finite()
        || !rect.origin.y.is_finite()
        || !rect.size.width.is_finite()
        || !rect.size.height.is_finite()
        || rect.size.width <= 0.
        || rect.size.height <= 0.
    {
        return Err("actual host desktop geometry is unavailable".into());
    }
    Ok(rect)
}
pub(super) struct App {
    pool: Id,
    app: Id,
    mouse: Mutex<MouseQueue>,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct MouseEvent {
    pub window: i64,
    pub message: u32,
    pub keys: u32,
    pub client: [i32; 2],
    pub screen: [i32; 2],
    pub timestamp_ns: u64,
    pub clicks: u64,
}
#[derive(Default)]
struct MouseQueue {
    buttons: u32,
    events: VecDeque<MouseEvent>,
    capture: Option<i64>,
    capture_foreground: bool,
}
// Background capture remains local to the visible owning window. Only an
// established foreground capture is lost on a subsequent real focus loss.
fn capture_mode(was_foreground: bool, visible: bool, foreground: bool) -> Option<bool> {
    (visible && (!was_foreground || foreground)).then_some(foreground)
}
fn mouse_message(kind: u64, button: u64, buttons: &mut u32, modifiers: u64) -> Option<(u32, u32)> {
    let (message, xbutton) = match kind {
        5..=7 | 27 => (0x200, 0),
        1..=4 | 25 | 26 => {
            if button > 4 {
                return None;
            }
            let down = matches!(kind, 1 | 3 | 25);
            if down {
                *buttons |= 1 << button;
            } else {
                *buttons &= !(1 << button);
            }
            let base = match button {
                0 => 0x201,
                1 => 0x204,
                2 => 0x207,
                _ => 0x20b,
            };
            (
                base + u32::from(!down),
                if button >= 3 { (button - 2) as u32 } else { 0 },
            )
        }
        _ => return None,
    };
    let keys = (*buttons & 3)
        | ((*buttons & 0x1c) << 2)
        | if modifiers & (1 << 17) != 0 { 4 } else { 0 }
        | if modifiers & (1 << 18) != 0 { 8 } else { 0 };
    Some((message, keys | (xbutton << 16)))
}
fn mouse_point(point: Point) -> crate::Result<[i32; 2]> {
    if [point.x, point.y]
        .iter()
        .any(|n| !n.is_finite() || n.floor() < i32::MIN as f64 || n.floor() > i32::MAX as f64)
    {
        return Err("native mouse coordinates exceed Win32 LONG bounds".into());
    }
    Ok([point.x.floor() as i32, point.y.floor() as i32])
}
// SAFETY: opaque AppKit tokens are only used on main_exec; metadata is immutable.
unsafe impl Send for App {}
unsafe impl Sync for App {}
impl App {
    pub(super) fn new() -> crate::Result<Self> {
        main_exec(move || {
            if unsafe { pthread_main_np() } != 1 {
                return Err("AppKit GUI operations require the host main thread".into());
            }
            let pool = unsafe { send0::<Id>(class("NSAutoreleasePool")?, "new") };
            let app = unsafe { send0::<Id>(class("NSApplication")?, "sharedApplication") };
            if pool.is_null() || app.is_null() {
                return Err("AppKit application initialization failed".into());
            }
            let policy: i8 = unsafe { send1(app, "setActivationPolicy:", 0isize) };
            let actual: isize = unsafe { send0(app, "activationPolicy") };
            if actual != 0 {
                return Err(format!(
                    "AppKit regular activation policy was not established; request_result={policy}; actual_policy={actual}"
                ));
            }
            unsafe { send0::<()>(app, "finishLaunching") };
            current_desktop()?;
            let mut mouse = MouseQueue::default();
            for button in 0..5 {
                if unsafe { CGEventSourceButtonState(1, button) } {
                    mouse.buttons |= 1 << button;
                }
            }
            Ok(Self {
                pool,
                app,
                mouse: Mutex::new(mouse),
            })
        })
    }
    pub(super) fn desktop(&self) -> crate::Result<Rect> {
        main_exec(current_desktop)
    }
    pub(super) fn frame_for_client(&self, rect: Rect, style: u64) -> crate::Result<Rect> {
        main_exec(move || unsafe { convert(rect, style, "frameRectForContentRect:styleMask:") })
    }
    pub(super) fn create(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        style: u64,
        title: &[u16],
    ) -> crate::Result<Window> {
        main_exec(move || {
            if width <= 0 || height <= 0 {
                return Err("native window dimensions must be positive".into());
            }
            let frame = Rect {
                origin: Point {
                    x: x as f64,
                    y: flipped_y(current_desktop()?.size.height, y as f64, height as f64),
                },
                size: Size {
                    width: width as f64,
                    height: height as f64,
                },
            };
            let content = unsafe { convert(frame, style, "contentRectForFrameRect:styleMask:") }?;
            let allocated = unsafe { send0::<Id>(class("NSWindow")?, "alloc") };
            let init: unsafe extern "C" fn(Id, Sel, Rect, u64, u64, i8) -> Id =
                unsafe { std::mem::transmute(objc_msgSend as *const ()) };
            let window = unsafe {
                init(
                    allocated,
                    selector("initWithContentRect:styleMask:backing:defer:"),
                    content,
                    style,
                    2,
                    0,
                )
            };
            if window.is_null() {
                return Err("native NSWindow creation failed".into());
            }
            unsafe {
                send1::<_, ()>(window, "setReleasedWhenClosed:", 0i8);
                send1::<_, ()>(window, "setAcceptsMouseMovedEvents:", 1i8);
                send1::<_, ()>(window, "setTitle:", string(title)?)
            };
            GUEST_WINDOWS.lock().unwrap().insert(window as usize);
            Ok(Window { window })
        })
    }
    pub(super) fn pump(&self) -> crate::Result<()> {
        main_exec(move || {
            let _pool = FramePool::new()?;
            let date = unsafe { send0::<Id>(class("NSDate")?, "distantPast") };
            let next: unsafe extern "C" fn(Id, Sel, u64, Id, Id, i8) -> Id =
                unsafe { std::mem::transmute(objc_msgSend as *const ()) };
            for _ in 0..128 {
                let event = unsafe {
                    next(
                        self.app,
                        selector("nextEventMatchingMask:untilDate:inMode:dequeue:"),
                        u64::MAX,
                        date,
                        NSDefaultRunLoopMode,
                        1,
                    )
                };
                if event.is_null() {
                    break;
                }
                self.capture_mouse(event)?;
                let kind = unsafe { send0::<u64>(event, "type") };
                let consume = if matches!(kind, 10 | 11) {
                    let window = unsafe { send0::<Id>(event, "window") };
                    let owned = GUEST_WINDOWS.lock().unwrap().contains(&(window as usize));
                    consume_game_key(owned, kind, unsafe { send0::<u64>(event, "modifierFlags") })
                } else {
                    false
                };
                if !consume {
                    unsafe { send1::<_, ()>(self.app, "sendEvent:", event) };
                }
            }
            unsafe { send0::<()>(self.app, "updateWindows") };
            Ok(())
        })
    }
    // Copy native event values while their autorelease pool is alive. Guest
    // callbacks run only through the owner's ordinary Win32 message dispatcher.
    fn capture_mouse(&self, event: Id) -> crate::Result<()> {
        let kind = unsafe { send0::<u64>(event, "type") };
        if !matches!(kind, 1..=7 | 25..=27) {
            return Ok(());
        }
        let mut window = unsafe { send0::<Id>(event, "window") };
        if window.is_null() {
            return Ok(());
        }
        let mut location = unsafe { send0::<Point>(event, "locationInWindow") };
        let screen = unsafe { send1::<Point, Point>(window, "convertPointToScreen:", location) };
        let mut queue = self.mouse.lock().unwrap();
        if let Some(number) = queue.capture {
            let captured =
                unsafe { send1::<_, Id>(self.app, "windowWithWindowNumber:", number as isize) };
            let active = unsafe { send0::<i8>(self.app, "isActive") } != 0
                && !captured.is_null()
                && unsafe { send0::<i8>(captured, "isKeyWindow") } != 0;
            let another_down = matches!(kind, 1 | 3 | 25)
                && unsafe { send0::<i64>(window, "windowNumber") } != number;
            let visible = !captured.is_null()
                && unsafe { send0::<i8>(captured, "isVisible") } != 0
                && unsafe { send0::<i8>(self.app, "isHidden") } == 0;
            if let Some(foreground) =
                capture_mode(queue.capture_foreground, visible, active).filter(|_| !another_down)
            {
                queue.capture_foreground = foreground;
                // WindowServer delivers background events only over the visible
                // owner. Do not reroute another window's events into this scope.
                if foreground || unsafe { send0::<i64>(window, "windowNumber") } == number {
                    window = captured;
                    location =
                        unsafe { send1::<Point, Point>(window, "convertPointFromScreen:", screen) };
                }
            } else {
                queue.capture = None;
                queue.capture_foreground = false;
            }
        }
        let view = unsafe { send0::<Id>(window, "contentView") };
        if view.is_null() {
            return Ok(());
        }
        let convert: unsafe extern "C" fn(Id, Sel, Point, Id) -> Point =
            unsafe { std::mem::transmute(objc_msgSend as *const ()) };
        let local = unsafe {
            convert(
                view,
                selector("convertPoint:fromView:"),
                location,
                ptr::null_mut(),
            )
        };
        let bounds = unsafe { rectangle(view, "bounds") };
        let x = local.x - bounds.origin.x;
        let y = local.y - bounds.origin.y;
        let flipped = unsafe { send0::<i8>(view, "isFlipped") } != 0;
        let client = mouse_point(Point {
            x,
            y: if flipped { y } else { bounds.size.height - y },
        })?;
        let screen = mouse_point(Point {
            x: screen.x,
            y: flipped_y(current_desktop()?.size.height, screen.y, 0.),
        })?;
        let seconds = unsafe { send0::<f64>(event, "timestamp") };
        if !seconds.is_finite() || seconds < 0.0 || seconds * 1e9 >= u64::MAX as f64 {
            return Err("native mouse timestamp exceeds bounds".into());
        }
        let button = if matches!(kind, 1..=4 | 25 | 26) {
            unsafe { send0::<u64>(event, "buttonNumber") }
        } else {
            0
        };
        let modifiers = unsafe { send0::<u64>(event, "modifierFlags") };
        if let Some((message, keys)) = mouse_message(kind, button, &mut queue.buttons, modifiers) {
            if !(queue.capture.is_some() && queue.capture_foreground)
                && (x < 0.0 || y < 0.0 || x >= bounds.size.width || y >= bounds.size.height)
            {
                return Ok(()); // Nonclient events still update the physical button state.
            }
            if queue.events.len() >= 4096 {
                return Err("native mouse queue exceeds bounds".into());
            }
            queue.events.push_back(MouseEvent {
                window: unsafe { send0(window, "windowNumber") },
                message,
                keys,
                client,
                screen,
                timestamp_ns: (seconds * 1e9) as u64,
                clicks: if matches!(kind, 1..=4 | 25 | 26) {
                    unsafe { send0(event, "clickCount") }
                } else {
                    0
                },
            });
        }
        Ok(())
    }
    pub(super) fn take_mouse(&self) -> Vec<MouseEvent> {
        self.mouse.lock().unwrap().events.drain(..).collect()
    }
    /// Foreground capture routes the native drag/up stream outside the owner.
    /// Background capture is limited to actual WindowServer events over its
    /// visible window; it never intercepts another process's mouse input.
    pub(super) fn set_mouse_capture(&self, number: Option<i64>) -> crate::Result<Option<i64>> {
        main_exec(move || {
            let foreground = if let Some(number) = number {
                let window =
                    unsafe { send1::<_, Id>(self.app, "windowWithWindowNumber:", number as isize) };
                if window.is_null()
                    || unsafe { send0::<i8>(window, "isVisible") } == 0
                    || unsafe { send0::<i8>(self.app, "isHidden") } != 0
                {
                    return Err("mouse capture requires a visible owned window".into());
                }
                (unsafe { send0::<i8>(self.app, "isActive") }) != 0
                    && (unsafe { send0::<i8>(window, "isKeyWindow") }) != 0
            } else {
                false
            };
            let mut mouse = self.mouse.lock().unwrap();
            let previous = mouse.capture;
            mouse.capture = number;
            mouse.capture_foreground = foreground;
            Ok(previous)
        })
    }
    pub(super) fn mouse_capture(&self) -> Option<i64> {
        main_exec(move || {
            let mut mouse = self.mouse.lock().unwrap();
            if let Some(number) = mouse.capture {
                let window =
                    unsafe { send1::<_, Id>(self.app, "windowWithWindowNumber:", number as isize) };
                let visible = !window.is_null()
                    && unsafe { send0::<i8>(window, "isVisible") } != 0
                    && unsafe { send0::<i8>(self.app, "isHidden") } == 0;
                let foreground = visible
                    && unsafe { send0::<i8>(self.app, "isActive") } != 0
                    && unsafe { send0::<i8>(window, "isKeyWindow") } != 0;
                if let Some(mode) = capture_mode(mouse.capture_foreground, visible, foreground) {
                    mouse.capture_foreground = mode;
                } else {
                    mouse.capture = None;
                    mouse.capture_foreground = false;
                }
            }
            mouse.capture
        })
    }
    pub(super) fn screen_geometry(&self) -> crate::Result<(Rect, u32)> {
        main_exec(move || {
            let screens = unsafe { send0::<Id>(class("NSScreen")?, "screens") };
            let count = unsafe { send0::<usize>(screens, "count") };
            if count == 0 || count > 32 {
                return Err(
                    "actual active screen inventory is unavailable or exceeds bounds".into(),
                );
            }
            let desktop_height = current_desktop()?.size.height;
            let mut left = f64::INFINITY;
            let mut top = f64::INFINITY;
            let mut right = f64::NEG_INFINITY;
            let mut bottom = f64::NEG_INFINITY;
            for index in 0..count {
                let screen = unsafe { send1::<_, Id>(screens, "objectAtIndex:", index) };
                let mut rect = unsafe { rectangle(screen, "frame") };
                rect.origin.y = flipped_y(desktop_height, rect.origin.y, rect.size.height);
                left = left.min(rect.origin.x);
                top = top.min(rect.origin.y);
                right = right.max(rect.origin.x + rect.size.width);
                bottom = bottom.max(rect.origin.y + rect.size.height);
            }
            Ok((
                Rect {
                    origin: Point { x: left, y: top },
                    size: Size {
                        width: right - left,
                        height: bottom - top,
                    },
                },
                count as u32,
            ))
        })
    }
    pub(super) fn activate(&self) {
        main_exec(move || {
            unsafe { send1::<_, ()>(self.app, "activateIgnoringOtherApps:", 1i8) };
        })
    }
}
impl Drop for App {
    fn drop(&mut self) {
        let pool = self.pool as usize;
        main_exec(move || unsafe { send0::<()>(pool as Id, "drain") });
    }
}
pub(super) struct Window {
    window: Id,
}
// SAFETY: every native operation and destruction is marshalled to main.
unsafe impl Send for Window {}
unsafe impl Sync for Window {}
impl Window {
    pub(super) fn present(&self, width: u32, height: u32, pixels: &[u8]) -> crate::Result<()> {
        if width == 0
            || height == 0
            || width > 8192
            || height > 8192
            || pixels.len() != width as usize * height as usize * 4
        {
            return Err("invalid GDI presentation surface".into());
        }
        if pixels.as_chunks::<4>().0.iter().any(|p| p[3] != 255) {
            return Err("GDI device presentation requires opaque RGB pixels".into());
        }
        main_exec(move || {
            let _pool = FramePool::new()?;
            let allocation = unsafe { send0::<Id>(class("NSBitmapImageRep")?, "alloc") };
            let initialize: unsafe extern "C" fn(
                Id,
                Sel,
                *mut *mut u8,
                isize,
                isize,
                isize,
                isize,
                i8,
                i8,
                Id,
                isize,
                isize,
            ) -> Id = unsafe { std::mem::transmute(objc_msgSend as *const ()) };
            let representation = unsafe {
                initialize(
                    allocation,
                    selector(
                        "initWithBitmapDataPlanes:pixelsWide:pixelsHigh:bitsPerSample:samplesPerPixel:hasAlpha:isPlanar:colorSpaceName:bytesPerRow:bitsPerPixel:",
                    ),
                    ptr::null_mut(),
                    width as isize,
                    height as isize,
                    8,
                    4,
                    1,
                    0,
                    NSDeviceRGBColorSpace,
                    width as isize * 4,
                    32,
                )
            };
            if representation.is_null() {
                return Err("AppKit bitmap allocation failed".into());
            }
            let data = unsafe { send0::<*mut u8>(representation, "bitmapData") };
            if data.is_null() {
                unsafe { send0::<()>(representation, "release") };
                return Err("AppKit bitmap storage unavailable".into());
            }
            let dest = unsafe { std::slice::from_raw_parts_mut(data, pixels.len()) };
            for (from, to) in pixels
                .as_chunks::<4>()
                .0
                .iter()
                .zip(dest.as_chunks_mut::<4>().0.iter_mut())
            {
                *to = [from[2], from[1], from[0], from[3]];
            }
            let image = unsafe {
                send1::<_, Id>(
                    send0::<Id>(class("NSImage")?, "alloc"),
                    "initWithSize:",
                    Size {
                        width: width as f64,
                        height: height as f64,
                    },
                )
            };
            if image.is_null() {
                unsafe { send0::<()>(representation, "release") };
                return Err("AppKit image allocation failed".into());
            }
            unsafe {
                send1::<_, ()>(image, "addRepresentation:", representation);
            }
            let view = unsafe {
                send1::<_, Id>(
                    send0::<Id>(class("NSImageView")?, "alloc"),
                    "initWithFrame:",
                    Rect {
                        origin: Point { x: 0., y: 0. },
                        size: Size {
                            width: width as f64,
                            height: height as f64,
                        },
                    },
                )
            };
            if view.is_null() {
                unsafe {
                    send0::<()>(image, "release");
                    send0::<()>(representation, "release")
                };
                return Err("AppKit image view allocation failed".into());
            }
            unsafe {
                send1::<_, ()>(view, "setImageScaling:", 1isize);
                send1::<_, ()>(view, "setAutoresizingMask:", 18usize);
                send1::<_, ()>(view, "setImage:", image);
                send1::<_, ()>(self.window, "setContentView:", view);
                send0::<()>(self.window, "displayIfNeeded");
                send0::<()>(view, "release");
                send0::<()>(image, "release");
                send0::<()>(representation, "release");
            }
            Ok(())
        })
    }
    pub(super) fn topmost(&self, value: bool) {
        main_exec(move || {
            let level = unsafe { CGWindowLevelForKey(if value { 5 } else { 4 }) };
            unsafe {
                send1::<_, ()>(self.window, "setLevel:", level as i64);
            }
        })
    }
    pub(super) fn disabled(&self, value: bool) {
        main_exec(move || unsafe {
            send1::<_, ()>(self.window, "setIgnoresMouseEvents:", value as i8)
        });
    }
    pub(super) fn show_unactivated(&self, visible: bool) {
        main_exec(move || unsafe {
            if visible {
                send0::<()>(self.window, "orderFrontRegardless");
            } else {
                send1::<_, ()>(self.window, "orderOut:", ptr::null_mut::<c_void>());
            }
        });
    }
    pub(super) fn show(&self, visible: bool) {
        main_exec(move || {
            unsafe {
                send1::<_, ()>(
                    self.window,
                    if visible {
                        "makeKeyAndOrderFront:"
                    } else {
                        "orderOut:"
                    },
                    ptr::null_mut::<c_void>(),
                )
            };
        })
    }
    pub(super) fn visible(&self) -> bool {
        main_exec(move || unsafe { send0::<i8>(self.window, "isVisible") != 0 })
    }
    pub(super) fn onscreen(&self) -> crate::Result<bool> {
        main_exec(move || {
            let number = self.number();
            if number <= 0 || number > u32::MAX as i64 {
                return Err("native window number is unavailable".into());
            }
            let list = unsafe { CGWindowListCopyWindowInfo(8, number as u32) };
            if list.is_null() {
                return Err("actual WindowServer metadata is unavailable".into());
            }
            let value = if unsafe { CFArrayGetCount(list) } == 1 {
                let row = unsafe { CFArrayGetValueAtIndex(list, 0) };
                let flag = unsafe { CFDictionaryGetValue(row, kCGWindowIsOnscreen) };
                if flag.is_null() {
                    None
                } else {
                    Some(unsafe { CFBooleanGetValue(flag) != 0 })
                }
            } else {
                None
            };
            unsafe { CFRelease(list) };
            value.ok_or("native window is absent from WindowServer metadata".into())
        })
    }
    pub(super) fn number(&self) -> i64 {
        main_exec(move || unsafe { send0(self.window, "windowNumber") })
    }
    pub(super) fn frame(&self) -> crate::Result<Rect> {
        main_exec(move || {
            let mut rect = unsafe { rectangle(self.window, "frame") };
            rect.origin.y = flipped_y(
                current_desktop()?.size.height,
                rect.origin.y,
                rect.size.height,
            );
            Ok(rect)
        })
    }
    pub(super) fn client_frame(&self) -> crate::Result<Rect> {
        main_exec(move || {
            let frame = unsafe { rectangle(self.window, "frame") };
            let style = unsafe { send0::<u64>(self.window, "styleMask") };
            let mut content =
                unsafe { convert(frame, style, "contentRectForFrameRect:styleMask:") }?;
            content.origin.y = flipped_y(
                current_desktop()?.size.height,
                content.origin.y,
                content.size.height,
            );
            Ok(content)
        })
    }
    pub(super) fn key(&self) -> bool {
        main_exec(move || unsafe { send0::<i8>(self.window, "isKeyWindow") != 0 })
    }
    pub(super) fn limits(&self) -> (Size, Size) {
        main_exec(move || unsafe { (send0(self.window, "minSize"), send0(self.window, "maxSize")) })
    }
    pub(super) fn client(&self) -> Rect {
        main_exec(move || {
            let view = unsafe { send0::<Id>(self.window, "contentView") };
            unsafe { rectangle(view, "bounds") }
        })
    }
    pub(super) fn title(&self, text: &[u16]) -> crate::Result<()> {
        main_exec(move || {
            unsafe { send1::<_, ()>(self.window, "setTitle:", string(text)?) };
            Ok(())
        })
    }
    /// Apply a real title/border/resizability transition while keeping the outer
    /// desktop rectangle. Win32 invokes this when SWP_FRAMECHANGED is requested.
    pub(super) fn frame_style(&self, style: u64) -> crate::Result<()> {
        main_exec(move || {
            let frame = unsafe { rectangle(self.window, "frame") };
            unsafe {
                send1::<_, ()>(self.window, "setStyleMask:", style);
            }
            let call: unsafe extern "C" fn(Id, Sel, Rect, i8) =
                unsafe { std::mem::transmute(objc_msgSend as *const ()) };
            unsafe {
                call(self.window, selector("setFrame:display:"), frame, 0);
            }
            if unsafe { send0::<u64>(self.window, "styleMask") } != style {
                return Err("native window style transition was not observed".into());
            }
            Ok(())
        })
    }
    pub(super) fn position(&self, x: i32, y: i32, width: i32, height: i32) -> crate::Result<()> {
        main_exec(move || {
            let frame = Rect {
                origin: Point {
                    x: x as f64,
                    y: flipped_y(current_desktop()?.size.height, y as f64, height as f64),
                },
                size: Size {
                    width: width as f64,
                    height: height as f64,
                },
            };
            let call: unsafe extern "C" fn(Id, Sel, Rect, i8) =
                unsafe { std::mem::transmute(objc_msgSend as *const ()) };
            unsafe { call(self.window, selector("setFrame:display:"), frame, 1) };
            Ok(())
        })
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        let window = self.window as usize;
        main_exec(move || unsafe {
            GUEST_WINDOWS.lock().unwrap().remove(&window);
            send0::<()>(window as Id, "close");
            send0::<()>(window as Id, "release")
        });
    }
}

// Nested pools bound temporary CoreGraphics/AppKit frame images to one host
// operation. NSWindow/NSImageView retain the current image beyond this scope.
struct FramePool(Id);
impl FramePool {
    fn new() -> crate::Result<Self> {
        let pool = unsafe { send0::<Id>(class("NSAutoreleasePool")?, "new") };
        if pool.is_null() {
            return Err("native frame autorelease pool allocation failed".into());
        }
        Ok(Self(pool))
    }
}
impl Drop for FramePool {
    fn drop(&mut self) {
        unsafe { send0::<()>(self.0, "drain") };
    }
}

pub(super) struct Cursor {
    cursor: Id,
}
// All native operations, including destruction, marshal to the host main thread.
unsafe impl Send for Cursor {}
unsafe impl Sync for Cursor {}
impl Cursor {
    pub(super) fn new(
        bytes: &[u8],
        width: u32,
        height: u32,
        x: u32,
        y: u32,
    ) -> crate::Result<Self> {
        let bytes = bytes.to_vec();
        main_exec(move || unsafe {
            let data_call: unsafe extern "C" fn(Id, Sel, *const u8, usize) -> Id =
                std::mem::transmute(objc_msgSend as *const ());
            let data = data_call(
                class("NSData")?,
                selector("dataWithBytes:length:"),
                bytes.as_ptr(),
                bytes.len(),
            );
            let image: Id = send1(
                send0::<Id>(class("NSImage")?, "alloc"),
                "initWithData:",
                data,
            );
            if image.is_null() || send0::<i8>(image, "isValid") == 0 {
                if !image.is_null() {
                    send0::<()>(image, "release");
                }
                return Err("AppKit could not decode the original CUR data".into());
            }
            let size: Size = send0(image, "size");
            if size.width != width as f64 || size.height != height as f64 {
                send0::<()>(image, "release");
                return Err("native cursor dimensions differ from the original directory".into());
            }
            let initialize: unsafe extern "C" fn(Id, Sel, Id, Point) -> Id =
                std::mem::transmute(objc_msgSend as *const ());
            let cursor = initialize(
                send0::<Id>(class("NSCursor")?, "alloc"),
                selector("initWithImage:hotSpot:"),
                image,
                Point {
                    x: x as f64,
                    y: y as f64,
                },
            );
            send0::<()>(image, "release");
            if cursor.is_null() {
                return Err("native cursor allocation failed".into());
            }
            let actual: Point = send0(cursor, "hotSpot");
            if actual.x != x as f64 || actual.y != y as f64 {
                send0::<()>(cursor, "release");
                return Err("native cursor hotspot differs from the original directory".into());
            }
            Ok(Cursor { cursor })
        })
    }
    pub(super) fn same(&self, other: &Self) -> bool {
        self.cursor == other.cursor
    }
    pub(super) fn retain_current() -> crate::Result<Option<Self>> {
        main_exec(|| unsafe {
            let cursor: Id = send0(class("NSCursor")?, "currentCursor");
            if cursor.is_null() {
                Ok(None)
            } else {
                send0::<()>(cursor, "retain");
                Ok(Some(Cursor { cursor }))
            }
        })
    }
    pub(super) fn replace(&self) -> crate::Result<Option<Self>> {
        let cursor = self.cursor as usize;
        main_exec(move || unsafe {
            let old: Id = send0(class("NSCursor")?, "currentCursor");
            if !old.is_null() {
                send0::<()>(old, "retain");
            }
            send0::<()>(cursor as Id, "set");
            if send0::<Id>(class("NSCursor")?, "currentCursor") != cursor as Id {
                if !old.is_null() {
                    send0::<()>(old, "release");
                }
                return Err("native cursor selection was not observed".into());
            }
            Ok(if old.is_null() {
                None
            } else {
                Some(Cursor { cursor: old })
            })
        })
    }
    pub(super) fn is_current(&self) -> bool {
        let cursor = self.cursor as usize;
        main_exec(move || unsafe {
            class("NSCursor").is_ok_and(|class| send0::<Id>(class, "currentCursor") == cursor as Id)
        })
    }
}
impl Drop for Cursor {
    fn drop(&mut self) {
        let cursor = self.cursor as usize;
        main_exec(move || unsafe { send0::<()>(cursor as Id, "release") });
    }
}

fn alert_buttons(flags: u32) -> crate::Result<Vec<(u32, &'static str)>> {
    if flags & !0x0005137f != 0 {
        return Err("message box modality/help/desktop flags are unsupported".into());
    }
    let buttons = match flags & 15 {
        0 => vec![(1, "OK")],
        1 => vec![(1, "OK"), (2, "Cancel")],
        2 => vec![(3, "Abort"), (4, "Retry"), (5, "Ignore")],
        3 => vec![(6, "Yes"), (7, "No"), (2, "Cancel")],
        4 => vec![(6, "Yes"), (7, "No")],
        5 => vec![(4, "Retry"), (2, "Cancel")],
        6 => vec![(2, "Cancel"), (10, "Try Again"), (11, "Continue")],
        _ => return Err("invalid message box button group".into()),
    };
    if (flags >> 8 & 3) as usize >= buttons.len() {
        return Err("message box default button outside its group".into());
    }
    Ok(buttons)
}
pub(super) fn message_box(title: Vec<u16>, text: Vec<u16>, flags: u32) -> crate::Result<u32> {
    let buttons = alert_buttons(flags)?;
    main_exec(move || unsafe {
        let alert: Id = send0(send0::<Id>(class("NSAlert")?, "alloc"), "init");
        if alert.is_null() {
            return Err("AppKit alert allocation failed".into());
        }
        send1::<_, ()>(alert, "setMessageText:", string(&title)?);
        send1::<_, ()>(alert, "setInformativeText:", string(&text)?);
        let style: isize = match flags & 0x70 {
            0x10 => 2,
            0x30 => 1,
            _ => 0,
        };
        send1::<_, ()>(alert, "setAlertStyle:", style);
        for (index, (_, label)) in buttons.iter().enumerate() {
            let button: Id = send1(
                alert,
                "addButtonWithTitle:",
                string(&label.encode_utf16().collect::<Vec<_>>())?,
            );
            let key = if index == (flags >> 8 & 3) as usize {
                "\r"
            } else if buttons[index].0 == 2 {
                "\u{1b}"
            } else {
                ""
            };
            send1::<_, ()>(
                button,
                "setKeyEquivalent:",
                string(&key.encode_utf16().collect::<Vec<_>>())?,
            );
        }
        if flags & 0x41000 != 0 {
            let window: Id = send0(alert, "window");
            send1::<_, ()>(window, "setLevel:", CGWindowLevelForKey(5) as i64);
        }
        let app: Id = send0(class("NSApplication")?, "sharedApplication");
        send1::<_, ()>(app, "activateIgnoringOtherApps:", 1i8);
        let response: isize = send0(alert, "runModal");
        send0::<()>(alert, "release");
        let index = response
            .checked_sub(1000)
            .and_then(|n| usize::try_from(n).ok());
        index
            .and_then(|i| buttons.get(i))
            .map(|p| p.0)
            .ok_or_else(|| format!("native alert returned unknown response {response}"))
    })
}

#[cfg(test)]
mod capture_tests {
    #[test]
    fn native_bitmap_row_orientation_rgb_conversion_and_opacity_are_observed() {
        assert_eq!(unsafe { super::alkyhol_pe32_capture_color_contract() }, 1);
    }
}

#[cfg(test)]
mod handoff_tests {
    #[test]
    fn capture_tracks_background_upgrade_real_focus_loss_and_visibility() {
        assert_eq!(super::capture_mode(false, true, false), Some(false));
        assert_eq!(super::capture_mode(false, true, true), Some(true));
        assert_eq!(super::capture_mode(true, true, true), Some(true));
        assert_eq!(super::capture_mode(true, true, false), None);
        for foreground in [false, true] {
            for old in [false, true] {
                assert_eq!(super::capture_mode(old, false, foreground), None);
            }
        }
    }
    #[test]
    fn native_mouse_buttons_retain_transitions_and_win32_modifier_bits() {
        use super::mouse_message;
        let mut buttons = 0;
        assert_eq!(mouse_message(1, 0, &mut buttons, 1 << 17), Some((0x201, 5)));
        assert_eq!(
            mouse_message(3, 1, &mut buttons, 1 << 18),
            Some((0x204, 11))
        );
        assert_eq!(mouse_message(6, 0, &mut buttons, 0), Some((0x200, 3)));
        assert_eq!(mouse_message(2, 0, &mut buttons, 0), Some((0x202, 2)));
        assert_eq!(mouse_message(4, 1, &mut buttons, 0), Some((0x205, 0)));
        for (button, base, mask) in [(2, 0x207, 16), (3, 0x20b, 0x10020), (4, 0x20b, 0x20040)] {
            assert_eq!(
                mouse_message(25, button, &mut buttons, 0),
                Some((base, mask))
            );
            assert_eq!(
                mouse_message(26, button, &mut buttons, 0),
                Some((base + 1, mask & 0xffff0000))
            );
        }
        assert_eq!(buttons, 0);
        assert_eq!(mouse_message(25, 5, &mut buttons, 0), None);
        assert_eq!(mouse_message(10, 0, &mut buttons, 0), None);
        assert_eq!(buttons, 0);
    }
    #[test]
    fn native_mouse_points_floor_signed_logical_cells_and_reject_invalid_coordinates() {
        use super::{Point, mouse_point};
        assert_eq!(mouse_point(Point { x: -0.2, y: 37.9 }).unwrap(), [-1, 37]);
        assert_eq!(
            mouse_point(Point {
                x: i32::MIN as f64,
                y: i32::MAX as f64 + 0.25
            })
            .unwrap(),
            [i32::MIN, i32::MAX]
        );
        for x in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            i32::MAX as f64 + 1.,
            i32::MIN as f64 - 0.25,
        ] {
            assert!(mouse_point(Point { x, y: 0. }).is_err());
            assert!(mouse_point(Point { x: 0., y: x }).is_err());
        }
    }
    #[test]
    fn message_box_button_ids_and_default_selection_are_bounded() {
        assert_eq!(super::alert_buttons(0).unwrap(), vec![(1, "OK")]);
        assert_eq!(
            super::alert_buttons(0x134).unwrap(),
            vec![(6, "Yes"), (7, "No")]
        );
        assert_eq!(
            super::alert_buttons(6).unwrap(),
            vec![(2, "Cancel"), (10, "Try Again"), (11, "Continue")]
        );
        for flags in [7, 0x100, 0x201, 0x2000, 0x400] {
            assert!(super::alert_buttons(flags).is_err());
        }
    }
    #[test]
    fn submitting_thread_can_destroy_each_completed_request_immediately() {
        let worker = std::thread::spawn(|| {
            for n in 0..512 {
                let payload = vec![n as u8; 257];
                let output = super::main_exec(move || (n, payload));
                assert_eq!(output, (n, vec![n as u8; 257]));
            }
        });
        while !worker.is_finished() {
            let task = super::tasks().lock().unwrap().pop_front();
            if let Some(task) = task {
                // Pure owned closures only: this test performs no AppKit work.
                unsafe {
                    (task.invoke)(task.address);
                }
            } else {
                std::thread::yield_now();
            }
        }
        worker.join().unwrap();
    }
}

#[cfg(test)]
mod display_mode_tests {
    use super::display_mode;
    #[test]
    fn mode_projection_preserves_unknown_refresh_and_checks_host_values() {
        assert_eq!(display_mode(1512, 982, 0.).unwrap(), [1512, 982, 0, 22]);
        assert_eq!(
            display_mode(1920, 1080, 59.94).unwrap(),
            [1920, 1080, 60, 22]
        );
        for (w, h, r) in [
            (0, 1080, 60.),
            (1, 0, 60.),
            (1, 1, f64::NAN),
            (1, 1, -1.),
            (1, 1, f64::INFINITY),
            (1, 1, u32::MAX as f64 + 1.),
        ] {
            assert!(display_mode(w, h, r).is_err());
        }
    }
}

#[cfg(test)]
mod desktop_geometry_tests {
    use super::{Point, Rect, Size, checked_desktop, current_desktop, flipped_y};
    #[test]
    fn display_change_keeps_message_points_and_window_rectangles_consistent() {
        // Reproduce the actual DAO mode-change diagnostic: cached 663 reported
        // y=320 for a physical y=900 event after the desktop became 1243 high.
        let native_y = 343.;
        assert_eq!(flipped_y(663., native_y, 0.), 320.);
        assert_eq!(flipped_y(1243., native_y, 0.), 900.);
        for height in [663., 1107., 1243., 1326.] {
            for (top, extent) in [(26., 1232.), (-120., 300.), (900., 0.)] {
                let native = flipped_y(height, top, extent);
                assert_eq!(flipped_y(height, native, extent), top);
                // The rectangle's top point and an event at that top have the
                // same Win32 Y even when its native origin is below the screen.
                assert_eq!(flipped_y(height, native + extent, 0.), top);
            }
        }
    }
    #[test]
    fn desktop_bounds_reject_unknown_geometry_without_a_stale_fallback() {
        let valid = Rect {
            origin: Point { x: 0., y: 0. },
            size: Size {
                width: 1710.,
                height: 1107.,
            },
        };
        assert_eq!(checked_desktop(valid).unwrap().size.height, 1107.);
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut rect = valid;
            rect.origin.x = value;
            assert!(checked_desktop(rect).is_err());
            let mut rect = valid;
            rect.origin.y = value;
            assert!(checked_desktop(rect).is_err());
        }
        for value in [0., -1., f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut rect = valid;
            rect.size.width = value;
            assert!(checked_desktop(rect).is_err());
            let mut rect = valid;
            rect.size.height = value;
            assert!(checked_desktop(rect).is_err());
        }
    }
    #[test]
    fn current_primary_desktop_query_returns_actual_coregraphics_bounds() {
        // Read-only physical-host evidence; this never switches modes or opens
        // a window. A live mode-change window/input replay remains separate.
        let rect = current_desktop().unwrap();
        let mode = unsafe { super::CGDisplayCopyDisplayMode(super::CGMainDisplayID()) };
        assert!(!mode.is_null());
        let (width, height) = unsafe {
            (
                super::CGDisplayModeGetWidth(mode),
                super::CGDisplayModeGetHeight(mode),
            )
        };
        unsafe {
            super::CFRelease(mode);
        }
        assert_eq!(
            (rect.size.width, rect.size.height),
            (width as f64, height as f64)
        );
    }
}
