//! Checked PE32 unnamed events and counting semaphores backed by real kernel state.
//! Named/security/inheritance profiles and guest-thread waits remain separate.
use super::{LoadedImage32, unsupported};
use crate::Result;
use std::collections::BTreeSet;
// A shared live-object bound, not a lifetime allocation counter. Real kernel
// objects back every slot; CloseHandle releases capacity for either object kind.
const MAX_SYNC_HANDLES: usize = 16_384;
fn capacity_failure() -> u32 {
    alkyhol::ffi::SetLastError(8); // ERROR_NOT_ENOUGH_MEMORY, never successful creation.
    0
}
#[derive(Default)]
pub(super) struct State {
    handles: BTreeSet<u32>,
    semaphores: BTreeSet<u32>,
}
impl State {
    pub(super) fn is_event(&self, h: u32) -> bool {
        self.handles.contains(&h)
    }
    pub(super) fn contains(&self, h: u32) -> bool {
        self.handles.contains(&h) || self.semaphores.contains(&h)
    }
    fn create(&mut self, manual: bool, initial: bool) -> Result<u32> {
        if self.handles.len() + self.semaphores.len() >= MAX_SYNC_HANDLES {
            return Ok(capacity_failure());
        }
        let h = unsafe {
            alkyhol::ffi::CreateEventW(
                std::ptr::null(),
                manual.into(),
                initial.into(),
                std::ptr::null(),
            )
        };
        if h == 0 {
            return Ok(0);
        } // Preserve a real kernel allocation error.
        if h > u32::MAX as usize || (0x70000001..=0x70000003).contains(&h) {
            if h != 0 {
                alkyhol::ffi::CloseHandle(h);
            }
            return Err("event handle is unavailable or outside the checked PE32 namespace".into());
        }
        self.handles.insert(h as u32);
        Ok(h as u32)
    }
    fn semaphore(&mut self, initial: i32, maximum: i32) -> Result<u32> {
        if self.handles.len() + self.semaphores.len() >= MAX_SYNC_HANDLES {
            return Ok(capacity_failure());
        }
        let h = unsafe {
            alkyhol::ffi::CreateSemaphoreW(std::ptr::null(), initial, maximum, std::ptr::null())
        };
        if h == 0 {
            return Ok(0);
        } // Actual checked kernel failure, including invalid signed counts.
        if h > u32::MAX as usize || (0x70000001..=0x70000003).contains(&h) {
            alkyhol::ffi::CloseHandle(h);
            return Err("semaphore handle outside checked PE32 namespace".into());
        }
        self.semaphores.insert(h as u32);
        Ok(h as u32)
    }
    pub(super) fn close(&mut self, h: u32) -> Result<bool> {
        if !self.contains(h) {
            return Ok(false);
        }
        if alkyhol::ffi::CloseHandle(h as usize) == 0 {
            return Err("owned event backing handle was closed externally".into());
        }
        self.handles.remove(&h);
        self.semaphores.remove(&h);
        Ok(true)
    }
    pub(super) fn clear(&mut self) {
        let error = alkyhol::ffi::GetLastError();
        for h in std::mem::take(&mut self.handles)
            .into_iter()
            .chain(std::mem::take(&mut self.semaphores))
        {
            alkyhol::ffi::CloseHandle(h as usize);
        }
        alkyhol::ffi::SetLastError(error);
    }
    fn change(&self, h: u32, set: bool) -> Result<u32> {
        if !self.handles.contains(&h) {
            return Err("event state change requires an owned live event".into());
        }
        let result = if set {
            alkyhol::ffi::SetEvent(h as usize)
        } else {
            alkyhol::ffi::ResetEvent(h as usize)
        };
        if result == 0 {
            return Err("event backing state is no longer owned".into());
        }
        Ok(1)
    }
    pub(super) fn poll_if_owned(&self, h: u32) -> Option<u32> {
        self.contains(h)
            .then(|| alkyhol::ffi::WaitForSingleObject(h as usize, 0))
    }

    #[cfg(test)]
    fn poll(&self, h: u32) -> Result<u32> {
        let result = self
            .poll_if_owned(h)
            .ok_or("event wait requires an owned live event")?;
        if ![0, 258].contains(&result) {
            return Err("event backing wait failed outside its admitted state".into());
        }
        Ok(result)
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.clear();
    }
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    if id == 119 {
        if args[0] != 0 || args[3] != 0 {
            unsupported(
                "PE32 CreateSemaphoreW security attributes and named semaphores are unsupported",
            );
        }
        let allocation = image
            .cpu
            .events
            .write()
            .unwrap()
            .semaphore(args[1] as i32, args[2] as i32);
        // Drop the registry borrow before any fatal diagnostic can terminate
        // while another guest worker is reacquiring the CPU lease.
        let h = allocation.unwrap_or_else(|e| unsupported(&e));
        if h != 0 {
            alkyhol::ffi::SetLastError(0);
        }
        h
    } else if id == 116 {
        if args[0] != 0 || args[3] != 0 {
            unsupported("PE32 CreateEventW security attributes and named events are unsupported");
        }
        let allocation = image
            .cpu
            .events
            .write()
            .unwrap()
            .create(args[1] != 0, args[2] != 0);
        let h = allocation.unwrap_or_else(|e| unsupported(&e));
        if h != 0 {
            alkyhol::ffi::SetLastError(0);
        }
        h
    } else if id == 113 {
        if (args[1] != 0 || args[2] != 0)
            && !image.cpu.threads.borrow().has_workers()
            && !image.cpu.watches.borrow().completion_event(args[0])
            && !super::dsound32::completion_event(image, args[0])
        {
            if image
                .cpu
                .events
                .read()
                .unwrap()
                .semaphores
                .contains(&args[0])
            {
                unsupported("semaphore waits require zero timeout and alertable FALSE");
            }
            unsupported("event waits require zero timeout and alertable FALSE");
        }
        if args[2] != 0 {
            unsupported("alertable event/semaphore waits require APC delivery");
        }
        // No event registry borrow crosses the released CPU lease.
        if !image.cpu.events.read().unwrap().contains(args[0]) {
            unsupported("wait requires an owned event/semaphore");
        }
        super::thread32::wait_handle(image, args[0], args[1])
    } else {
        let result = image.cpu.events.read().unwrap().change(args[0], id == 117);
        result.unwrap_or_else(|e| unsupported(&e))
    }
}
/// Win32 event/thread completion sets do not depend on Winsock initialization.
pub(super) fn wait_set(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let fail = |error| {
        alkyhol::ffi::SetLastError(error);
        u32::MAX
    };
    if !(1..=64).contains(&args[0]) {
        return fail(87);
    }
    if !image.permits(args[1] as usize, args[0] as usize * 4, false) {
        unsupported("wait set array outside readable guest storage");
    }
    let handles: Vec<_> = (0..args[0] as usize)
        .map(|index| unsafe { (args[1] as *const u32).add(index).read_unaligned() as usize })
        .collect();
    for &handle in &handles {
        let state = image.cpu.events.read().unwrap();
        if !state.is_event(handle as u32) && !image.cpu.threads.borrow().contains(handle as u32) {
            if state.contains(handle as u32) {
                unsupported(
                    "event-set wait contains a semaphore; mixed semaphore sets are not implemented",
                );
            }
            return fail(6);
        }
    }
    match super::thread32::wait_event_set(image, &handles, args[2] != 0, args[3]) {
        Ok(result) => result,
        Err(error) if error.0 == 50 => {
            unsupported("wait set contains an unsupported event alias or object kind")
        }
        Err(error) => fail(error.0),
    }
}
pub(super) fn release(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    if !image
        .cpu
        .events
        .read()
        .unwrap()
        .semaphores
        .contains(&args[0])
    {
        unsupported("semaphore release requires an owned live counting semaphore");
    }
    if args[2] != 0 && (!args[2].is_multiple_of(4) || !image.permits(args[2] as usize, 4, true)) {
        unsupported("semaphore previous-count output outside writable guest storage");
    }
    let mut previous = 0i32;
    let result =
        unsafe { alkyhol::ffi::ReleaseSemaphore(args[0] as usize, args[1] as i32, &mut previous) }
            as u32;
    if result != 0 && args[2] != 0 {
        unsafe { (args[2] as *mut i32).write(previous) };
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn live_capacity_is_shared_and_recovered_without_disturbing_remaining_objects() {
        let mut state = State::default();
        let mut events = Vec::new();
        for _ in 0..MAX_SYNC_HANDLES - 1 {
            let h = state.create(true, false).unwrap();
            assert_ne!(h, 0);
            events.push(h);
        }
        let semaphore = state.semaphore(1, 2).unwrap();
        assert_ne!(semaphore, 0);
        assert_eq!(state.create(false, true).unwrap(), 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 8);
        assert_eq!(state.semaphore(1, 2).unwrap(), 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 8);
        // Every slot has a distinct, actual kernel object; no dropped allocation.
        assert_eq!(
            state.handles.len() + state.semaphores.len(),
            MAX_SYNC_HANDLES
        );
        assert_eq!(state.poll(events[129]).unwrap(), 258);
        state.change(events[129], true).unwrap();
        assert_eq!(state.poll(events[129]).unwrap(), 0);
        for h in events.iter().step_by(2) {
            assert!(state.close(*h).unwrap());
        }
        let available = MAX_SYNC_HANDLES - state.handles.len() - state.semaphores.len();
        for _ in 0..available {
            assert_ne!(state.semaphore(1, 1).unwrap(), 0);
        }
        assert_eq!(state.poll(events[129]).unwrap(), 0);
        assert_eq!(state.poll(semaphore).unwrap(), 0);
        assert_eq!(state.poll(semaphore).unwrap(), 258);
        assert_eq!(state.create(true, false).unwrap(), 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 8);
        assert!(state.close(semaphore).unwrap());
        let replacement = state.create(false, true).unwrap();
        assert_ne!(replacement, 0);
        assert_eq!(state.poll(replacement).unwrap(), 0);
        assert_eq!(state.poll(replacement).unwrap(), 258);
        alkyhol::ffi::SetLastError(0xface1234);
        state.clear();
        assert_eq!(alkyhol::ffi::GetLastError(), 0xface1234);
        assert!(state.handles.is_empty() && state.semaphores.is_empty());
        assert!(state.poll(replacement).is_err());
    }
    #[test]
    fn real_manual_and_automatic_state_close_and_kind_ownership() {
        let mut state = State::default();
        let manual = state.create(true, false).unwrap();
        let automatic = state.create(false, true).unwrap();
        assert_ne!(manual, automatic);
        assert_eq!(state.poll(manual).unwrap(), 258);
        assert_eq!(state.poll(automatic).unwrap(), 0);
        assert_eq!(state.poll(automatic).unwrap(), 258);
        assert_eq!(state.change(manual, true).unwrap(), 1);
        assert_eq!(state.change(manual, true).unwrap(), 1);
        assert_eq!(state.poll(manual).unwrap(), 0);
        assert_eq!(state.poll(manual).unwrap(), 0);
        assert_eq!(state.change(manual, false).unwrap(), 1);
        assert_eq!(state.poll(manual).unwrap(), 258);
        assert!(state.close(manual).unwrap());
        assert!(!state.close(manual).unwrap());
        assert!(state.poll(manual).is_err());
        assert!(state.change(manual, true).is_err());
        state.clear();
        assert!(state.poll(automatic).is_err());
    }
    #[test]
    fn concurrent_pollers_consume_one_automatic_event_signal() {
        use std::sync::{Arc, Barrier, RwLock};

        let mut state = State::default();
        let event = state.create(false, true).unwrap();
        let state = Arc::new(RwLock::new(state));
        let ready = Arc::new(Barrier::new(2));
        let workers = std::array::from_fn::<_, 2, _>(|_| {
            let state = Arc::clone(&state);
            let ready = Arc::clone(&ready);
            std::thread::spawn(move || {
                ready.wait();
                state.read().unwrap().poll_if_owned(event).unwrap()
            })
        });
        let mut results = workers.map(|worker| worker.join().unwrap());
        results.sort_unstable();
        assert_eq!(results, [0, 258]);
        state.write().unwrap().close(event).unwrap();
        assert_eq!(state.read().unwrap().poll_if_owned(event), None);
    }

    #[test]
    fn counting_semaphores_consume_real_permits_and_preserve_type_and_signed_ranges() {
        let mut state = State::default();
        let h = state.semaphore(2, 3).unwrap();
        assert_ne!(h, 0);
        assert_eq!(state.poll(h).unwrap(), 0);
        assert_eq!(state.poll(h).unwrap(), 0);
        assert_eq!(state.poll(h).unwrap(), 258);
        assert!(state.change(h, true).is_err());
        assert_eq!(
            unsafe { alkyhol::ffi::ReleaseSemaphore(h as usize, 3, std::ptr::null_mut()) },
            1
        );
        for _ in 0..3 {
            assert_eq!(state.poll(h).unwrap(), 0);
        }
        assert_eq!(state.poll(h).unwrap(), 258);
        for (initial, max) in [(-1, 3), (4, 3), (0, 0)] {
            assert_eq!(state.semaphore(initial, max).unwrap(), 0);
            assert_eq!(alkyhol::ffi::GetLastError(), 87);
        }
        assert!(state.close(h).unwrap());
        assert!(state.poll(h).is_err());
    }
}
