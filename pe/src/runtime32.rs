//! Experimental PE32 integer/stdcall path in a 64-bit Rosetta host.
//! No Windows FS/TEB, CRT, DLL loading, callbacks, SEH or graphics ABI yet.
use crate::{ExportTable, ExportTarget, Image, ImportSymbol, ProcessContext, Result};
use std::{cell::Cell, ffi::c_void, marker::PhantomData, ptr, sync::OnceLock};
std::arch::global_asm!(include_str!("transition32.S"), options(att_syntax));
unsafe extern "C" {
    fn mmap(
        address: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
    fn mprotect(address: *mut c_void, len: usize, prot: i32) -> i32;
    fn munmap(address: *mut c_void, len: usize) -> i32;
    fn getpagesize() -> i32;
    fn i386_get_ldt(start: i32, entries: *mut u64, count: i32) -> i32;
    fn i386_set_ldt(start: i32, entries: *const u64, count: i32) -> i32;
    fn alkyhol_enter_pe32(entry: u32, stack: u32, code: u32, data: u32) -> u32;
    static alkyhol_pe32_gate_start: u8;
    static alkyhol_pe32_gate_end: u8;
    static alkyhol_pe32_dispatch_slot: u8;
}
const PAGE: usize = 4096;
const STACK: usize = 1024 * 1024;
const APIS: [(&str, u16); 7] = [
    ("GetLastError", 0),
    ("SetLastError", 4),
    ("GetStdHandle", 4),
    ("WriteFile", 20),
    ("GetModuleHandleA", 4),
    ("GetModuleHandleW", 4),
    ("GetProcAddress", 8),
];
static SELECTORS: OnceLock<Result<(u16, u16)>> = OnceLock::new();
fn selectors() -> Result<(u16, u16)> {
    SELECTORS
        .get_or_init(|| {
            // Rosetta aborts for LDT_AUTO_ALLOC. Inspect the task's existing table,
            // reserve two unused entries, and retain them for this process lifetime.
            let mut entries = [0u64; 2];
            let count = unsafe { i386_get_ldt(512, entries.as_mut_ptr(), 2) };
            if !(0..=8192).contains(&count) {
                return Err("cannot inspect task LDT".into());
            }
            if (count > 512 && entries[0] != 0) || (count > 513 && entries[1] != 0) {
                return Err("PE32 host LDT slots 512/513 are already occupied".into());
            }
            let descriptors = [0x00cffa000000ffffu64, 0x00cff2000000ffff];
            for (index, descriptor) in descriptors.iter().enumerate() {
                // Rosetta also rejects installing multiple selectors at once.
                let slot = 512 + index as i32;
                if unsafe { i386_set_ldt(slot, descriptor, 1) } != slot {
                    return Err("cannot install flat 32-bit code/data descriptors".into());
                }
            }
            Ok((0x1007, 0x100f))
        })
        .clone()
}
struct LowMapping {
    base: *mut u8,
    len: usize,
}
impl LowMapping {
    fn new(len: usize, alignment: usize, hint: usize) -> Result<Self> {
        let reserve = len
            .checked_add(alignment)
            .ok_or("low allocation overflow")?;
        let raw = unsafe { mmap(hint as *mut c_void, reserve, 3, 0x1002 | 0x8000, -1, 0) };
        if raw as isize == -1 {
            return Err("low memory unavailable; host must link with a small __PAGEZERO".into());
        }
        let first = raw as usize;
        if first + reserve > 0x100000000 {
            unsafe {
                munmap(raw, reserve);
            };
            return Err("mapping is not below 4 GiB".into());
        }
        let base = (first + alignment - 1) & !(alignment - 1);
        unsafe {
            if base > first {
                munmap(raw, base - first);
            }
            if first + reserve > base + len {
                munmap((base + len) as *mut c_void, first + reserve - base - len);
            }
        }
        Ok(Self {
            base: base as *mut u8,
            len,
        })
    }
    fn protect(&self, start: usize, len: usize, rights: i32) -> Result<()> {
        if start.checked_add(len).is_none_or(|end| end > self.len) {
            return Err("low protection extent".into());
        }
        if unsafe { mprotect(self.base.add(start).cast(), len, rights) } != 0 {
            return Err("low protection failed".into());
        }
        Ok(())
    }
}
impl Drop for LowMapping {
    fn drop(&mut self) {
        unsafe {
            munmap(self.base.cast(), self.len);
        }
    }
}
// The pointer is installed only while enter holds &self; ActiveGuard clears it
// before that borrow ends. No cross-thread/nested entry or guest callback path.
thread_local! {static ACTIVE:Cell<*const c_void>=const {Cell::new(ptr::null())};}
struct ActiveGuard;
impl Drop for ActiveGuard {
    fn drop(&mut self) {
        ACTIVE.with(|slot| slot.set(ptr::null()));
    }
}
fn unsupported(reason: &str) -> ! {
    eprintln!("Alkyhol PE32 unsupported: {reason}");
    std::process::exit(78)
}
/// Trusted i386 EXE mapping with separate low stack and stdcall API gates.
/// The process guard and all three mappings remain alive until drop. No guest
/// threads, FS/TEB, callbacks, SEH, arbitrary syscalls or escaping pointers.
/// The host executable must use a small __PAGEZERO (see translate_pe32.py).
///
/// ```compile_fail,E0505
/// # use alkyhol_pe::{ProcessContext,LoadedImage32};
/// # fn example(bytes:&[u8])->alkyhol_pe::Result<()> {
/// let process=ProcessContext::install(&["guest32".into()],&[])?;
/// let image=LoadedImage32::load(bytes,false,&process)?;
/// drop(process);
/// drop(image);
/// # Ok(()) }
/// ```
pub struct LoadedImage32<'a> {
    image: LowMapping,
    exports: Option<ExportTable>,
    gates: LowMapping,
    stack: LowMapping,
    entry: u32,
    relocated: bool,
    code: u16,
    data: u16,
    regions: Vec<(usize, usize, u32)>,
    handles: [usize; 3],
    _process: PhantomData<&'a ProcessContext>,
}
impl<'a> LoadedImage32<'a> {
    /// Preflight all features/imports, map checked sections and HIGHLOW fixups,
    /// build low stdcall gates and a guarded stack. No guest code runs here.
    pub fn load(bytes: &[u8], rebase: bool, _process: &'a ProcessContext) -> Result<Self> {
        if unsafe { getpagesize() } != PAGE as i32 {
            return Err("PE32 requires 4-KiB host pages".into());
        }
        let parsed = Image::parse_i386(bytes)?;
        parsed.check_startup()?;
        let imports = parsed.imports()?;
        let fixups = parsed.relocations()?;
        let mut bindings = Vec::new();
        for import in imports {
            let id = if import.module.eq_ignore_ascii_case("kernel32.dll") {
                match &import.symbol {
                    ImportSymbol::Name(n) => APIS.iter().position(|&(name, _)| name == n),
                    _ => None,
                }
            } else {
                None
            }
            .ok_or_else(|| {
                format!(
                    "unsupported Win32 import {}!{:?}",
                    import.module, import.symbol
                )
            })?;
            if fixups
                .iter()
                .any(|&at| at < import.iat + 4 && import.iat < at + 4)
            {
                return Err("IAT overlaps HIGHLOW fixup".into());
            }
            bindings.push((import.iat, id));
        }
        let (code, data) = selectors()?;
        let image = LowMapping::new(
            parsed.size,
            65536,
            if rebase {
                0x10000000
            } else {
                parsed.base as usize
            },
        )?;
        let relocated = image.base as u64 != parsed.base;
        if rebase && !relocated {
            return Err("forced PE32 rebase did not move".into());
        }
        if relocated && fixups.is_empty() {
            return Err("PE32 preferred base unavailable and no HIGHLOW fixups".into());
        }
        let gates = LowMapping::new(PAGE, PAGE, 0)?;
        let stack = LowMapping::new(STACK + 2 * PAGE, PAGE, 0)?;
        let mut cs64 = 0u16;
        unsafe {
            std::arch::asm!("mov {0:x}, cs",out(reg) cs64,options(nomem,nostack,preserves_flags));
        }
        let mut template = vec![0u8; PAGE];
        // 32-bit near return -> far jump into low 64-bit gate -> host continuation.
        template[..2].copy_from_slice(&[0xff, 0x2d]);
        template[2..6].copy_from_slice(&((gates.base as u32) + 8).to_le_bytes());
        template[8..12].copy_from_slice(&((gates.base as u32) + 16).to_le_bytes());
        template[12..14].copy_from_slice(&cs64.to_le_bytes());
        template[16..22].copy_from_slice(&[0x4c, 0x89, 0xe4, 0x41, 0xff, 0xe5]);
        let start = ptr::addr_of!(alkyhol_pe32_gate_start) as usize;
        let end = ptr::addr_of!(alkyhol_pe32_gate_end) as usize;
        let slot = ptr::addr_of!(alkyhol_pe32_dispatch_slot) as usize - start;
        if end - start > 192 || slot + 8 > end - start {
            return Err("invalid CPU gate template".into());
        }
        template[64..64 + end - start].copy_from_slice(unsafe {
            std::slice::from_raw_parts(start as *const u8, end - start)
        });
        template[64 + slot..72 + slot]
            .copy_from_slice(&(dispatch as *const () as u64).to_le_bytes());
        for (id, &(_, argument_bytes)) in APIS.iter().enumerate() {
            let at = 256 + id * 32;
            let base = gates.base as u32 + at as u32;
            template[at] = 0x68;
            template[at + 1..at + 5].copy_from_slice(&(base + 16).to_le_bytes());
            template[at + 5] = 0x68;
            template[at + 6..at + 10].copy_from_slice(&(id as u32).to_le_bytes());
            template[at + 10..at + 12].copy_from_slice(&[0xff, 0x2d]);
            template[at + 12..at + 16].copy_from_slice(&(base + 20).to_le_bytes());
            template[at + 16] = 0xc2;
            template[at + 17..at + 19].copy_from_slice(&argument_bytes.to_le_bytes());
            template[at + 20..at + 24].copy_from_slice(&(gates.base as u32 + 64).to_le_bytes());
            template[at + 24..at + 26].copy_from_slice(&cs64.to_le_bytes());
        }
        unsafe {
            ptr::copy_nonoverlapping(template.as_ptr(), gates.base, PAGE);
            ptr::copy_nonoverlapping(bytes.as_ptr(), image.base, parsed.headers);
            for section in &parsed.sections {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(section.raw),
                    image.base.add(section.rva),
                    section.raw_size,
                );
            }
            if relocated {
                let delta = (image.base as u32).wrapping_sub(parsed.base as u32);
                for at in fixups {
                    let p = image.base.add(at).cast::<u32>();
                    p.write_unaligned(p.read_unaligned().wrapping_add(delta));
                }
            }
            for (at, id) in bindings {
                image
                    .base
                    .add(at)
                    .cast::<u32>()
                    .write_unaligned(gates.base as u32 + 256 + id as u32 * 32);
            }
            stack
                .base
                .add(PAGE + STACK - 4)
                .cast::<u32>()
                .write_unaligned(gates.base as u32);
        }
        gates.protect(0, PAGE, 5)?;
        stack.protect(0, PAGE, 0)?;
        stack.protect(PAGE + STACK, PAGE, 0)?;
        image.protect(0, parsed.size, 0)?;
        image.protect(0, parsed.headers, 1)?;
        let mut regions = Vec::new();
        for section in &parsed.sections {
            let rights = ((section.rights >> 30) & 1)
                | (((section.rights >> 31) & 1) << 1)
                | (((section.rights >> 29) & 1) << 2);
            image.protect(section.rva, section.size, rights as i32)?;
            regions.push((
                image.base as usize + section.rva,
                image.base as usize + section.rva + section.size,
                section.rights,
            ));
        }
        let standard: unsafe extern "win64" fn(u32) -> usize = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "GetStdHandle").unwrap())
        };
        let handles = unsafe {
            [
                standard(0xfffffff6),
                standard(0xfffffff5),
                standard(0xfffffff4),
            ]
        };
        let entry = image.base as u32 + parsed.entry as u32;
        Ok(Self {
            image,
            exports: parsed.exports()?,
            gates,
            stack,
            entry,
            relocated,
            code,
            data,
            regions,
            handles,
            _process: PhantomData,
        })
    }
    /// Whether HIGHLOW relocation was required for this mapping.
    pub fn relocated(&self) -> bool {
        self.relocated
    }
    fn permits(&self, address: usize, len: usize, write: bool) -> bool {
        let Some(end) = address.checked_add(len) else {
            return false;
        };
        let first = self.stack.base as usize + PAGE;
        if address >= first && end <= first + STACK {
            return true;
        }
        let mask = if write { 0x80000000 } else { 0x40000000 };
        self.regions
            .iter()
            .any(|&(first, last, rights)| rights & mask != 0 && address >= first && end <= last)
    }
    fn export_name(&self, address: u32) -> Result<String> {
        query_name(address, |at| {
            if !self.permits(at as usize, 1, false) {
                return Err("export query name outside readable guest storage".into());
            }
            Ok(unsafe { (at as usize as *const u8).read() })
        })
    }
    fn export_address(&self, module: u32, symbol: u32) -> Result<Option<u32>> {
        if module != self.image.base as u32 {
            return Err("GetProcAddress requires the current PE32 executable module".into());
        }
        if self.exports.is_none() {
            return Err("GetProcAddress without a PE32 export table is unsupported".into());
        }
        let name = if symbol > u16::MAX as u32 {
            Some(self.export_name(symbol)?)
        } else {
            None
        };
        let entry = self.exports.as_ref().and_then(|table| {
            table.entries.iter().find(|e| match &name {
                Some(n) => e.names.contains(n),
                None => e.ordinal == symbol,
            })
        });
        match entry.map(|e| &e.target) {
            None => Ok(None),
            Some(ExportTarget::Address { rva }) => (self.image.base as u32)
                .checked_add(*rva as u32)
                .map(Some)
                .ok_or_else(|| "export address overflow".into()),
            Some(ExportTarget::Forwarder(_)) => {
                Err("PE32 export forwarders are unsupported".into())
            }
        }
    }
    /// Execute trusted i386 code and return its DWORD result to the host.
    ///
    /// # Safety
    /// Input must obey this type's stated restricted native-code contract.
    /// This is execution, not an instruction validator or security sandbox.
    pub unsafe fn enter(&self) -> Result<u32> {
        ACTIVE.with(|slot| {
            if !slot.get().is_null() {
                return Err("nested PE32 entry unsupported".into());
            }
            slot.set(ptr::from_ref(self).cast());
            let _guard = ActiveGuard;
            let sp = self.stack.base as u32 + PAGE as u32 + STACK as u32 - 4;
            unsafe {
                self.stack
                    .base
                    .add(PAGE + STACK - 4)
                    .cast::<u32>()
                    .write_unaligned(self.gates.base as u32);
            }
            Ok(unsafe {
                alkyhol_enter_pe32(self.entry, sp, u32::from(self.code), u32::from(self.data))
            })
        })
    }
}
// Keep the process borrow live through field destruction, including implicit drop.
impl Drop for LoadedImage32<'_> {
    fn drop(&mut self) {}
}
fn query_name(address: u32, mut read: impl FnMut(u32) -> Result<u8>) -> Result<String> {
    let mut text = Vec::new();
    for offset in 0..4096u32 {
        let at = address
            .checked_add(offset)
            .ok_or("export query pointer overflow")?;
        let byte = read(at)?;
        if byte == 0 {
            if text.is_empty() {
                return Err("empty export query name is unsupported".into());
            }
            return String::from_utf8(text).map_err(|_| "invalid export query name".into());
        }
        if !byte.is_ascii() {
            return Err("non-ASCII export query is unsupported".into());
        }
        text.push(byte);
    }
    Err("export query exceeds 4096-byte bound".into())
}
extern "C" fn dispatch(id: u32, arguments: *const u32) -> u32 {
    ACTIVE.with(|slot| {
        let Some(image) = (unsafe { slot.get().cast::<LoadedImage32<'_>>().as_ref() }) else {
            unsupported("no active Win32 image");
        };
        let Some(&(_, bytes)) = APIS.get(id as usize) else {
            unsupported("unknown gateway");
        };
        if !(arguments as usize).is_multiple_of(4)
            || !image.permits(arguments as usize, bytes as usize, false)
        {
            unsupported("argument stack outside checked storage");
        }
        // Snapshot values rather than borrowing guest argument memory across a
        // host call: a valid output pointer may alias those stack words.
        let mut args = [0u32; 5];
        for (index, value) in args.iter_mut().enumerate().take(bytes as usize / 4) {
            *value = unsafe { arguments.add(index).read_unaligned() };
        }
        match id {
            0 => alkyhol::ffi::GetLastError(),
            1 => {
                alkyhol::ffi::SetLastError(args[0]);
                0
            }
            2 => {
                let index = match args[0] {
                    0xfffffff6 => 0,
                    0xfffffff5 => 1,
                    0xfffffff4 => 2,
                    _ => {
                        alkyhol::ffi::SetLastError(87);
                        return u32::MAX;
                    }
                };
                match image.handles[index] {
                    0 => 0,
                    usize::MAX => u32::MAX,
                    _ => 0x70000001 + index as u32,
                }
            }
            3 => {
                let index = args[0].wrapping_sub(0x70000001) as usize;
                if index >= 3 {
                    unsupported("WriteFile handle is not a standard-stream proxy");
                }
                if args[4] != 0 {
                    unsupported("overlapped Win32 I/O");
                }
                if args[2] > 1024 * 1024
                    || !args[3].is_multiple_of(4)
                    || !image.permits(args[1] as usize, args[2] as usize, false)
                    || !image.permits(args[3] as usize, 4, true)
                {
                    unsupported("WriteFile buffers outside checked storage/bounds");
                }
                let write: unsafe extern "win64" fn(
                    usize,
                    *const c_void,
                    u32,
                    *mut u32,
                    *mut c_void,
                ) -> i32 = unsafe {
                    std::mem::transmute(crate::abi::resolve("kernel32.dll", "WriteFile").unwrap())
                };
                unsafe {
                    write(
                        image.handles[index],
                        args[1] as usize as *const c_void,
                        args[2],
                        args[3] as usize as *mut u32,
                        ptr::null_mut(),
                    ) as u32
                }
            }
            4 | 5 => {
                if args[0] != 0 {
                    unsupported("named PE32 module lookup is unsupported");
                }
                image.image.base as u32
            }
            6 => match image.export_address(args[0], args[1]) {
                Ok(Some(address)) => address,
                Ok(None) => {
                    // Native PE32 distinguishes absent names from out-of-range
                    // ordinals (ERROR_PROC_NOT_FOUND / ERROR_INVALID_ORDINAL).
                    alkyhol::ffi::SetLastError(if args[1] <= u16::MAX as u32 { 182 } else { 127 });
                    0
                }
                Err(error) => unsupported(&error),
            },
            _ => unreachable!(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    // No mapping is dereferenced or dropped: pure address/metadata assertions,
    // separate from the actual unchanged-PE execution probe.
    fn layout() -> std::mem::ManuallyDrop<LoadedImage32<'static>> {
        std::mem::ManuallyDrop::new(LoadedImage32 {
            image: LowMapping {
                base: ptr::null_mut(),
                len: 0,
            },
            exports: None,
            gates: LowMapping {
                base: ptr::null_mut(),
                len: 0,
            },
            stack: LowMapping {
                base: 0x400000usize as *mut u8,
                len: STACK + 2 * PAGE,
            },
            entry: 0,
            relocated: false,
            code: 0,
            data: 0,
            handles: [0; 3],
            regions: vec![
                (0x100000, 0x101000, 0x60000000),
                (0x200000, 0x201000, 0xc0000000),
            ],
            _process: PhantomData,
        })
    }
    #[test]
    fn buffer_validation_obeys_actual_section_permissions_stack_bounds_and_overflow() {
        let image = layout();
        assert!(image.permits(0x100000, 4096, false));
        assert!(!image.permits(0x100000, 1, true));
        assert!(image.permits(0x200000, 4096, true));
        assert!(!image.permits(0x200fff, 2, true));
        assert!(!image.permits(0x101000, 1, false));
        assert!(!image.permits(0, 4, false));
        assert!(!image.permits(0x400fff, 1, false));
        assert!(image.permits(0x401000, STACK, true));
        assert!(!image.permits(0x401000, STACK + 1, true));
        assert!(!image.permits(usize::MAX - 1, 4, false));
    }
    #[test]
    fn export_query_names_are_owned_bounded_ascii_and_never_read_after_failure() {
        assert_eq!(
            query_name(100, |a| Ok(b"Data\0"[(a - 100) as usize])).unwrap(),
            "Data"
        );
        for data in [&b"\0"[..], &b"\xff\0"[..]] {
            assert!(query_name(0, |a| Ok(data[a as usize])).is_err());
        }
        let mut calls = 0;
        assert!(
            query_name(0, |_| {
                calls += 1;
                Ok(b'X')
            })
            .is_err()
        );
        assert_eq!(calls, 4096);
        calls = 0;
        assert!(
            query_name(u32::MAX, |_| {
                calls += 1;
                Ok(b'X')
            })
            .is_err()
        );
        assert_eq!(calls, 1);
        calls = 0;
        assert!(
            query_name(100, |_| {
                calls += 1;
                Err("unreadable".into())
            })
            .is_err()
        );
        assert_eq!(calls, 1);
        assert!(layout().export_name(0).is_err());
    }
    #[test]
    fn self_export_addresses_require_own_module_and_preserve_biased_ordinals() {
        let mut image = layout();
        image.image.base = 0x100000usize as *mut u8;
        assert!(image.export_address(0x100000, 17).is_err());
        image.exports = Some(ExportTable {
            dll_name: "owned.exe".into(),
            ordinal_base: 17,
            entries: vec![
                crate::Export {
                    ordinal: 17,
                    names: vec!["Direct".into(), "Alias".into()],
                    target: ExportTarget::Address { rva: 0x1000 },
                },
                crate::Export {
                    ordinal: 18,
                    names: vec![],
                    target: ExportTarget::Address { rva: 0x2000 },
                },
            ],
        });
        assert_eq!(image.export_address(0x100000, 17).unwrap(), Some(0x101000));
        assert_eq!(image.export_address(0x100000, 18).unwrap(), Some(0x102000));
        for ordinal in [0, 16, 19, 65535] {
            assert_eq!(image.export_address(0x100000, ordinal).unwrap(), None);
        }
        assert!(image.export_address(0, 17).is_err());
        assert!(image.export_address(0x100000, 0xdead0000).is_err());
        image.exports.as_mut().unwrap().entries[0].target =
            ExportTarget::Forwarder("kernel32.Sleep".into());
        assert!(image.export_address(0x100000, 17).is_err());
        image.exports.as_mut().unwrap().entries[0].target =
            ExportTarget::Address { rva: 0xffff0000 };
        assert!(image.export_address(0x100000, 17).is_err());
    }
}
