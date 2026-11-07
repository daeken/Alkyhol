//! Native C ABI. Pointer parameters must reference valid host memory of the documented
//! length/alignment, including writable output pointers. Non-null does not prove validity.
//! This is not a guest-memory boundary or a memory safety sandbox.
#![allow(non_snake_case)]
#![allow(clippy::missing_safety_doc)] // Shared pointer contract above and in the C header.
use crate::kernel::*;
use std::{ffi::c_void, path::Path};

fn boolean(result: WinResult<()>) -> i32 {
    match result {
        Ok(()) => 1,
        Err(error) => {
            set_last_error(error.0);
            0
        }
    }
}
fn handle(result: WinResult<Handle>, failure: Handle) -> Handle {
    match result {
        Ok(handle) => handle,
        Err(error) => {
            set_last_error(error.0);
            failure
        }
    }
}
#[unsafe(no_mangle)]
pub extern "C" fn GetLastError() -> u32 {
    get_last_error()
}
#[unsafe(no_mangle)]
pub extern "C" fn SetLastError(error: u32) {
    set_last_error(error);
}
#[unsafe(no_mangle)]
pub extern "C" fn CloseHandle(h: Handle) -> i32 {
    boolean(kernel().close_handle(h))
}

unsafe fn wide_string(ptr: *const u16) -> WinResult<String> {
    if ptr.is_null() {
        return Err(WinError(ERROR_INVALID_PARAMETER));
    }
    let mut units = Vec::new();
    // Callers guarantee readable memory through the first NUL (at most 32768 units).
    for i in 0..32768 {
        let unit = unsafe { *ptr.add(i) };
        if unit == 0 {
            return String::from_utf16(&units).map_err(|_| WinError(ERROR_NO_UNICODE_TRANSLATION));
        }
        units.push(unit);
    }
    Err(WinError(ERROR_INVALID_PARAMETER))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CreateFileW(
    name: *const u16,
    access: u32,
    share: u32,
    security: *const c_void,
    disposition: u32,
    flags: u32,
    template: Handle,
) -> Handle {
    let result = (|| {
        if !security.is_null() || template != 0 {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        let path = unsafe { wide_string(name)? };
        if path.starts_with("\\\\") || path.as_bytes().get(1) == Some(&b':') || path.contains('\\')
        {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        kernel().create_file(Path::new(&path), access, share, disposition, flags)
    })();
    handle(result, INVALID_HANDLE_VALUE)
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ReadFile(
    h: Handle,
    buffer: *mut c_void,
    count: u32,
    transferred: *mut u32,
    overlapped: *mut c_void,
) -> i32 {
    if !transferred.is_null() {
        unsafe {
            *transferred = 0;
        }
    }
    if !overlapped.is_null() {
        return boolean(Err(WinError(ERROR_NOT_SUPPORTED)));
    }
    if transferred.is_null() || (count != 0 && buffer.is_null()) {
        return boolean(Err(WinError(ERROR_INVALID_PARAMETER)));
    }
    let bytes = if count == 0 {
        &mut []
    } else {
        unsafe { std::slice::from_raw_parts_mut(buffer.cast::<u8>(), count as usize) }
    };
    boolean(kernel().read_file(h, bytes).map(|n| unsafe {
        *transferred = n as u32;
    }))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn WriteFile(
    h: Handle,
    buffer: *const c_void,
    count: u32,
    transferred: *mut u32,
    overlapped: *mut c_void,
) -> i32 {
    if !transferred.is_null() {
        unsafe {
            *transferred = 0;
        }
    }
    if !overlapped.is_null() {
        return boolean(Err(WinError(ERROR_NOT_SUPPORTED)));
    }
    if transferred.is_null() || (count != 0 && buffer.is_null()) {
        return boolean(Err(WinError(ERROR_INVALID_PARAMETER)));
    }
    let bytes = if count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(buffer.cast::<u8>(), count as usize) }
    };
    boolean(kernel().write_file(h, bytes).map(|n| unsafe {
        *transferred = n as u32;
    }))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetFileSizeEx(h: Handle, size: *mut i64) -> i32 {
    if size.is_null() {
        return boolean(Err(WinError(ERROR_INVALID_PARAMETER)));
    }
    boolean(kernel().file_size(h).and_then(|n| {
        let n = i64::try_from(n).map_err(|_| WinError(ERROR_INVALID_PARAMETER))?;
        unsafe {
            *size = n;
        }
        Ok(())
    }))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn SetFilePointerEx(
    h: Handle,
    distance: i64,
    position: *mut i64,
    method: u32,
) -> i32 {
    boolean(kernel().seek(h, distance, method).map(|n| {
        if !position.is_null() {
            unsafe {
                *position = n as i64;
            }
        }
    }))
}
#[unsafe(no_mangle)]
pub extern "C" fn SetEndOfFile(h: Handle) -> i32 {
    boolean(kernel().set_end_of_file(h))
}
#[unsafe(no_mangle)]
pub extern "C" fn FlushFileBuffers(h: Handle) -> i32 {
    boolean(kernel().flush_file(h))
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CreateEventW(
    security: *const c_void,
    manual: i32,
    initial: i32,
    name: *const u16,
) -> Handle {
    if !security.is_null() || !name.is_null() {
        return handle(Err(WinError(ERROR_NOT_SUPPORTED)), 0);
    }
    kernel().create_event(manual != 0, initial != 0)
}
#[unsafe(no_mangle)]
pub extern "C" fn SetEvent(h: Handle) -> i32 {
    boolean(kernel().set_event(h))
}
#[unsafe(no_mangle)]
pub extern "C" fn ResetEvent(h: Handle) -> i32 {
    boolean(kernel().reset_event(h))
}
#[unsafe(no_mangle)]
pub extern "C" fn WaitForSingleObject(h: Handle, ms: u32) -> u32 {
    match kernel().wait(h, ms) {
        Ok(result) => result,
        Err(e) => {
            set_last_error(e.0);
            WAIT_FAILED
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn QueryPerformanceCounter(out: *mut i64) -> i32 {
    if out.is_null() {
        return boolean(Err(WinError(ERROR_INVALID_PARAMETER)));
    }
    unsafe {
        *out = performance_counter();
    }
    1
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn QueryPerformanceFrequency(out: *mut i64) -> i32 {
    if out.is_null() {
        return boolean(Err(WinError(ERROR_INVALID_PARAMETER)));
    }
    unsafe {
        *out = PERFORMANCE_FREQUENCY;
    }
    1
}
#[unsafe(no_mangle)]
pub extern "C" fn Sleep(ms: u32) {
    sleep(ms);
}
