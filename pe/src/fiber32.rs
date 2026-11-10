//! One guest thread's initial fiber. Fiber switching/thread creation remain
//! unsupported. Values are opaque DWORDs, never dereferenced as native pointers.
use super::{LoadedImage32, unsupported};
use alkyhol::ffi;

#[derive(Clone)]
struct Slot {
    callback: u32,
    values: std::collections::BTreeMap<u32, u32>,
    freeing: bool,
}
#[derive(Default)]
pub(super) struct Slots(Vec<Option<Slot>>);
impl Slots {
    fn allocate(&mut self, callback: u32) -> u32 {
        let index = self
            .0
            .iter()
            .position(Option::is_none)
            .unwrap_or(self.0.len());
        if index == 128 {
            ffi::SetLastError(8);
            return u32::MAX;
        }
        let slot = Some(Slot {
            callback,
            values: Default::default(),
            freeing: false,
        });
        if index == self.0.len() {
            self.0.push(slot);
        } else {
            self.0[index] = slot;
        }
        index as u32
    }
    fn get(&mut self, index: u32) -> Option<&mut Slot> {
        self.0
            .get_mut(index as usize)?
            .as_mut()
            .filter(|s| !s.freeing)
    }
    pub(super) fn pending_callbacks(&self) -> bool {
        self.0
            .iter()
            .flatten()
            .any(|s| s.callback != 0 && s.values.values().any(|&v| v != 0))
    }
    pub(super) fn clear(&mut self) {
        self.0.clear();
    }
}
fn fail() -> u32 {
    ffi::SetLastError(87);
    0
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    if (23..=25).contains(&id)
        && image
            .cpu
            .fls
            .borrow()
            .0
            .get(args[0] as usize)
            .and_then(Option::as_ref)
            .is_some_and(|slot| slot.freeing)
    {
        unsupported("reentrant access to the FLS index being freed is unsupported");
    }
    match id {
        22 => {
            let callback = args[0];
            if callback != 0
                && !image.regions.iter().any(|&(first, last, rights)| {
                    rights & 0x20000000 != 0
                        && super::host_code_address(callback) >= first
                        && super::host_code_address(callback) < last
                })
            {
                unsupported("FlsAlloc callback outside executable guest storage");
            }
            image.cpu.fls.borrow_mut().allocate(callback)
        }
        23 => {
            let slot = {
                let mut slots = image.cpu.fls.borrow_mut();
                let Some(slot) = slots.get(args[0]) else {
                    return fail();
                };
                slot.freeing = true;
                slot.clone()
            };
            if slot.callback != 0 {
                for value in slot.values.values().copied().filter(|&v| v != 0) {
                    unsafe { image.callback(slot.callback, value) }
                        .unwrap_or_else(|e| unsupported(&e));
                }
            }
            image.cpu.fls.borrow_mut().0[args[0] as usize] = None;
            1
        }
        24 => {
            let mut slots = image.cpu.fls.borrow_mut();
            let Some(slot) = slots.get(args[0]) else {
                return fail();
            };
            ffi::SetLastError(0);
            slot.values
                .get(&ffi::GetCurrentThreadId())
                .copied()
                .unwrap_or(0)
        }
        25 => {
            let mut slots = image.cpu.fls.borrow_mut();
            let Some(slot) = slots.get(args[0]) else {
                return fail();
            };
            slot.values.insert(ffi::GetCurrentThreadId(), args[1]);
            1
        }
        26 => {
            let mut slots = image.cpu.dynamic_tls.borrow_mut();
            let Some(index) = slots.iter().position(|&used| !used) else {
                ffi::SetLastError(8);
                return u32::MAX;
            };
            slots[index] = true;
            super::thread32::clear_tls(image, index);
            index as u32
        }
        27..=29 => {
            let index = args[0] as usize;
            let mut slots = image.cpu.dynamic_tls.borrow_mut();
            if index >= slots.len() || !slots[index] {
                return fail();
            }
            let value = unsafe { image.cpu.teb().base.add(0xe10 + index * 4).cast::<u32>() };
            match id {
                27 => {
                    slots[index] = false;
                    super::thread32::clear_tls(image, index);
                    1
                }
                28 => {
                    ffi::SetLastError(0);
                    unsafe { value.read() }
                }
                29 => {
                    unsafe {
                        value.write(args[1]);
                    }
                    1
                }
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

pub(super) fn thread_exit(image: &LoadedImage32<'_>) {
    let id = ffi::GetCurrentThreadId();
    let pending: Vec<_> = {
        let mut slots = image.cpu.fls.borrow_mut();
        slots
            .0
            .iter_mut()
            .flatten()
            .filter_map(|s| {
                s.values
                    .remove(&id)
                    .filter(|&v| v != 0 && s.callback != 0)
                    .map(|v| (s.callback, v))
            })
            .collect()
    };
    for (callback, value) in pending {
        unsafe { image.callback(callback, value) }.unwrap_or_else(|e| unsupported(&e));
    }
    if image
        .cpu
        .fls
        .borrow()
        .0
        .iter()
        .flatten()
        .any(|s| s.values.get(&id).is_some_and(|&v| v != 0))
    {
        unsupported("FLS callbacks rearmed values during worker exit");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_slots_are_zero_initialized_and_reuse_freed_indices() {
        let mut slots = Slots::default();
        for index in 0..128 {
            assert_eq!(slots.allocate(42), index);
            assert!(slots.get(index).unwrap().values.is_empty());
        }
        assert_eq!(slots.allocate(0), u32::MAX);
        assert_eq!(ffi::GetLastError(), 8);
        assert!(slots.get(128).is_none());
        slots.get(19).unwrap().values.insert(7, 33);
        assert!(slots.pending_callbacks());
        slots.0[19] = None;
        assert_eq!(slots.allocate(0), 19);
        assert!(!slots.pending_callbacks());
        slots.clear();
        assert!(slots.get(0).is_none());
    }
}
