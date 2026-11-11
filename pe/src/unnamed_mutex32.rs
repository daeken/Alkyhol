//! Unnamed process-local mutexes with independently synchronized ownership.
//! Native worker exit abandons owned mutexes before signaling completion.
use super::{LoadedImage32, LowMapping, PAGE, unsupported};
use crate::Result;
use std::{
    collections::BTreeMap,
    sync::{Arc, Condvar, Mutex},
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
    fn abandon(&mut self, thread: u32) -> bool {
        if self.depth == 0 || self.owner != thread {
            return false;
        }
        self.depth = 0;
        self.owner = 0;
        self.abandoned = true;
        true
    }
}
#[derive(Default)]
struct Waitable {
    ownership: Mutex<Ownership>,
    available: Condvar,
}
impl Waitable {
    fn poll(&self, thread: u32) -> Result<u32> {
        self.ownership.lock().unwrap().poll(thread)
    }

    fn wait(&self, thread: u32, deadline: Option<Instant>) -> Result<u32> {
        let main = super::window_host32::is_main_thread();
        let mut ownership = self.ownership.lock().unwrap();
        loop {
            let result = ownership.poll(thread)?;
            if result != 258 {
                return Ok(result);
            }
            let remaining = deadline.map(|end| end.saturating_duration_since(Instant::now()));
            if remaining.is_some_and(|duration| duration.is_zero()) {
                return Ok(258);
            }
            if main {
                // The owner may require synchronous AppKit work to release.
                let slice = Duration::from_millis(1);
                (ownership, _) = self
                    .available
                    .wait_timeout(ownership, remaining.map_or(slice, |time| time.min(slice)))
                    .unwrap();
                drop(ownership);
                super::window_host32::pump_tasks();
                ownership = self.ownership.lock().unwrap();
            } else if let Some(remaining) = remaining {
                (ownership, _) = self.available.wait_timeout(ownership, remaining).unwrap();
            } else {
                ownership = self.available.wait(ownership).unwrap();
            }
        }
    }

    fn release(&self, thread: u32) -> bool {
        let mut ownership = self.ownership.lock().unwrap();
        let released = ownership.release(thread);
        if released && ownership.depth == 0 {
            self.available.notify_one();
        }
        released
    }

    fn abandon(&self, thread: u32) {
        if self.ownership.lock().unwrap().abandon(thread) {
            self.available.notify_one();
        }
    }
}

struct Entry {
    _token: LowMapping,
    state: Arc<Waitable>,
}
#[derive(Default)]
pub(super) struct State {
    entries: Mutex<BTreeMap<u32, Entry>>,
}
impl State {
    pub(super) fn contains(&self, handle: u32) -> bool {
        self.entries.lock().unwrap().contains_key(&handle)
    }
    pub(super) fn clear(&self) {
        self.entries.lock().unwrap().clear();
    }
    pub(super) fn abandon(&self, thread: u32) {
        for entry in self.entries.lock().unwrap().values() {
            entry.state.abandon(thread);
        }
    }
    pub(super) fn close(&self, handle: u32) -> Result<bool> {
        let mut entries = self.entries.lock().unwrap();
        if entries
            .get_mut(&handle)
            .is_some_and(|entry| Arc::get_mut(&mut entry.state).is_none())
        {
            return Err(
                "closing a mutex during an active operation is outside the defined profile".into(),
            );
        }
        Ok(entries.remove(&handle).is_some())
    }
}

pub(super) fn create(image: &LoadedImage32<'_>, initial: bool) -> u32 {
    let mut entries = image.cpu.unnamed_mutexes.entries.lock().unwrap();
    if entries.len() >= 1024 {
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
    let state = Arc::new(Waitable::default());
    if initial {
        state.poll(alkyhol::ffi::GetCurrentThreadId()).unwrap();
    }
    entries.insert(
        handle,
        Entry {
            _token: token,
            state,
        },
    );
    alkyhol::ffi::SetLastError(0);
    handle
}

pub(super) fn operation(image: &LoadedImage32<'_>, args: &[u32; 8], wait: bool) -> u32 {
    let thread = alkyhol::ffi::GetCurrentThreadId();
    let state = Arc::clone(
        &image
            .cpu
            .unnamed_mutexes
            .entries
            .lock()
            .unwrap()
            .get(&args[0])
            .unwrap_or_else(|| unsupported("unknown unnamed mutex handle"))
            .state,
    );
    if !wait {
        let released = state.release(thread);
        if !released {
            alkyhol::ffi::SetLastError(288);
        }
        return released as u32;
    }
    if args[2] != 0 {
        unsupported("alertable unnamed mutex waits require APC delivery");
    }
    let start = Instant::now();
    let result = state.poll(thread).unwrap_or_else(|e| unsupported(&e));
    if result != 258 || args[1] == 0 {
        return result;
    }
    let deadline = (args[1] != u32::MAX).then(|| start + Duration::from_millis(u64::from(args[1])));
    super::thread32::without_cpu(image, || state.wait(thread, deadline))
        .unwrap_or_else(|e| unsupported(&e))
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
