//! Bounded system CRT calls reached by DAO's original dependency startup.
//! Allocations are real guest storage, isolated from the process default heap.
use super::{LoadedImage32, unsupported};
#[derive(Default)]
pub(super) struct State {
    heap: Option<u32>,
    error_mode: u32,
    terminate: u32,
    unexpected: u32,
    app_type: u32,
    errno: std::collections::BTreeMap<u32, u32>,
    locks: std::collections::BTreeMap<u32, super::sync32::Section>,
}
impl State {
    fn lock(&mut self, index: u32, thread: u32, enter: bool) -> crate::Result<()> {
        if index != 8 {
            return Err("CRT lock index outside the observed onexit lock profile".into());
        }
        if enter {
            self.locks
                .entry(index)
                .or_insert_with(|| super::sync32::Section::new(0))
                .enter(thread)
        } else {
            self.locks
                .get_mut(&index)
                .ok_or("CRT unlock requires an owned initialized lock")?
                .leave(thread)
        }
    }
    fn set_error_mode(&mut self, mode: u32) -> crate::Result<u32> {
        let old = self.error_mode;
        match mode {
            0..=2 => self.error_mode = mode,
            3 => (), // _REPORT_ERRMODE queries without changing the setting.
            _ => return Err("CRT invalid-parameter/errno handling is unsupported".into()),
        }
        Ok(old)
    }
}
fn abort_unsupported(image: &LoadedImage32<'_>, branch: &str) -> ! {
    let state = image.cpu.crt.borrow();
    unsupported(&format!(
        "{branch} requires unsupported abort/reporting behavior (error_mode={}, app_type={})",
        state.error_mode, state.app_type
    ));
}
pub(super) fn initializers(image: &LoadedImage32<'_>, begin: u32, end: u32) -> u32 {
    let size = end
        .checked_sub(begin)
        .filter(|&n| n <= 4096 * 4)
        .unwrap_or_else(|| unsupported("CRT initializer table range or budget is unsupported"));
    if !begin.is_multiple_of(4) || !end.is_multiple_of(4) {
        unsupported("CRT initializer table is not DWORD aligned");
    }
    if size == 0 {
        return 0;
    }
    if !image.permits(begin as usize, size as usize, false) {
        unsupported("CRT initializer table outside readable guest storage");
    }
    for entry in (begin..end).step_by(4) {
        // Read in execution order: an initializer can modify a later entry.
        let callback = unsafe { (entry as *const u32).read() };
        if callback == 0 {
            continue;
        }
        let result =
            unsafe { image.callback_frame(callback, &[], super::CallingConvention32::Cdecl) }
                .unwrap_or_else(|e| unsupported(&e));
        if result.is_none() {
            return image.cpu.exit.get().unwrap_or_else(|| {
                unsupported("initializer exited without an actual guest exit status")
            });
        }
    }
    0 // void return; actual callbacks and their side effects are the operation.
}
pub(super) fn dll_onexit(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    if args[0] == 0
        || !image.regions.iter().any(|&(first, last, rights)| {
            rights & 0x20000000 != 0 && args[0] as usize >= first && (args[0] as usize) < last
        })
    {
        unsupported("DLL onexit callback outside executable guest storage");
    }
    if args[1] == args[2]
        || [args[1], args[2]]
            .iter()
            .any(|&p| !p.is_multiple_of(4) || !image.permits(p as usize, 4, true))
    {
        unsupported("DLL onexit output cells outside separate writable guest storage");
    }
    let begin = unsafe { (args[1] as *const u32).read() };
    let end = unsafe { (args[2] as *const u32).read() };
    if begin == 0 || end == 0 {
        unsupported(
            "DLL onexit requires a preallocated owned table; null-table behavior is unverified",
        );
    }
    let size = end
        .checked_sub(begin)
        .filter(|&n| n.is_multiple_of(4) && n < 4096 * 4)
        .unwrap_or_else(|| unsupported("DLL onexit table range or budget is unsupported"));
    if !begin.is_multiple_of(4) || (begin == 0 && end != 0) {
        unsupported("invalid DLL onexit table endpoints");
    }
    let mut state = image.cpu.crt.borrow_mut();
    let mut heap = image.cpu.heap.borrow_mut();
    let handle = if let Some(h) = state.heap {
        h
    } else {
        let h = heap.create(0, 0, 0).unwrap_or_else(|e| unsupported(&e));
        if h == 0 {
            unsupported("DLL onexit heap creation failed");
        }
        state.heap = Some(h);
        h
    };
    if begin != 0 {
        let allocated = heap
            .size(handle, 0, begin)
            .unwrap_or_else(|e| unsupported(&e));
        if size > allocated {
            unsupported("DLL onexit entries exceed the owned allocation");
        }
        for output in [args[1], args[2]] {
            if (output as u64) < u64::from(begin) + u64::from(allocated)
                && (output as u64) + 4 > u64::from(begin)
            {
                unsupported("DLL onexit output cells overlap the resized table");
            }
        }
    }
    let replacement = heap
        .allocate(handle, 0, size + 4)
        .unwrap_or_else(|e| unsupported(&e));
    if replacement == 0 {
        unsupported("DLL onexit allocation failure/errno behavior is unsupported");
    }
    unsafe {
        if size != 0 {
            std::ptr::copy_nonoverlapping(
                begin as *const u8,
                replacement as *mut u8,
                size as usize,
            );
        }
        ((replacement + size) as *mut u32).write(args[0]);
    }
    if begin != 0 {
        heap.free(handle, 0, begin)
            .unwrap_or_else(|e| unsupported(&e));
    }
    unsafe {
        (args[1] as *mut u32).write(replacement);
        (args[2] as *mut u32).write(replacement + size + 4);
    }
    args[0]
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    if id == 85 {
        let handler = image.cpu.crt.borrow().unexpected;
        if handler != 0 {
            let result =
                unsafe { image.callback_frame(handler, &[], super::CallingConvention32::Cdecl) }
                    .unwrap_or_else(|e| unsupported(&e));
            if result.is_none() {
                return image.cpu.exit.get().unwrap_or_else(|| {
                    unsupported("unexpected handler did not exit the guest process")
                });
            }
        }
        return dispatch(image, 84, args);
    }
    if id == 84 {
        let handler = image.cpu.crt.borrow().terminate;
        if handler == 0 {
            abort_unsupported(image, "CRT default terminate");
        }
        let result =
            unsafe { image.callback_frame(handler, &[], super::CallingConvention32::Cdecl) }
                .unwrap_or_else(|e| unsupported(&e));
        if result.is_some() {
            abort_unsupported(image, "returning CRT terminate handler");
        }
        return image
            .cpu
            .exit
            .get()
            .unwrap_or_else(|| unsupported("terminate handler did not exit the guest process"));
    }
    let mut state = image.cpu.crt.borrow_mut();
    if matches!(id, 94 | 95) {
        state
            .lock(args[0], alkyhol::ffi::GetCurrentThreadId(), id == 94)
            .unwrap_or_else(|e| unsupported(&e));
        return 0; // void: the owned lock mutation, not this register, is the operation.
    }
    if id == 91
        && let Some(&pointer) = state.errno.get(&alkyhol::ffi::GetCurrentThreadId())
    {
        return pointer;
    }
    if id == 86 {
        if args[0] > 2 {
            unsupported("unsupported CRT application type");
        }
        state.app_type = args[0];
        return 0; // void result: only the owned state mutation is observable.
    }
    if id == 81 {
        return state.terminate;
    }
    if id == 83 {
        return state.unexpected;
    }
    if id == 79 {
        // This records the actual CRT policy. Error reporting itself remains
        // unsupported rather than claiming to show a message box or terminate.
        return state
            .set_error_mode(args[0])
            .unwrap_or_else(|e| unsupported(&e));
    }
    if id == 80 || id == 82 {
        if args[0] != 0
            && !image.regions.iter().any(|&(first, last, rights)| {
                rights & 0x20000000 != 0 && args[0] as usize >= first && (args[0] as usize) < last
            })
        {
            unsupported("CRT exception policy handler outside executable guest storage");
        }
        // A NULL previous handler is a documented restorable default, not a
        // synthetic successful callback. Default abort/reporting remains unsupported.
        let slot = if id == 80 {
            &mut state.terminate
        } else {
            &mut state.unexpected
        };
        return std::mem::replace(slot, args[0]);
    }
    let mut heap = image.cpu.heap.borrow_mut();
    let handle = if let Some(handle) = state.heap {
        handle
    } else {
        let handle = heap.create(0, 0, 0).unwrap_or_else(|e| unsupported(&e));
        if handle == 0 {
            unsupported("CRT heap creation failed; errno/new-handler policy is unsupported");
        }
        state.heap = Some(handle);
        handle
    };
    let pointer = heap
        .allocate(
            handle,
            if id == 91 { 8 } else { 0 },
            if id == 91 { 4 } else { args[0] },
        )
        .unwrap_or_else(|e| unsupported(&e));
    if pointer == 0 {
        unsupported("CRT malloc exhausted backed storage; errno/new-handler policy is unsupported");
    }
    if id == 91 {
        state
            .errno
            .insert(alkyhol::ffi::GetCurrentThreadId(), pointer);
    }
    pointer
}
// Scalar/array delete and free are void operations on the same real private
// CRT heap as malloc/new. Invalid or foreign bases fail before mutation.
pub(super) fn deallocate(image: &LoadedImage32<'_>, pointer: u32) -> u32 {
    if pointer == 0 {
        return 0;
    }
    let handle = image
        .cpu
        .crt
        .borrow()
        .heap
        .unwrap_or_else(|| unsupported("CRT delete/free requires an owned allocation"));
    image
        .cpu
        .heap
        .borrow_mut()
        .free(handle, 0, pointer)
        .unwrap_or_else(|e| unsupported(&e));
    0 // void ABI; the owned allocation was actually freed.
}
fn checked_byte(image: &LoadedImage32<'_>, base: u32, index: u32) -> u8 {
    let address = base
        .checked_add(index)
        .unwrap_or_else(|| unsupported("CRT string address overflow"));
    if !image.permits(address as usize, 1, false) {
        unsupported("CRT string outside owned readable storage");
    }
    unsafe { (address as *const u8).read() }
}
pub(super) fn memory(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    if id >= 268 {
        let limit = if id == 269 { args[2] } else { 1024 * 1024 };
        if limit > 1024 * 1024 {
            unsupported("CRT string traversal budget exhausted");
        }
        for i in 0..limit {
            let a = checked_byte(image, args[0], i);
            if id == 268 {
                if a == args[1] as u8 {
                    return args[0] + i;
                }
                if a == 0 {
                    return 0;
                }
            } else {
                let b = checked_byte(image, args[1], i);
                if a != b {
                    return (a as i32 - b as i32) as u32;
                }
                if a == 0 {
                    return 0;
                }
            }
        }
        if id == 269 {
            return 0;
        }
        unsupported("CRT strchr traversal budget exhausted");
    }
    let n = args[2] as usize;
    if n > 64 * 1024 * 1024 {
        unsupported("CRT memory extent budget exhausted");
    }
    if n == 0 {
        return args[0];
    }
    if !image.permits(args[0] as usize, n, true)
        || id != 267 && !image.permits(args[1] as usize, n, false)
    {
        unsupported("CRT memory transfer outside owned storage");
    }
    unsafe {
        match id {
            265 => {
                let destination = args[0] as usize;
                let source = args[1] as usize;
                if destination < source + n && source < destination + n {
                    unsupported("CRT memcpy overlapping ranges require memmove");
                }
                std::ptr::copy_nonoverlapping(source as *const u8, destination as *mut u8, n);
            }
            266 => std::ptr::copy(args[1] as *const u8, args[0] as *mut u8, n),
            267 => std::ptr::write_bytes(args[0] as *mut u8, args[1] as u8, n),
            _ => unreachable!(),
        }
    }
    args[0]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn onexit_lock_tracks_real_recursion_ownership_and_rejects_unbalanced_release() {
        let mut state = State::default();
        assert!(state.lock(8, 7, false).is_err());
        state.lock(8, 7, true).unwrap();
        state.lock(8, 7, true).unwrap();
        assert!(state.lock(8, 9, true).is_err());
        assert!(state.lock(8, 9, false).is_err());
        state.lock(8, 7, false).unwrap();
        assert!(state.lock(8, 9, true).is_err());
        state.lock(8, 7, false).unwrap();
        assert!(state.lock(8, 7, false).is_err());
        state.lock(8, 9, true).unwrap();
        state.lock(8, 9, false).unwrap();
        for index in [0, 7, 9, u32::MAX] {
            assert!(state.lock(index, 7, true).is_err());
        }
    }
    #[test]
    fn error_policy_queries_preserve_settings_and_setters_return_previous() {
        let mut state = State::default();
        assert_eq!(state.set_error_mode(3).unwrap(), 0);
        assert_eq!(state.set_error_mode(1).unwrap(), 0);
        assert_eq!(state.set_error_mode(3).unwrap(), 1);
        assert_eq!(state.set_error_mode(2).unwrap(), 1);
        for invalid in [4, u32::MAX] {
            assert!(state.set_error_mode(invalid).is_err());
            assert_eq!(state.set_error_mode(3).unwrap(), 2);
        }
        assert_eq!(state.set_error_mode(0).unwrap(), 2);
        assert_eq!(state.set_error_mode(3).unwrap(), 0);
    }
}
