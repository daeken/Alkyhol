//! Restricted Win32 kernel contracts. Handles are tokens, never host pointers.
use std::{
    cell::Cell,
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        Arc, Condvar, Mutex, OnceLock,
        atomic::{AtomicU32, AtomicUsize, Ordering},
        mpsc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub type Handle = usize;
pub const INVALID_HANDLE_VALUE: Handle = usize::MAX;
pub const ERROR_INVALID_FUNCTION: u32 = 1;
pub const ERROR_FILE_NOT_FOUND: u32 = 2;
pub const ERROR_PATH_NOT_FOUND: u32 = 3;
pub const ERROR_ACCESS_DENIED: u32 = 5;
pub const ERROR_INVALID_HANDLE: u32 = 6;
pub const ERROR_NOT_ENOUGH_MEMORY: u32 = 8;
pub const ERROR_NOT_SUPPORTED: u32 = 50;
pub const ERROR_FILE_EXISTS: u32 = 80;
pub const ERROR_INVALID_PARAMETER: u32 = 87;
pub const ERROR_NEGATIVE_SEEK: u32 = 131;
pub const ERROR_ALREADY_EXISTS: u32 = 183;
pub const ERROR_TOO_MANY_POSTS: u32 = 298;
pub const ERROR_PROCESS_ABORTED: u32 = 1067;
pub const ERROR_NO_UNICODE_TRANSLATION: u32 = 1113;
pub const GENERIC_READ: u32 = 0x8000_0000;
pub const GENERIC_WRITE: u32 = 0x4000_0000;
pub const FILE_SHARE_ALL: u32 = 7;
pub const FILE_ATTRIBUTE_NORMAL: u32 = 0x80;
pub const CREATE_NEW: u32 = 1;
pub const CREATE_ALWAYS: u32 = 2;
pub const OPEN_EXISTING: u32 = 3;
pub const OPEN_ALWAYS: u32 = 4;
pub const TRUNCATE_EXISTING: u32 = 5;
pub const WAIT_OBJECT_0: u32 = 0;
pub const WAIT_TIMEOUT: u32 = 258;
pub const WAIT_FAILED: u32 = u32::MAX;
pub const INFINITE: u32 = u32::MAX;
pub const STILL_ACTIVE: u32 = 259;

thread_local! { static LAST_ERROR: Cell<u32> = const { Cell::new(0) }; }
pub fn get_last_error() -> u32 {
    LAST_ERROR.get()
}
pub fn set_last_error(error: u32) {
    LAST_ERROR.set(error);
}

thread_local! { static THREAD_ID: Cell<u32> = const { Cell::new(0) }; }
static NEXT_THREAD_ID: AtomicU32 = AtomicU32::new(1);
#[allow(deprecated)]
fn allocate_thread_id() -> WinResult<u32> {
    NEXT_THREAD_ID
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| WinError(ERROR_NOT_ENOUGH_MEMORY))
}
/// Opaque identity within Alkyhol, not the host OS's system-wide thread ID.
pub fn current_thread_id() -> u32 {
    THREAD_ID.with(|id| {
        if id.get() == 0 {
            id.set(allocate_thread_id().expect("thread ID namespace exhausted"));
        }
        id.get()
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WinError(pub u32);
pub type WinResult<T> = Result<T, WinError>;
impl From<std::io::Error> for WinError {
    fn from(error: std::io::Error) -> Self {
        use std::io::ErrorKind::*;
        Self(match error.kind() {
            NotFound => ERROR_FILE_NOT_FOUND,
            PermissionDenied => ERROR_ACCESS_DENIED,
            AlreadyExists => ERROR_FILE_EXISTS,
            InvalidInput | InvalidData => ERROR_INVALID_PARAMETER,
            // Do not silently label unknown errors as success.
            _ => ERROR_INVALID_FUNCTION,
        })
    }
}

struct FileObject {
    file: Mutex<File>,
    read: bool,
    write: bool,
}
struct EventState {
    signaled: bool,
    generation: u64,
}
struct EventObject {
    manual: bool,
    state: Mutex<EventState>,
    changed: Condvar,
}
struct SemaphoreObject {
    count: Mutex<i32>,
    maximum: i32,
    changed: Condvar,
}
impl SemaphoreObject {
    fn wait(&self, milliseconds: u32) -> u32 {
        let deadline = (milliseconds != INFINITE)
            .then(|| Instant::now() + Duration::from_millis(milliseconds.into()));
        let mut count = self.count.lock().unwrap();
        loop {
            if *count > 0 {
                *count -= 1;
                return WAIT_OBJECT_0;
            }
            match deadline {
                None => count = self.changed.wait(count).unwrap(),
                Some(deadline) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return WAIT_TIMEOUT;
                    }
                    count = self.changed.wait_timeout(count, remaining).unwrap().0;
                }
            }
        }
    }
}
struct ThreadObject {
    outcome: Mutex<Option<WinResult<u32>>>,
    changed: Condvar,
}
impl ThreadObject {
    fn wait(&self, milliseconds: u32) -> WinResult<u32> {
        let deadline = (milliseconds != INFINITE)
            .then(|| Instant::now() + Duration::from_millis(milliseconds.into()));
        let mut outcome = self.outcome.lock().unwrap();
        loop {
            if let Some(result) = *outcome {
                return result.map(|_| WAIT_OBJECT_0);
            }
            match deadline {
                None => outcome = self.changed.wait(outcome).unwrap(),
                Some(end) => {
                    let remaining = end.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return Ok(WAIT_TIMEOUT);
                    }
                    outcome = self.changed.wait_timeout(outcome, remaining).unwrap().0;
                }
            }
        }
    }
}
enum Object {
    File(FileObject),
    Event(EventObject),
    Semaphore(SemaphoreObject),
    Thread(Arc<ThreadObject>),
}

/// A retained unnamed event reference for asynchronous completions. Closing its
/// original handle does not invalidate this owned reference or make it alias a
/// new handle. It grants signaling only, not a new public handle or wait right.
#[derive(Clone)]
pub struct EventNotifier {
    object: Arc<Object>,
}
impl EventNotifier {
    pub fn signal(&self) {
        let Object::Event(event) = &*self.object else {
            unreachable!("validated event reference")
        };
        signal_event(event);
    }
}
fn signal_event(event: &EventObject) {
    let mut state = event.state.lock().unwrap();
    if !state.signaled {
        state.signaled = true;
        state.generation = state.generation.wrapping_add(1);
        if event.manual {
            event.changed.notify_all();
        } else {
            event.changed.notify_one();
        }
    }
}

#[derive(Default)]
pub struct Kernel {
    objects: Mutex<HashMap<Handle, Arc<Object>>>,
}
// Global token allocation prevents accidental handle aliasing between Kernel instances.
static NEXT_HANDLE: AtomicUsize = AtomicUsize::new(1);
/// Reserve a unique process-local token for a companion compatibility subsystem.
/// No kernel object is created: the owning subsystem must validate its own token
/// type/lifetime. Sharing this allocator prevents aliases with kernel handles.
#[allow(deprecated)]
pub fn reserve_handle_token() -> WinResult<Handle> {
    NEXT_HANDLE
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| {
            n.checked_add(1).filter(|v| *v < i32::MAX as usize)
        })
        .map_err(|_| WinError(ERROR_NOT_ENOUGH_MEMORY))
}
impl Kernel {
    #[allow(deprecated)] // fetch_update remains available on older Rust releases.
    fn insert(&self, object: Object) -> Handle {
        let handle = reserve_handle_token().expect("handle namespace exhausted");
        self.objects
            .lock()
            .unwrap()
            .insert(handle, Arc::new(object));
        handle
    }
    fn object(&self, handle: Handle) -> WinResult<Arc<Object>> {
        self.objects
            .lock()
            .unwrap()
            .get(&handle)
            .cloned()
            .ok_or(WinError(ERROR_INVALID_HANDLE))
    }
    pub fn close_handle(&self, handle: Handle) -> WinResult<()> {
        self.objects
            .lock()
            .unwrap()
            .remove(&handle)
            .map(|_| ())
            .ok_or(WinError(ERROR_INVALID_HANDLE))
    }
    /// Host paths, synchronous disk files, unrestricted sharing only. See docs/contracts.md.
    pub fn create_file(
        &self,
        path: &Path,
        access: u32,
        share: u32,
        disposition: u32,
        flags: u32,
    ) -> WinResult<Handle> {
        if access == 0 || access & !(GENERIC_READ | GENERIC_WRITE) != 0 {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        if share != FILE_SHARE_ALL || flags != FILE_ATTRIBUTE_NORMAL {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        if !(CREATE_NEW..=TRUNCATE_EXISTING).contains(&disposition) {
            return Err(WinError(ERROR_INVALID_PARAMETER));
        }
        let read = access & GENERIC_READ != 0;
        let write = access & GENERIC_WRITE != 0;
        if !write && disposition != OPEN_EXISTING {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        let mut options = OpenOptions::new();
        options.read(read).write(write);
        let (file, existed) = match disposition {
            CREATE_NEW => (options.create_new(true).open(path)?, false),
            CREATE_ALWAYS | OPEN_ALWAYS => match options.clone().create_new(true).open(path) {
                Ok(file) => (file, false),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (
                    options.truncate(disposition == CREATE_ALWAYS).open(path)?,
                    true,
                ),
                Err(e) => return Err(e.into()),
            },
            OPEN_EXISTING => (options.open(path)?, true),
            TRUNCATE_EXISTING => (options.truncate(true).open(path)?, true),
            _ => unreachable!(),
        };
        if !file.metadata()?.is_file() {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        if disposition == CREATE_ALWAYS || disposition == OPEN_ALWAYS {
            set_last_error(if existed { ERROR_ALREADY_EXISTS } else { 0 });
        }
        Ok(self.insert(Object::File(FileObject {
            file: Mutex::new(file),
            read,
            write,
        })))
    }
    /// Adopt an already-open regular file, restricting its interface permissions.
    /// The host descriptor continues enforcing its own access rights as well.
    pub fn adopt_file(&self, file: File, access: u32) -> WinResult<Handle> {
        if access & !(GENERIC_READ | GENERIC_WRITE) != 0 || !file.metadata()?.is_file() {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        Ok(self.insert(Object::File(FileObject {
            file: Mutex::new(file),
            read: access & GENERIC_READ != 0,
            write: access & GENERIC_WRITE != 0,
        })))
    }
    pub fn read_file(&self, handle: Handle, bytes: &mut [u8]) -> WinResult<usize> {
        let object = self.object(handle)?;
        let Object::File(file) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        if !file.read {
            return Err(WinError(ERROR_ACCESS_DENIED));
        }
        Ok(file.file.lock().unwrap().read(bytes)?)
    }
    pub fn write_file(&self, handle: Handle, bytes: &[u8]) -> WinResult<usize> {
        let object = self.object(handle)?;
        let Object::File(file) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        if !file.write {
            return Err(WinError(ERROR_ACCESS_DENIED));
        }
        Ok(file.file.lock().unwrap().write(bytes)?)
    }
    pub fn file_size(&self, handle: Handle) -> WinResult<u64> {
        Ok(self.file_metadata(handle)?.len())
    }
    /// Live metadata of the owned file descriptor, including after seeking.
    pub fn file_metadata(&self, handle: Handle) -> WinResult<std::fs::Metadata> {
        let object = self.object(handle)?;
        let Object::File(file) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        Ok(file.file.lock().unwrap().metadata()?)
    }
    pub fn seek(&self, handle: Handle, distance: i64, method: u32) -> WinResult<u64> {
        let object = self.object(handle)?;
        let Object::File(file) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        let mut file = file.file.lock().unwrap();
        // Compute the target explicitly so negative seeks have Windows' specific error.
        let base = match method {
            0 => 0_i128,
            1 => file.stream_position()? as i128,
            2 => file.metadata()?.len() as i128,
            _ => return Err(WinError(ERROR_INVALID_PARAMETER)),
        };
        let target = base + i128::from(distance);
        if target < 0 {
            return Err(WinError(ERROR_NEGATIVE_SEEK));
        }
        if target > i64::MAX as i128 {
            return Err(WinError(ERROR_INVALID_PARAMETER));
        }
        Ok(file.seek(SeekFrom::Start(target as u64))?)
    }
    pub fn set_end_of_file(&self, handle: Handle) -> WinResult<()> {
        let object = self.object(handle)?;
        let Object::File(file) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        if !file.write {
            return Err(WinError(ERROR_ACCESS_DENIED));
        }
        let mut file = file.file.lock().unwrap();
        let position = file.stream_position()?;
        Ok(file.set_len(position)?)
    }
    pub fn flush_file(&self, handle: Handle) -> WinResult<()> {
        let object = self.object(handle)?;
        let Object::File(file) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        if !file.write {
            return Err(WinError(ERROR_ACCESS_DENIED));
        }
        Ok(file.file.lock().unwrap().sync_all()?)
    }
    pub fn create_event(&self, manual: bool, initial: bool) -> Handle {
        self.insert(Object::Event(EventObject {
            manual,
            state: Mutex::new(EventState {
                signaled: initial,
                generation: 0,
            }),
            changed: Condvar::new(),
        }))
    }
    /// Retain a validated event for asynchronous notification, independently of
    /// the lifetime of the public handle. Files, threads and semaphores fail.
    pub fn event_notifier(&self, handle: Handle) -> WinResult<EventNotifier> {
        let object = self.object(handle)?;
        if !matches!(&*object, Object::Event(_)) {
            return Err(WinError(ERROR_INVALID_HANDLE));
        }
        Ok(EventNotifier { object })
    }
    pub fn set_event(&self, handle: Handle) -> WinResult<()> {
        self.event_notifier(handle)?.signal();
        Ok(())
    }
    pub fn reset_event(&self, handle: Handle) -> WinResult<()> {
        let object = self.object(handle)?;
        let Object::Event(event) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        event.state.lock().unwrap().signaled = false;
        Ok(())
    }
    /// Unnamed, process-local counting semaphore. No access-mask or namespace support.
    pub fn create_semaphore(&self, initial: i32, maximum: i32) -> WinResult<Handle> {
        if maximum <= 0 || initial < 0 || initial > maximum {
            return Err(WinError(ERROR_INVALID_PARAMETER));
        }
        Ok(self.insert(Object::Semaphore(SemaphoreObject {
            count: Mutex::new(initial),
            maximum,
            changed: Condvar::new(),
        })))
    }
    /// Atomically add permits and return the prior count. Failure leaves it unchanged.
    pub fn release_semaphore(&self, handle: Handle, release: i32) -> WinResult<i32> {
        let object = self.object(handle)?;
        let Object::Semaphore(semaphore) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        if release <= 0 {
            return Err(WinError(ERROR_INVALID_PARAMETER));
        }
        let mut count = semaphore.count.lock().unwrap();
        // Both operands are within [0, maximum]; no signed addition can overflow.
        if release > semaphore.maximum - *count {
            return Err(WinError(ERROR_TOO_MANY_POSTS));
        }
        let previous = *count;
        *count += release;
        // Wake every waiter to compete for the new permits. Ordering is unspecified.
        semaphore.changed.notify_all();
        Ok(previous)
    }
    /// Start a native host worker. Default host stack, normal callback return only.
    pub fn create_thread(
        &self,
        start: impl FnOnce() -> u32 + Send + 'static,
    ) -> WinResult<(Handle, u32)> {
        let id = allocate_thread_id()?;
        let completion = Arc::new(ThreadObject {
            outcome: Mutex::new(None),
            changed: Condvar::new(),
        });
        let monitor = completion.clone();
        let (sender, receiver) = mpsc::channel::<JoinHandle<u32>>();
        // Start the monitor before the user worker. Failure to create either host
        // thread cannot report a successful CreateThread with no completion path.
        // If worker creation fails, dropping sender lets the monitor exit quietly.
        let reaper = thread::Builder::new().spawn(move || {
            if let Ok(worker) = receiver.recv() {
                let result = worker.join().map_err(|_| WinError(ERROR_PROCESS_ABORTED));
                *monitor.outcome.lock().unwrap() = Some(result);
                monitor.changed.notify_all();
            }
        })?;
        drop(reaper); // Internal monitor owns its completion state until join finishes.
        let worker = thread::Builder::new().spawn(move || {
            THREAD_ID.with(|current| current.set(id));
            start()
        })?;
        sender
            .send(worker)
            .expect("thread completion monitor ended unexpectedly");
        let handle = self.insert(Object::Thread(completion));
        Ok((handle, id))
    }
    pub fn thread_exit_code(&self, handle: Handle) -> WinResult<u32> {
        let object = self.object(handle)?;
        let Object::Thread(worker) = &*object else {
            return Err(WinError(ERROR_INVALID_HANDLE));
        };
        Ok(worker
            .outcome
            .lock()
            .unwrap()
            .transpose()?
            .unwrap_or(STILL_ACTIVE))
    }
    pub fn wait(&self, handle: Handle, milliseconds: u32) -> WinResult<u32> {
        let object = self.object(handle)?;
        if let Object::Semaphore(semaphore) = &*object {
            return Ok(semaphore.wait(milliseconds));
        }
        if let Object::Thread(worker) = &*object {
            return worker.wait(milliseconds);
        }
        let Object::Event(event) = &*object else {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        };
        let deadline = (milliseconds != INFINITE)
            .then(|| Instant::now() + Duration::from_millis(milliseconds.into()));
        let mut state = event.state.lock().unwrap();
        let generation = state.generation;
        loop {
            if state.signaled || (event.manual && state.generation != generation) {
                if !event.manual {
                    state.signaled = false;
                }
                return Ok(WAIT_OBJECT_0);
            }
            match deadline {
                None => state = event.changed.wait(state).unwrap(),
                Some(deadline) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return Ok(WAIT_TIMEOUT);
                    }
                    state = event.changed.wait_timeout(state, remaining).unwrap().0;
                }
            }
        }
    }
}

pub fn kernel() -> &'static Kernel {
    static KERNEL: OnceLock<Kernel> = OnceLock::new();
    KERNEL.get_or_init(Kernel::default)
}
pub const PERFORMANCE_FREQUENCY: i64 = 1_000_000_000;
pub fn performance_counter() -> i64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_nanos() as i64
}
pub fn sleep(milliseconds: u32) {
    if milliseconds == 0 {
        std::thread::yield_now();
    } else {
        std::thread::sleep(Duration::from_millis(milliseconds.into()));
    }
}
