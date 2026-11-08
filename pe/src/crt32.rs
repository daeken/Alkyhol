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
}
impl State {
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
        .allocate(handle, 0, args[0])
        .unwrap_or_else(|e| unsupported(&e));
    if pointer == 0 {
        unsupported("CRT malloc exhausted backed storage; errno/new-handler policy is unsupported");
    }
    pointer
}
#[cfg(test)]
mod tests {
    use super::*;
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
