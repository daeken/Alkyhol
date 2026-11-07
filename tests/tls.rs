use alkyhol::{ffi::*, kernel::*, tls::*};
use std::{
    ptr::null_mut,
    sync::{Arc, Barrier, Mutex, mpsc},
    thread,
};

// Capacity/reuse assertions intentionally own the whole Alkyhol pool. This lock
// serializes this test binary's TLS tests; spawned workers still run concurrently.
static TEST_POOL: Mutex<()> = Mutex::new(());

#[test]
fn null_nonnull_and_opaque_pointer_values_have_the_documented_last_error_behavior() {
    let _guard = TEST_POOL.lock().unwrap();
    SetLastError(123);
    let index = TlsAlloc();
    assert_ne!(index, TLS_OUT_OF_INDEXES);
    assert_eq!(GetLastError(), 123);
    assert_eq!(TlsGetValue(index), null_mut());
    assert_eq!(GetLastError(), 0);
    let mut marker = 42_u64;
    let pointer = (&mut marker as *mut u64).cast();
    for value in [
        pointer,
        std::ptr::without_provenance_mut(usize::MAX),
        null_mut(),
    ] {
        SetLastError(123);
        assert_eq!(TlsSetValue(index, value), 1);
        assert_eq!(GetLastError(), 123);
        assert_eq!(TlsGetValue(index), value);
        assert_eq!(GetLastError(), 0);
    }
    SetLastError(123);
    assert_eq!(TlsFree(index), 1);
    assert_eq!(GetLastError(), 123);
    assert_eq!(marker, 42); // TLS never dereferences or frees the stored address.
}

#[test]
fn live_slots_are_independent_within_and_between_threads() {
    let _guard = TEST_POOL.lock().unwrap();
    let a = TlsAlloc();
    let b = TlsAlloc();
    assert_ne!(a, TLS_OUT_OF_INDEXES);
    assert_ne!(b, TLS_OUT_OF_INDEXES);
    assert_ne!(a, b);
    assert_eq!(TlsSetValue(a, std::ptr::without_provenance_mut(123)), 1);
    let barrier = Arc::new(Barrier::new(4));
    let workers: Vec<_> = (1..=4)
        .map(|id| {
            let barrier = barrier.clone();
            thread::spawn(move || {
                assert_eq!(TlsGetValue(a), null_mut());
                assert_eq!(TlsGetValue(b), null_mut());
                assert_eq!(TlsSetValue(a, std::ptr::without_provenance_mut(id)), 1);
                assert_eq!(TlsSetValue(b, std::ptr::without_provenance_mut(id + 10)), 1);
                barrier.wait();
                assert_eq!(TlsGetValue(a) as usize, id);
                assert_eq!(TlsGetValue(b) as usize, id + 10);
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
    assert_eq!(TlsGetValue(a) as usize, 123);
    assert_eq!(TlsGetValue(b), null_mut());
    assert_eq!(TlsFree(a), 1);
    assert_eq!(TlsFree(b), 1);
}

#[test]
fn exhaustion_is_an_error_and_reallocated_slots_are_zero_in_existing_threads() {
    let _guard = TEST_POOL.lock().unwrap();
    let indices: Vec<_> = (0..TLS_CAPACITY).map(|_| TlsAlloc()).collect();
    assert!(indices.iter().all(|index| *index != TLS_OUT_OF_INDEXES));
    assert_eq!(
        indices
            .iter()
            .collect::<std::collections::HashSet<_>>()
            .len(),
        TLS_CAPACITY
    );
    assert_eq!(TlsAlloc(), TLS_OUT_OF_INDEXES);
    assert_eq!(GetLastError(), ERROR_NOT_ENOUGH_MEMORY);
    let index = indices[0];
    let (ready_tx, ready_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let worker = thread::spawn(move || {
        assert_eq!(TlsSetValue(index, std::ptr::without_provenance_mut(456)), 1);
        ready_tx.send(()).unwrap();
        resume_rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .unwrap();
        assert_eq!(TlsGetValue(index), null_mut());
        assert_eq!(GetLastError(), 0);
    });
    ready_rx
        .recv_timeout(std::time::Duration::from_secs(2))
        .unwrap();
    assert_eq!(TlsSetValue(index, std::ptr::without_provenance_mut(123)), 1);
    assert_eq!(TlsFree(index), 1);
    let reused = TlsAlloc();
    assert_eq!(reused, index); // The only available slot, not a general ordering contract.
    assert_eq!(TlsGetValue(reused), null_mut());
    resume_tx.send(()).unwrap();
    worker.join().unwrap();
    for index in indices {
        assert_eq!(TlsFree(index), 1);
    }
}

#[test]
fn invalid_and_unallocated_indices_fail_in_the_restricted_host_contract() {
    let _guard = TEST_POOL.lock().unwrap();
    // Windows does minimal validation of unallocated in-range indices. This
    // intentionally stricter out-of-contract behavior is NOT a Windows observation.
    let index = TlsAlloc();
    assert_ne!(index, TLS_OUT_OF_INDEXES);
    assert_eq!(TlsFree(index), 1);
    for invalid in [index, TLS_CAPACITY as u32, u32::MAX] {
        assert_eq!(TlsSetValue(invalid, null_mut()), 0);
        assert_eq!(GetLastError(), ERROR_INVALID_PARAMETER);
        assert_eq!(TlsGetValue(invalid), null_mut());
        assert_eq!(GetLastError(), ERROR_INVALID_PARAMETER);
        assert_eq!(TlsFree(invalid), 0);
        assert_eq!(GetLastError(), ERROR_INVALID_PARAMETER);
    }
}
