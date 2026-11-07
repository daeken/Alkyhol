use alkyhol::{ffi, kernel::*};
use std::{
    ptr::{null, null_mut},
    sync::{Arc, Barrier},
    thread,
};

#[test]
fn creation_validates_counts_including_signed_boundaries() {
    let k = Kernel::default();
    for (initial, maximum) in [(-1, 1), (0, 0), (0, -1), (2, 1), (i32::MIN, i32::MAX)] {
        assert_eq!(
            k.create_semaphore(initial, maximum),
            Err(WinError(ERROR_INVALID_PARAMETER))
        );
    }
    let h = k.create_semaphore(i32::MAX, i32::MAX).unwrap();
    assert_eq!(
        k.release_semaphore(h, 1),
        Err(WinError(ERROR_TOO_MANY_POSTS))
    );
    assert_eq!(k.wait(h, 0).unwrap(), WAIT_OBJECT_0);
    assert_eq!(k.release_semaphore(h, 1).unwrap(), i32::MAX - 1);
    k.close_handle(h).unwrap();
}

#[test]
fn waits_consume_exactly_one_permit_and_failures_leave_count_unchanged() {
    let k = Kernel::default();
    let h = k.create_semaphore(1, 3).unwrap();
    for release in [0, -1, i32::MIN] {
        assert_eq!(
            k.release_semaphore(h, release),
            Err(WinError(ERROR_INVALID_PARAMETER))
        );
    }
    assert_eq!(
        k.release_semaphore(h, i32::MAX),
        Err(WinError(ERROR_TOO_MANY_POSTS))
    );
    assert_eq!(k.wait(h, INFINITE).unwrap(), WAIT_OBJECT_0);
    assert_eq!(k.wait(h, 0).unwrap(), WAIT_TIMEOUT);
    assert_eq!(k.wait(h, 1).unwrap(), WAIT_TIMEOUT);
    assert_eq!(k.release_semaphore(h, 2).unwrap(), 0);
    assert_eq!(k.release_semaphore(h, 1).unwrap(), 2);
    assert_eq!(
        k.release_semaphore(h, 1),
        Err(WinError(ERROR_TOO_MANY_POSTS))
    );
    for _ in 0..3 {
        assert_eq!(k.wait(h, 0).unwrap(), WAIT_OBJECT_0);
    }
    assert_eq!(k.wait(h, 0).unwrap(), WAIT_TIMEOUT);
    k.close_handle(h).unwrap();
}

#[test]
fn release_makes_multiple_permits_available_to_waiters_without_losing_any() {
    let k = Arc::new(Kernel::default());
    let h = k.create_semaphore(0, 4).unwrap();
    let ready = Arc::new(Barrier::new(5));
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let k = k.clone();
            let ready = ready.clone();
            thread::spawn(move || {
                ready.wait();
                k.wait(h, 2000)
            })
        })
        .collect();
    ready.wait();
    assert_eq!(k.release_semaphore(h, 4).unwrap(), 0);
    for worker in workers {
        assert_eq!(worker.join().unwrap(), Ok(WAIT_OBJECT_0));
    }
    assert_eq!(k.wait(h, 0).unwrap(), WAIT_TIMEOUT);
    k.close_handle(h).unwrap();
}

#[test]
fn concurrent_releases_report_a_serialized_previous_count() {
    let k = Arc::new(Kernel::default());
    let h = k.create_semaphore(0, 4).unwrap();
    let workers: Vec<_> = (0..4)
        .map(|_| {
            let k = k.clone();
            thread::spawn(move || k.release_semaphore(h, 1).unwrap())
        })
        .collect();
    let mut previous: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().unwrap())
        .collect();
    previous.sort();
    assert_eq!(previous, [0, 1, 2, 3]);
    k.close_handle(h).unwrap();
}

#[test]
fn wrong_object_cross_kernel_and_stale_tokens_fail() {
    let a = Kernel::default();
    let b = Kernel::default();
    let event = a.create_event(false, false);
    assert_eq!(
        a.release_semaphore(event, 1),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    let semaphore = a.create_semaphore(0, 1).unwrap();
    assert_eq!(a.set_event(semaphore), Err(WinError(ERROR_INVALID_HANDLE)));
    assert_eq!(
        a.reset_event(semaphore),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    assert_eq!(a.file_size(semaphore), Err(WinError(ERROR_INVALID_HANDLE)));
    assert_eq!(
        b.release_semaphore(semaphore, 1),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    assert_eq!(b.wait(semaphore, 0), Err(WinError(ERROR_INVALID_HANDLE)));
    a.close_handle(semaphore).unwrap();
    assert_eq!(
        a.release_semaphore(semaphore, 1),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    assert_eq!(a.wait(semaphore, 0), Err(WinError(ERROR_INVALID_HANDLE)));
    a.close_handle(event).unwrap();
}

#[test]
fn ffi_reports_previous_count_optional_outputs_and_failure_errors() {
    unsafe {
        ffi::SetLastError(123);
        let h = ffi::CreateSemaphoreW(null(), 0, 2, null());
        assert_ne!(h, 0);
        assert_eq!(ffi::GetLastError(), 123);
        let mut previous = -99;
        assert_eq!(ffi::ReleaseSemaphore(h, 1, &mut previous), 1);
        assert_eq!(previous, 0);
        assert_eq!(ffi::GetLastError(), 123);
        assert_eq!(ffi::ReleaseSemaphore(h, 1, null_mut()), 1);
        assert_eq!(ffi::ReleaseSemaphore(h, 1, &mut previous), 0);
        assert_eq!(ffi::GetLastError(), ERROR_TOO_MANY_POSTS);
        assert_eq!(previous, 0);
        assert_eq!(ffi::ReleaseSemaphore(h, 0, &mut previous), 0);
        assert_eq!(ffi::GetLastError(), ERROR_INVALID_PARAMETER);
        assert_eq!(previous, 0);
        assert_eq!(ffi::WaitForSingleObject(h, 0), WAIT_OBJECT_0);
        assert_eq!(ffi::WaitForSingleObject(h, 0), WAIT_OBJECT_0);
        assert_eq!(ffi::WaitForSingleObject(h, 0), WAIT_TIMEOUT);
        assert_eq!(ffi::CloseHandle(h), 1);
        assert_eq!(ffi::ReleaseSemaphore(h, 1, &mut previous), 0);
        assert_eq!(ffi::GetLastError(), ERROR_INVALID_HANDLE);
        assert_eq!(previous, 0);
    }
}

#[test]
fn ffi_rejects_invalid_counts_named_objects_and_security() {
    unsafe {
        assert_eq!(ffi::CreateSemaphoreW(null(), -1, 1, null()), 0);
        assert_eq!(ffi::GetLastError(), ERROR_INVALID_PARAMETER);
        let marker = 0_u8;
        let name = [0_u16];
        for (security, name) in [
            ((&marker as *const u8).cast(), null()),
            (null(), name.as_ptr()),
        ] {
            assert_eq!(ffi::CreateSemaphoreW(security, 0, 1, name), 0);
            assert_eq!(ffi::GetLastError(), ERROR_NOT_SUPPORTED);
        }
    }
}
