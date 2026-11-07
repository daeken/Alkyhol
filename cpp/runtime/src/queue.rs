//! Real queue-ordered Metal completion and CPU-signaled D3D fence subset.
//! Queue.Wait/timestamps remain explicit unsupported operations.
use super::*;
use alkyhol::kernel::{EventNotifier, kernel};
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
};
const FENCE: Guid = guid!(
    0x0a753dcf, 0xc4d8, 0x4b91, 0xad, 0xf6, 0xbe, 0x5a, 0x60, 0xd9, 0x5a, 0x76
);
const QUEUE: Guid = guid!(
    0x0ec870a6, 0x5d7e, 0x4c22, 0x8c, 0xfc, 0x5b, 0xaa, 0xe0, 0x76, 0x16, 0xed
);
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct QueueDesc {
    kind: i32,
    priority: i32,
    flags: u32,
    node: u32,
}
struct Counter {
    value: u64,
    events: Vec<(u64, EventNotifier)>,
}
struct Timeline {
    counter: Mutex<Counter>,
    changed: Condvar,
}
impl Timeline {
    fn new(value: u64) -> Self {
        Self {
            counter: Mutex::new(Counter {
                value,
                events: Vec::new(),
            }),
            changed: Condvar::new(),
        }
    }
    fn value(&self) -> u64 {
        self.counter.lock().unwrap().value
    }
    fn signal(&self, value: u64) {
        let notify = {
            let mut c = self.counter.lock().unwrap();
            c.value = value;
            let mut ready = Vec::new();
            let mut i = 0;
            while i < c.events.len() {
                if c.events[i].0 <= value {
                    ready.push(c.events.swap_remove(i).1);
                } else {
                    i += 1;
                }
            }
            self.changed.notify_all();
            ready
        };
        for event in notify {
            event.signal();
        }
    }
    fn arm(&self, target: u64, handle: usize) -> i32 {
        if handle == 0 {
            let mut c = self.counter.lock().unwrap();
            while c.value < target {
                c = self.changed.wait(c).unwrap();
            }
            return S_OK;
        }
        let event = match kernel().event_notifier(handle) {
            Ok(e) => e,
            Err(e) => return (0x80070000 | e.0) as i32,
        };
        let mut c = self.counter.lock().unwrap();
        if c.value >= target {
            drop(c);
            event.signal();
            return S_OK;
        }
        if c.events.try_reserve(1).is_err() {
            return E_OUTOFMEMORY;
        }
        c.events.push((target, event));
        S_OK
    }
}
#[repr(C)]
struct Fence {
    vtable: *const FenceVtable,
    refs: AtomicU32,
    metadata: Metadata,
    timeline: Arc<Timeline>,
    parent: Parent,
}
#[repr(C)]
struct Queue {
    vtable: *const QueueVtable,
    refs: AtomicU32,
    metadata: Metadata,
    native: metal::CommandQueue,
    desc: QueueDesc,
    order: Arc<Mutex<Order>>,
    submit: Mutex<()>,
    identity: Arc<()>,
    parent: Parent,
}
struct Action {
    timeline: Option<Arc<Timeline>>,
    value: u64,
    done: bool,
    failed: bool,
    id: u64,
}
#[derive(Default)]
struct Order {
    next: u64,
    actions: VecDeque<Action>,
    failed: bool,
}
impl Order {
    fn complete(&mut self, id: u64, failed: bool) {
        self.failed |= failed;
        if let Some(action) = self.actions.iter_mut().find(|a| a.id == id) {
            action.done = true;
            action.failed = failed;
        }
        while self.actions.front().is_some_and(|a| a.done) {
            let a = self.actions.pop_front().unwrap();
            if let Some(timeline) = a.timeline {
                timeline.signal(if a.failed || self.failed {
                    u64::MAX
                } else {
                    a.value
                });
            }
        }
    }
}
macro_rules! ownership {
    ($ty:ty,$iid:ident,$q:ident,$a:ident,$r:ident) => {
        unsafe extern "C" fn $q(this: *mut $ty, iid: *const Guid, out: *mut *mut c_void) -> i32 {
            if let Err(e) = unsafe { clear(out) } {
                return e;
            }
            if iid.is_null() {
                return E_POINTER;
            }
            if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, $iid].contains(unsafe { &*iid }) {
                return E_NOINTERFACE;
            }
            unsafe {
                $a(this);
                *out = this.cast();
            }
            S_OK
        }
        unsafe extern "C" fn $a(this: *mut $ty) -> u32 {
            increment(unsafe { &(*this).refs })
        }
        unsafe extern "C" fn $r(this: *mut $ty) -> u32 {
            let n = unsafe { (*this).refs.fetch_sub(1, Ordering::Release) };
            if n == 1 {
                fence(Ordering::Acquire);
                unsafe {
                    drop(Box::from_raw(this));
                }
                0
            } else {
                n - 1
            }
        }
    };
}
ownership!(Fence, FENCE, fence_query, fence_add, fence_release);
ownership!(Queue, QUEUE, queue_query, queue_add, queue_release);
metadata_methods!(Fence, fence_set, fence_interface, fence_get);
metadata_methods!(Queue, queue_set, queue_interface, queue_get);
unsafe extern "C" fn fence_name(this: *mut Fence, name: *const u32) -> i32 {
    let m = unsafe { (*this).metadata.0 };
    unsafe { ((*(*m).vtable).set_name)(m, name) }
}
unsafe extern "C" fn queue_name(this: *mut Queue, name: *const u32) -> i32 {
    let m = unsafe { (*this).metadata.0 };
    unsafe { ((*(*m).vtable).set_name)(m, name) }
}
unsafe extern "C" fn fence_device(
    this: *mut Fence,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { device_query((*this).parent.0, iid, out) }
}
unsafe extern "C" fn queue_device(
    this: *mut Queue,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    unsafe { device_query((*this).parent.0, iid, out) }
}
unsafe extern "C" fn completed(this: *mut Fence) -> u64 {
    unsafe { (*this).timeline.value() }
}
unsafe extern "C" fn event(this: *mut Fence, value: u64, handle: usize) -> i32 {
    unsafe { (*this).timeline.arm(value, handle) }
}
unsafe extern "C" fn cpu_signal(this: *mut Fence, value: u64) -> i32 {
    unsafe {
        (*this).timeline.signal(value);
    }
    S_OK
}
unsafe extern "C" fn gpu_signal(this: *mut Queue, fence: *mut Fence, value: u64) -> i32 {
    if fence.is_null() {
        return E_POINTER;
    }
    if unsafe { (*fence).vtable } != &FENCE_VTABLE
        || unsafe { (*fence).parent.0 } != unsafe { (*this).parent.0 }
    {
        return E_INVALIDARG;
    }
    unsafe { submit(this, Some(Arc::clone(&(*fence).timeline)), value, None) }
}
// A single submission mutex covers validation/encoding/commit for both work and
// signals. CPU completion callbacks may arrive in any order; Order retires them
// in submission order, propagating workload errors to subsequent fence signals.
unsafe fn submit(
    this: *mut Queue,
    timeline: Option<Arc<Timeline>>,
    value: u64,
    lists: Option<(u32, usize)>,
) -> i32 {
    autoreleasepool(|| {
        let _submission = unsafe { (*this).submit.lock().unwrap() };
        let batches = if let Some((count, lists)) = lists {
            match unsafe {
                super::commands::prepare(
                    (*this).parent.0,
                    (*this).desc.kind,
                    count,
                    lists,
                    &(*this).identity,
                )
            } {
                Ok(batches) => batches,
                Err(e) => return e,
            }
        } else {
            Vec::new()
        };
        let order = unsafe { Arc::clone(&(*this).order) };
        let mut state = order.lock().unwrap();
        if state.failed {
            return 0x887a0005u32 as i32;
        }
        if state.actions.try_reserve(1).is_err() {
            return E_OUTOFMEMORY;
        }
        let Some(next) = state.next.checked_add(1) else {
            return E_OUTOFMEMORY;
        };
        let id = state.next;
        state.next = next;
        let command = unsafe { (*this).native.new_command_buffer() };
        if !batches.is_empty() {
            let encoder = command.new_blit_command_encoder();
            for batch in &batches {
                batch.encode(encoder);
            }
            encoder.end_encoding();
        }
        for batch in &batches {
            batch.commit();
        }
        let callback_order = Arc::clone(&order);
        let handler = block::ConcreteBlock::new(move |buffer: &metal::CommandBufferRef| {
            let failed = buffer.status() != metal::MTLCommandBufferStatus::Completed;
            if failed {
                eprintln!(
                    "Alkyhol Metal queue completion failed: {:?}",
                    buffer.status()
                );
            }
            for batch in &batches {
                batch.finish(failed);
            }
            callback_order.lock().unwrap().complete(id, failed);
        })
        .copy();
        command.add_completed_handler(&handler);
        state.actions.push_back(Action {
            timeline,
            value,
            done: false,
            failed: false,
            id,
        });
        drop(state);
        command.commit();
        S_OK
    })
}

unsafe extern "C" fn queue_wait(_: *mut Queue, _: *mut Fence, _: u64) -> i32 {
    unavailable("GPU queue wait")
}
unsafe extern "C" fn frequency(_: *mut Queue, out: *mut u64) -> i32 {
    if out.is_null() {
        return E_POINTER;
    }
    unsafe {
        *out = 0;
    }
    unavailable("queue timestamp frequency")
}
unsafe extern "C" fn calibration(_: *mut Queue, gpu: *mut u64, cpu: *mut u64) -> i32 {
    if gpu.is_null() || cpu.is_null() {
        return E_POINTER;
    }
    unsafe {
        *gpu = 0;
        *cpu = 0;
    }
    unavailable("queue clock calibration")
}
unsafe extern "C" fn description(this: *mut Queue) -> QueueDesc {
    unsafe { (*this).desc }
}
unsafe extern "C" fn tiles(
    _: *mut Queue,
    _: usize,
    _: u32,
    _: usize,
    _: usize,
    _: usize,
    _: u32,
    _: usize,
    _: usize,
    _: usize,
    _: u32,
) {
    unsupported_void("queue tile mappings")
}
unsafe extern "C" fn copy_tiles(
    _: *mut Queue,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: usize,
    _: u32,
) {
    unsupported_void("queue tile copy")
}
unsafe extern "C" fn execute(this: *mut Queue, count: u32, lists: usize) {
    let result = unsafe { submit(this, None, 0, Some((count, lists))) };
    if result != S_OK {
        eprintln!(
            "Alkyhol ExecuteCommandLists rejected: 0x{:08x}",
            result as u32
        );
        unsupported_void("invalid or unsupported command list execution");
    }
}
unsafe extern "C" fn marker(_: *mut Queue, _: u32, _: usize, _: u32) {
    unsupported_void("queue marker/event")
}
unsafe extern "C" fn end(_: *mut Queue) {
    unsupported_void("queue end event")
}
#[repr(C)]
struct FenceVtable {
    query: unsafe extern "C" fn(*mut Fence, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Fence) -> u32,
    release: unsafe extern "C" fn(*mut Fence) -> u32,
    get: unsafe extern "C" fn(*mut Fence, *const Guid, *mut u32, *mut c_void) -> i32,
    set: unsafe extern "C" fn(*mut Fence, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Fence, *const Guid, *mut c_void) -> i32,
    name: unsafe extern "C" fn(*mut Fence, *const u32) -> i32,
    device: unsafe extern "C" fn(*mut Fence, *const Guid, *mut *mut c_void) -> i32,
    completed: unsafe extern "C" fn(*mut Fence) -> u64,
    event: unsafe extern "C" fn(*mut Fence, u64, usize) -> i32,
    signal: unsafe extern "C" fn(*mut Fence, u64) -> i32,
}
static FENCE_VTABLE: FenceVtable = FenceVtable {
    query: fence_query,
    add: fence_add,
    release: fence_release,
    get: fence_get,
    set: fence_set,
    interface: fence_interface,
    name: fence_name,
    device: fence_device,
    completed,
    event,
    signal: cpu_signal,
};
#[repr(C)]
struct QueueVtable {
    query: unsafe extern "C" fn(*mut Queue, *const Guid, *mut *mut c_void) -> i32,
    add: unsafe extern "C" fn(*mut Queue) -> u32,
    release: unsafe extern "C" fn(*mut Queue) -> u32,
    get: unsafe extern "C" fn(*mut Queue, *const Guid, *mut u32, *mut c_void) -> i32,
    set: unsafe extern "C" fn(*mut Queue, *const Guid, u32, *const c_void) -> i32,
    interface: unsafe extern "C" fn(*mut Queue, *const Guid, *mut c_void) -> i32,
    name: unsafe extern "C" fn(*mut Queue, *const u32) -> i32,
    device: unsafe extern "C" fn(*mut Queue, *const Guid, *mut *mut c_void) -> i32,
    tiles: unsafe extern "C" fn(
        *mut Queue,
        usize,
        u32,
        usize,
        usize,
        usize,
        u32,
        usize,
        usize,
        usize,
        u32,
    ),
    copy: unsafe extern "C" fn(*mut Queue, usize, usize, usize, usize, usize, u32),
    execute: unsafe extern "C" fn(*mut Queue, u32, usize),
    marker: unsafe extern "C" fn(*mut Queue, u32, usize, u32),
    begin: unsafe extern "C" fn(*mut Queue, u32, usize, u32),
    end: unsafe extern "C" fn(*mut Queue),
    signal: unsafe extern "C" fn(*mut Queue, *mut Fence, u64) -> i32,
    wait: unsafe extern "C" fn(*mut Queue, *mut Fence, u64) -> i32,
    frequency: unsafe extern "C" fn(*mut Queue, *mut u64) -> i32,
    calibration: unsafe extern "C" fn(*mut Queue, *mut u64, *mut u64) -> i32,
    desc: unsafe extern "C" fn(*mut Queue) -> QueueDesc,
}
static QUEUE_VTABLE: QueueVtable = QueueVtable {
    query: queue_query,
    add: queue_add,
    release: queue_release,
    get: queue_get,
    set: queue_set,
    interface: queue_interface,
    name: queue_name,
    device: queue_device,
    tiles,
    copy: copy_tiles,
    execute,
    marker,
    begin: marker,
    end,
    signal: gpu_signal,
    wait: queue_wait,
    frequency,
    calibration,
    desc: description,
};
pub(super) unsafe extern "C" fn create_queue(
    this: *mut Device,
    desc: *const c_void,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if desc.is_null() || iid.is_null() {
        return E_POINTER;
    }
    let mut desc = unsafe { *desc.cast::<QueueDesc>() };
    if ![0, 2, 3].contains(&desc.kind) || desc.priority != 0 || desc.flags != 0 || desc.node > 1 {
        return E_NOTIMPL;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, QUEUE].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    // Native WARP normalizes the default single-node mask from 0 to 1.
    // Preserve that observed GetDesc result, rather than echoing the input.
    desc.node = 1;
    autoreleasepool(|| {
        let native = unsafe { (*this).metal.new_command_queue() };
        if native.as_ptr().is_null() {
            return E_OUTOFMEMORY;
        }
        unsafe {
            device_add(this);
        }
        let queue = Box::new(Queue {
            vtable: &QUEUE_VTABLE,
            refs: AtomicU32::new(1),
            metadata: Metadata::new(),
            native,
            desc,
            order: Arc::new(Mutex::new(Order::default())),
            submit: Mutex::new(()),
            identity: Arc::new(()),
            parent: Parent(this),
        });
        unsafe {
            *out = Box::into_raw(queue).cast();
        }
        S_OK
    })
}
pub(super) unsafe extern "C" fn create_fence(
    this: *mut Device,
    value: u64,
    flags: u32,
    iid: *const Guid,
    out: *mut *mut c_void,
) -> i32 {
    if let Err(e) = unsafe { clear(out) } {
        return e;
    }
    if iid.is_null() {
        return E_POINTER;
    }
    if flags != 0 {
        return E_NOTIMPL;
    }
    if ![IID_IUNKNOWN, IID_ID3D12OBJECT, CHILD, PAGEABLE, FENCE].contains(unsafe { &*iid }) {
        return E_NOINTERFACE;
    }
    unsafe {
        device_add(this);
    }
    let fence = Box::new(Fence {
        vtable: &FENCE_VTABLE,
        refs: AtomicU32::new(1),
        metadata: Metadata::new(),
        timeline: Arc::new(Timeline::new(value)),
        parent: Parent(this),
    });
    unsafe {
        *out = Box::into_raw(fence).cast();
    }
    S_OK
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_vtable_sizes_match_sdk_source_interfaces() {
        assert_eq!(
            std::mem::size_of::<FenceVtable>(),
            11 * std::mem::size_of::<usize>()
        );
        assert_eq!(
            std::mem::size_of::<QueueVtable>(),
            19 * std::mem::size_of::<usize>()
        );
        assert_eq!(std::mem::size_of::<QueueDesc>(), 16);
    }
    #[test]
    fn cpu_timeline_rewinds_and_notifies_only_satisfied_events() {
        let timeline = Timeline::new(5);
        let a = kernel().create_event(false, false);
        let b = kernel().create_event(false, false);
        assert_eq!(timeline.arm(7, a), S_OK);
        assert_eq!(timeline.arm(9, b), S_OK);
        timeline.signal(6);
        assert_eq!(kernel().wait(a, 0).unwrap(), 258);
        timeline.signal(7);
        assert_eq!(kernel().wait(a, 0).unwrap(), 0);
        assert_eq!(kernel().wait(b, 0).unwrap(), 258);
        timeline.signal(3);
        assert_eq!(timeline.value(), 3);
        timeline.signal(9);
        assert_eq!(kernel().wait(b, 0).unwrap(), 0);
        assert_eq!(timeline.arm(8, a), S_OK);
        assert_eq!(kernel().wait(a, 0).unwrap(), 0);
        assert_eq!(timeline.arm(10, usize::MAX), 0x80070006u32 as i32);
        kernel().close_handle(a).unwrap();
        kernel().close_handle(b).unwrap();
    }
    #[test]
    fn null_event_wait_blocks_until_the_actual_target_is_reached() {
        let timeline = Arc::new(Timeline::new(0));
        let other = Arc::clone(&timeline);
        let (sender, receiver) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let result = other.arm(5, 0);
            sender.send(result).unwrap();
        });
        assert!(
            receiver
                .recv_timeout(std::time::Duration::from_millis(20))
                .is_err()
        );
        timeline.signal(4);
        assert!(
            receiver
                .recv_timeout(std::time::Duration::from_millis(20))
                .is_err()
        );
        timeline.signal(5);
        assert_eq!(
            receiver
                .recv_timeout(std::time::Duration::from_secs(1))
                .unwrap(),
            S_OK
        );
        thread.join().unwrap();
    }
    #[test]
    fn out_of_order_callbacks_cannot_reorder_fence_updates_and_errors_cannot_pass() {
        let timeline = Arc::new(Timeline::new(0));
        let mut order = Order::default();
        for (id, value) in [(0, 10), (1, 3)] {
            order.actions.push_back(Action {
                timeline: Some(Arc::clone(&timeline)),
                value,
                done: false,
                failed: false,
                id,
            });
        }
        order.complete(1, false);
        assert_eq!(timeline.value(), 0);
        order.complete(0, false);
        assert_eq!(timeline.value(), 3);
        assert!(order.actions.is_empty());
        order.actions.push_back(Action {
            timeline: Some(Arc::clone(&timeline)),
            value: 99,
            done: false,
            failed: false,
            id: 2,
        });
        order.complete(2, true);
        assert!(order.failed);
        assert_eq!(timeline.value(), u64::MAX);
    }
    #[test]
    fn actual_metal_queue_completion_notifies_and_retains_released_fence_state() {
        autoreleasepool(|| unsafe {
            let mut raw = ptr::null_mut();
            assert_eq!(
                AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, &mut raw),
                S_OK
            );
            let device = raw.cast::<Device>();
            let d = QueueDesc {
                kind: 3,
                priority: 0,
                flags: 0,
                node: 0,
            };
            assert_eq!(
                create_queue(device, (&d as *const QueueDesc).cast(), &QUEUE, &mut raw),
                S_OK
            );
            let queue = raw.cast::<Queue>();
            assert_eq!(create_fence(device, 0, 0, &FENCE, &mut raw), S_OK);
            let fence = raw.cast::<Fence>();
            let event = kernel().create_event(false, false);
            assert_eq!(super::event(fence, 7, event), S_OK);
            assert_eq!(gpu_signal(queue, fence, 7), S_OK);
            assert_eq!(kernel().wait(event, 5000).unwrap(), 0);
            assert_eq!(completed(fence), 7);
            assert_eq!(super::event(fence, 8, event), S_OK);
            assert_eq!(gpu_signal(queue, fence, 8), S_OK);
            assert_eq!(fence_release(fence), 0);
            assert_eq!(kernel().wait(event, 5000).unwrap(), 0);
            assert_eq!(device_release(device), 1);
            assert_eq!(queue_release(queue), 0);
            kernel().close_handle(event).unwrap();
        });
    }
    #[test]
    fn constructor_flags_types_and_foreign_fences_fail_without_outputs() {
        autoreleasepool(|| unsafe {
            let mut raw = ptr::null_mut();
            assert_eq!(
                AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, &mut raw),
                S_OK
            );
            let device = raw.cast::<Device>();
            let d = QueueDesc {
                kind: 1,
                priority: 0,
                flags: 0,
                node: 0,
            };
            raw = ptr::dangling_mut();
            assert_eq!(
                create_queue(device, (&d as *const QueueDesc).cast(), &QUEUE, &mut raw),
                E_NOTIMPL
            );
            assert!(raw.is_null());
            assert_eq!(create_fence(device, 0, 1, &FENCE, &mut raw), E_NOTIMPL);
            assert!(raw.is_null());
            assert_eq!(
                create_fence(device, 0, 0, &RESOURCE, &mut raw),
                E_NOINTERFACE
            );
            assert!(raw.is_null());
            let mut queues = Vec::new();
            for kind in [0, 2, 3] {
                let desc = QueueDesc { kind, ..d };
                assert_eq!(
                    create_queue(device, (&desc as *const QueueDesc).cast(), &QUEUE, &mut raw),
                    S_OK
                );
                let queue = raw.cast::<Queue>();
                assert_eq!(description(queue).kind, kind);
                assert_eq!(description(queue).node, 1);
                queues.push(queue);
            }
            let mut other = ptr::null_mut();
            assert_eq!(
                AlkyholCreateMetalDeviceSubset(ptr::null_mut(), &DEVICE, &mut other),
                S_OK
            );
            assert_eq!(create_fence(other.cast(), 0, 0, &FENCE, &mut raw), S_OK);
            let foreign = raw.cast::<Fence>();
            assert_eq!(gpu_signal(queues[0], foreign, 1), E_INVALIDARG);
            assert_eq!(gpu_signal(queues[0], ptr::null_mut(), 1), E_POINTER);
            assert_eq!(completed(foreign), 0);
            assert_eq!(fence_release(foreign), 0);
            assert_eq!(device_release(other.cast()), 0);
            for queue in queues {
                assert_eq!(queue_release(queue), 0);
            }
            assert_eq!(device_release(device), 0);
        });
    }
}
