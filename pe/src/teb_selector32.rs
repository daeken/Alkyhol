//! Per-thread FS selector ownership, independent of guest CPU admission.
use crate::Result;

#[cfg(target_arch = "x86_64")]
use super::{i386_get_ldt, i386_set_ldt, teb_descriptor};
#[cfg(target_arch = "x86_64")]
use std::sync::Mutex;

#[cfg(target_arch = "x86_64")]
const FIRST_TEB: i32 = 514;
#[cfg(target_arch = "x86_64")]
const LDT_SIZE: i32 = 8192;
#[cfg(target_arch = "x86_64")]
static LDT: Mutex<()> = Mutex::new(());

pub(super) struct TebSelector {
    #[cfg(target_arch = "x86_64")]
    slot: i32,
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

impl TebSelector {
    pub(super) fn allocate(base: u32) -> Result<Self> {
        #[cfg(target_arch = "x86_64")]
        {
            let _ldt = LDT.lock().map_err(|_| "PE32 selector table poisoned")?;
            for slot in FIRST_TEB..LDT_SIZE {
                let mut descriptor = 0;
                let count = unsafe { i386_get_ldt(slot, &mut descriptor, 1) };
                if !(0..=LDT_SIZE).contains(&count) {
                    return Err("cannot inspect PE32 TEB descriptor".into());
                }
                if count > slot && descriptor != 0 {
                    continue;
                }
                if unsafe { i386_set_ldt(slot, &teb_descriptor(base), 1) } != slot {
                    return Err("cannot install PE32 TEB descriptor".into());
                }
                return Ok(Self {
                    slot,
                    _thread: std::marker::PhantomData,
                });
            }
            Err("no free PE32 TEB selector".into())
        }
        #[cfg(target_arch = "aarch64")]
        {
            let _ = base;
            Ok(Self {
                _thread: std::marker::PhantomData,
            })
        }
    }

    pub(super) fn value(&self) -> u16 {
        #[cfg(target_arch = "x86_64")]
        {
            ((self.slot as u16) << 3) | 7
        }
        #[cfg(target_arch = "aarch64")]
        {
            // The ARM executor models FS against the active thread's TEB.
            0x1017
        }
    }
}

#[cfg(target_arch = "x86_64")]
impl Drop for TebSelector {
    fn drop(&mut self) {
        let _ldt = LDT.lock().unwrap();
        let empty = 0;
        if unsafe { i386_set_ldt(self.slot, &empty, 1) } != self.slot {
            // Do not release the TEB while the descriptor still points to it.
            eprintln!("Alkyhol PE32 unsupported: cannot clear owned TEB descriptor");
            std::process::abort();
        }
    }
}

#[cfg(all(test, target_arch = "x86_64"))]
mod tests {
    use super::*;

    fn descriptor(selector: u16) -> u64 {
        let _ldt = LDT.lock().unwrap();
        let slot = i32::from(selector >> 3);
        let mut value = 0;
        let count = unsafe { i386_get_ldt(slot, &mut value, 1) };
        assert!((0..=LDT_SIZE).contains(&count));
        if count > slot { value } else { 0 }
    }

    #[test]
    fn live_threads_own_distinct_descriptors_and_release_only_their_own() {
        // Exclude runtime entries while observing the retired slots.
        let _cpu = super::super::GUEST_CPU.lock().unwrap();
        let first_base = 0x3210_1000;
        let second_base = 0x7654_3000;
        let first = TebSelector::allocate(first_base).unwrap();
        let first_fs = first.value();
        let second_fs = std::thread::spawn(move || {
            let second = TebSelector::allocate(second_base).unwrap();
            let second_fs = second.value();
            assert_ne!(first_fs, second_fs);
            assert_eq!(descriptor(first_fs), teb_descriptor(first_base));
            assert_eq!(descriptor(second_fs), teb_descriptor(second_base));
            second_fs
        })
        .join()
        .unwrap();
        assert_eq!(descriptor(first_fs), teb_descriptor(first_base));
        assert_eq!(descriptor(second_fs), 0);
        drop(first);
        assert_eq!(descriptor(first_fs), 0);
    }
}
