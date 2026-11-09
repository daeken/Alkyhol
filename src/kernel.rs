//! Restricted Win32 kernel contracts. Handles are tokens, never host pointers.
use std::{
    cell::Cell,
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{
        Arc, Condvar, Mutex, OnceLock, Weak,
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
#[derive(Clone)]
struct EventWaitLink {
    waiter: Weak<EventWaitRecord>,
    index: u32,
}
struct EventWaitRecord {
    all: bool,
    members: Vec<Weak<Object>>,
    result: Mutex<Option<u32>>,
    changed: Condvar,
}
struct EventState {
    signaled: bool,
    waiters: Vec<EventWaitLink>,
}
struct EventObject {
    dispatcher: Arc<Mutex<()>>,
    manual: bool,
    state: Mutex<EventState>,
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
    // A single dispatcher orders set/reset, wait registration and timeouts.
    // Wait records are completed at the signal, not by a later polling snapshot.
    let _dispatcher = event.dispatcher.lock().unwrap();
    let links = {
        let mut state = event.state.lock().unwrap();
        if state.signaled {
            return;
        }
        state.signaled = true;
        state.waiters.clone()
    };
    for link in links {
        let Some(waiter) = link.waiter.upgrade() else {
            continue;
        };
        let mut result = waiter.result.lock().unwrap();
        if result.is_some() {
            continue;
        }
        let members: Vec<_> = waiter
            .members
            .iter()
            .map(|member| {
                member
                    .upgrade()
                    .expect("registered wait retains its event objects")
            })
            .collect();
        let satisfied = if waiter.all {
            members.iter().all(|object| {
                let Object::Event(member) = &**object else {
                    unreachable!()
                };
                member.state.lock().unwrap().signaled
            })
        } else {
            event.state.lock().unwrap().signaled
        };
        if satisfied {
            if waiter.all {
                for object in &members {
                    let Object::Event(member) = &**object else {
                        unreachable!()
                    };
                    if !member.manual {
                        member.state.lock().unwrap().signaled = false;
                    }
                }
            } else if !event.manual {
                event.state.lock().unwrap().signaled = false;
            }
            *result = Some(if waiter.all { 0 } else { link.index });
            waiter.changed.notify_one();
        }
    }
}

#[derive(Default)]
pub struct Kernel {
    objects: Mutex<HashMap<Handle, Arc<Object>>>,
    event_dispatcher: Arc<Mutex<()>>,
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
    /// Add an independently owned handle to the same live kernel object.
    pub fn duplicate_handle(&self, handle: Handle) -> WinResult<Handle> {
        let object = self.object(handle)?;
        let duplicate = reserve_handle_token()?;
        self.objects.lock().unwrap().insert(duplicate, object);
        Ok(duplicate)
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
            dispatcher: self.event_dispatcher.clone(),
            manual,
            state: Mutex::new(EventState {
                signaled: initial,
                waiters: Vec::new(),
            }),
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
        let _dispatcher = event.dispatcher.lock().unwrap();
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
    /// Atomic any/all waits on 1..64 events. A successful wait consumes only
    /// satisfied automatic events; manual states are retained. Other object
    /// types and APC delivery are separate. All-wait duplicate handles fail.
    pub fn wait_events(&self, handles: &[Handle], all: bool, milliseconds: u32) -> WinResult<u32> {
        self.wait_events_inner(handles, all, milliseconds, None)
    }
    /// Keep one atomic wait registered while servicing calling-thread host tasks.
    /// The callback runs without dispatcher/event/result locks between millisecond
    /// wait slices. Signaling during a callback is still latched.
    pub fn wait_events_with_tasks(
        &self,
        handles: &[Handle],
        all: bool,
        milliseconds: u32,
        tasks: &mut dyn FnMut(),
    ) -> WinResult<u32> {
        self.wait_events_inner(handles, all, milliseconds, Some(tasks))
    }
    fn wait_events_inner(
        &self,
        handles: &[Handle],
        all: bool,
        milliseconds: u32,
        tasks: Option<&mut dyn FnMut()>,
    ) -> WinResult<u32> {
        if handles.is_empty() || handles.len() > 64 {
            return Err(WinError(ERROR_INVALID_PARAMETER));
        }
        let mut unique = std::collections::HashSet::new();
        if all && handles.iter().any(|&h| !unique.insert(h)) {
            return Err(WinError(ERROR_INVALID_PARAMETER));
        }
        let objects: Vec<_> = handles
            .iter()
            .map(|&h| self.object(h))
            .collect::<WinResult<_>>()?;
        self.wait_event_objects(objects, all, milliseconds, tasks)
    }
    fn wait_event_objects(
        &self,
        objects: Vec<Arc<Object>>,
        all: bool,
        milliseconds: u32,
        mut tasks: Option<&mut dyn FnMut()>,
    ) -> WinResult<u32> {
        let events: Vec<_> = objects
            .iter()
            .map(|object| match &**object {
                Object::Event(event) => Ok(event),
                _ => Err(WinError(ERROR_NOT_SUPPORTED)),
            })
            .collect::<WinResult<_>>()?;
        // Different handles aliasing one automatic event in wait-all need a
        // separate native contract. Do not count one signal twice implicitly.
        if all {
            for (index, object) in objects.iter().enumerate() {
                if !events[index].manual
                    && objects[..index]
                        .iter()
                        .any(|prior| Arc::ptr_eq(prior, object))
                {
                    return Err(WinError(ERROR_NOT_SUPPORTED));
                }
            }
        }
        let deadline = (milliseconds != INFINITE)
            .then(|| Instant::now() + Duration::from_millis(milliseconds.into()));
        let waiter = {
            let _dispatcher = self.event_dispatcher.lock().unwrap();
            let states: Vec<_> = events
                .iter()
                .map(|event| event.state.lock().unwrap().signaled)
                .collect();
            let ready = if all {
                states.iter().all(|&state| state).then_some(0)
            } else {
                states
                    .iter()
                    .position(|&state| state)
                    .map(|index| index as u32)
            };
            if let Some(index) = ready {
                for (position, event) in events.iter().enumerate() {
                    if !event.manual && (all || position == index as usize) {
                        event.state.lock().unwrap().signaled = false;
                    }
                }
                return Ok(index);
            }
            if milliseconds == 0 {
                return Ok(WAIT_TIMEOUT);
            }
            let waiter = Arc::new(EventWaitRecord {
                all,
                members: objects.iter().map(Arc::downgrade).collect(),
                result: Mutex::new(None),
                changed: Condvar::new(),
            });
            for (index, event) in events.iter().enumerate() {
                event.state.lock().unwrap().waiters.push(EventWaitLink {
                    waiter: Arc::downgrade(&waiter),
                    index: index as u32,
                });
            }
            waiter
        };
        let result = loop {
            if let Some(tasks) = tasks.as_mut() {
                tasks();
            }
            let state = waiter.result.lock().unwrap();
            if let Some(result) = *state {
                break result;
            }
            if let Some(deadline) = deadline {
                let remaining = deadline.saturating_duration_since(Instant::now());
                if remaining.is_zero() {
                    drop(state);
                    let _dispatcher = self.event_dispatcher.lock().unwrap();
                    let mut state = waiter.result.lock().unwrap();
                    break *state.get_or_insert(WAIT_TIMEOUT);
                }
                let slice = if tasks.is_some() {
                    remaining.min(Duration::from_millis(1))
                } else {
                    remaining
                };
                drop(waiter.changed.wait_timeout(state, slice).unwrap());
            } else if tasks.is_some() {
                drop(
                    waiter
                        .changed
                        .wait_timeout(state, Duration::from_millis(1))
                        .unwrap(),
                );
            } else {
                drop(waiter.changed.wait(state).unwrap());
            }
        };
        // Remove all links, including duplicate wait-any members, before the
        // record dies. Timeout loops cannot accumulate dormant registrations.
        let _dispatcher = self.event_dispatcher.lock().unwrap();
        for event in events {
            event
                .state
                .lock()
                .unwrap()
                .waiters
                .retain(|link| link.waiter.as_ptr() != Arc::as_ptr(&waiter));
        }
        Ok(result)
    }
    pub fn wait(&self, handle: Handle, milliseconds: u32) -> WinResult<u32> {
        let object = self.object(handle)?;
        if let Object::Semaphore(semaphore) = &*object {
            return Ok(semaphore.wait(milliseconds));
        }
        if let Object::Thread(worker) = &*object {
            return worker.wait(milliseconds);
        }
        if !matches!(&*object, Object::Event(_)) {
            return Err(WinError(ERROR_NOT_SUPPORTED));
        }
        self.wait_event_objects(vec![object], false, milliseconds, None)
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

#[cfg(test)]
mod event_set_tests {
    use super::*;
    fn links(kernel: &Kernel, handle: Handle) -> usize {
        let object = kernel.object(handle).unwrap();
        let Object::Event(event) = &*object else {
            panic!("event expected")
        };
        event.state.lock().unwrap().waiters.len()
    }
    fn registered(kernel: &Kernel, handle: Handle) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while links(kernel, handle) == 0 {
            assert!(Instant::now() < deadline, "wait did not register");
            thread::yield_now();
        }
    }
    #[test]
    fn manual_event_sets_keep_index_priority_all_state_and_timeout_cleanup() {
        let kernel = Kernel::default();
        let a = kernel.create_event(true, false);
        let b = kernel.create_event(true, false);
        assert_eq!(kernel.wait_events(&[a, b], false, 0).unwrap(), WAIT_TIMEOUT);
        kernel.set_event(b).unwrap();
        assert_eq!(kernel.wait_events(&[a, b], false, 0).unwrap(), 1);
        kernel.set_event(a).unwrap();
        assert_eq!(kernel.wait_events(&[a, b], false, 0).unwrap(), 0);
        assert_eq!(kernel.wait_events(&[a, b], true, 0).unwrap(), 0);
        kernel.reset_event(a).unwrap();
        assert_eq!(kernel.wait_events(&[a, b], true, 2).unwrap(), WAIT_TIMEOUT);
        assert_eq!(links(&kernel, a), 0);
        assert_eq!(links(&kernel, b), 0);
        assert_eq!(kernel.wait(b, 0).unwrap(), 0);
        assert_eq!(kernel.wait_events(&[b, b], false, 0).unwrap(), 0);
        assert_eq!(
            kernel.wait_events(&[b, b], true, 0).unwrap_err().0,
            ERROR_INVALID_PARAMETER
        );
        assert_eq!(
            kernel.wait_events(&[], false, 0).unwrap_err().0,
            ERROR_INVALID_PARAMETER
        );
        assert_eq!(
            kernel.wait_events(&[a; 65], false, 0).unwrap_err().0,
            ERROR_INVALID_PARAMETER
        );
        let semaphore = kernel.create_semaphore(1, 1).unwrap();
        assert_eq!(
            kernel.wait_events(&[a, semaphore], false, 0).unwrap_err().0,
            ERROR_NOT_SUPPORTED
        );
        assert_eq!(kernel.wait(semaphore, 0).unwrap(), 0);
        kernel.close_handle(a).unwrap();
        assert_eq!(
            kernel.wait_events(&[a], false, 0).unwrap_err().0,
            ERROR_INVALID_HANDLE
        );
    }
    #[test]
    fn automatic_sets_consume_only_satisfied_signals_and_keep_manual_members() {
        let kernel = Kernel::default();
        let a = kernel.create_event(false, false);
        let b = kernel.create_event(false, true);
        let manual = kernel.create_event(true, true);
        assert_eq!(kernel.wait_events(&[a, b], false, 0).unwrap(), 1);
        assert_eq!(kernel.wait_events(&[a, b], false, 0).unwrap(), WAIT_TIMEOUT);
        kernel.set_event(a).unwrap();
        kernel.set_event(a).unwrap();
        assert_eq!(kernel.wait_events(&[a, b], true, 0).unwrap(), WAIT_TIMEOUT);
        assert_eq!(kernel.wait(a, 0).unwrap(), 0);
        assert_eq!(kernel.wait(a, 0).unwrap(), WAIT_TIMEOUT);
        kernel.set_event(a).unwrap();
        kernel.set_event(b).unwrap();
        assert_eq!(kernel.wait_events(&[a, b, manual], true, 0).unwrap(), 0);
        assert_eq!(kernel.wait(a, 0).unwrap(), WAIT_TIMEOUT);
        assert_eq!(kernel.wait(b, 0).unwrap(), WAIT_TIMEOUT);
        assert_eq!(kernel.wait(manual, 0).unwrap(), 0);
        let alias = kernel.duplicate_handle(a).unwrap();
        kernel.set_event(a).unwrap();
        assert_eq!(
            kernel.wait_events(&[a, alias], true, 0).unwrap_err().0,
            ERROR_NOT_SUPPORTED
        );
        assert_eq!(kernel.wait(a, 0).unwrap(), 0);
    }
    #[test]
    fn two_registered_waiters_cannot_spend_one_automatic_signal_twice() {
        let kernel = Arc::new(Kernel::default());
        let auto = kernel.create_event(false, false);
        let mut workers = Vec::new();
        for _ in 0..2 {
            let kernel = kernel.clone();
            workers.push(thread::spawn(move || kernel.wait(auto, 5000).unwrap()));
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while links(&kernel, auto) != 2 {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        kernel.set_event(auto).unwrap();
        assert_eq!(kernel.wait(auto, 0).unwrap(), WAIT_TIMEOUT);
        // A second real set releases the other wait; the first cannot release it.
        kernel.set_event(auto).unwrap();
        for worker in workers {
            assert_eq!(worker.join().unwrap(), 0);
        }
        assert_eq!(kernel.wait(auto, 0).unwrap(), WAIT_TIMEOUT);
        assert_eq!(links(&kernel, auto), 0);
    }
    #[test]
    fn incomplete_wait_all_does_not_reserve_an_automatic_signal_from_single_waits() {
        let kernel = Arc::new(Kernel::default());
        let manual = kernel.create_event(true, false);
        let auto = kernel.create_event(false, false);
        let all_kernel = kernel.clone();
        let all =
            thread::spawn(move || all_kernel.wait_events(&[manual, auto], true, 5000).unwrap());
        registered(&kernel, manual);
        let single_kernel = kernel.clone();
        let single = thread::spawn(move || single_kernel.wait(auto, 5000).unwrap());
        let deadline = Instant::now() + Duration::from_secs(5);
        while links(&kernel, auto) != 2 {
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        kernel.set_event(auto).unwrap();
        assert_eq!(single.join().unwrap(), 0);
        kernel.set_event(manual).unwrap();
        assert_eq!(kernel.wait(auto, 0).unwrap(), WAIT_TIMEOUT);
        kernel.set_event(auto).unwrap();
        assert_eq!(all.join().unwrap(), 0);
        assert_eq!(kernel.wait(auto, 0).unwrap(), WAIT_TIMEOUT);
        assert_eq!(kernel.wait(manual, 0).unwrap(), 0);
    }
    #[test]
    fn host_tasks_preserve_one_registration_and_latched_signals() {
        let kernel = Kernel::default();
        let a = kernel.create_event(true, false);
        let b = kernel.create_event(true, false);
        let mut calls = 0;
        let result = kernel
            .wait_events_with_tasks(&[a, b], false, 5000, &mut || {
                calls += 1;
                assert_eq!(links(&kernel, a), 1);
                if calls == 2 {
                    kernel.set_event(b).unwrap();
                    kernel.set_event(a).unwrap();
                    kernel.reset_event(b).unwrap();
                    kernel.reset_event(a).unwrap();
                }
            })
            .unwrap();
        assert_eq!(calls, 2);
        assert_eq!(result, 1);
        assert_eq!(links(&kernel, a), 0);
        assert_eq!(links(&kernel, b), 0);
    }
    #[test]
    fn a_registered_wait_any_latches_the_first_signal_not_a_later_snapshot() {
        let kernel = Arc::new(Kernel::default());
        let a = kernel.create_event(true, false);
        let b = kernel.create_event(true, false);
        let worker_kernel = kernel.clone();
        let worker =
            thread::spawn(move || worker_kernel.wait_events(&[a, b], false, 5000).unwrap());
        registered(&kernel, a);
        kernel.set_event(b).unwrap();
        kernel.set_event(a).unwrap();
        kernel.reset_event(b).unwrap();
        assert_eq!(worker.join().unwrap(), 1);
        assert_eq!(links(&kernel, a), 0);
        assert_eq!(links(&kernel, b), 0);
    }
    #[test]
    fn a_registered_wait_all_latches_overlap_but_not_disjoint_pulses() {
        let kernel = Arc::new(Kernel::default());
        let a = kernel.create_event(true, false);
        let b = kernel.create_event(true, false);
        let worker_kernel = kernel.clone();
        let worker = thread::spawn(move || worker_kernel.wait_events(&[a, b], true, 5000).unwrap());
        registered(&kernel, a);
        kernel.set_event(a).unwrap();
        kernel.reset_event(a).unwrap();
        kernel.set_event(b).unwrap();
        assert!(links(&kernel, a) > 0); // A was reset before B became signaled.
        kernel.set_event(a).unwrap();
        kernel.reset_event(a).unwrap();
        kernel.reset_event(b).unwrap();
        assert_eq!(worker.join().unwrap(), 0); // Both were ready at the set operation.
        assert_eq!(links(&kernel, a), 0);
        assert_eq!(links(&kernel, b), 0);
    }
}
