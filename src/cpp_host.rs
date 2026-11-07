//! Unix C++ source ABI bridges. wchar_t is UTF-32 here; the core Win32 ABI
//! deliberately retains UTF-16. These entry points must not be mixed up.
#![allow(non_snake_case, clippy::missing_safety_doc)]
use crate::kernel::*;
use std::{
    ffi::{CStr, c_char, c_void},
    os::unix::fs::MetadataExt,
    path::Path,
    ptr,
};

unsafe fn wide(p: *const u32) -> WinResult<String> {
    if p.is_null() {
        return Err(WinError(ERROR_INVALID_PARAMETER));
    }
    let mut result = String::new();
    for i in 0..32768 {
        let v = unsafe { *p.add(i) };
        if v == 0 {
            return Ok(result);
        }
        result.push(char::from_u32(v).ok_or(WinError(ERROR_NO_UNICODE_TRANSLATION))?);
    }
    Err(WinError(ERROR_INVALID_PARAMETER))
}
fn failure(error: u32) -> i32 {
    set_last_error(error);
    0
}
#[repr(C)]
pub struct CreateFile2Parameters {
    pub size: u32,
    pub attributes: u32,
    pub flags: u32,
    pub qos: u32,
    pub security: *const c_void,
    pub template: Handle,
}
#[repr(C)]
#[derive(Debug, Default)]
pub struct FileStandardInfo {
    pub allocation_size: i64,
    pub end_of_file: i64,
    pub links: u32,
    pub delete_pending: u8,
    pub directory: u8,
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn CreateFile2(
    name: *const u32,
    access: u32,
    share: u32,
    disposition: u32,
    parameters: *const CreateFile2Parameters,
) -> Handle {
    let result = (|| {
        let attributes = if parameters.is_null() {
            FILE_ATTRIBUTE_NORMAL
        } else {
            let p = unsafe { &*parameters };
            if p.size as usize != std::mem::size_of::<CreateFile2Parameters>() {
                return Err(WinError(ERROR_INVALID_PARAMETER));
            }
            // Sequential scan is a performance hint, with no observable I/O change.
            if p.flags & !0x08000000 != 0 || p.qos != 0 || !p.security.is_null() || p.template != 0
            {
                return Err(WinError(ERROR_NOT_SUPPORTED));
            }
            if p.attributes == 0 {
                FILE_ATTRIBUTE_NORMAL
            } else {
                p.attributes
            }
        };
        let path = unsafe { wide(name)? };
        if path.starts_with("\\\\") || path.as_bytes().get(1) == Some(&b':') {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        kernel().create_file(
            Path::new(&path.replace('\\', "/")),
            access,
            share,
            disposition,
            attributes,
        )
    })();
    match result {
        Ok(h) => h,
        Err(e) => {
            set_last_error(e.0);
            INVALID_HANDLE_VALUE
        }
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetFileInformationByHandleEx(
    handle: Handle,
    class: i32,
    out: *mut FileStandardInfo,
    size: u32,
) -> i32 {
    if class != 1 {
        return failure(ERROR_NOT_SUPPORTED);
    }
    if out.is_null() || (size as usize) < std::mem::size_of::<FileStandardInfo>() {
        return failure(ERROR_INVALID_PARAMETER);
    }
    match kernel().file_metadata(handle) {
        Ok(m) => {
            let Some(allocation) = m
                .blocks()
                .checked_mul(512)
                .and_then(|v| i64::try_from(v).ok())
            else {
                return failure(ERROR_INVALID_PARAMETER);
            };
            let Ok(length) = i64::try_from(m.len()) else {
                return failure(ERROR_INVALID_PARAMETER);
            };
            let Ok(links) = u32::try_from(m.nlink()) else {
                return failure(ERROR_INVALID_PARAMETER);
            };
            // The kernel has no delete-on-close mode; successful handles cannot be delete-pending.
            unsafe {
                ptr::write(
                    out,
                    FileStandardInfo {
                        allocation_size: allocation,
                        end_of_file: length,
                        links,
                        delete_pending: 0,
                        directory: u8::from(m.is_dir()),
                    },
                );
            }
            1
        }
        Err(e) => failure(e.0),
    }
}
static MAIN_MODULE: u8 = 0;
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetModuleHandleW(name: *const u32) -> *const c_void {
    if !name.is_null() {
        set_last_error(ERROR_NOT_SUPPORTED);
        return ptr::null();
    }
    (&raw const MAIN_MODULE).cast()
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn GetModuleFileNameW(
    module: *const c_void,
    out: *mut u32,
    capacity: u32,
) -> u32 {
    if !module.is_null() && module != (&raw const MAIN_MODULE).cast() {
        return failure(ERROR_NOT_SUPPORTED) as u32;
    }
    if out.is_null() || capacity == 0 {
        return failure(ERROR_INVALID_PARAMETER) as u32;
    }
    let path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => return failure(WinError::from(e).0) as u32,
    };
    let Some(text) = path.to_str() else {
        return failure(ERROR_NO_UNICODE_TRANSLATION) as u32;
    };
    let mut chars: Vec<u32> = text.chars().map(u32::from).collect();
    let length = chars.len();
    chars.push(0);
    if chars.len() > capacity as usize {
        unsafe {
            ptr::copy_nonoverlapping(chars.as_ptr(), out, capacity as usize - 1);
            *out.add(capacity as usize - 1) = 0;
        }
        set_last_error(122);
        capacity
    } else {
        unsafe {
            ptr::copy_nonoverlapping(chars.as_ptr(), out, chars.len());
        }
        length as u32
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn OutputDebugStringA(text: *const c_char) {
    if !text.is_null() {
        eprint!("{}", unsafe { CStr::from_ptr(text) }.to_string_lossy());
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn OutputDebugStringW(text: *const u32) {
    match unsafe { wide(text) } {
        Ok(s) => eprint!("{s}"),
        Err(e) => set_last_error(e.0),
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn MultiByteToWideChar(
    page: u32,
    flags: u32,
    input: *const u8,
    length: i32,
    out: *mut u32,
    capacity: i32,
) -> i32 {
    if length == 0
        || length < -1
        || capacity < 0
        || input.is_null()
        || (capacity > 0 && out.is_null())
    {
        return failure(ERROR_INVALID_PARAMETER);
    }
    if (page == 65001 && flags & !8 != 0)
        || (page == 0 && flags & !1 != 0)
        || (page != 0 && page != 65001)
    {
        return failure(ERROR_NOT_SUPPORTED);
    }
    let bytes = if length == -1 {
        unsafe { CStr::from_ptr(input.cast()) }.to_bytes_with_nul()
    } else {
        unsafe { std::slice::from_raw_parts(input, length as usize) }
    };
    // CP_ACP is intentionally bounded to portable ASCII until an ACP is configured.
    if page == 0 && !bytes.is_ascii() {
        return failure(ERROR_NO_UNICODE_TRANSLATION);
    }
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => std::borrow::Cow::Borrowed(s),
        Err(_) if flags & 8 == 0 => String::from_utf8_lossy(bytes),
        Err(_) => return failure(ERROR_NO_UNICODE_TRANSLATION),
    };
    let chars: Vec<u32> = text.chars().map(u32::from).collect();
    let Ok(required) = i32::try_from(chars.len()) else {
        return failure(ERROR_INVALID_PARAMETER);
    };
    if capacity == 0 {
        return required;
    }
    if capacity < required {
        return failure(122);
    }
    unsafe {
        ptr::copy_nonoverlapping(chars.as_ptr(), out, chars.len());
    }
    required
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn WideCharToMultiByte(
    page: u32,
    flags: u32,
    input: *const u32,
    length: i32,
    out: *mut u8,
    capacity: i32,
    default: *const u8,
    used: *mut i32,
) -> i32 {
    if length == 0
        || length < -1
        || capacity < 0
        || input.is_null()
        || (capacity > 0 && out.is_null())
    {
        return failure(ERROR_INVALID_PARAMETER);
    }
    if (page != 0 && page != 65001) || flags != 0 || !default.is_null() || !used.is_null() {
        return failure(ERROR_NOT_SUPPORTED);
    }
    let text = if length == -1 {
        match unsafe { wide(input) } {
            Ok(mut s) => {
                s.push('\0');
                s
            }
            Err(e) => return failure(e.0),
        }
    } else {
        let mut s = String::new();
        for v in unsafe { std::slice::from_raw_parts(input, length as usize) } {
            let Some(c) = char::from_u32(*v) else {
                return failure(ERROR_NO_UNICODE_TRANSLATION);
            };
            s.push(c);
        }
        s
    };
    if page == 0 && !text.is_ascii() {
        return failure(ERROR_NO_UNICODE_TRANSLATION);
    }
    let Ok(required) = i32::try_from(text.len()) else {
        return failure(ERROR_INVALID_PARAMETER);
    };
    if capacity == 0 {
        return required;
    }
    if capacity < required {
        return failure(122);
    }
    unsafe {
        ptr::copy_nonoverlapping(text.as_ptr(), out, text.len());
    }
    required
}
