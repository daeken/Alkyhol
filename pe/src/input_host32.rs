//! Real Quartz events for the virtual DirectInput keyboard/mouse. Keyboard taps
//! are process-scoped. Mouse events are accepted only while this process is
//! foreground. No guest pointer or guest callback enters a native event tap.
use std::{
    collections::VecDeque,
    ffi::c_void,
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    thread::JoinHandle,
};
type Id = *mut c_void;
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        mask: u64,
        callback: unsafe extern "C" fn(Id, u32, Id, Id) -> Id,
        context: Id,
    ) -> Id;
    fn CGEventTapCreateForPid(
        pid: i32,
        place: u32,
        options: u32,
        mask: u64,
        callback: unsafe extern "C" fn(Id, u32, Id, Id) -> Id,
        context: Id,
    ) -> Id;
    fn CGEventTapEnable(tap: Id, enable: bool);
    fn CGEventTapIsEnabled(tap: Id) -> bool;
    fn CGEventGetIntegerValueField(event: Id, field: u32) -> i64;
    fn CGEventGetTimestamp(event: Id) -> u64;
    fn CGEventSourceKeyState(state: i32, key: u16) -> bool;
    fn CGEventSourceButtonState(state: i32, button: u32) -> bool;
}
#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    static kCFRunLoopDefaultMode: Id;
    fn CFRunLoopGetCurrent() -> Id;
    fn CFRunLoopAddSource(runloop: Id, source: Id, mode: Id);
    fn CFRunLoopRemoveSource(runloop: Id, source: Id, mode: Id);
    fn CFRunLoopRunInMode(mode: Id, seconds: f64, one: bool) -> i32;
    fn CFRunLoopStop(runloop: Id);
    fn CFRetain(value: Id) -> Id;
    fn CFRelease(value: *const c_void);
    fn CFMachPortCreateRunLoopSource(allocator: Id, port: Id, order: isize) -> Id;
    fn CFMachPortInvalidate(port: Id);
}
#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn GetFrontProcess(serial: *mut [u32; 2]) -> i16;
    fn GetProcessPID(serial: *const [u32; 2], pid: *mut i32) -> i32;
}
unsafe extern "C" {
    fn getpid() -> i32;
}
pub(super) fn foreground() -> bool {
    let mut serial = [0; 2];
    let mut pid = 0;
    unsafe {
        GetFrontProcess(&mut serial) == 0
            && GetProcessPID(&serial, &mut pid) == 0
            && pid == getpid()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    Keyboard,
    Mouse,
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Binding {
    pub offset: usize,
    pub component: u16,
    pub appdata: u32,
}
// Mac physical key codes to DIK set-1 positions; these are API constants, not a
// character/layout conversion. Unrepresented physical keys remain absent.
const KEYS: &[(u16, u16)] = &[
    (0, 0x1e),
    (1, 0x1f),
    (2, 0x20),
    (3, 0x21),
    (4, 0x23),
    (5, 0x22),
    (6, 0x2c),
    (7, 0x2d),
    (8, 0x2e),
    (9, 0x2f),
    (10, 0x56),
    (11, 0x30),
    (12, 0x10),
    (13, 0x11),
    (14, 0x12),
    (15, 0x13),
    (16, 0x15),
    (17, 0x14),
    (18, 2),
    (19, 3),
    (20, 4),
    (21, 5),
    (22, 7),
    (23, 6),
    (24, 0x0d),
    (25, 0x0a),
    (26, 8),
    (27, 0x0c),
    (28, 9),
    (29, 0x0b),
    (30, 0x1b),
    (31, 0x18),
    (32, 0x16),
    (33, 0x1a),
    (34, 0x17),
    (35, 0x19),
    (36, 0x1c),
    (37, 0x26),
    (38, 0x24),
    (39, 0x28),
    (40, 0x25),
    (41, 0x27),
    (42, 0x2b),
    (43, 0x33),
    (44, 0x35),
    (45, 0x31),
    (46, 0x32),
    (47, 0x34),
    (48, 0x0f),
    (49, 0x39),
    (50, 0x29),
    (51, 0x0e),
    (53, 1),
    (54, 0xdc),
    (55, 0xdb),
    (56, 0x2a),
    (57, 0x3a),
    (58, 0x38),
    (59, 0x1d),
    (60, 0x36),
    (61, 0xb8),
    (62, 0x9d),
    (65, 0x53),
    (67, 0x37),
    (69, 0x4e),
    (71, 0x45),
    (75, 0xb5),
    (76, 0x9c),
    (78, 0x4a),
    (81, 0x8d),
    (82, 0x52),
    (83, 0x4f),
    (84, 0x50),
    (85, 0x51),
    (86, 0x4b),
    (87, 0x4c),
    (88, 0x4d),
    (89, 0x47),
    (91, 0x48),
    (92, 0x49),
    (96, 0x3f),
    (97, 0x40),
    (98, 0x41),
    (99, 0x3d),
    (100, 0x42),
    (101, 0x43),
    (103, 0x57),
    (105, 0x64),
    (107, 0x65),
    (109, 0x44),
    (111, 0x58),
    (113, 0x66),
    (114, 0xd2),
    (115, 0xc7),
    (116, 0xc9),
    (117, 0xd3),
    (118, 0x3e),
    (119, 0xcf),
    (120, 0x3c),
    (121, 0xd1),
    (122, 0x3b),
    (123, 0xcb),
    (124, 0xcd),
    (125, 0xd0),
    (126, 0xc8),
];
pub(super) fn key_components() -> Vec<u16> {
    let mut keys: Vec<_> = KEYS.iter().map(|&(_, dik)| dik).collect();
    keys.sort_unstable();
    keys.dedup();
    keys
}
static SEQUENCE: AtomicU32 = AtomicU32::new(0);
struct Data {
    bytes: Vec<u8>,
    queue: VecDeque<[u32; 5]>,
    overflow: bool,
    lost: bool,
}
struct Context {
    kind: Kind,
    bindings: Vec<Binding>,
    capacity: usize,
    data: Mutex<Data>,
    stop: AtomicBool,
}
impl Context {
    fn update(&self, component: u16, value: i32, time: u32, relative: bool) {
        let mut data = self.data.lock().unwrap();
        for binding in self.bindings.iter().filter(|b| b.component == component) {
            if relative {
                let old = i32::from_le_bytes(
                    data.bytes[binding.offset..binding.offset + 4]
                        .try_into()
                        .unwrap(),
                );
                data.bytes[binding.offset..binding.offset + 4]
                    .copy_from_slice(&old.wrapping_add(value).to_le_bytes());
            } else {
                data.bytes[binding.offset] = value as u8;
            }
            if self.capacity != 0 {
                if data.queue.len() == self.capacity {
                    data.queue.pop_front();
                    data.overflow = true;
                }
                data.queue.push_back([
                    binding.offset as u32,
                    value as u32,
                    time,
                    SEQUENCE.fetch_add(1, Ordering::Relaxed),
                    binding.appdata,
                ]);
            }
        }
    }
}
unsafe extern "C" fn event(_proxy: Id, ty: u32, event: Id, user: Id) -> Id {
    let context = unsafe { &*(user as *const Context) };
    if ty >= 0xfffffffe {
        context.data.lock().unwrap().lost = true;
        return event;
    }
    if !foreground() {
        context.data.lock().unwrap().lost = true;
        return event;
    }
    let time = (unsafe { CGEventGetTimestamp(event) } / 1_000_000) as u32;
    if context.kind == Kind::Keyboard {
        if unsafe { CGEventGetIntegerValueField(event, 8) } != 0 {
            return event;
        }
        let key = unsafe { CGEventGetIntegerValueField(event, 9) } as u16;
        if let Some(&(_, component)) = KEYS.iter().find(|&&(native, _)| native == key) {
            let down = if ty == 12 {
                unsafe { CGEventSourceKeyState(1, key) }
            } else {
                ty == 10
            };
            context.update(component, if down { 128 } else { 0 }, time, false);
        }
    } else {
        match ty {
            5..=7 | 27 => {
                for (field, component) in [(4, 256), (5, 257)] {
                    let delta = unsafe { CGEventGetIntegerValueField(event, field) } as i32;
                    if delta != 0 {
                        context.update(component, delta, time, true);
                    }
                }
            }
            22 => {
                let delta = unsafe { CGEventGetIntegerValueField(event, 11) } as i32;
                if delta != 0 {
                    context.update(258, delta.wrapping_mul(120), time, true);
                }
            }
            1..=4 | 25 | 26 => {
                let button = unsafe { CGEventGetIntegerValueField(event, 3) };
                if (0..8).contains(&button) {
                    context.update(
                        512 + button as u16,
                        if [1, 3, 25].contains(&ty) { 128 } else { 0 },
                        time,
                        false,
                    );
                }
            }
            _ => (),
        }
    }
    event
}
pub(super) struct Source {
    context: Arc<Context>,
    runloop: usize,
    thread: Option<JoinHandle<()>>,
}
impl Source {
    pub(super) fn new(
        kind: Kind,
        size: usize,
        bindings: Vec<Binding>,
        capacity: usize,
    ) -> Result<Self, String> {
        if !foreground() {
            return Err("input window process is not foreground".into());
        }
        let context = Arc::new(Context {
            kind,
            bindings,
            capacity,
            data: Mutex::new(Data {
                bytes: vec![0; size],
                queue: VecDeque::new(),
                overflow: false,
                lost: false,
            }),
            stop: AtomicBool::new(false),
        });
        {
            let mut data = context.data.lock().unwrap();
            for b in &context.bindings {
                if kind == Kind::Keyboard {
                    if let Some(&(native, _)) = KEYS.iter().find(|&&(_, dik)| dik == b.component) {
                        data.bytes[b.offset] = if unsafe { CGEventSourceKeyState(1, native) } {
                            128
                        } else {
                            0
                        };
                    }
                } else if (512..520).contains(&b.component) {
                    data.bytes[b.offset] =
                        if unsafe { CGEventSourceButtonState(1, (b.component - 512) as u32) } {
                            128
                        } else {
                            0
                        };
                }
            }
        }
        let state = context.clone();
        let (send, receive) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let mask = if kind == Kind::Keyboard {
                (1u64 << 10) | (1 << 11) | (1 << 12)
            } else {
                [1u32, 2, 3, 4, 5, 6, 7, 22, 25, 26, 27]
                    .iter()
                    .fold(0u64, |n, &ty| n | (1 << ty))
            };
            let user = Arc::as_ptr(&state) as Id;
            let port = unsafe {
                if kind == Kind::Keyboard {
                    CGEventTapCreateForPid(getpid(), 0, 1, mask, event, user)
                } else {
                    CGEventTapCreate(1, 0, 1, mask, event, user)
                }
            };
            if port.is_null() {
                let _ = send.send(Err("native event tap denied or unavailable".to_string()));
                return;
            }
            let source = unsafe { CFMachPortCreateRunLoopSource(ptr::null_mut(), port, 0) };
            if source.is_null() {
                unsafe {
                    CFMachPortInvalidate(port);
                    CFRelease(port);
                }
                let _ = send.send(Err("cannot create native input run-loop source".to_string()));
                return;
            }
            let runloop = unsafe { CFRunLoopGetCurrent() };
            unsafe {
                CFRunLoopAddSource(runloop, source, kCFRunLoopDefaultMode);
                CGEventTapEnable(port, true);
            }
            if !unsafe { CGEventTapIsEnabled(port) } {
                unsafe {
                    CFRunLoopRemoveSource(runloop, source, kCFRunLoopDefaultMode);
                    CFRelease(source);
                    CFMachPortInvalidate(port);
                    CFRelease(port);
                }
                let _ = send.send(Err("native input tap is disabled".to_string()));
                return;
            }
            unsafe {
                CFRetain(runloop);
            }
            if send.send(Ok(runloop as usize)).is_err() {
                unsafe {
                    CFRelease(runloop);
                }
                state.stop.store(true, Ordering::Release);
            }
            while !state.stop.load(Ordering::Acquire) && !state.data.lock().unwrap().lost {
                unsafe {
                    CFRunLoopRunInMode(kCFRunLoopDefaultMode, 0.016, true);
                }
                if !foreground() {
                    state.data.lock().unwrap().lost = true;
                }
            }
            unsafe {
                CFRunLoopRemoveSource(runloop, source, kCFRunLoopDefaultMode);
                CFRelease(source);
                CFMachPortInvalidate(port);
                CFRelease(port);
            }
        });
        match receive.recv_timeout(std::time::Duration::from_secs(3)) {
            Ok(Ok(runloop)) => Ok(Self {
                context,
                runloop,
                thread: Some(thread),
            }),
            result => {
                context.stop.store(true, Ordering::Release);
                let _ = thread.join();
                if let Ok(Ok(runloop)) = receive.try_recv() {
                    unsafe {
                        CFRelease(runloop as Id);
                    }
                }
                Err(match result {
                    Ok(Err(e)) => e,
                    _ => "native input creation timed out".into(),
                })
            }
        }
    }
    pub(super) fn lost(&self) -> bool {
        self.context.data.lock().unwrap().lost
    }
    pub(super) fn state(&self) -> Result<Vec<u8>, String> {
        let mut data = self.context.data.lock().unwrap();
        if data.lost {
            return Err("input source lost foreground or event tap".into());
        }
        let bytes = data.bytes.clone();
        for b in &self.context.bindings {
            if (256..259).contains(&b.component) {
                data.bytes[b.offset..b.offset + 4].fill(0);
            }
        }
        Ok(bytes)
    }
    pub(super) fn records(
        &self,
        count: usize,
        peek: bool,
    ) -> Result<(Vec<[u32; 5]>, bool), String> {
        let mut data = self.context.data.lock().unwrap();
        if data.lost {
            return Err("input source lost foreground or event tap".into());
        }
        let n = count.min(data.queue.len());
        let records = data.queue.iter().take(n).copied().collect();
        let overflow = data.overflow;
        if !peek {
            data.queue.drain(..n);
            data.overflow = false;
        }
        Ok((records, overflow))
    }
}
impl Drop for Source {
    fn drop(&mut self) {
        self.context.stop.store(true, Ordering::Release);
        unsafe {
            CFRunLoopStop(self.runloop as Id);
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
        unsafe {
            CFRelease(self.runloop as Id);
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn captured_events_wrap_accumulators_and_report_bounded_queue_overflow() {
        let context = Context {
            kind: Kind::Mouse,
            bindings: vec![
                Binding {
                    offset: 0,
                    component: 256,
                    appdata: u32::MAX,
                },
                Binding {
                    offset: 4,
                    component: 512,
                    appdata: u32::MAX,
                },
            ],
            capacity: 2,
            data: Mutex::new(Data {
                bytes: vec![0; 5],
                queue: VecDeque::new(),
                overflow: false,
                lost: false,
            }),
            stop: AtomicBool::new(false),
        };
        context.update(256, i32::MAX, 1, true);
        context.update(256, 1, 2, true);
        context.update(512, 128, 3, false);
        let data = context.data.lock().unwrap();
        assert_eq!(
            i32::from_le_bytes(data.bytes[..4].try_into().unwrap()),
            i32::MIN
        );
        assert_eq!(data.bytes[4], 128);
        assert_eq!(data.queue.len(), 2);
        assert!(data.overflow);
        assert_eq!(data.queue[0][2], 2);
    }
}
