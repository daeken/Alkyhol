//! Exercise the Rust C facade directly, including its output and last-error contract.
//! These are host assertions; real Windows comparison lives in the portable samples.
use alkyhol::{ffi::*, kernel::*};
use std::{
    ffi::c_void,
    path::PathBuf,
    ptr::{null, null_mut},
};

struct FileFixture {
    path: PathBuf,
    handle: Handle,
}
impl FileFixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "alkyhol-ffi-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let name: Vec<_> = path.to_str().unwrap().encode_utf16().chain([0]).collect();
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_ALL,
                null(),
                CREATE_NEW,
                FILE_ATTRIBUTE_NORMAL,
                0,
            )
        };
        assert_ne!(handle, INVALID_HANDLE_VALUE);
        Self { path, handle }
    }
}
impl Drop for FileFixture {
    fn drop(&mut self) {
        assert_eq!(CloseHandle(self.handle), 1);
        std::fs::remove_file(&self.path).unwrap();
    }
}

#[test]
fn last_error_and_close_handle() {
    SetLastError(0xdead_beef);
    assert_eq!(GetLastError(), 0xdead_beef);
    let event = unsafe { CreateEventW(null(), 0, 0, null()) };
    assert_ne!(event, 0);
    assert_eq!(GetLastError(), 0xdead_beef);
    assert_eq!(CloseHandle(event), 1);
    assert_eq!(GetLastError(), 0xdead_beef);
    assert_eq!(CloseHandle(event), 0);
    assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
}

#[test]
fn create_file_rejects_invalid_utf16_null_and_unterminated_paths() {
    let names = [vec![0xd800, 0], vec![65; 32768]];
    for (name, error) in names
        .iter()
        .zip([ERROR_NO_UNICODE_TRANSLATION, ERROR_INVALID_PARAMETER])
    {
        assert_eq!(
            unsafe {
                CreateFileW(
                    name.as_ptr(),
                    GENERIC_READ,
                    FILE_SHARE_ALL,
                    null(),
                    OPEN_EXISTING,
                    FILE_ATTRIBUTE_NORMAL,
                    0,
                )
            },
            INVALID_HANDLE_VALUE
        );
        assert_eq!(GetLastError(), error);
    }
    assert_eq!(
        unsafe {
            CreateFileW(
                null(),
                GENERIC_READ,
                FILE_SHARE_ALL,
                null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                0,
            )
        },
        INVALID_HANDLE_VALUE
    );
    assert_eq!(GetLastError(), ERROR_INVALID_PARAMETER);
}

#[test]
fn create_file_rejects_foreign_paths_security_and_template() {
    let marker = 1_u8;
    let security = (&marker as *const u8).cast::<c_void>();
    for path in ["C:/file", "\\\\server\\file", "relative\\file"] {
        let name: Vec<_> = path.encode_utf16().chain([0]).collect();
        assert_eq!(
            unsafe {
                CreateFileW(
                    name.as_ptr(),
                    GENERIC_WRITE,
                    FILE_SHARE_ALL,
                    null(),
                    CREATE_NEW,
                    FILE_ATTRIBUTE_NORMAL,
                    0,
                )
            },
            INVALID_HANDLE_VALUE
        );
        assert_eq!(GetLastError(), ERROR_NOT_SUPPORTED);
    }
    for (attributes, template) in [(security, 0), (null(), 1)] {
        assert_eq!(
            unsafe {
                CreateFileW(
                    null(),
                    GENERIC_WRITE,
                    FILE_SHARE_ALL,
                    attributes,
                    CREATE_NEW,
                    FILE_ATTRIBUTE_NORMAL,
                    template,
                )
            },
            INVALID_HANDLE_VALUE
        );
        assert_eq!(GetLastError(), ERROR_NOT_SUPPORTED);
    }
}

#[test]
fn read_write_size_seek_truncate_flush_and_eof() {
    let f = FileFixture::new();
    unsafe {
        let mut count = 99;
        SetLastError(123);
        assert_eq!(
            WriteFile(
                f.handle,
                b"hello".as_ptr().cast(),
                5,
                &mut count,
                null_mut()
            ),
            1
        );
        assert_eq!(count, 5);
        assert_eq!(GetLastError(), 123);
        let mut size = -1;
        assert_eq!(GetFileSizeEx(f.handle, &mut size), 1);
        assert_eq!(size, 5);
        let mut position = -1;
        assert_eq!(SetFilePointerEx(f.handle, -2, &mut position, 2), 1);
        assert_eq!(position, 3);
        let mut bytes = [0xcc_u8; 4];
        assert_eq!(
            ReadFile(
                f.handle,
                bytes.as_mut_ptr().cast(),
                4,
                &mut count,
                null_mut()
            ),
            1
        );
        assert_eq!(count, 2);
        assert_eq!(bytes, [b'l', b'o', 0xcc, 0xcc]);
        assert_eq!(
            ReadFile(
                f.handle,
                bytes.as_mut_ptr().cast(),
                4,
                &mut count,
                null_mut()
            ),
            1
        );
        assert_eq!(count, 0);
        assert_eq!(SetFilePointerEx(f.handle, 2, null_mut(), 0), 1);
        assert_eq!(SetEndOfFile(f.handle), 1);
        assert_eq!(FlushFileBuffers(f.handle), 1);
        assert_eq!(GetFileSizeEx(f.handle, &mut size), 1);
        assert_eq!(size, 2);
    }
}

#[test]
fn io_null_zero_length_and_overlapped_contract() {
    let f = FileFixture::new();
    unsafe {
        let mut count = 99;
        assert_eq!(ReadFile(f.handle, null_mut(), 0, &mut count, null_mut()), 1);
        assert_eq!(count, 0);
        assert_eq!(WriteFile(f.handle, null(), 0, &mut count, null_mut()), 1);
        assert_eq!(count, 0);
        for write in [false, true] {
            let mut marker = 0_u8;
            let marker = (&mut marker as *mut u8).cast::<c_void>();
            for (buffer, length, output, overlap, error) in [
                (
                    null_mut(),
                    1,
                    &mut count as *mut u32,
                    null_mut(),
                    ERROR_INVALID_PARAMETER,
                ),
                (marker, 1, null_mut(), null_mut(), ERROR_INVALID_PARAMETER),
                (marker, 1, &mut count, marker, ERROR_NOT_SUPPORTED),
            ] {
                count = 99;
                let result = if write {
                    WriteFile(f.handle, buffer, length, output, overlap)
                } else {
                    ReadFile(f.handle, buffer, length, output, overlap)
                };
                assert_eq!(result, 0);
                assert_eq!(GetLastError(), error);
                if !output.is_null() {
                    assert_eq!(count, 0);
                }
            }
        }
    }
}

#[test]
fn failed_file_calls_report_errors_and_preserve_optional_outputs() {
    let f = FileFixture::new();
    unsafe {
        assert_eq!(GetFileSizeEx(f.handle, null_mut()), 0);
        assert_eq!(GetLastError(), ERROR_INVALID_PARAMETER);
        let mut output = 123;
        assert_eq!(GetFileSizeEx(0, &mut output), 0);
        assert_eq!(output, 123);
        assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
        for (distance, method, error) in [
            (-1, 0, ERROR_NEGATIVE_SEEK),
            (0, 99, ERROR_INVALID_PARAMETER),
        ] {
            assert_eq!(SetFilePointerEx(f.handle, distance, &mut output, method), 0);
            assert_eq!(GetLastError(), error);
            assert_eq!(output, 123);
        }
        assert_eq!(SetFilePointerEx(0, 0, &mut output, 0), 0);
        assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
        assert_eq!(SetEndOfFile(0), 0);
        assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
        assert_eq!(FlushFileBuffers(0), 0);
        assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
        let mut count = 123;
        assert_eq!(ReadFile(0, null_mut(), 0, &mut count, null_mut()), 0);
        assert_eq!(count, 0);
        assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
        count = 123;
        assert_eq!(WriteFile(0, null(), 0, &mut count, null_mut()), 0);
        assert_eq!(count, 0);
        assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
    }
}

#[test]
fn event_exports_accept_nonzero_bools_and_reject_unknown_handles() {
    let event = unsafe { CreateEventW(null(), -7, 42, null()) };
    assert_ne!(event, 0);
    assert_eq!(WaitForSingleObject(event, 0), WAIT_OBJECT_0);
    assert_eq!(WaitForSingleObject(event, 0), WAIT_OBJECT_0);
    assert_eq!(ResetEvent(event), 1);
    assert_eq!(WaitForSingleObject(event, 1), WAIT_TIMEOUT);
    assert_eq!(SetEvent(event), 1);
    assert_eq!(WaitForSingleObject(event, INFINITE), WAIT_OBJECT_0);
    assert_eq!(CloseHandle(event), 1);
    assert_eq!(SetEvent(event), 0);
    assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
    assert_eq!(ResetEvent(event), 0);
    assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
    assert_eq!(WaitForSingleObject(event, 0), WAIT_FAILED);
    assert_eq!(GetLastError(), ERROR_INVALID_HANDLE);
    let automatic = unsafe { CreateEventW(null(), 0, 1, null()) };
    assert_eq!(WaitForSingleObject(automatic, 0), WAIT_OBJECT_0);
    assert_eq!(WaitForSingleObject(automatic, 0), WAIT_TIMEOUT);
    assert_eq!(CloseHandle(automatic), 1);
}

#[test]
fn event_creation_rejects_security_and_names() {
    let marker = 0_u8;
    let security = (&marker as *const u8).cast();
    for (security, name) in [(security, null()), (null(), [65_u16, 0].as_ptr())] {
        assert_eq!(unsafe { CreateEventW(security, 0, 0, name) }, 0);
        assert_eq!(GetLastError(), ERROR_NOT_SUPPORTED);
    }
}

#[test]
fn performance_exports_and_sleep_preserve_last_error() {
    unsafe {
        assert_eq!(QueryPerformanceCounter(null_mut()), 0);
        assert_eq!(GetLastError(), ERROR_INVALID_PARAMETER);
        assert_eq!(QueryPerformanceFrequency(null_mut()), 0);
        assert_eq!(GetLastError(), ERROR_INVALID_PARAMETER);
        let (mut a, mut b, mut frequency) = (-1, -1, -1);
        SetLastError(123);
        assert_eq!(QueryPerformanceFrequency(&mut frequency), 1);
        assert_eq!(frequency, PERFORMANCE_FREQUENCY);
        assert_eq!(QueryPerformanceCounter(&mut a), 1);
        Sleep(0);
        Sleep(1);
        assert_eq!(QueryPerformanceCounter(&mut b), 1);
        assert!(a >= 0 && b >= a);
        assert_eq!(GetLastError(), 123);
    }
}
