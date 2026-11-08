//! Checked PE32 unnamed events backed by Alkyhol's real event state machine.
//! Named/security/inheritance profiles and guest-thread waits remain separate.
use super::{LoadedImage32, unsupported};
use crate::Result;
use std::collections::BTreeSet;
#[derive(Default)]
pub(super) struct State {
    handles: BTreeSet<u32>,
}
impl State {
    pub(super) fn contains(&self, h: u32) -> bool {
        self.handles.contains(&h)
    }
    fn create(&mut self, manual: bool, initial: bool) -> Result<u32> {
        if self.handles.len() >= 128 {
            return Err("PE32 event handle budget exceeded".into());
        }
        let h = unsafe {
            alkyhol::ffi::CreateEventW(
                std::ptr::null(),
                manual.into(),
                initial.into(),
                std::ptr::null(),
            )
        };
        if h == 0 || h > u32::MAX as usize || (0x70000001..=0x70000003).contains(&h) {
            if h != 0 {
                alkyhol::ffi::CloseHandle(h);
            }
            return Err("event handle is unavailable or outside the checked PE32 namespace".into());
        }
        self.handles.insert(h as u32);
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
        Ok(true)
    }
    pub(super) fn clear(&mut self) {
        let error = alkyhol::ffi::GetLastError();
        for h in std::mem::take(&mut self.handles) {
            alkyhol::ffi::CloseHandle(h as usize);
        }
        alkyhol::ffi::SetLastError(error);
    }
    fn change(&self, h: u32, set: bool) -> Result<u32> {
        if !self.contains(h) {
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
    fn poll(&self, h: u32) -> Result<u32> {
        if !self.contains(h) {
            return Err("event wait requires an owned live event".into());
        }
        let result = alkyhol::ffi::WaitForSingleObject(h as usize, 0);
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
    if id == 116 {
        if args[0] != 0 || args[3] != 0 {
            unsupported("PE32 CreateEventW security attributes and named events are unsupported");
        }
        let h = image
            .cpu
            .events
            .borrow_mut()
            .create(args[1] != 0, args[2] != 0)
            .unwrap_or_else(|e| unsupported(&e));
        alkyhol::ffi::SetLastError(0);
        h
    } else if id == 113 {
        if args[1] != 0 || args[2] != 0 {
            unsupported("event waits require zero timeout and alertable FALSE");
        }
        image
            .cpu
            .events
            .borrow()
            .poll(args[0])
            .unwrap_or_else(|e| unsupported(&e))
    } else {
        image
            .cpu
            .events
            .borrow()
            .change(args[0], id == 117)
            .unwrap_or_else(|e| unsupported(&e))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
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
}
