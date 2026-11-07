use alkyhol::kernel::*;
use std::{
    path::PathBuf,
    sync::{Arc, Barrier, mpsc},
    thread,
    time::Duration,
};
fn path() -> PathBuf {
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    std::env::temp_dir().join(format!(
        "alkyhol-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ))
}
fn open(k: &Kernel, p: &std::path::Path, access: u32, disposition: u32) -> WinResult<Handle> {
    k.create_file(
        p,
        access,
        FILE_SHARE_ALL,
        disposition,
        FILE_ATTRIBUTE_NORMAL,
    )
}
#[test]
fn stale_and_cross_kernel_handles_are_invalid() {
    let a = Kernel::default();
    let b = Kernel::default();
    let h = a.create_event(false, false);
    assert_eq!(b.close_handle(h), Err(WinError(ERROR_INVALID_HANDLE)));
    a.close_handle(h).unwrap();
    let next = a.create_event(false, true);
    assert_ne!(h, next);
    assert_eq!(a.wait(h, 0), Err(WinError(ERROR_INVALID_HANDLE)));
}
#[test]
fn file_lifecycle_sparse_seek_and_rights() {
    let k = Kernel::default();
    let p = path();
    let h = open(&k, &p, GENERIC_READ | GENERIC_WRITE, CREATE_NEW).unwrap();
    assert_eq!(
        open(&k, &p, GENERIC_WRITE, CREATE_NEW),
        Err(WinError(ERROR_FILE_EXISTS))
    );
    assert_eq!(k.write_file(h, b"abc").unwrap(), 3);
    assert_eq!(k.seek(h, 2, 2).unwrap(), 5);
    k.write_file(h, b"z").unwrap();
    k.seek(h, 0, 0).unwrap();
    let mut bytes = [0xff; 6];
    assert_eq!(k.read_file(h, &mut bytes).unwrap(), 6);
    assert_eq!(&bytes, b"abc\0\0z");
    assert_eq!(k.read_file(h, &mut bytes).unwrap(), 0);
    assert_eq!(k.seek(h, -7, 1), Err(WinError(ERROR_NEGATIVE_SEEK)));
    assert_eq!(k.seek(h, 0, 1).unwrap(), 6); // failure preserves position
    k.seek(h, 2, 0).unwrap();
    k.set_end_of_file(h).unwrap();
    assert_eq!(k.file_size(h).unwrap(), 2);
    k.close_handle(h).unwrap();
    let h = open(&k, &p, GENERIC_READ, OPEN_EXISTING).unwrap();
    assert_eq!(k.write_file(h, b"x"), Err(WinError(ERROR_ACCESS_DENIED)));
    assert_eq!(k.set_end_of_file(h), Err(WinError(ERROR_ACCESS_DENIED)));
    k.close_handle(h).unwrap();
    std::fs::remove_file(p).unwrap();
}
#[test]
fn disposition_and_last_error() {
    let k = Kernel::default();
    let p = path();
    set_last_error(42);
    let h = open(&k, &p, GENERIC_WRITE, OPEN_ALWAYS).unwrap();
    assert_eq!(get_last_error(), 0);
    k.write_file(h, b"test").unwrap();
    k.close_handle(h).unwrap();
    let h = open(&k, &p, GENERIC_WRITE, OPEN_ALWAYS).unwrap();
    assert_eq!(get_last_error(), ERROR_ALREADY_EXISTS);
    assert_eq!(k.file_size(h).unwrap(), 4);
    k.close_handle(h).unwrap();
    let h = open(&k, &p, GENERIC_WRITE, CREATE_ALWAYS).unwrap();
    assert_eq!(get_last_error(), ERROR_ALREADY_EXISTS);
    assert_eq!(k.file_size(h).unwrap(), 0);
    k.close_handle(h).unwrap();
    std::fs::remove_file(p).unwrap();
}
#[test]
fn unsupported_flags_do_not_create_files() {
    let k = Kernel::default();
    let p = path();
    assert_eq!(
        k.create_file(&p, GENERIC_WRITE, 0, CREATE_NEW, FILE_ATTRIBUTE_NORMAL),
        Err(WinError(ERROR_NOT_SUPPORTED))
    );
    assert_eq!(
        k.create_file(&p, GENERIC_WRITE, FILE_SHARE_ALL, CREATE_NEW, 0x40000000),
        Err(WinError(ERROR_NOT_SUPPORTED))
    );
    assert!(!p.exists());
}
#[test]
fn wrong_object_types_fail() {
    let k = Kernel::default();
    let h = k.create_event(false, false);
    assert_eq!(k.file_size(h), Err(WinError(ERROR_INVALID_HANDLE)));
    let p = path();
    let f = open(&k, &p, GENERIC_WRITE, CREATE_NEW).unwrap();
    assert_eq!(k.set_event(f), Err(WinError(ERROR_INVALID_HANDLE)));
    assert_eq!(k.wait(f, 0), Err(WinError(ERROR_NOT_SUPPORTED)));
    k.close_handle(f).unwrap();
    std::fs::remove_file(p).unwrap();
}
#[test]
fn automatic_event_is_saturating() {
    let k = Kernel::default();
    let h = k.create_event(false, false);
    k.set_event(h).unwrap();
    k.set_event(h).unwrap();
    assert_eq!(k.wait(h, 0).unwrap(), WAIT_OBJECT_0);
    assert_eq!(k.wait(h, 0).unwrap(), WAIT_TIMEOUT);
}
#[test]
fn manual_event_remains_signaled_until_reset() {
    let k = Kernel::default();
    let h = k.create_event(true, true);
    for _ in 0..5 {
        assert_eq!(k.wait(h, 0).unwrap(), WAIT_OBJECT_0);
    }
    k.reset_event(h).unwrap();
    assert_eq!(k.wait(h, 1).unwrap(), WAIT_TIMEOUT);
}
#[test]
fn signal_releases_a_waiting_thread() {
    let k = Arc::new(Kernel::default());
    let h = k.create_event(false, false);
    let barrier = Arc::new(Barrier::new(2));
    let (tx, rx) = mpsc::channel();
    let worker = {
        let k = k.clone();
        let barrier = barrier.clone();
        thread::spawn(move || {
            barrier.wait();
            tx.send(k.wait(h, 2000)).unwrap();
        })
    };
    barrier.wait();
    k.set_event(h).unwrap();
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(3)).unwrap(),
        Ok(WAIT_OBJECT_0)
    );
    worker.join().unwrap();
    assert_eq!(k.wait(h, 0).unwrap(), WAIT_TIMEOUT);
}
#[test]
fn manual_signal_releases_multiple_threads() {
    let k = Arc::new(Kernel::default());
    let h = k.create_event(true, false);
    let barrier = Arc::new(Barrier::new(5));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let k = k.clone();
            let b = barrier.clone();
            thread::spawn(move || {
                b.wait();
                k.wait(h, 2000)
            })
        })
        .collect();
    barrier.wait();
    k.set_event(h).unwrap();
    for worker in workers {
        assert_eq!(worker.join().unwrap(), Ok(WAIT_OBJECT_0));
    }
}
#[test]
fn last_error_is_per_thread() {
    set_last_error(123);
    thread::spawn(|| {
        assert_eq!(get_last_error(), 0);
        set_last_error(456);
    })
    .join()
    .unwrap();
    assert_eq!(get_last_error(), 123);
}
#[test]
fn ffi_rejects_null_and_overlapped_without_fake_success() {
    use alkyhol::ffi::*;
    use std::ptr::{null, null_mut};
    unsafe {
        assert_eq!(
            CreateFileW(
                null(),
                GENERIC_READ,
                FILE_SHARE_ALL,
                null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                0
            ),
            INVALID_HANDLE_VALUE
        );
        assert_eq!(GetLastError(), ERROR_INVALID_PARAMETER);
        let mut count = 999;
        let mut overlap = 0_u8;
        assert_eq!(
            ReadFile(
                0,
                null_mut(),
                0,
                &mut count,
                (&mut overlap as *mut u8).cast()
            ),
            0
        );
        assert_eq!(count, 0);
        assert_eq!(GetLastError(), ERROR_NOT_SUPPORTED);
        assert_eq!(QueryPerformanceCounter(null_mut()), 0);
        assert_eq!(CreateEventW(null(), 0, 0, [65_u16, 0].as_ptr()), 0);
        assert_eq!(GetLastError(), ERROR_NOT_SUPPORTED);
    }
}

#[test]
fn file_access_disposition_and_seek_boundaries() {
    let k = Kernel::default();
    let p = path();
    for access in [0, 1, GENERIC_READ | 1] {
        assert_eq!(
            open(&k, &p, access, CREATE_NEW),
            Err(WinError(ERROR_NOT_SUPPORTED))
        );
    }
    for disposition in [0, 6, u32::MAX] {
        assert_eq!(
            open(&k, &p, GENERIC_WRITE, disposition),
            Err(WinError(ERROR_INVALID_PARAMETER))
        );
    }
    assert_eq!(
        open(&k, &p, GENERIC_READ, CREATE_NEW),
        Err(WinError(ERROR_NOT_SUPPORTED))
    );
    assert_eq!(
        open(&k, &p, GENERIC_WRITE, TRUNCATE_EXISTING),
        Err(WinError(ERROR_FILE_NOT_FOUND))
    );
    assert!(!p.exists());
    let h = open(&k, &p, GENERIC_WRITE, CREATE_ALWAYS).unwrap();
    assert_eq!(get_last_error(), 0);
    assert_eq!(
        k.read_file(h, &mut [0; 1]),
        Err(WinError(ERROR_ACCESS_DENIED))
    );
    k.write_file(h, b"abcdef").unwrap();
    k.flush_file(h).unwrap();
    assert_eq!(k.seek(h, 1, 0).unwrap(), 1);
    assert_eq!(
        k.seek(h, i64::MAX, 1),
        Err(WinError(ERROR_INVALID_PARAMETER))
    );
    assert_eq!(k.seek(h, 0, 99), Err(WinError(ERROR_INVALID_PARAMETER)));
    assert_eq!(k.seek(h, 0, 1).unwrap(), 1);
    k.close_handle(h).unwrap();
    let h = open(&k, &p, GENERIC_WRITE, TRUNCATE_EXISTING).unwrap();
    assert_eq!(k.file_size(h).unwrap(), 0);
    k.close_handle(h).unwrap();
    let h = open(&k, &p, GENERIC_READ, OPEN_EXISTING).unwrap();
    assert_eq!(k.flush_file(h), Err(WinError(ERROR_ACCESS_DENIED)));
    k.close_handle(h).unwrap();
    std::fs::remove_file(p).unwrap();
}

#[test]
fn all_file_methods_reject_event_handles() {
    let k = Kernel::default();
    let h = k.create_event(false, false);
    assert_eq!(
        k.read_file(h, &mut [0; 1]),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    assert_eq!(k.write_file(h, b"a"), Err(WinError(ERROR_INVALID_HANDLE)));
    assert_eq!(k.seek(h, 0, 0), Err(WinError(ERROR_INVALID_HANDLE)));
    assert_eq!(k.set_end_of_file(h), Err(WinError(ERROR_INVALID_HANDLE)));
    assert_eq!(k.flush_file(h), Err(WinError(ERROR_INVALID_HANDLE)));
    k.close_handle(h).unwrap();
}

#[test]
fn common_io_errors_and_unknown_errors_remain_errors() {
    use std::io::ErrorKind;
    for (kind, expected) in [
        (ErrorKind::NotFound, ERROR_FILE_NOT_FOUND),
        (ErrorKind::PermissionDenied, ERROR_ACCESS_DENIED),
        (ErrorKind::AlreadyExists, ERROR_FILE_EXISTS),
        (ErrorKind::InvalidInput, ERROR_INVALID_PARAMETER),
        (ErrorKind::InvalidData, ERROR_INVALID_PARAMETER),
        (ErrorKind::Other, ERROR_INVALID_FUNCTION),
    ] {
        assert_eq!(
            WinError::from(std::io::Error::from(kind)),
            WinError(expected)
        );
    }
}

#[test]
fn retained_event_notifications_validate_kind_keep_identity_and_preserve_reset_modes() {
    let kernel = Kernel::default();
    let auto = kernel.create_event(false, false);
    let manual = kernel.create_event(true, false);
    let a = kernel.event_notifier(auto).unwrap();
    let m = kernel.event_notifier(manual).unwrap();
    a.signal();
    a.signal();
    m.signal();
    assert_eq!(kernel.wait(auto, 0).unwrap(), WAIT_OBJECT_0);
    assert_eq!(kernel.wait(auto, 0).unwrap(), WAIT_TIMEOUT);
    assert_eq!(kernel.wait(manual, 0).unwrap(), WAIT_OBJECT_0);
    assert_eq!(kernel.wait(manual, 0).unwrap(), WAIT_OBJECT_0);
    kernel.reset_event(manual).unwrap();
    assert_eq!(kernel.wait(manual, 0).unwrap(), WAIT_TIMEOUT);
    let semaphore = kernel.create_semaphore(0, 1).unwrap();
    assert!(matches!(
        kernel.event_notifier(semaphore),
        Err(WinError(ERROR_INVALID_HANDLE))
    ));
    let foreign = Kernel::default();
    assert!(matches!(
        foreign.event_notifier(auto),
        Err(WinError(ERROR_INVALID_HANDLE))
    ));
    kernel.close_handle(auto).unwrap();
    assert!(matches!(
        kernel.event_notifier(auto),
        Err(WinError(ERROR_INVALID_HANDLE))
    ));
    let next = kernel.create_event(false, false);
    a.signal();
    assert_eq!(kernel.wait(next, 0).unwrap(), WAIT_TIMEOUT);
    // The retained reference can still be used from another thread after close.
    std::thread::spawn(move || a.signal()).join().unwrap();
}
