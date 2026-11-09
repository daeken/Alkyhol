//! Native host threads with distinct guest stacks/TEBs and serialized CPU entries.
//! Guest execution yields at API boundaries. Windows CPU affinity/preemption and
//! arbitrary asynchronous thread suspension are not implemented.
use super::{
    CallingConvention32, Cpu32, LoadedImage32, LowMapping, PAGE, STACK, TebLease, unsupported,
};
use crate::Result;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    ffi::c_void,
    ptr,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicU32, Ordering},
    },
    thread::JoinHandle,
};
thread_local! {
    static CURRENT: Cell<*const Record> = const { Cell::new(ptr::null()) };
    static LEASE: RefCell<Option<TebLease>> = const { RefCell::new(None) };
    static QUANTUM: RefCell<Option<(u32,std::time::Instant)>> = const { RefCell::new(None) };
}
struct Record {
    cpu: *const Cpu32,
    stack: LowMapping,
    teb: LowMapping,
    tls: Option<LowMapping>,
    tls_size: usize,
    host_fs: Cell<u16>,
    callback_stack: Cell<u32>,
    callback_depth: Cell<u32>,
    requested_exit: Cell<Option<u32>>,
    control: Arc<Control>,
    closed: bool,
}
struct Control {
    id: AtomicU32,
    exit: AtomicU32,
    done: AtomicU32,
    suspended: Mutex<u32>,
    ready: Condvar,
    event: u32,
    retained: u32,
}
#[derive(Default)]
pub(super) struct State {
    records: BTreeMap<u32, Box<Record>>,
    joins: Vec<JoinHandle<()>>,
    notification_owner: Option<u32>,
}
impl State {
    pub(super) fn has_workers(&self) -> bool {
        !self.records.is_empty()
    }
    pub(super) fn contains(&self, h: u32) -> bool {
        self.records.get(&h).is_some_and(|r| !r.closed)
    }
    pub(super) fn require_completed(&self) {
        if self
            .records
            .values()
            .any(|r| r.control.done.load(Ordering::Acquire) == 0)
        {
            unsupported(
                "return/termination with live guest threads requires process-wide termination support",
            );
        }
    }
    pub(super) fn shutdown(&mut self) {
        self.require_completed();
        for join in self.joins.drain(..) {
            if join.join().is_err() {
                unsupported("native guest worker panicked");
            }
        }
        for r in self.records.values() {
            if !r.closed {
                alkyhol::ffi::CloseHandle(r.control.event as usize);
            }
            alkyhol::ffi::CloseHandle(r.control.retained as usize);
        }
        self.records.clear();
    }
    pub(super) fn close(&mut self, h: u32) -> bool {
        let Some(r) = self.records.get_mut(&h).filter(|r| !r.closed) else {
            return false;
        };
        if alkyhol::ffi::CloseHandle(h as usize) == 0 {
            unsupported("thread completion handle backing disappeared");
        }
        r.closed = true;
        true
    }
    pub(super) fn permits(&self, at: usize, len: usize) -> bool {
        at.checked_add(len).is_some_and(|end| {
            self.records.values().any(|r| {
                let first = r.stack.base as usize + PAGE;
                (at >= first && end <= first + STACK)
                    || (at >= r.teb.base as usize && end <= r.teb.base as usize + PAGE)
                    || r.tls.as_ref().is_some_and(|t| {
                        at >= t.base as usize && end <= t.base as usize + r.tls_size
                    })
            })
        })
    }
}
fn current(cpu: &Cpu32) -> Option<&Record> {
    CURRENT
        .with(|p| unsafe { p.get().as_ref() })
        .filter(|r| std::ptr::eq(r.cpu, cpu))
}
pub(super) fn stack(cpu: &Cpu32) -> Option<&LowMapping> {
    current(cpu).map(|r| &r.stack)
}
pub(super) fn teb(cpu: &Cpu32) -> Option<&LowMapping> {
    current(cpu).map(|r| &r.teb)
}
pub(super) fn callback_stack(cpu: &Cpu32) -> Option<&Cell<u32>> {
    current(cpu).map(|r| &r.callback_stack)
}
pub(super) fn callback_depth(cpu: &Cpu32) -> Option<&Cell<u32>> {
    current(cpu).map(|r| &r.callback_depth)
}
pub(super) fn exit_state() -> Option<u32> {
    CURRENT
        .with(|p| unsafe { p.get().as_ref() })
        .and_then(|r| r.requested_exit.get())
}
pub(super) struct MainLease;
impl MainLease {
    pub(super) fn install(teb: u32) -> Result<Self> {
        let lease = TebLease::install(teb)?;
        LEASE.with(|slot| {
            if slot.borrow().is_some() {
                return Err::<(), String>("nested native CPU lease".into());
            }
            *slot.borrow_mut() = Some(lease);
            Ok(())
        })?;
        Ok(Self)
    }
}
impl Drop for MainLease {
    fn drop(&mut self) {
        LEASE.with(|slot| drop(slot.borrow_mut().take()));
    }
}
// All process RefCell borrows must end before this function. The closure may
// touch only independently synchronized host objects, never shared guest state.
pub(super) fn without_cpu<T>(image: &LoadedImage32<'_>, call: impl FnOnce() -> T) -> T {
    let teb = image.cpu.teb().base as u32;
    let saved = alkyhol::ffi::GetLastError();
    let lease = LEASE
        .with(|slot| slot.borrow_mut().take())
        .unwrap_or_else(|| unsupported("guest wait without an exclusive CPU lease"));
    drop(lease);
    super::window_host32::pump_tasks();
    let value = call();
    super::window_host32::pump_tasks();
    let lease = TebLease::acquire(teb, true).unwrap_or_else(|e| unsupported(&e));
    LEASE.with(|slot| *slot.borrow_mut() = Some(lease));
    QUANTUM.with(|q| *q.borrow_mut() = Some((0, std::time::Instant::now())));
    alkyhol::ffi::SetLastError(saved);
    value
}
pub(super) fn yield_cpu() {
    super::ACTIVE.with(|slot| {
        let Some(image) = (unsafe { slot.get().cast::<LoadedImage32<'_>>().as_ref() }) else {
            return;
        };
        let running = image
            .cpu
            .threads
            .borrow()
            .records
            .values()
            .any(|r| r.control.done.load(Ordering::Acquire) == 0);
        if running {
            let due = QUANTUM.with(|q| {
                let mut q = q.borrow_mut();
                let (count, start) = q.get_or_insert_with(|| (0, std::time::Instant::now()));
                *count += 1;
                *count >= 64 || start.elapsed() >= std::time::Duration::from_millis(2)
            });
            if due {
                without_cpu(image, std::thread::yield_now);
            }
        }
    });
}
fn fail(error: u32) -> u32 {
    alkyhol::ffi::SetLastError(error);
    0
}
fn output(image: &LoadedImage32<'_>, at: u32, value: u32) {
    if !at.is_multiple_of(4) || !image.permits(at as usize, 4, true) {
        unsupported("thread result outside checked writable guest storage");
    }
    unsafe {
        (at as *mut u32).write(value);
    }
}
pub(super) fn create(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[0] != 0 || a[1] > STACK as u32 || a[4] & !4 != 0 {
        unsupported("thread security/stack/creation flags outside the owned synchronous profile");
    }
    if !image
        .code_regions
        .iter()
        .any(|&(first, last)| a[2] as usize >= first && (a[2] as usize) < last)
    {
        unsupported("thread entry outside executable guest storage");
    }
    if image.cpu.threads.borrow().records.len() >= 128 {
        return fail(8);
    }
    if a[5] != 0 && (!a[5].is_multiple_of(4) || !image.permits(a[5] as usize, 4, true)) {
        unsupported("thread ID output outside writable guest storage");
    }
    let stack = image
        .cpu
        .mapping(STACK + 2 * PAGE, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    let teb = image
        .cpu
        .mapping(PAGE, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    let bytes = super::teb_bytes(
        teb.base as u32,
        stack.base as u32 + PAGE as u32,
        STACK as u32,
    )
    .unwrap_or_else(|e| unsupported(&e));
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), teb.base, PAGE);
    }
    stack
        .protect(0, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    stack
        .protect(PAGE + STACK, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    let h = unsafe { alkyhol::ffi::CreateEventW(ptr::null(), 1, 0, ptr::null()) };
    if h == 0 || h > u32::MAX as usize {
        return fail(8);
    }
    // Keep an independent completion reference: CloseHandle is allowed while
    // the worker still runs. The caller's event handle is never the owner.
    let retained = alkyhol::kernel::kernel()
        .duplicate_handle(h)
        .unwrap_or_else(|_| unsupported("thread completion reference allocation failed"));
    let control = Arc::new(Control {
        id: AtomicU32::new(0),
        exit: AtomicU32::new(259),
        done: AtomicU32::new(0),
        suspended: Mutex::new(if a[4] & 4 != 0 { 1 } else { 0 }),
        ready: Condvar::new(),
        event: h as u32,
        retained: retained as u32,
    });
    let tls = image.tls.as_ref().map(|original| {
        let memory = image
            .cpu
            .mapping(original.mapping.len, original.alignment, 0)
            .unwrap_or_else(|e| unsupported(&e));
        if original.initial.len() != original.size {
            unsupported("static TLS template snapshot unavailable");
        }
        unsafe {
            ptr::copy_nonoverlapping(original.initial.as_ptr(), memory.base, original.size);
            teb.base
                .add(0x2c)
                .cast::<u32>()
                .write(teb.base as u32 + 0x100);
            teb.base.add(0x100).cast::<u32>().write(memory.base as u32);
        }
        memory
    });
    let record = Box::new(Record {
        cpu: RcPtr::cpu(image),
        stack,
        teb,
        tls,
        tls_size: image.tls.as_ref().map_or(0, |t| t.size),
        host_fs: Cell::new(0),
        callback_stack: Cell::new(0),
        callback_depth: Cell::new(0),
        requested_exit: Cell::new(None),
        control: control.clone(),
        closed: false,
    });
    let record_at = ptr::from_ref(&*record) as usize;
    let image_at = ptr::from_ref(image) as usize;
    let entry = a[2];
    let parameter = a[3];
    let snapshot = crate::process::child_snapshot().unwrap_or_else(|e| unsupported(&e));
    let (send, receive) = std::sync::mpsc::channel();
    let join = std::thread::Builder::new()
        .name("Alkyhol PE32 worker".into())
        .spawn(move || {
            let id = alkyhol::ffi::GetCurrentThreadId();
            control.id.store(id, Ordering::Release);
            if send.send(id).is_err() {
                return;
            }
            let mut suspended = control.suspended.lock().unwrap();
            while *suspended != 0 {
                suspended = control.ready.wait(suspended).unwrap();
            }
            drop(suspended);
            // SAFETY: Box allocation and image outlive every worker. The parent
            // holds the CPU lease until insertion is complete. Every subsequent
            // guest/process access is under that same exclusive lease; suspended
            // host waits touch only Arc/atomic/Mutex state. Workers never clone Rc.
            unsafe {
                run(
                    image_at as *const LoadedImage32<'_>,
                    record_at as *const Record,
                    entry,
                    parameter,
                    snapshot,
                );
            }
        })
        .unwrap_or_else(|e| unsupported(&format!("native thread creation failed: {e}")));
    let id = receive
        .recv()
        .unwrap_or_else(|_| unsupported("native thread failed before its identity handoff"));
    unsafe {
        record.teb.base.add(0x24).cast::<u32>().write(id);
    }
    if let Some(activation) = image.cpu.bound_activation.borrow_mut().as_mut() {
        activation
            .inherit_thread(id)
            .unwrap_or_else(|e| unsupported(&e));
    }
    let mut state = image.cpu.threads.borrow_mut();
    state.records.insert(h as u32, record);
    state.joins.push(join);
    drop(state);
    if a[5] != 0 {
        output(image, a[5], id);
    }
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 native worker created: handle={h:x}; id={id}; entry={entry:08x}; suspended={}",
            a[4] & 4 != 0
        );
    }
    h as u32
}
// Avoid even an Rc refcount operation on another host thread.
struct RcPtr;
impl RcPtr {
    fn cpu(image: &LoadedImage32<'_>) -> *const Cpu32 {
        std::rc::Rc::as_ptr(&image.cpu)
    }
}
unsafe fn run(
    image: *const LoadedImage32<'_>,
    record: *const Record,
    entry: u32,
    parameter: u32,
    snapshot: crate::process::ChildSnapshot,
) {
    let record = unsafe { &*record };
    let lease = TebLease::acquire(record.teb.base as u32, true).unwrap_or_else(|e| unsupported(&e));
    LEASE.with(|s| *s.borrow_mut() = Some(lease));
    let _lease = MainLease;
    let native_fs = super::segment_state()[3];
    if ![0, 0x23].contains(&native_fs) {
        unsupported("worker native FS is outside the observed Darwin profile");
    }
    record.host_fs.set(native_fs);
    CURRENT.set(record);
    super::ACTIVE.set(image.cast::<c_void>());
    let _process =
        crate::ProcessContext::install_snapshot(snapshot).unwrap_or_else(|e| unsupported(&e));
    let image = unsafe { &*image };
    let top = record.stack.base as u32 + PAGE as u32 + STACK as u32 - 32;
    record.callback_stack.set(top);
    notify(image, 2);
    let frame = top - 8;
    unsafe {
        (frame as *mut u32).write(image.gates.base as u32);
        ((frame + 4) as *mut u32).write(parameter);
    }
    let result = unsafe {
        super::alkyhol_enter_pe32(entry, frame, u32::from(image.code), u32::from(image.data))
    };
    let observed = (result >> 32) as u32;
    let status = if observed == 0 {
        record
            .requested_exit
            .take()
            .filter(|&v| v == result as u32)
            .unwrap_or_else(|| unsupported("worker escaped without matching ExitThread"))
    } else {
        if observed != frame + 8 {
            unsupported("worker entry did not restore stdcall parameter stack");
        }
        result as u32
    };
    super::fiber32::thread_exit(image);
    record.callback_stack.set(top);
    notify(image, 3);
    if let Some(activation) = image.cpu.bound_activation.borrow_mut().as_mut() {
        activation.exit_thread().unwrap_or_else(|e| unsupported(&e));
    }
    record.control.exit.store(status, Ordering::Release);
    // Signal the independently retained object even if the guest closed its handle.
    if alkyhol::ffi::SetEvent(record.control.retained as usize) == 0 {
        unsupported("thread completion signal failed");
    }
    record.control.done.store(1, Ordering::Release);
    super::ACTIVE.set(ptr::null());
    CURRENT.set(ptr::null());
}
pub(super) struct LoaderLock<'a>(&'a Cpu32);
impl Drop for LoaderLock<'_> {
    fn drop(&mut self) {
        self.0.threads.borrow_mut().notification_owner = None;
    }
}
pub(super) fn loader_lock<'a>(image: &'a LoadedImage32<'_>) -> LoaderLock<'a> {
    let id = alkyhol::ffi::GetCurrentThreadId();
    loop {
        let owner = image.cpu.threads.borrow().notification_owner;
        if owner.is_none() {
            image.cpu.threads.borrow_mut().notification_owner = Some(id);
            break;
        }
        if owner == Some(id) {
            unsupported("nested DLL thread-notification lifecycle");
        }
        without_cpu(image, || {
            std::thread::sleep(std::time::Duration::from_millis(1))
        });
    }
    LoaderLock(&image.cpu)
}
fn notify(image: &LoadedImage32<'_>, reason: u32) {
    let _notifications = loader_lock(image);
    let mut entries: Vec<_> = image
        .cpu
        .modules
        .borrow()
        .iter()
        .filter(|m| {
            m.loaded
                && m.base != image.cpu.executable.get()
                && m.thread_notifications
                && m.entry != 0
        })
        .map(|m| (m.base, m.entry))
        .collect();
    if reason == 3 {
        entries.reverse();
    }
    for (base, entry) in entries {
        let _scope = super::activation32::enter_scope(&image.cpu, base);
        let result = unsafe {
            image.callback_frame(entry, &[base, reason, 0], CallingConvention32::Stdcall)
        }
        .unwrap_or_else(|e| unsupported(&e));
        if result.is_none() {
            unsupported("DLL thread notification exited nonlocally");
        }
    }
}
pub(super) fn query(image: &LoadedImage32<'_>, a: &[u32; 8], kind: u32) -> u32 {
    let state = image.cpu.threads.borrow();
    let Some(record) = state.records.get(&a[0]).filter(|r| !r.closed) else {
        alkyhol::ffi::SetLastError(6);
        return if kind == 2 { u32::MAX } else { 0 };
    };
    match kind {
        0 => {
            output(image, a[1], record.control.exit.load(Ordering::Acquire));
            1
        }
        1 => record.control.id.load(Ordering::Acquire),
        2 => {
            let mut count = record.control.suspended.lock().unwrap();
            let old = *count;
            if old != 0 {
                *count -= 1;
                if *count == 0 {
                    record.control.ready.notify_one();
                }
            }
            old
        }
        _ => unreachable!(),
    }
}
pub(super) fn exit_thread(status: u32) -> u32 {
    CURRENT.with(|p| {
        let r = unsafe { p.get().as_ref() }.unwrap_or_else(|| {
            unsupported("ExitThread on the primary guest thread requires process shutdown support")
        });
        r.requested_exit.set(Some(status));
    });
    status
}
pub(super) fn wait(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[2] != 0 {
        unsupported("alertable thread waits require APC delivery");
    }
    wait_handle(image, a[0], a[1])
}

pub(super) fn clear_tls(image: &LoadedImage32<'_>, index: usize) {
    unsafe {
        image
            .cpu
            .teb
            .base
            .add(0xe10 + index * 4)
            .cast::<u32>()
            .write(0);
    }
    for r in image.cpu.threads.borrow().records.values() {
        unsafe {
            r.teb.base.add(0xe10 + index * 4).cast::<u32>().write(0);
        }
    }
}

pub(super) fn host_fs_valid(fs: u16) -> bool {
    CURRENT
        .with(|p| unsafe { p.get().as_ref() })
        .map_or(fs == 0, |r| r.host_fs.get() == fs)
}

pub(super) fn wait_handle(image: &LoadedImage32<'_>, handle: u32, milliseconds: u32) -> u32 {
    without_cpu(image, || {
        if !super::window_host32::is_main_thread() || milliseconds == 0 {
            return alkyhol::ffi::WaitForSingleObject(handle as usize, milliseconds);
        }
        let start = std::time::Instant::now();
        loop {
            super::window_host32::pump_tasks();
            let slice = if milliseconds == u32::MAX {
                1
            } else {
                milliseconds
                    .saturating_sub(start.elapsed().as_millis().min(u32::MAX as u128) as u32)
                    .min(1)
            };
            let result = alkyhol::ffi::WaitForSingleObject(handle as usize, slice);
            if result != 258 {
                return result;
            }
            if milliseconds != u32::MAX
                && start.elapsed() >= std::time::Duration::from_millis(milliseconds as u64)
            {
                return 258;
            }
        }
    })
}

pub(super) fn switch_to_thread(image: &LoadedImage32<'_>) -> u32 {
    let before = super::CPU_TICKETS.lock().unwrap().1;
    without_cpu(image, std::thread::yield_now);
    let after = super::CPU_TICKETS.lock().unwrap().1;
    // Releasing and immediately reacquiring costs one ticket. More than one
    // means another guest thread actually ran under the FIFO CPU lease.
    u32::from(after - before > 1)
}
