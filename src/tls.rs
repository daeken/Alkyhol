//! Dynamic process TLS slots for native host threads, separate from PE static TLS/FLS.
use crate::kernel::{ERROR_INVALID_PARAMETER, ERROR_NOT_ENOUGH_MEMORY, WinError, WinResult};
use std::{cell::RefCell, sync::Mutex};

pub const TLS_CAPACITY: usize = 1088;
pub const TLS_OUT_OF_INDEXES: u32 = u32::MAX;

#[derive(Clone, Copy)]
struct Slot {
    allocated: bool,
    generation: u64,
}
static SLOTS: Mutex<[Slot; TLS_CAPACITY]> = Mutex::new(
    [Slot {
        allocated: false,
        generation: 0,
    }; TLS_CAPACITY],
);
thread_local! {
    // Fixed size bounds per-thread metadata. Generation mismatch means NULL after
    // free/reallocation, including in threads that already existed at allocation.
    static VALUES: RefCell<[(u64, usize); TLS_CAPACITY]> = const { RefCell::new([(0, 0); TLS_CAPACITY]) };
}

fn generation(slots: &[Slot], index: u32) -> WinResult<u64> {
    slots
        .get(index as usize)
        .filter(|slot| slot.allocated)
        .map(|slot| slot.generation)
        .ok_or(WinError(ERROR_INVALID_PARAMETER))
}

pub fn alloc() -> WinResult<u32> {
    let mut slots = SLOTS.lock().unwrap();
    let (index, slot) = slots
        .iter_mut()
        .enumerate()
        .find(|(_, slot)| !slot.allocated && slot.generation < u64::MAX)
        .ok_or(WinError(ERROR_NOT_ENOUGH_MEMORY))?;
    slot.generation += 1;
    slot.allocated = true;
    Ok(index as u32)
}

pub fn free(index: u32) -> WinResult<()> {
    let mut slots = SLOTS.lock().unwrap();
    generation(&*slots, index)?;
    slots[index as usize].allocated = false;
    // Stored addresses are borrowed opaque values, never allocations owned by TLS.
    Ok(())
}

pub fn get(index: u32) -> WinResult<usize> {
    let slots = SLOTS.lock().unwrap();
    let current = generation(&*slots, index)?;
    Ok(VALUES.with(|values| {
        let (stored, value) = values.borrow()[index as usize];
        if stored == current { value } else { 0 }
    }))
}

pub fn set(index: u32, value: usize) -> WinResult<()> {
    let slots = SLOTS.lock().unwrap();
    let current = generation(&*slots, index)?;
    VALUES.with(|values| values.borrow_mut()[index as usize] = (current, value));
    Ok(())
}
