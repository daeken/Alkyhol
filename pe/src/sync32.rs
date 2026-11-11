//! Recursive critical sections with actual guest-thread ownership and waits.
//! Each object synchronizes its own state. Contended waits release the CPU lease.
use super::{LoadedImage32, unsupported};
use std::{
    ptr,
    sync::{Arc, Condvar, Mutex},
    time::Duration,
};

pub(super) struct Section {
    depth: u32,
    owner: u32,
    spin: u32,
}
impl Section {
    pub(super) fn new(spin: u32) -> Self {
        Self {
            depth: 0,
            owner: 0,
            spin,
        }
    }
    pub(super) fn enter(&mut self, thread: u32) -> crate::Result<()> {
        if self.depth != 0 && self.owner != thread {
            return Err("contended PE32 critical section requires guest thread scheduling".into());
        }
        self.depth = self
            .depth
            .checked_add(1)
            .filter(|&v| v <= i32::MAX as u32)
            .ok_or("PE32 critical section recursion overflow")?;
        self.owner = thread;
        Ok(())
    }
    pub(super) fn leave(&mut self, thread: u32) -> crate::Result<()> {
        if self.depth == 0 || self.owner != thread {
            return Err("PE32 critical section leave without ownership".into());
        }
        self.depth -= 1;
        if self.depth == 0 {
            self.owner = 0;
        }
        Ok(())
    }
    unsafe fn write(&self, address: *mut u32) {
        // The public Win32 width is 24 bytes. Debug/semaphore extensions are not
        // exposed; copying/modifying a live object is unsupported by Win32 too.
        let words = [
            0,
            self.depth.wrapping_sub(1),
            self.depth,
            self.owner,
            0,
            self.spin,
        ];
        unsafe {
            ptr::copy_nonoverlapping(words.as_ptr(), address, words.len());
        }
    }
}
pub(super) struct CriticalSection {
    ownership: Mutex<Section>,
    released: Condvar,
}
impl CriticalSection {
    fn new(spin: u32) -> Self {
        Self {
            ownership: Mutex::new(Section::new(spin)),
            released: Condvar::new(),
        }
    }

    unsafe fn try_enter(&self, thread: u32, address: *mut u32) -> crate::Result<bool> {
        let mut state = self.ownership.lock().unwrap();
        if state.depth != 0 && state.owner != thread {
            return Ok(false);
        }
        state.enter(thread)?;
        unsafe { state.write(address) };
        Ok(true)
    }

    unsafe fn leave(&self, thread: u32, address: *mut u32) -> crate::Result<()> {
        let mut state = self.ownership.lock().unwrap();
        state.leave(thread)?;
        unsafe { state.write(address) };
        if state.depth == 0 {
            self.released.notify_one();
        }
        Ok(())
    }

    fn wait_until_unowned(&self) {
        let mut state = self.ownership.lock().unwrap();
        let owned = |state: &mut Section| state.depth != 0;
        if !super::window_host32::is_main_thread() {
            drop(self.released.wait_while(state, owned).unwrap());
            return;
        }
        // The section owner may need synchronous AppKit work from this thread.
        while owned(&mut state) {
            (state, _) = self
                .released
                .wait_timeout_while(state, Duration::from_millis(1), owned)
                .unwrap();
            drop(state);
            super::window_host32::pump_tasks();
            state = self.ownership.lock().unwrap();
        }
    }
}

pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    let address = args[0];
    if matches!(id, 38 | 51 | 75 | 76) {
        if !address.is_multiple_of(4) || !image.permits(address as usize, 4, true) {
            unsupported("PE32 interlocked target outside writable aligned guest storage");
        }
        let atomic = unsafe { std::sync::atomic::AtomicU32::from_ptr(address as *mut u32) };
        if id == 75 {
            return compare_exchange(atomic, args[1], args[2]);
        }
        if id == 76 {
            return atomic.swap(args[1], std::sync::atomic::Ordering::SeqCst);
        }
        return if id == 38 {
            atomic
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                .wrapping_add(1)
        } else {
            atomic
                .fetch_sub(1, std::sync::atomic::Ordering::SeqCst)
                .wrapping_sub(1)
        };
    }
    if !address.is_multiple_of(4) || !image.permits(address as usize, 24, true) {
        unsupported("PE32 critical section outside writable aligned guest storage");
    }
    let mut sections = image.cpu.critical_sections.lock().unwrap();
    if id == 32 || id == 33 {
        if sections.contains_key(&address) {
            unsupported("PE32 critical section already initialized");
        }
        let mut section = CriticalSection::new(if id == 32 { args[1] } else { 0 });
        unsafe {
            section
                .ownership
                .get_mut()
                .unwrap()
                .write(address as *mut u32)
        };
        sections.insert(address, Arc::new(section));
        return 1;
    }
    let entry = sections
        .get_mut(&address)
        .unwrap_or_else(|| unsupported("unknown/uninitialized PE32 critical section"));
    if id == 36 {
        let section = Arc::get_mut(entry).unwrap_or_else(|| {
            unsupported("deleting a PE32 critical section during an active operation")
        });
        if section.ownership.get_mut().unwrap().depth != 0 {
            unsupported("deleting an owned PE32 critical section");
        }
        sections.remove(&address);
        return 0;
    }
    let section = Arc::clone(entry);
    drop(sections);
    let thread = alkyhol::ffi::GetCurrentThreadId();
    if id == 35 {
        unsafe { section.leave(thread, address as *mut u32) }.unwrap_or_else(|e| unsupported(&e));
        return 1;
    }
    debug_assert!(matches!(id, 34 | 37));
    loop {
        let acquired = unsafe { section.try_enter(thread, address as *mut u32) }
            .unwrap_or_else(|e| unsupported(&e));
        if acquired || id == 37 {
            return acquired as u32;
        }
        super::thread32::without_cpu(image, || section.wait_until_unowned());
        if !image.permits(address as usize, 24, true) {
            unsupported("PE32 critical section storage disappeared during a pending wait");
        }
    }
}

fn compare_exchange(value: &std::sync::atomic::AtomicU32, exchange: u32, comparand: u32) -> u32 {
    match value.compare_exchange(
        comparand,
        exchange,
        std::sync::atomic::Ordering::SeqCst,
        std::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(old) | Err(old) => old,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compare_exchange_returns_old_bits_on_success_and_failure() {
        let value = std::sync::atomic::AtomicU32::new(0);
        assert_eq!(compare_exchange(&value, u32::MAX, 0), 0);
        assert_eq!(compare_exchange(&value, 7, 0), u32::MAX);
        assert_eq!(value.load(std::sync::atomic::Ordering::SeqCst), u32::MAX);
        assert_eq!(compare_exchange(&value, 0x80000000, u32::MAX), u32::MAX);
        assert_eq!(value.load(std::sync::atomic::Ordering::SeqCst), 0x80000000);
        assert_eq!(
            value.swap(0, std::sync::atomic::Ordering::SeqCst),
            0x80000000
        );
        assert_eq!(value.load(std::sync::atomic::Ordering::SeqCst), 0);
    }
    #[test]
    fn ownership_recursion_and_unbalanced_operations_are_checked() {
        let mut s = Section::new(4000);
        assert!(s.leave(7).is_err());
        s.enter(7).unwrap();
        s.enter(7).unwrap();
        assert_eq!(s.depth, 2);
        assert!(s.enter(8).is_err());
        assert!(s.leave(8).is_err());
        s.leave(7).unwrap();
        assert_eq!(s.owner, 7);
        s.leave(7).unwrap();
        assert_eq!((s.depth, s.owner), (0, 0));
        s.enter(8).unwrap();
        assert_eq!(s.owner, 8);
    }

    #[test]
    fn recursive_owner_wakes_native_waiter_only_on_final_release() {
        let section = Arc::new(CriticalSection::new(4000));
        let mut mirror = [0; 6];
        assert!(unsafe { section.try_enter(7, mirror.as_mut_ptr()) }.unwrap());
        assert!(unsafe { section.try_enter(7, mirror.as_mut_ptr()) }.unwrap());
        assert_eq!(&mirror[1..4], &[1, 2, 7]);

        let worker_section = Arc::clone(&section);
        let (contended, started) = std::sync::mpsc::channel();
        let (acquired, finished) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let mut mirror = [0; 6];
            assert!(!unsafe { worker_section.try_enter(9, mirror.as_mut_ptr()) }.unwrap());
            contended.send(()).unwrap();
            worker_section.wait_until_unowned();
            assert!(unsafe { worker_section.try_enter(9, mirror.as_mut_ptr()) }.unwrap());
            assert_eq!(&mirror[1..4], &[0, 1, 9]);
            unsafe { worker_section.leave(9, mirror.as_mut_ptr()) }.unwrap();
            acquired.send(()).unwrap();
        });

        started.recv_timeout(Duration::from_secs(2)).unwrap();
        unsafe { section.leave(7, mirror.as_mut_ptr()) }.unwrap();
        assert_eq!(&mirror[1..4], &[0, 1, 7]);
        assert!(matches!(
            finished.recv_timeout(Duration::from_millis(20)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        unsafe { section.leave(7, mirror.as_mut_ptr()) }.unwrap();
        assert_eq!(&mirror[1..4], &[u32::MAX, 0, 0]);
        finished.recv_timeout(Duration::from_secs(2)).unwrap();
        worker.join().unwrap();
    }
}
