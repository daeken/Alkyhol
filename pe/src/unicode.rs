//! Bounded Win64 UTF-8/UTF-16 conversion, including default replacement.
//! Other code pages are unsupported; the 32-bit wchar_t source ABI is separate.
#![allow(non_snake_case)]
use alkyhol::ffi;
use std::ptr;
const LIMIT: usize = 1024 * 1024;
fn fail(error: u32) -> i32 {
    ffi::SetLastError(error);
    0
}
unsafe fn input<T: Copy + Default + PartialEq>(
    pointer: *const T,
    length: i32,
) -> Result<Vec<T>, u32> {
    if pointer.is_null()
        || !(pointer as usize).is_multiple_of(std::mem::align_of::<T>())
        || length == 0
        || length < -1
    {
        return Err(87);
    }
    let limit = LIMIT / std::mem::size_of::<T>();
    if length == -1 {
        for i in 0..limit {
            if unsafe { pointer.add(i).read() } == T::default() {
                return Ok(unsafe { std::slice::from_raw_parts(pointer, i + 1) }.to_vec());
            }
        }
        return Err(50);
    }
    if length as usize > limit {
        return Err(50);
    }
    Ok(unsafe { std::slice::from_raw_parts(pointer, length as usize) }.to_vec())
}
unsafe fn output<T: Copy>(value: &[T], target: *mut T, capacity: i32) -> i32 {
    if capacity < 0
        || (capacity > 0
            && (target.is_null() || !(target as usize).is_multiple_of(std::mem::align_of::<T>())))
    {
        return fail(87);
    }
    if capacity == 0 {
        return value.len() as i32;
    }
    if (capacity as usize) < value.len() {
        return fail(122);
    }
    unsafe {
        ptr::copy_nonoverlapping(value.as_ptr(), target, value.len());
    }
    value.len() as i32
}
// Native Windows rejects an out-of-range second continuation byte after
// consuming the leading pair. Rust's maximal-subpart decoder consumes only
// the lead byte in these cases. Keep valid text and other errors on std's path.
fn windows_utf8_lossy(bytes: &[u8]) -> String {
    let mut text = String::new();
    let mut start = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        let first = bytes[i];
        let second = bytes[i + 1];
        let rejected = (0x80..=0xbf).contains(&second)
            && match first {
                0xe0 => second < 0xa0,
                0xed => second >= 0xa0,
                0xf0 => second < 0x90,
                0xf4 => second >= 0x90,
                _ => false,
            };
        if rejected {
            text.push_str(&String::from_utf8_lossy(&bytes[start..i]));
            text.push('�');
            i += 2;
            start = i;
        } else {
            i += 1;
        }
    }
    text.push_str(&String::from_utf8_lossy(&bytes[start..]));
    text
}
#[alkyhol_host_abi::host_abi]
unsafe extern "win64" fn MultiByteToWideChar(
    page: u32,
    flags: u32,
    source: *const u8,
    length: i32,
    target: *mut u16,
    capacity: i32,
) -> i32 {
    if page != 65001 || ![0, 8].contains(&flags) {
        return fail(50);
    }
    if capacity < 0 || (capacity > 0 && (target.is_null() || source.cast::<u16>() == target)) {
        return fail(87);
    }
    let bytes = match unsafe { input(source, length) } {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    let text = match std::str::from_utf8(&bytes) {
        Ok(v) => std::borrow::Cow::Borrowed(v),
        Err(_) if flags == 0 => std::borrow::Cow::Owned(windows_utf8_lossy(&bytes)),
        Err(_) => return fail(1113),
    };
    let wide: Vec<u16> = text.encode_utf16().collect();
    unsafe { output(&wide, target, capacity) }
}
#[alkyhol_host_abi::host_abi]
unsafe extern "win64" fn WideCharToMultiByte(
    page: u32,
    flags: u32,
    source: *const u16,
    length: i32,
    target: *mut u8,
    capacity: i32,
    default: *const u8,
    used: *mut i32,
) -> i32 {
    if page != 65001 || ![0, 128].contains(&flags) {
        return fail(50);
    }
    if !default.is_null()
        || !used.is_null()
        || capacity < 0
        || (capacity > 0 && (target.is_null() || source.cast::<u8>() == target))
    {
        return fail(87);
    }
    let wide = match unsafe { input(source, length) } {
        Ok(v) => v,
        Err(e) => return fail(e),
    };
    let text = match String::from_utf16(&wide) {
        Ok(v) => v,
        Err(_) if flags == 0 => String::from_utf16_lossy(&wide),
        Err(_) => return fail(1113),
    };
    unsafe { output(text.as_bytes(), target, capacity) }
}
pub(crate) fn address(name: &str) -> Option<usize> {
    match name {
        "MultiByteToWideChar" => Some(MultiByteToWideChar as *const () as usize),
        "WideCharToMultiByte" => Some(WideCharToMultiByte as *const () as usize),
        _ => None,
    }
}
#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;
    #[test]
    fn win64_round_trips_supplementary_embedded_and_terminated_nuls_with_exact_units() {
        let mb: unsafe extern "win64" fn(u32, u32, *const u8, i32, *mut u16, i32) -> i32 = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "MultiByteToWideChar").unwrap())
        };
        let wc: unsafe extern "win64" fn(
            u32,
            u32,
            *const u16,
            i32,
            *mut u8,
            i32,
            *const u8,
            *mut i32,
        ) -> i32 = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "WideCharToMultiByte").unwrap())
        };
        for text in ["", "ASCII", "λ🌸", "a\0b", "\u{10ffff}"] {
            for terminated in [false, true] {
                if text.is_empty() && !terminated {
                    continue;
                }
                let value = if terminated {
                    format!("{text}\0")
                } else {
                    text.into()
                };
                let bytes = value.as_bytes();
                let expected: Vec<u16> = value.encode_utf16().collect();
                let length = if terminated { -1 } else { bytes.len() as i32 };
                let units = if terminated {
                    value.split('\0').next().unwrap().encode_utf16().count() + 1
                } else {
                    expected.len()
                };
                ffi::SetLastError(0xbeef);
                assert_eq!(
                    unsafe { mb(65001, 0, bytes.as_ptr(), length, ptr::null_mut(), 0) },
                    units as i32
                );
                let mut wide = vec![0xabab; units + 1];
                assert_eq!(
                    unsafe {
                        mb(
                            65001,
                            8,
                            bytes.as_ptr(),
                            length,
                            wide.as_mut_ptr(),
                            units as i32,
                        )
                    },
                    units as i32
                );
                assert_eq!(&wide[..units], &expected[..units]);
                assert_eq!(wide[units], 0xabab);
                let result_length = if terminated { -1 } else { units as i32 };
                let size = unsafe {
                    wc(
                        65001,
                        0,
                        wide.as_ptr(),
                        result_length,
                        ptr::null_mut(),
                        0,
                        ptr::null(),
                        ptr::null_mut(),
                    )
                };
                let wanted = String::from_utf16(&expected[..units]).unwrap().into_bytes();
                assert_eq!(size as usize, wanted.len());
                let mut out = vec![0xab; wanted.len() + 1];
                assert_eq!(
                    unsafe {
                        wc(
                            65001,
                            128,
                            wide.as_ptr(),
                            result_length,
                            out.as_mut_ptr(),
                            size,
                            ptr::null(),
                            ptr::null_mut(),
                        )
                    },
                    size
                );
                assert_eq!(&out[..wanted.len()], wanted);
                assert_eq!(out[wanted.len()], 0xab);
                assert_eq!(ffi::GetLastError(), 0xbeef);
            }
        }
    }
    #[test]
    fn default_replacement_preserves_valid_text_and_native_invalid_prefix_groups() {
        for (bytes, expected) in [
            (vec![0xff], "�"),
            (vec![65, 0xff, 66], "A�B"),
            (vec![0xf0, 0x9f], "�"),
            (vec![0xed, 0xa0, 0x80], "��"),
            (vec![0xc0, 0xaf], "��"),
            (vec![0xe0, 0x80, 0x80], "��"),
            (vec![0xf0, 0x80, 0x80, 0x80], "���"),
            (
                vec![65, 0xed, 0xa0, 0x80, 66, 0xf4, 0x90, 0x80, 0x80, 67],
                "A��B���C",
            ),
            (vec![0xf4, 0x90, 0x80, 0x80], "���"),
        ] {
            let mut out = [0xababu16; 16];
            ffi::SetLastError(0xbeef);
            let count = unsafe {
                MultiByteToWideChar(
                    65001,
                    0,
                    bytes.as_ptr(),
                    bytes.len() as i32,
                    out.as_mut_ptr(),
                    16,
                )
            };
            let expected: Vec<u16> = expected.encode_utf16().collect();
            assert_eq!(count, expected.len() as i32);
            assert_eq!(&out[..expected.len()], expected);
            assert_eq!(out[expected.len()], 0xabab);
            assert_eq!(ffi::GetLastError(), 0xbeef);
        }
        for (wide, expected) in [
            (vec![0xd800], "�"),
            (vec![0xdc00], "�"),
            (vec![65, 0xd800, 66], "A�B"),
            (vec![0xd800, 0xd800, 0xdc00], "�𐀀"),
        ] {
            let mut out = [0xabu8; 16];
            ffi::SetLastError(0xbeef);
            let count = unsafe {
                WideCharToMultiByte(
                    65001,
                    0,
                    wide.as_ptr(),
                    wide.len() as i32,
                    out.as_mut_ptr(),
                    16,
                    ptr::null(),
                    ptr::null_mut(),
                )
            };
            assert_eq!(count, expected.len() as i32);
            assert_eq!(&out[..expected.len()], expected.as_bytes());
            assert_eq!(out[expected.len()], 0xab);
            assert_eq!(ffi::GetLastError(), 0xbeef);
        }
    }
    #[test]
    fn unsupported_encodings_invalid_unicode_flags_and_buffer_errors_do_not_become_success() {
        let mut wide = [0xababu16; 8];
        let mut bytes = [0xabu8; 8];
        let text = b"hello\0";
        for (page, flags, source, length, capacity, error) in [
            (0, 0, text.as_ptr(), -1, 8, 50),
            (65001, 1, text.as_ptr(), -1, 8, 50),
            (65001, 0, ptr::null(), -1, 8, 87),
            (65001, 0, text.as_ptr(), 0, 8, 87),
            (65001, 0, text.as_ptr(), -2, 8, 87),
            (65001, 0, text.as_ptr(), -1, -1, 87),
            (65001, 0, text.as_ptr(), -1, 5, 122),
            (65001, 0, text.as_ptr(), (LIMIT + 1) as i32, 8, 50),
        ] {
            assert_eq!(
                unsafe {
                    MultiByteToWideChar(page, flags, source, length, wide.as_mut_ptr(), capacity)
                },
                0
            );
            assert_eq!(ffi::GetLastError(), error);
            assert_eq!(wide, [0xabab; 8]);
        }
        {
            let flags = 8;
            assert_eq!(
                unsafe {
                    MultiByteToWideChar(65001, flags, [0xff].as_ptr(), 1, wide.as_mut_ptr(), 8)
                },
                0
            );
            assert_eq!(ffi::GetLastError(), 1113);
        }
        let valid = [65u16, 0];
        let invalid = [0xd800u16];
        {
            let flags = 128;
            assert_eq!(
                unsafe {
                    WideCharToMultiByte(
                        65001,
                        flags,
                        invalid.as_ptr(),
                        1,
                        bytes.as_mut_ptr(),
                        8,
                        ptr::null(),
                        ptr::null_mut(),
                    )
                },
                0
            );
            assert_eq!(ffi::GetLastError(), 1113);
        }
        assert_eq!(
            unsafe {
                WideCharToMultiByte(
                    65001,
                    0,
                    valid.as_ptr(),
                    -1,
                    bytes.as_mut_ptr(),
                    1,
                    ptr::null(),
                    ptr::null_mut(),
                )
            },
            0
        );
        assert_eq!(ffi::GetLastError(), 122);
        let mut used = 17;
        assert_eq!(
            unsafe {
                WideCharToMultiByte(
                    65001,
                    0,
                    valid.as_ptr(),
                    -1,
                    bytes.as_mut_ptr(),
                    8,
                    ptr::null(),
                    &mut used,
                )
            },
            0
        );
        assert_eq!(ffi::GetLastError(), 87);
        assert_eq!(used, 17);
        assert_eq!(bytes, [0xab; 8]);
        assert_eq!(
            unsafe { MultiByteToWideChar(65001, 0, text.as_ptr(), -1, ptr::null_mut(), 8) },
            0
        );
        assert_eq!(
            unsafe {
                MultiByteToWideChar(
                    65001,
                    0,
                    text.as_ptr(),
                    -1,
                    wide.as_mut_ptr().cast::<u8>().wrapping_add(1).cast(),
                    8,
                )
            },
            0
        );
        assert_eq!(ffi::GetLastError(), 87);
        let unterminated = vec![65u8; LIMIT];
        assert_eq!(
            unsafe {
                MultiByteToWideChar(65001, 0, unterminated.as_ptr(), -1, wide.as_mut_ptr(), 8)
            },
            0
        );
        assert_eq!(ffi::GetLastError(), 50);
    }
}
