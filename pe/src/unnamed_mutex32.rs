//! Unnamed process-local mutexes with real guest-thread ownership. All metadata
//! transitions occur under the CPU lease; contended waits release it. Native
//! worker exit abandons owned mutexes before signaling thread completion.
use super::{LoadedImage32, LowMapping, PAGE, unsupported};
use crate::Result;
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};
#[derive(Default)]
struct Ownership {
    owner: u32,
    depth: u32,
    abandoned: bool,
}
impl Ownership {
    fn poll(&mut self, thread: u32) -> Result<u32> {
        if self.depth != 0 && self.owner != thread {
            return Ok(258);
        }
        self.depth = self
            .depth
            .checked_add(1)
            .filter(|&n| n <= 1024)
            .ok_or("mutex recursion budget exceeded")?;
        self.owner = thread;
        Ok(if std::mem::take(&mut self.abandoned) {
            0x80
        } else {
            0
        })
    }
    fn release(&mut self, thread: u32) -> bool {
        if self.depth == 0 || self.owner != thread {
            return false;
        }
        self.depth -= 1;
        if self.depth == 0 {
            self.owner = 0;
        }
        true
    }
    fn abandon(&mut self, thread: u32) {
        if self.depth != 0 && self.owner == thread {
            self.depth = 0;
            self.owner = 0;
            self.abandoned = true;
        }
    }
}
struct Entry {
    _token: LowMapping,
    ownership: Ownership,
    waiters: u32,
}
#[derive(Default)]
pub(super) struct State {
    entries: BTreeMap<u32, Entry>,
}
impl State {
    pub(super) fn contains(&self, handle: u32) -> bool {
        self.entries.contains_key(&handle)
    }
    pub(super) fn clear(&mut self) {
        self.entries.clear();
    }
    pub(super) fn abandon(&mut self, thread: u32) {
        for entry in self.entries.values_mut() {
            entry.ownership.abandon(thread);
        }
    }
    pub(super) fn close(&mut self, handle: u32) -> Result<bool> {
        if self.entries.get(&handle).is_some_and(|e| e.waiters != 0) {
            return Err(
                "closing a mutex during a pending wait is outside the defined profile".into(),
            );
        }
        Ok(self.entries.remove(&handle).is_some())
    }
}
pub(super) fn create(image: &LoadedImage32<'_>, initial: bool) -> u32 {
    let mut state = image.cpu.unnamed_mutexes.borrow_mut();
    if state.entries.len() >= 1024 {
        unsupported("unnamed mutex handle budget exhausted");
    }
    let token = image
        .cpu
        .mapping(PAGE, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    token
        .protect(0, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    let handle = token.base as u32;
    let mut ownership = Ownership::default();
    if initial {
        ownership.poll(alkyhol::ffi::GetCurrentThreadId()).unwrap();
    }
    state.entries.insert(
        handle,
        Entry {
            _token: token,
            ownership,
            waiters: 0,
        },
    );
    alkyhol::ffi::SetLastError(0);
    handle
}
pub(super) fn operation(image: &LoadedImage32<'_>, args: &[u32; 8], wait: bool) -> u32 {
    let thread = alkyhol::ffi::GetCurrentThreadId();
    if !wait {
        let released = image
            .cpu
            .unnamed_mutexes
            .borrow_mut()
            .entries
            .get_mut(&args[0])
            .unwrap_or_else(|| unsupported("unknown unnamed mutex handle"))
            .ownership
            .release(thread);
        if !released {
            alkyhol::ffi::SetLastError(288);
        }
        return released as u32;
    }
    if args[2] != 0 {
        unsupported("alertable unnamed mutex waits require APC delivery");
    }
    let start = Instant::now();
    let mut registered = false;
    loop {
        let mut state = image.cpu.unnamed_mutexes.borrow_mut();
        let entry = state
            .entries
            .get_mut(&args[0])
            .unwrap_or_else(|| unsupported("mutex closed during an active wait"));
        let result = entry
            .ownership
            .poll(thread)
            .unwrap_or_else(|e| unsupported(&e));
        let timeout =
            args[1] != u32::MAX && start.elapsed() >= Duration::from_millis(args[1] as u64);
        if result != 258 || timeout {
            if registered {
                entry.waiters -= 1;
            }
            return result;
        }
        if !registered {
            entry.waiters += 1;
            registered = true;
        }
        drop(state);
        super::thread32::without_cpu(image, || {
            super::window_host32::pump_tasks();
            std::thread::sleep(Duration::from_millis(1));
        });
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_contends_recurses_and_rejects_a_foreign_release() {
        let mut state = Ownership::default();
        assert!(!state.release(7));
        assert_eq!(state.poll(7).unwrap(), 0);
        assert_eq!(state.poll(7).unwrap(), 0);
        assert_eq!(state.poll(9).unwrap(), 258);
        assert!(!state.release(9));
        assert!(state.release(7));
        assert_eq!(state.poll(9).unwrap(), 258);
        assert!(state.release(7));
        assert_eq!(state.poll(9).unwrap(), 0);
        assert!(state.release(9));
    }
    #[test]
    fn real_owner_exit_abandons_once_and_clears_recursive_depth() {
        let mut state = Ownership::default();
        state.poll(7).unwrap();
        state.poll(7).unwrap();
        state.abandon(9);
        assert_eq!(state.poll(9).unwrap(), 258);
        state.abandon(7);
        assert_eq!(state.poll(9).unwrap(), 0x80);
        assert_eq!(state.poll(9).unwrap(), 0);
        assert!(state.release(9));
        assert!(state.release(9));
        assert_eq!(state.poll(7).unwrap(), 0);
    }
}
