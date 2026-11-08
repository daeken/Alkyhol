//! Recursive critical sections for the single admitted guest thread. The CPU
//! lease excludes concurrent entries; guest thread creation remains unsupported.
//! Track real ownership/depth rather than treating synchronization as a no-op.
use super::{LoadedImage32, unsupported};
use std::ptr;
pub(super) struct Section {
    depth: u32,
    owner: u32,
    spin: u32,
}
impl Section {
    fn enter(&mut self, thread: u32) -> crate::Result<()> {
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
    fn leave(&mut self, thread: u32) -> crate::Result<()> {
        if self.depth == 0 || self.owner != thread {
            return Err("PE32 critical section leave without ownership".into());
        }
        self.depth -= 1;
        if self.depth == 0 {
            self.owner = 0;
        }
        Ok(())
    }
    fn write(&self, address: u32) {
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
            ptr::copy_nonoverlapping(words.as_ptr(), address as *mut u32, words.len());
        }
    }
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    let address = args[0];
    if id == 38 {
        if !address.is_multiple_of(4) || !image.permits(address as usize, 4, true) {
            unsupported("PE32 interlocked target outside writable aligned guest storage");
        }
        let atomic = unsafe { std::sync::atomic::AtomicU32::from_ptr(address as *mut u32) };
        return atomic
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            .wrapping_add(1);
    }
    if !address.is_multiple_of(4) || !image.permits(address as usize, 24, true) {
        unsupported("PE32 critical section outside writable aligned guest storage");
    }
    let mut sections = image.cpu.critical_sections.borrow_mut();
    if id == 32 || id == 33 {
        if sections.contains_key(&address) {
            unsupported("PE32 critical section already initialized");
        }
        let section = Section {
            depth: 0,
            owner: 0,
            spin: if id == 32 { args[1] } else { 0 },
        };
        section.write(address);
        sections.insert(address, section);
        return 1;
    }
    let section = sections
        .get_mut(&address)
        .unwrap_or_else(|| unsupported("unknown/uninitialized PE32 critical section"));
    let thread = alkyhol::ffi::GetCurrentThreadId();
    match id {
        34 | 37 => section.enter(thread).unwrap_or_else(|e| unsupported(&e)),
        35 => section.leave(thread).unwrap_or_else(|e| unsupported(&e)),
        36 => {
            if section.depth != 0 {
                unsupported("deleting an owned PE32 critical section");
            }
            sections.remove(&address);
            return 0;
        }
        _ => unreachable!(),
    }
    section.write(address);
    1
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ownership_recursion_and_unbalanced_operations_are_checked() {
        let mut s = Section {
            depth: 0,
            owner: 0,
            spin: 4000,
        };
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
}
