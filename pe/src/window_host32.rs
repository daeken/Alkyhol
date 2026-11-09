//! Direct AppKit window ownership for the PE32 main-thread GUI adapter.
//! No guest pointer is used as an Objective-C object or native window handle.
use std::{
    collections::VecDeque,
    ffi::{CString, c_void},
    ptr,
    sync::{
        Condvar, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
pub(super) type Id = *mut c_void;
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
    fn CGWindowLevelForKey(key: i32) -> i32;
    fn CGDisplayBounds(display: u32) -> Rect;
    fn CGWindowListCopyWindowInfo(options: u32, window: u32) -> Id;
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
unsafe fn rectangle(object: Id, name: &str) -> Rect {
    let mut out = Rect::default();
    let call: unsafe extern "C" fn(*mut Rect, Id, Sel) =
        unsafe { std::mem::transmute(objc_msgSend_stret as *const ()) };
    unsafe { call(&mut out, object, selector(name)) };
    out
}
unsafe fn convert(rect: Rect, style: u64, method: &str) -> crate::Result<Rect> {
    let mut out = Rect::default();
    let call: unsafe extern "C" fn(*mut Rect, Id, Sel, Rect, u64) =
        unsafe { std::mem::transmute(objc_msgSend_stret as *const ()) };
    unsafe { call(&mut out, class("NSWindow")?, selector(method), rect, style) };
    Ok(out)
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
pub(super) struct App {
    pool: Id,
    app: Id,
    pub desktop: Rect,
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
            if policy == 0 {
                return Err("AppKit activation policy was rejected".into());
            }
            unsafe { send0::<()>(app, "finishLaunching") };
            let desktop = unsafe { CGDisplayBounds(CGMainDisplayID()) };
            if !desktop.size.width.is_finite()
                || !desktop.size.height.is_finite()
                || desktop.size.width <= 0.
                || desktop.size.height <= 0.
            {
                return Err("actual host desktop geometry is unavailable".into());
            }
            Ok(Self { pool, app, desktop })
        })
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
                    y: self.desktop.size.height - y as f64 - height as f64,
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
                send1::<_, ()>(window, "setTitle:", string(title)?)
            };
            Ok(Window {
                window,
                desktop_height: self.desktop.size.height,
            })
        })
    }
    pub(super) fn pump(&self) -> crate::Result<()> {
        main_exec(move || {
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
                unsafe { send1::<_, ()>(self.app, "sendEvent:", event) };
            }
            unsafe { send0::<()>(self.app, "updateWindows") };
            Ok(())
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
            let mut left = f64::INFINITY;
            let mut top = f64::INFINITY;
            let mut right = f64::NEG_INFINITY;
            let mut bottom = f64::NEG_INFINITY;
            for index in 0..count {
                let screen = unsafe { send1::<_, Id>(screens, "objectAtIndex:", index) };
                let mut rect = unsafe { rectangle(screen, "frame") };
                rect.origin.y = self.desktop.size.height - rect.origin.y - rect.size.height;
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
    desktop_height: f64,
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
    pub(super) fn frame(&self) -> Rect {
        main_exec(move || {
            let mut rect = unsafe { rectangle(self.window, "frame") };
            rect.origin.y = self.desktop_height - rect.origin.y - rect.size.height;
            rect
        })
    }
    pub(super) fn client_frame(&self) -> crate::Result<Rect> {
        main_exec(move || {
            let frame = unsafe { rectangle(self.window, "frame") };
            let style = unsafe { send0::<u64>(self.window, "styleMask") };
            let mut content =
                unsafe { convert(frame, style, "contentRectForFrameRect:styleMask:") }?;
            content.origin.y = self.desktop_height - content.origin.y - content.size.height;
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
    pub(super) fn position(&self, x: i32, y: i32, width: i32, height: i32) {
        main_exec(move || {
            let frame = Rect {
                origin: Point {
                    x: x as f64,
                    y: self.desktop_height - y as f64 - height as f64,
                },
                size: Size {
                    width: width as f64,
                    height: height as f64,
                },
            };
            let call: unsafe extern "C" fn(Id, Sel, Rect, i8) =
                unsafe { std::mem::transmute(objc_msgSend as *const ()) };
            unsafe { call(self.window, selector("setFrame:display:"), frame, 1) };
        })
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        let window = self.window as usize;
        main_exec(move || unsafe {
            send0::<()>(window as Id, "close");
            send0::<()>(window as Id, "release")
        });
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
