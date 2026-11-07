//! Explicitly unavailable C++ application entry points. These are linkable
//! failure boundaries, not implementations or successful capability probes.
#![allow(non_snake_case, clippy::missing_safety_doc)]
use crate::{
    com::{E_POINTER, Guid},
    kernel::{ERROR_NOT_SUPPORTED, set_last_error},
};
use std::{ffi::c_void, ptr};
pub const E_NOTIMPL: i32 = 0x80004001u32 as i32;
fn unavailable(name: &str) {
    set_last_error(ERROR_NOT_SUPPORTED);
    eprintln!("Alkyhol: {name} is not implemented");
}
unsafe fn interface_failure(name: &str, out: *mut *mut c_void) -> i32 {
    unavailable(name);
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = ptr::null_mut();
    }
    E_NOTIMPL
}
#[cfg(not(feature = "cpp-metal-factory"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CreateDXGIFactory2(
    _flags: u32,
    _iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { interface_failure("CreateDXGIFactory2", out) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn D3D12CreateDevice(
    _adapter: *mut c_void,
    _level: i32,
    _iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    // A null output is also a feature probe. Never claim that a device can be created.
    unavailable("D3D12CreateDevice");
    if !out.is_null() {
        unsafe {
            *out = ptr::null_mut();
        }
    }
    E_NOTIMPL
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn D3D12GetDebugInterface(_iid: *const Guid, out: *mut *mut c_void) -> i32 {
    unsafe { interface_failure("D3D12GetDebugInterface", out) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn D3D12EnableExperimentalFeatures(
    _count: u32,
    _features: *const Guid,
    _config: *const c_void,
    _sizes: *const u32,
) -> i32 {
    unavailable("D3D12EnableExperimentalFeatures");
    E_NOTIMPL
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn DirectInput8Create(
    _instance: *mut c_void,
    _version: u32,
    _iid: *const Guid,
    out: *mut *mut c_void,
    _outer: *mut c_void,
) -> i32 {
    unsafe { interface_failure("DirectInput8Create", out) }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn XInputGetState(_user: u32, _state: *mut c_void) -> u32 {
    unavailable("XInputGetState");
    ERROR_NOT_SUPPORTED
}
macro_rules! unavailable_export{
    ($name:ident($($arg:ident:$type:ty),*) -> $ret:ty = $failure:expr)=>{
        #[unsafe(no_mangle)]pub extern "C" fn $name($($arg:$type),*)->$ret{unavailable(stringify!($name));$failure}
    };
}
unavailable_export!(LoadIconW(_instance:usize,_name:usize)->usize=0);
unavailable_export!(LoadCursorW(_instance:usize,_name:usize)->usize=0);
unavailable_export!(RegisterClassExW(_class:usize)->u16=0);
unavailable_export!(AdjustWindowRect(_rect:usize,_style:u32,_menu:i32)->i32=0);
unavailable_export!(CreateWindowExW(_ex:u32,_class:usize,_title:usize,_style:u32,_x:i32,_y:i32,_width:i32,_height:i32,_parent:usize,_menu:usize,_instance:usize,_parameter:usize)->usize=0);
unavailable_export!(ShowWindow(_window:usize,_command:i32)->i32=0);
unavailable_export!(SetWindowPos(_window:usize,_after:usize,_x:i32,_y:i32,_width:i32,_height:i32,_flags:u32)->i32=0);
unavailable_export!(GetClientRect(_window:usize,_rect:usize)->i32=0);
unavailable_export!(GetForegroundWindow()->usize=0);
unavailable_export!(IsWindowVisible(_window:usize)->i32=0);
unavailable_export!(PeekMessageW(_message:usize,_window:usize,_first:u32,_last:u32,_remove:u32)->i32=0);
unavailable_export!(TranslateMessage(_message:usize)->i32=0);
unavailable_export!(DispatchMessageW(_message:usize)->isize=0);
unavailable_export!(DefWindowProcW(_window:usize,_message:u32,_w:usize,_l:isize)->isize=0);
unavailable_export!(PostQuitMessage(_code:i32)->()=());
unavailable_export!(GetCommandLineW()->usize=0);
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CommandLineToArgvW(_text: *const u32, count: *mut i32) -> *mut *mut u32 {
    unavailable("CommandLineToArgvW");
    if !count.is_null() {
        unsafe {
            *count = 0;
        }
    }
    ptr::null_mut()
}
