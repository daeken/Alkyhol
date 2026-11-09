//! Guest COM apartment initialization reached by the original FMOD startup.
//! This owns per-guest-process/thread balances, not host COM state. Activation,
//! proxies, marshaling, OLE/DDE and RPC remain separate unsupported operations.
use super::{LoadedImage32, unsupported};
use crate::Result;
use std::collections::BTreeMap;
#[derive(Default)]
pub(super) struct State {
    apartments: BTreeMap<u32, Apartment>,
}
struct Apartment {
    sta: bool,
    balance: u32,
}
impl State {
    fn initialize(&mut self, thread: u32, reserved: u32, flags: u32) -> Result<u32> {
        if reserved != 0 || flags & !14 != 0 {
            return Ok(0x80070057); // E_INVALIDARG
        }
        let sta = flags & 2 != 0;
        if let Some(apartment) = self.apartments.get_mut(&thread) {
            if apartment.sta != sta {
                return Ok(0x80010106); // RPC_E_CHANGED_MODE, no balance acquired.
            }
            apartment.balance = apartment
                .balance
                .checked_add(1)
                .filter(|&n| n <= 65536)
                .ok_or("COM initialization balance budget exhausted")?;
            Ok(1) // S_FALSE still acquires a balance.
        } else {
            if self.apartments.len() >= 1024 {
                return Err("COM apartment thread budget exhausted".into());
            }
            self.apartments
                .insert(thread, Apartment { sta, balance: 1 });
            Ok(0)
        }
    }
    fn uninitialize(&mut self, thread: u32) {
        if let Some(apartment) = self.apartments.get_mut(&thread) {
            apartment.balance -= 1;
            if apartment.balance == 0 {
                self.apartments.remove(&thread);
            }
        }
    }
    pub(super) fn exit(&mut self, thread: u32) {
        // No guest COM object/RPC registrations exist in this bounded profile.
        self.apartments.remove(&thread);
    }
    pub(super) fn clear(&mut self) {
        self.apartments.clear();
    }
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    let thread = alkyhol::ffi::GetCurrentThreadId();
    let mut state = image.cpu.com.borrow_mut();
    match id {
        272 | 273 => state
            .initialize(thread, args[0], if id == 272 { 2 } else { args[1] })
            .unwrap_or_else(|e| unsupported(&e)),
        274 => {
            state.uninitialize(thread);
            0
        }
        _ => unreachable!(),
    }
}
/// Real class-registration failure for explicitly configured unregistered classes.
/// A registered/unknown class stops until a real activation implementation exists.
pub(super) fn create(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[4] == 0 {
        return 0x80004003;
    } // E_POINTER
    if !image.permits(a[4] as usize, 4, true) {
        unsupported("CoCreateInstance output outside guest storage");
    }
    unsafe {
        (a[4] as *mut u32).write_unaligned(0);
    }
    if !image
        .cpu
        .com
        .borrow()
        .apartments
        .contains_key(&alkyhol::ffi::GetCurrentThreadId())
    {
        return 0x800401f0;
    }
    if a[1] != 0 || a[2] == 0 || a[2] & !7 != 0 {
        unsupported("COM aggregation or activation context outside admitted local-server profile");
    }
    if !image.permits(a[0] as usize, 16, false) || !image.permits(a[3] as usize, 16, false) {
        unsupported("CoCreateInstance GUID outside readable guest storage");
    }
    let g = unsafe { std::slice::from_raw_parts(a[0] as *const u8, 16) };
    let guid = format!(
        "{{{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
        u32::from_le_bytes(g[..4].try_into().unwrap()),
        u16::from_le_bytes(g[4..6].try_into().unwrap()),
        u16::from_le_bytes(g[6..8].try_into().unwrap()),
        g[8],
        g[9],
        g[10],
        g[11],
        g[12],
        g[13],
        g[14],
        g[15]
    );
    let catalog = image.cpu.registry.as_ref().unwrap_or_else(|| {
        unsupported("COM activation needs an explicit class registration catalog")
    });
    let registered = catalog
        .borrow()
        .class_registered(&guid)
        .unwrap_or_else(|e| unsupported(&e));
    if registered {
        unsupported("registered COM class activation is not implemented");
    }
    eprintln!("PE32 COM class absent from configured guest registration inventory: {guid}");
    0x80040154 // REGDB_E_CLASSNOTREG; caller may handle actual unavailable component.
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_balance_and_changed_mode_are_thread_local() {
        let mut s = State::default();
        assert_eq!(s.initialize(10, 0, 2).unwrap(), 0);
        assert_eq!(s.initialize(10, 0, 6).unwrap(), 1);
        assert_eq!(s.initialize(10, 0, 0).unwrap(), 0x80010106);
        assert_eq!(s.initialize(20, 0, 0).unwrap(), 0);
        s.uninitialize(10);
        assert_eq!(s.initialize(10, 0, 0).unwrap(), 0x80010106);
        s.uninitialize(10);
        assert_eq!(s.initialize(10, 0, 8).unwrap(), 0);
        s.exit(10);
        assert!(!s.apartments.contains_key(&10));
        assert_eq!(s.initialize(20, 0, 0).unwrap(), 1);
        s.clear();
        assert!(s.apartments.is_empty());
    }
    #[test]
    fn invalid_arguments_do_not_initialize_or_change_balances() {
        let mut s = State::default();
        assert_eq!(s.initialize(1, 1, 2).unwrap(), 0x80070057);
        assert_eq!(s.initialize(1, 0, 1).unwrap(), 0x80070057);
        assert!(s.apartments.is_empty());
        s.uninitialize(1);
        assert_eq!(s.initialize(1, 0, 0).unwrap(), 0);
        assert_eq!(s.initialize(1, 0, 16).unwrap(), 0x80070057);
        s.uninitialize(1);
        assert_eq!(s.initialize(1, 0, 2).unwrap(), 0);
    }
}
