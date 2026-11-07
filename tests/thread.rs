use alkyhol::{ffi, kernel::*};
use std::{
    cell::RefCell,
    ffi::c_void,
    ptr::{null, null_mut},
    sync::{Arc, mpsc},
    thread,
    time::Duration,
};

#[test]
fn pending_return_code_persistent_signal_and_stale_handle() {
    let k = Kernel::default();
    let (send, receive) = mpsc::channel();
    let (handle, id) = k
        .create_thread(move || {
            receive.recv_timeout(Duration::from_secs(2)).unwrap();
            0xdead_beef
        })
        .unwrap();
    assert_ne!(id, 0);
    assert_ne!(id, current_thread_id());
    assert_eq!(k.thread_exit_code(handle).unwrap(), STILL_ACTIVE);
    assert_eq!(k.wait(handle, 0).unwrap(), WAIT_TIMEOUT);
    assert_eq!(k.wait(handle, 1).unwrap(), WAIT_TIMEOUT);
    send.send(()).unwrap();
    assert_eq!(k.wait(handle, 2000).unwrap(), WAIT_OBJECT_0);
    assert_eq!(k.wait(handle, 0).unwrap(), WAIT_OBJECT_0);
    assert_eq!(k.wait(handle, INFINITE).unwrap(), WAIT_OBJECT_0);
    assert_eq!(k.thread_exit_code(handle).unwrap(), 0xdead_beef);
    k.close_handle(handle).unwrap();
    assert_eq!(
        k.thread_exit_code(handle),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    assert_eq!(k.wait(handle, 0), Err(WinError(ERROR_INVALID_HANDLE)));
}

#[test]
fn exit_code_259_does_not_make_a_terminated_thread_unsignaled() {
    let k = Kernel::default();
    let (handle, _) = k.create_thread(|| STILL_ACTIVE).unwrap();
    assert_eq!(k.wait(handle, 2000).unwrap(), WAIT_OBJECT_0);
    assert_eq!(k.thread_exit_code(handle).unwrap(), STILL_ACTIVE);
    k.close_handle(handle).unwrap();
}

struct DestructorGate {
    entered: mpsc::Sender<()>,
    resume: mpsc::Receiver<()>,
}
impl Drop for DestructorGate {
    fn drop(&mut self) {
        let _ = self.entered.send(());
        let _ = self.resume.recv_timeout(Duration::from_secs(2));
    }
}
thread_local! { static DESTRUCTOR_GATE: RefCell<Option<DestructorGate>> = const { RefCell::new(None) }; }

#[test]
fn termination_waits_for_host_thread_local_destructors_and_query_stays_nonblocking() {
    let k = Kernel::default();
    let (entered_send, entered_receive) = mpsc::channel();
    let (resume_send, resume_receive) = mpsc::channel();
    let (handle, _) = k
        .create_thread(move || {
            DESTRUCTOR_GATE.with(|gate| {
                *gate.borrow_mut() = Some(DestructorGate {
                    entered: entered_send,
                    resume: resume_receive,
                })
            });
            42
        })
        .unwrap();
    entered_receive
        .recv_timeout(Duration::from_secs(2))
        .unwrap();
    assert_eq!(k.thread_exit_code(handle).unwrap(), STILL_ACTIVE);
    assert_eq!(k.wait(handle, 0).unwrap(), WAIT_TIMEOUT);
    resume_send.send(()).unwrap();
    assert_eq!(k.wait(handle, 2000).unwrap(), WAIT_OBJECT_0);
    assert_eq!(k.thread_exit_code(handle).unwrap(), 42);
    k.close_handle(handle).unwrap();
}

#[test]
fn closing_a_running_handle_detaches_without_stopping_its_worker() {
    let k = Kernel::default();
    let (resume_send, resume_receive) = mpsc::channel();
    let (done_send, done_receive) = mpsc::channel();
    let (handle, _) = k
        .create_thread(move || {
            resume_receive.recv_timeout(Duration::from_secs(2)).unwrap();
            done_send.send(123).unwrap();
            0
        })
        .unwrap();
    k.close_handle(handle).unwrap();
    resume_send.send(()).unwrap();
    assert_eq!(
        done_receive.recv_timeout(Duration::from_secs(2)).unwrap(),
        123
    );
    assert_eq!(k.wait(handle, 0), Err(WinError(ERROR_INVALID_HANDLE)));
}

#[test]
fn panicking_rust_workers_are_explicit_host_errors_not_successful_zero_exits() {
    // This tests the host Rust adapter's failure path, not Windows exception behavior.
    let k = Kernel::default();
    let (handle, _) = k
        .create_thread(|| panic!("intentional host worker failure"))
        .unwrap();
    assert_eq!(k.wait(handle, 2000), Err(WinError(ERROR_PROCESS_ABORTED)));
    assert_eq!(
        k.thread_exit_code(handle),
        Err(WinError(ERROR_PROCESS_ABORTED))
    );
    k.close_handle(handle).unwrap();
}

#[test]
fn current_ids_are_stable_and_unique_for_created_and_foreign_host_threads() {
    let parent = current_thread_id();
    assert_eq!(current_thread_id(), parent);
    let k = Kernel::default();
    let (send, receive) = mpsc::channel();
    let (handle, expected) = k
        .create_thread(move || {
            let id = current_thread_id();
            assert_eq!(current_thread_id(), id);
            send.send(id).unwrap();
            0
        })
        .unwrap();
    assert_eq!(
        receive.recv_timeout(Duration::from_secs(2)).unwrap(),
        expected
    );
    assert_ne!(expected, parent);
    let foreign = thread::spawn(|| {
        let id = current_thread_id();
        assert_eq!(current_thread_id(), id);
        id
    })
    .join()
    .unwrap();
    assert_ne!(foreign, parent);
    assert_ne!(foreign, expected);
    assert_eq!(k.wait(handle, 2000).unwrap(), WAIT_OBJECT_0);
    k.close_handle(handle).unwrap();
}

#[test]
fn wrong_type_and_cross_kernel_exit_queries_fail() {
    let a = Arc::new(Kernel::default());
    let b = Kernel::default();
    let event = a.create_event(false, true);
    assert_eq!(
        a.thread_exit_code(event),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    let (handle, _) = a.create_thread(|| 0).unwrap();
    assert_eq!(
        b.thread_exit_code(handle),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    assert_eq!(a.set_event(handle), Err(WinError(ERROR_INVALID_HANDLE)));
    assert_eq!(
        a.release_semaphore(handle, 1),
        Err(WinError(ERROR_INVALID_HANDLE))
    );
    assert_eq!(a.file_size(handle), Err(WinError(ERROR_INVALID_HANDLE)));
    assert_eq!(a.wait(handle, 2000).unwrap(), WAIT_OBJECT_0);
    a.close_handle(handle).unwrap();
    a.close_handle(event).unwrap();
}

#[repr(C)]
struct CallbackData {
    input: u32,
    observed_id: u32,
    output: u32,
}
unsafe extern "C" fn callback(parameter: *mut c_void) -> u32 {
    let data = unsafe { &mut *parameter.cast::<CallbackData>() };
    data.observed_id = ffi::GetCurrentThreadId();
    data.output = data.input.wrapping_add(1);
    data.output
}
unsafe extern "C" fn null_parameter_callback(parameter: *mut c_void) -> u32 {
    assert!(parameter.is_null());
    17
}

#[test]
fn ffi_callback_parameters_ids_exit_bits_and_optional_id_output() {
    unsafe {
        let mut data = Box::new(CallbackData {
            input: u32::MAX,
            observed_id: 0,
            output: 99,
        });
        let mut id = 0;
        ffi::SetLastError(123);
        let parent = ffi::GetCurrentThreadId();
        assert_eq!(ffi::GetCurrentThreadId(), parent);
        assert_eq!(ffi::GetLastError(), 123);
        let handle = ffi::CreateThread(
            null(),
            0,
            Some(callback),
            (&mut *data as *mut CallbackData).cast(),
            0,
            &mut id,
        );
        assert_ne!(handle, 0);
        assert_ne!(id, 0);
        assert_ne!(id, parent);
        assert_eq!(ffi::GetLastError(), 123);
        assert_eq!(ffi::WaitForSingleObject(handle, 2000), WAIT_OBJECT_0);
        assert_eq!(data.observed_id, id);
        assert_eq!(data.output, 0);
        let mut code = 99;
        assert_eq!(ffi::GetExitCodeThread(handle, &mut code), 1);
        assert_eq!(code, 0);
        assert_eq!(ffi::GetLastError(), 123);
        assert_eq!(ffi::CloseHandle(handle), 1);
        let handle = ffi::CreateThread(
            null(),
            0,
            Some(null_parameter_callback),
            null_mut(),
            0,
            null_mut(),
        );
        assert_ne!(handle, 0);
        assert_eq!(ffi::WaitForSingleObject(handle, 2000), WAIT_OBJECT_0);
        assert_eq!(ffi::GetExitCodeThread(handle, &mut code), 1);
        assert_eq!(code, 17);
        assert_eq!(ffi::CloseHandle(handle), 1);
    }
}

#[test]
fn ffi_rejects_unsupported_creation_and_invalid_query_outputs_without_writes() {
    unsafe {
        let marker = 0_u8;
        let mut id = 123;
        for (security, stack, flags, start, error) in [
            (
                (&marker as *const u8).cast(),
                0,
                0,
                Some(null_parameter_callback as ffi::ThreadStart),
                ERROR_NOT_SUPPORTED,
            ),
            (
                null(),
                4096,
                0,
                Some(null_parameter_callback as ffi::ThreadStart),
                ERROR_NOT_SUPPORTED,
            ),
            (
                null(),
                0,
                4,
                Some(null_parameter_callback as ffi::ThreadStart),
                ERROR_NOT_SUPPORTED,
            ),
            (null(), 0, 0, None, ERROR_INVALID_PARAMETER),
        ] {
            assert_eq!(
                ffi::CreateThread(security, stack, start, null_mut(), flags, &mut id),
                0
            );
            assert_eq!(ffi::GetLastError(), error);
            assert_eq!(id, 123);
        }
        assert_eq!(ffi::GetExitCodeThread(0, null_mut()), 0);
        assert_eq!(ffi::GetLastError(), ERROR_INVALID_PARAMETER);
        let mut code = 123;
        assert_eq!(ffi::GetExitCodeThread(0, &mut code), 0);
        assert_eq!(ffi::GetLastError(), ERROR_INVALID_HANDLE);
        assert_eq!(code, 123);
        let event = ffi::CreateEventW(null(), 0, 0, null());
        assert_eq!(ffi::GetExitCodeThread(event, &mut code), 0);
        assert_eq!(ffi::GetLastError(), ERROR_INVALID_HANDLE);
        assert_eq!(code, 123);
        assert_eq!(ffi::CloseHandle(event), 1);
    }
}
