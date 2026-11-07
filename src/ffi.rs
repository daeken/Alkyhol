//! Native C ABI. Pointer parameters must reference valid host memory of the documented
//! length/alignment, including writable output pointers. Non-null does not prove validity.
//! This is not a guest-memory boundary or a memory safety sandbox.
//! On Windows the Rust functions remain mangled to avoid interposing kernel32
//! used by std itself. The portable C probe uses the real Windows SDK there.
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
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn GetLastError() -> u32 {
    get_last_error()
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn SetLastError(error: u32) {
    set_last_error(error);
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn SetEndOfFile(h: Handle) -> i32 {
    boolean(kernel().set_end_of_file(h))
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn FlushFileBuffers(h: Handle) -> i32 {
    boolean(kernel().flush_file(h))
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
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
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn SetEvent(h: Handle) -> i32 {
    boolean(kernel().set_event(h))
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn ResetEvent(h: Handle) -> i32 {
    boolean(kernel().reset_event(h))
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub unsafe extern "C" fn CreateSemaphoreW(
    security: *const c_void,
    initial: i32,
    maximum: i32,
    name: *const u16,
) -> Handle {
    if !security.is_null() || !name.is_null() {
        return handle(Err(WinError(ERROR_NOT_SUPPORTED)), 0);
    }
    handle(kernel().create_semaphore(initial, maximum), 0)
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub unsafe extern "C" fn ReleaseSemaphore(h: Handle, release: i32, previous: *mut i32) -> i32 {
    boolean(kernel().release_semaphore(h, release).map(|count| {
        if !previous.is_null() {
            unsafe {
                *previous = count;
            }
        }
    }))
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn WaitForSingleObject(h: Handle, ms: u32) -> u32 {
    match kernel().wait(h, ms) {
        Ok(result) => result,
        Err(e) => {
            set_last_error(e.0);
            WAIT_FAILED
        }
    }
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub unsafe extern "C" fn QueryPerformanceCounter(out: *mut i64) -> i32 {
    if out.is_null() {
        return boolean(Err(WinError(ERROR_INVALID_PARAMETER)));
    }
    unsafe {
        *out = performance_counter();
    }
    1
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub unsafe extern "C" fn QueryPerformanceFrequency(out: *mut i64) -> i32 {
    if out.is_null() {
        return boolean(Err(WinError(ERROR_INVALID_PARAMETER)));
    }
    unsafe {
        *out = PERFORMANCE_FREQUENCY;
    }
    1
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn Sleep(ms: u32) {
    sleep(ms);
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn TlsAlloc() -> u32 {
    match crate::tls::alloc() {
        Ok(index) => index,
        Err(error) => {
            set_last_error(error.0);
            crate::tls::TLS_OUT_OF_INDEXES
        }
    }
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn TlsFree(index: u32) -> i32 {
    boolean(crate::tls::free(index))
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn TlsGetValue(index: u32) -> *mut c_void {
    match crate::tls::get(index) {
        Ok(value) => {
            // Unlike most Win32 APIs, successful TlsGetValue always clears last-error.
            set_last_error(0);
            value as *mut c_void
        }
        Err(error) => {
            set_last_error(error.0);
            std::ptr::null_mut()
        }
    }
}
#[cfg_attr(not(target_os = "windows"), unsafe(no_mangle))]
pub extern "C" fn TlsSetValue(index: u32, value: *mut c_void) -> i32 {
    boolean(crate::tls::set(index, value as usize))
}
