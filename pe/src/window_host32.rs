//! Direct AppKit window ownership for the PE32 main-thread GUI adapter.
//! No guest pointer is used as an Objective-C object or native window handle.
use std::{
    ffi::{CString, c_void},
    ptr,
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
}
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGMainDisplayID() -> u32;
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
    fn CFRelease(value: Id);
}
unsafe extern "C" {
    fn pthread_main_np() -> i32;
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
impl App {
    pub(super) fn new() -> crate::Result<Self> {
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
    }
    pub(super) fn frame_for_client(&self, rect: Rect, style: u64) -> crate::Result<Rect> {
        unsafe { convert(rect, style, "frameRectForContentRect:styleMask:") }
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
    }
    pub(super) fn pump(&self) -> crate::Result<()> {
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
    }
    pub(super) fn activate(&self) {
        unsafe { send1::<_, ()>(self.app, "activateIgnoringOtherApps:", 1i8) };
    }
}
impl Drop for App {
    fn drop(&mut self) {
        unsafe { send0::<()>(self.pool, "drain") };
    }
}
pub(super) struct Window {
    window: Id,
    desktop_height: f64,
}
impl Window {
    pub(super) fn show(&self, visible: bool) {
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
    }
    pub(super) fn visible(&self) -> bool {
        unsafe { send0::<i8>(self.window, "isVisible") != 0 }
    }
    pub(super) fn onscreen(&self) -> crate::Result<bool> {
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
    }
    pub(super) fn number(&self) -> i64 {
        unsafe { send0(self.window, "windowNumber") }
    }
    pub(super) fn frame(&self) -> Rect {
        let mut rect = unsafe { rectangle(self.window, "frame") };
        rect.origin.y = self.desktop_height - rect.origin.y - rect.size.height;
        rect
    }
    pub(super) fn client(&self) -> Rect {
        let view = unsafe { send0::<Id>(self.window, "contentView") };
        unsafe { rectangle(view, "bounds") }
    }
    pub(super) fn title(&self, text: &[u16]) -> crate::Result<()> {
        unsafe { send1::<_, ()>(self.window, "setTitle:", string(text)?) };
        Ok(())
    }
    pub(super) fn position(&self, x: i32, y: i32, width: i32, height: i32) {
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
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        unsafe {
            send0::<()>(self.window, "close");
            send0::<()>(self.window, "release")
        };
    }
}
