//! Experimental PE32 integer/stdcall path in a 64-bit Rosetta host.
//! Minimal FS/TEB, explicit shared-thread DLL dependencies and EXE static TLS.
//! No guest callbacks, PEB, CRT, process DLL loader/cache or exception dispatch.
use crate::{ExportTable, ExportTarget, Image, ImportSymbol, ProcessContext, Result};
use std::{
    cell::Cell,
    ffi::c_void,
    marker::PhantomData,
    ptr,
    rc::Rc,
    sync::{Mutex, MutexGuard, OnceLock},
};
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
    fn getentropy(buffer: *mut c_void, len: usize) -> i32;
    fn i386_get_ldt(start: i32, entries: *mut u64, count: i32) -> i32;
    fn i386_set_ldt(start: i32, entries: *const u64, count: i32) -> i32;
    fn alkyhol_enter_pe32(entry: u32, stack: u32, code: u32, data: u32) -> u64;
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
            // reserve two flat entries for this process lifetime and check the
            // third slot used only during an exclusively leased FS entry.
            let mut entries = [0u64; 3];
            let count = unsafe { i386_get_ldt(512, entries.as_mut_ptr(), 3) };
            if !(0..=8192).contains(&count) {
                return Err("cannot inspect task LDT".into());
            }
            if entries
                .iter()
                .enumerate()
                .any(|(index, &value)| count > 512 + index as i32 && value != 0)
            {
                return Err("PE32 host LDT slots 512/513/514 are already occupied".into());
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
// LDT entries belong to the process, while FS state belongs to a thread.
// Serialize CPU entries so a second thread cannot retarget the active TEB slot.
static FS_SLOT: Mutex<()> = Mutex::new(());
fn teb_descriptor(base: u32) -> u64 {
    0x0040f20000000fff
        | (u64::from(base & 0xffff) << 16)
        | (u64::from((base >> 16) & 0xff) << 32)
        | (u64::from(base >> 24) << 56)
}
struct TebLease {
    _guard: MutexGuard<'static, ()>,
}
impl TebLease {
    fn install(base: u32) -> Result<Self> {
        let guard = FS_SLOT
            .try_lock()
            .map_err(|_| "another PE32 entry owns the FS slot")?;
        let mut descriptor = 0u64;
        let count = unsafe { i386_get_ldt(514, &mut descriptor, 1) };
        if !(0..=8192).contains(&count) || (count > 514 && descriptor != 0) {
            return Err("PE32 FS slot 514 is already occupied or cannot be inspected".into());
        }
        descriptor = teb_descriptor(base);
        if unsafe { i386_set_ldt(514, &descriptor, 1) } != 514 {
            return Err("cannot install bounded PE32 TEB descriptor".into());
        }
        Ok(Self { _guard: guard })
    }
}
impl Drop for TebLease {
    fn drop(&mut self) {
        let empty = 0u64;
        if unsafe { i386_set_ldt(514, &empty, 1) } != 514 {
            // Host FS is restored before this guard is dropped. Do not leave an
            // apparently reusable descriptor pointing at soon-to-be-freed memory.
            eprintln!("Alkyhol PE32 unsupported: cannot clear owned FS descriptor");
            std::process::abort();
        }
    }
}
fn segment_state() -> [u16; 5] {
    let (mut ds, mut es, mut ss, mut fs, mut gs) = (0u16, 0u16, 0u16, 0u16, 0u16);
    unsafe {
        std::arch::asm!("mov {0:x}, ds", "mov {1:x}, es", "mov {2:x}, ss",
        "mov {3:x}, fs", "mov {4:x}, gs", out(reg) ds, out(reg) es, out(reg) ss,
        out(reg) fs, out(reg) gs, options(nomem, nostack, preserves_flags));
    }
    [ds, es, ss, fs, gs]
}
fn teb_bytes(base: u32, stack_limit: u32, stack_size: u32) -> Result<[u8; PAGE]> {
    let stack_base = stack_limit
        .checked_add(stack_size)
        .ok_or("TEB stack extent overflow")?;
    if stack_size == 0 || u64::from(base) + PAGE as u64 > 0x100000000 {
        return Err("invalid TEB/stack extent".into());
    }
    let mut bytes = [0u8; PAGE];
    for (at, value) in [
        (0, u32::MAX),
        (4, stack_base),
        (8, stack_limit),
        (0x18, base),
    ] {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}
struct TebError(*mut u32);
impl Drop for TebError {
    fn drop(&mut self) {
        unsafe {
            self.0.write_unaligned(alkyhol::ffi::GetLastError());
        }
    }
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
struct TlsBlock {
    mapping: LowMapping,
    size: usize,
}
// Linked images on one guest thread share one stack and FS/last-error state.
// An Rc keeps them thread-bound; the exclusive FS lease still guards CPU entry.
struct Cpu32 {
    stack: LowMapping,
    teb: LowMapping,
}
impl Cpu32 {
    fn new() -> Result<Self> {
        let stack = LowMapping::new(STACK + 2 * PAGE, PAGE, 0)?;
        let teb = LowMapping::new(PAGE, PAGE, 0)?;
        let bytes = teb_bytes(
            teb.base as u32,
            stack.base as u32 + PAGE as u32,
            STACK as u32,
        )?;
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), teb.base, PAGE);
        }
        stack.protect(0, PAGE, 0)?;
        stack.protect(PAGE + STACK, PAGE, 0)?;
        Ok(Self { stack, teb })
    }
}
enum Binding32 {
    Api(usize),
    Direct(u32),
}
fn dependency_cpu(dependencies: &[(&str, &LoadedLibrary32<'_>)]) -> Result<Option<Rc<Cpu32>>> {
    if dependencies.len() > 64 {
        return Err("PE32 dependency list exceeds 64 modules".into());
    }
    let mut names = std::collections::BTreeSet::new();
    let shared = dependencies.first().map(|(_, lib)| &lib.mapping.cpu);
    for &(name, library) in dependencies {
        if !names.insert(crate::runtime::dll_basename(name)?) {
            return Err("duplicate case-insensitive PE32 DLL dependency".into());
        }
        if shared.is_some_and(|cpu| !Rc::ptr_eq(cpu, &library.mapping.cpu)) {
            return Err("PE32 dependencies must share one guest thread context".into());
        }
    }
    Ok(shared.cloned())
}
const DEFAULT_GS_COOKIE: u32 = 0xbb40e64e;
fn select_cookie(
    declared_size: usize,
    initial: u32,
    mut entropy: impl FnMut() -> Result<u32>,
) -> Result<u32> {
    // Native observations: the declared structure length controls this action,
    // even when the data-directory length disagrees. Other tested seeds persist.
    if declared_size != 72 || initial != DEFAULT_GS_COOKIE {
        return Ok(initial);
    }
    for _ in 0..128 {
        let value = entropy()?;
        if value != 0 && value != DEFAULT_GS_COOKIE {
            return Ok(value);
        }
    }
    Err("system entropy did not yield a nondefault security cookie".into())
}
fn cookie_entropy() -> Result<u32> {
    let mut bytes = [0u8; 4];
    if unsafe { getentropy(bytes.as_mut_ptr().cast(), bytes.len()) } != 0 {
        return Err("system entropy unavailable for PE32 security cookie".into());
    }
    Ok(u32::from_ne_bytes(bytes))
}
/// Trusted i386 EXE mapping with separate low stack and stdcall API gates.
/// The process guard and all mappings remain alive until drop. Minimal FS/TEB
/// self/stack/last-error fields and single-thread static TLS data are supplied.
/// No guest threads, PEB, callbacks,
/// SEH, arbitrary syscalls or escaping pointers.
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
    cpu: Rc<Cpu32>,
    tls: Option<TlsBlock>,
    tls_entered: Cell<bool>,
    entry: u32,
    relocated: bool,
    code: u16,
    data: u16,
    dll: bool,
    regions: Vec<(usize, usize, u32)>,
    code_regions: Vec<(usize, usize)>,
    handles: [usize; 3],
    _process: PhantomData<&'a ProcessContext>,
}
impl<'a> LoadedImage32<'a> {
    /// Preflight all features/imports, map checked sections and HIGHLOW fixups,
    /// build low stdcall gates and a guarded stack. No guest code runs here.
    pub fn load(bytes: &[u8], rebase: bool, _process: &'a ProcessContext) -> Result<Self> {
        Self::load_kind(bytes, rebase, _process, false, &[])
    }
    fn load_kind(
        bytes: &[u8],
        rebase: bool,
        _process: &'a ProcessContext,
        dll: bool,
        dependencies: &[(&str, &'a LoadedLibrary32<'a>)],
    ) -> Result<Self> {
        if ACTIVE.with(|s| !s.get().is_null()) {
            return Err("PE32 loading during guest entry is unsupported".into());
        }
        let shared = dependency_cpu(dependencies)?;
        if unsafe { getpagesize() } != PAGE as i32 {
            return Err("PE32 requires 4-KiB host pages".into());
        }
        let parsed = if dll {
            Image::parse_i386_dll(bytes)?
        } else {
            Image::parse_i386(bytes)?
        };
        if dll {
            parsed.check_library_startup()?;
        } else {
            parsed.check_startup()?;
        }
        let tls_info = parsed.static_tls32()?;
        let load_config = parsed.legacy_load_config32()?;
        let imports = parsed.imports()?;
        let fixups = parsed.relocations()?;
        let mut bindings = Vec::new();
        for import in imports {
            let binding = if import.module.eq_ignore_ascii_case("kernel32.dll") {
                match &import.symbol {
                    ImportSymbol::Name(n) => APIS
                        .iter()
                        .position(|&(name, _)| name == n)
                        .map(Binding32::Api),
                    _ => None,
                }
            } else {
                match dependencies
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(&import.module))
                {
                    Some((_, library)) => {
                        Some(Binding32::Direct(library.import_address(&import.symbol)?))
                    }
                    None => None,
                }
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
            bindings.push((import.iat, binding));
        }
        let (code, data) = selectors()?;
        let image = LowMapping::new(
            parsed.size,
            65536,
            if rebase {
                if parsed.base == 0x10000000 {
                    0x20000000
                } else {
                    0x10000000
                }
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
        let cpu = match shared {
            Some(cpu) => cpu,
            None => Rc::new(Cpu32::new()?),
        };
        let stack = &cpu.stack;
        let teb = &cpu.teb;
        let tls = tls_info
            .as_ref()
            .map(|info| -> Result<TlsBlock> {
                let size = (info.total_size + PAGE - 1) & !(PAGE - 1);
                Ok(TlsBlock {
                    mapping: LowMapping::new(size, info.alignment.max(PAGE), 0)?,
                    size: info.total_size,
                })
            })
            .transpose()?;
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
        // Capture actual post-RET ESP before recovering the host stack. This
        // checks cdecl/stdcall cleanup rather than hiding a guest stack imbalance.
        template[16..24].copy_from_slice(&[0x89, 0xe2, 0x4c, 0x89, 0xe4, 0x41, 0xff, 0xe5]);
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
            for (at, binding) in bindings {
                let address = match binding {
                    Binding32::Api(id) => gates.base as u32 + 256 + id as u32 * 32,
                    Binding32::Direct(address) => address,
                };
                image.base.add(at).cast::<u32>().write_unaligned(address);
            }
            if let Some(config) = &load_config
                && let Some(rva) = config.cookie_rva
            {
                let target = image.base.add(rva).cast::<u32>();
                let value = select_cookie(
                    config.declared_size,
                    target.read_unaligned(),
                    cookie_entropy,
                )?;
                target.write_unaligned(value);
            }
            if let (Some(info), Some(block)) = (&tls_info, &tls) {
                // Copy the mapped template after relocations and import binding,
                // including any pointer initializer's adjusted image address.
                image
                    .base
                    .add(info.index_rva)
                    .cast::<u32>()
                    .write_unaligned(0);
                ptr::copy_nonoverlapping(
                    image.base.add(info.template_rva),
                    block.mapping.base,
                    info.template_size,
                );
                teb.base
                    .add(0x2c)
                    .cast::<u32>()
                    .write_unaligned(teb.base as u32 + 0x100);
                teb.base
                    .add(0x100)
                    .cast::<u32>()
                    .write_unaligned(block.mapping.base as u32);
            }
            stack
                .base
                .add(PAGE + STACK - 4)
                .cast::<u32>()
                .write_unaligned(gates.base as u32);
        }
        gates.protect(0, PAGE, 5)?;
        image.protect(0, parsed.size, 0)?;
        image.protect(0, parsed.headers, 1)?;
        let mut regions = Vec::new();
        let mut code_regions = Vec::new();
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
            if section.rights & 0x20000000 != 0 {
                code_regions.push((
                    image.base as usize + section.rva,
                    image.base as usize + section.rva + section.raw_size,
                ));
            }
        }
        // A direct dependency may call its imported API gate while this image
        // owns ACTIVE. All borrowed dependency data must retain its real rights.
        for &(_, dependency) in dependencies {
            regions.extend_from_slice(&dependency.mapping.regions);
        }
        regions.sort_unstable();
        regions.dedup();
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
        let entry = if parsed.entry == 0 {
            0
        } else {
            image.base as u32 + parsed.entry as u32
        };
        Ok(Self {
            image,
            exports: parsed.exports()?,
            gates,
            cpu,
            tls,
            tls_entered: Cell::new(false),
            entry,
            relocated,
            code,
            data,
            dll,
            regions,
            code_regions,
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
        if let Some(block) = &self.tls {
            let first = block.mapping.base as usize;
            if address >= first && end <= first + block.size {
                return true;
            }
        }
        let teb = self.cpu.teb.base as usize;
        if address >= teb && end <= teb + self.cpu.teb.len {
            return true;
        }
        let first = self.cpu.stack.base as usize + PAGE;
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
        unsafe { self.invoke(self.entry, &[], CallingConvention32::Cdecl) }
    }
    unsafe fn invoke(
        &self,
        entry: u32,
        args: &[u32],
        convention: CallingConvention32,
    ) -> Result<u32> {
        let (offset, expected_offset) = call_stack(args.len(), convention)?;
        if !self
            .code_regions
            .iter()
            .any(|&(first, last)| entry as usize >= first && (entry as usize) < last)
        {
            return Err("PE32 call target is outside executable image storage".into());
        }
        ACTIVE.with(|slot| {
            if !slot.get().is_null() {
                return Err("nested PE32 entry unsupported".into());
            }
            if self.tls.is_some() && self.tls_entered.get() {
                return Err("PE32 static TLS images support one entry".into());
            }
            let before = segment_state();
            if before[3] != 0 {
                return Err("PE32 requires an unused host FS selector".into());
            }
            let _teb_lease = TebLease::install(self.cpu.teb.base as u32)?;
            slot.set(ptr::from_ref(self).cast());
            let _guard = ActiveGuard;
            let sp = self.cpu.stack.base as u32 + offset as u32;
            unsafe {
                let frame = self.cpu.stack.base.add(offset).cast::<u32>();
                frame.write_unaligned(self.gates.base as u32);
                ptr::copy_nonoverlapping(args.as_ptr(), frame.add(1), args.len());
            }
            if self.tls.is_some() {
                self.tls_entered.set(true);
            }
            let result = unsafe {
                alkyhol_enter_pe32(entry, sp, u32::from(self.code), u32::from(self.data))
            };
            if segment_state() != before {
                unsupported("PE32 host segment selectors were not restored");
            }
            alkyhol::ffi::SetLastError(unsafe {
                self.cpu.teb.base.add(0x34).cast::<u32>().read_unaligned()
            });
            let returned_sp = (result >> 32) as u32;
            let expected_sp = self.cpu.stack.base as u32 + expected_offset as u32;
            if returned_sp != expected_sp {
                return Err(format!(
                    "PE32 {convention:?} call returned with an unbalanced guest stack"
                ));
            }
            Ok(result as u32)
        })
    }
}
/// Stack cleanup expected from an integer-only PE32 exported function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallingConvention32 {
    Cdecl,
    Stdcall,
}
fn call_stack(count: usize, convention: CallingConvention32) -> Result<(usize, usize)> {
    if count > 16 {
        return Err("PE32 calls support at most sixteen DWORD arguments".into());
    }
    // Align the argument vector to 16 bytes; the return address precedes it.
    let top = PAGE + STACK;
    let arguments = (top - count * 4) & !15;
    let frame = arguments - 4;
    let expected = arguments
        + if convention == CallingConvention32::Stdcall {
            count * 4
        } else {
            0
        };
    Ok((frame, expected))
}

/// A trusted i386 DLL with real dynamic PROCESS_ATTACH/DETACH and integer calls.
/// Explicit borrowed dependencies share one guest stack/TEB. This is not a DLL bundle,
/// filesystem search, process module cache or support for DLL TLS/SEH/CRT.
/// The ProcessContext borrow remains live through guest DETACH and unmapping.
///
/// ```compile_fail,E0505
/// # use alkyhol_pe::{ProcessContext, LoadedLibrary32};
/// # fn example(bytes: &[u8]) -> alkyhol_pe::Result<()> {
/// let process = ProcessContext::install(&["library32".into()], &[])?;
/// let library = unsafe { LoadedLibrary32::load(bytes, false, &process) }?;
/// drop(process);
/// drop(library);
/// # Ok(()) }
/// ```
pub struct LoadedLibrary32<'a> {
    mapping: LoadedImage32<'a>,
    attached: bool,
}
impl<'a> LoadedLibrary32<'a> {
    /// Map/bind the complete supported DLL before calling DllMain. FALSE attach
    /// receives one matching dynamic detach; the rejected library never escapes.
    ///
    /// # Safety
    /// DLL code must obey LoadedImage32's trusted integer-only, single-thread
    /// native-code contract and retain no pointers after this object is dropped.
    pub unsafe fn load(bytes: &[u8], rebase: bool, process: &'a ProcessContext) -> Result<Self> {
        unsafe { Self::load_linked(bytes, rebase, process, &[]) }
    }
    /// Bind an explicit set of already initialized DLL dependencies before
    /// attaching this DLL on their shared guest thread. References keep those
    /// mappings alive through this DLL's detach. No search/cache/forwarders.
    ///
    /// # Safety
    /// The same trusted-code contract as `load` applies to the entire dependency
    /// closure. Supplied basenames must identify the actual intended modules.
    ///
    /// ```compile_fail,E0505
    /// # use alkyhol_pe::{ProcessContext, LoadedLibrary32};
    /// # fn example(a: &[u8], b: &[u8]) -> alkyhol_pe::Result<()> {
    /// let process = ProcessContext::install(&["linked32".into()], &[])?;
    /// let leaf = unsafe { LoadedLibrary32::load(a, false, &process) }?;
    /// let child = unsafe { LoadedLibrary32::load_linked(b, false, &process, &[("leaf.dll", &leaf)]) }?;
    /// drop(leaf);
    /// drop(child);
    /// # Ok(()) }
    /// ```
    pub unsafe fn load_linked(
        bytes: &[u8],
        rebase: bool,
        process: &'a ProcessContext,
        dependencies: &[(&str, &'a LoadedLibrary32<'a>)],
    ) -> Result<Self> {
        let mut library = Self {
            mapping: LoadedImage32::load_kind(bytes, rebase, process, true, dependencies)?,
            attached: false,
        };
        if library.mapping.entry != 0 {
            if unsafe { library.entry(1)? } == 0 {
                unsafe {
                    library.entry(0)?;
                }
                return Err("PE32 DLL PROCESS_ATTACH returned FALSE".into());
            }
            library.attached = true;
        }
        Ok(library)
    }
    fn import_address(&self, symbol: &ImportSymbol) -> Result<u32> {
        if matches!(symbol, ImportSymbol::Ordinal(0)) {
            return Err("zero PE32 dependency import ordinals are unsupported".into());
        }
        let export = self
            .mapping
            .exports
            .as_ref()
            .and_then(|t| {
                t.entries.iter().find(|e| match symbol {
                    ImportSymbol::Name(name) => e.names.contains(name),
                    ImportSymbol::Ordinal(ordinal) => e.ordinal == u32::from(*ordinal),
                })
            })
            .ok_or_else(|| format!("missing PE32 dependency export {symbol:?}"))?;
        match export.target {
            ExportTarget::Address { rva } => (self.mapping.image.base as u32)
                .checked_add(rva as u32)
                .ok_or_else(|| "PE32 dependency export address overflow".into()),
            ExportTarget::Forwarder(_) => Err("PE32 dependency forwarders are unsupported".into()),
        }
    }
    unsafe fn entry(&self, reason: u32) -> Result<u32> {
        unsafe {
            self.mapping.invoke(
                self.mapping.entry,
                &[self.mapping.image.base as u32, reason, 0],
                CallingConvention32::Stdcall,
            )
        }
    }
    /// Whether this DLL actually required HIGHLOW relocation.
    pub fn relocated(&self) -> bool {
        self.mapping.relocated()
    }
    /// Invoke a named direct code export with up to sixteen DWORD arguments.
    /// Missing names, data exports, forwarders and stack-cleanup mismatches fail.
    ///
    /// # Safety
    /// The caller must supply the export's actual DWORD signature/convention;
    /// guest pointers must remain valid for the entire trusted call.
    pub unsafe fn call_name(
        &self,
        name: &str,
        args: &[u32],
        convention: CallingConvention32,
    ) -> Result<u32> {
        let export = self
            .mapping
            .exports
            .as_ref()
            .and_then(|t| t.entries.iter().find(|e| e.names.iter().any(|n| n == name)))
            .ok_or_else(|| format!("missing PE32 DLL export {name}"))?;
        let address = match export.target {
            ExportTarget::Address { rva } => self.mapping.image.base as u32 + rva as u32,
            ExportTarget::Forwarder(_) => {
                return Err("PE32 DLL export forwarders are unsupported".into());
            }
        };
        unsafe { self.mapping.invoke(address, args, convention) }
    }
}
impl Drop for LoadedLibrary32<'_> {
    fn drop(&mut self) {
        if self.attached {
            self.attached = false;
            if let Err(error) = unsafe { self.entry(0) } {
                // A failed teardown is not a successfully unloaded DLL.
                unsupported(&error);
            }
        }
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
        // Import calls observe direct guest FS writes. Reflect all host API
        // outcomes back into the same guest field, including early returns.
        let error = unsafe { image.cpu.teb.base.add(0x34).cast::<u32>() };
        alkyhol::ffi::SetLastError(unsafe { error.read_unaligned() });
        let _teb_error = TebError(error);
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
                if image.dll {
                    unsupported("isolated PE32 DLL has no process executable module lookup");
                }
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
            cpu: Rc::new(Cpu32 {
                stack: LowMapping {
                    base: 0x400000usize as *mut u8,
                    len: STACK + 2 * PAGE,
                },
                teb: LowMapping {
                    base: 0x600000usize as *mut u8,
                    len: PAGE,
                },
            }),
            tls: None,
            tls_entered: Cell::new(false),
            entry: 0,
            relocated: false,
            code: 0,
            data: 0,
            dll: false,
            handles: [0; 3],
            regions: vec![
                (0x100000, 0x101000, 0x60000000),
                (0x200000, 0x201000, 0xc0000000),
            ],
            code_regions: vec![(0x100000, 0x100200)],
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
    #[test]
    fn teb_descriptor_has_a_bounded_writable_user_data_segment_and_exact_base() {
        for base in [0, 0x12345678u32, 0xfffff000] {
            let d = teb_descriptor(base);
            let decoded_base =
                ((d >> 16) & 0xffff) | (((d >> 32) & 0xff) << 16) | (((d >> 56) & 0xff) << 24);
            let limit = (d & 0xffff) | (((d >> 48) & 0xf) << 16);
            assert_eq!(decoded_base, u64::from(base));
            assert_eq!(limit, PAGE as u64 - 1);
            assert_eq!((d >> 40) & 0xff, 0xf2); // Present, user, writable data.
            assert_eq!((d >> 52) & 0xf, 4); // Byte limit, 32-bit, no long-mode bit.
        }
    }
    #[test]
    fn minimal_teb_initializes_self_stack_and_error_without_inventing_tls_or_peb() {
        let b = teb_bytes(0x600000, 0x401000, STACK as u32).unwrap();
        let read = |at| u32::from_le_bytes(b[at..at + 4].try_into().unwrap());
        assert_eq!(read(0), u32::MAX);
        assert_eq!(read(4), 0x501000);
        assert_eq!(read(8), 0x401000);
        assert_eq!(read(0x18), 0x600000);
        for at in [0x2c, 0x30, 0x34] {
            assert_eq!(read(at), 0);
        }
        assert!(teb_bytes(0x600000, u32::MAX - 4, 8).is_err());
        assert!(teb_bytes(0x600000, 0x401000, 0).is_err());
        assert!(teb_bytes(u32::MAX - 4, 0x401000, 4096).is_err());
        let image = layout();
        assert!(image.permits(0x600000, PAGE, false));
        assert!(image.permits(0x600034, 4, true));
        assert!(!image.permits(0x600fff, 2, true));
        assert!(!image.permits(0x5fffff, 1, false));
    }
    #[test]
    fn last_error_guard_reflects_early_host_outcomes_and_exclusive_entry_fails_before_ldt() {
        let previous = alkyhol::ffi::GetLastError();
        let mut guest_error = 0x31415926u32;
        {
            let _sync = TebError(&mut guest_error);
            alkyhol::ffi::SetLastError(127);
        }
        assert_eq!(guest_error, 127);
        alkyhol::ffi::SetLastError(previous);
        let _occupied = FS_SLOT.lock().unwrap();
        assert!(TebLease::install(0x600000).is_err());
    }
    #[test]
    fn tls_buffer_permissions_use_the_logical_template_extent_without_page_padding() {
        let mut image = layout();
        image.tls = Some(TlsBlock {
            mapping: LowMapping {
                base: 0x700000usize as *mut u8,
                len: PAGE,
            },
            size: 28,
        });
        assert!(image.permits(0x700000, 28, false));
        assert!(image.permits(0x700018, 4, true));
        assert!(!image.permits(0x700000, 29, true));
        assert!(!image.permits(0x70001c, 1, false));
        assert!(!image.permits(0x6fffff, 1, false));
    }
    #[test]
    fn cookie_policy_preserves_legacy64_and_nondefault_seeds_without_requesting_entropy() {
        for (size, seed) in [
            (64, DEFAULT_GS_COOKIE),
            (64, 0x1234abcd),
            (72, 0),
            (72, 0x1234),
            (72, 0x1234abcd),
        ] {
            assert_eq!(
                select_cookie(size, seed, || panic!("entropy must not be requested")).unwrap(),
                seed
            );
        }
        let mut values = [0, DEFAULT_GS_COOKIE, 0x1234abcd].into_iter();
        assert_eq!(
            select_cookie(72, DEFAULT_GS_COOKIE, || Ok(values.next().unwrap())).unwrap(),
            0x1234abcd
        );
        assert_eq!(values.next(), None);
    }
    #[test]
    fn unavailable_or_exhausted_entropy_never_becomes_a_successful_fixed_cookie() {
        assert!(
            select_cookie(72, DEFAULT_GS_COOKIE, || Err("entropy unavailable".into())).is_err()
        );
        let mut requests = 0;
        assert!(
            select_cookie(72, DEFAULT_GS_COOKIE, || {
                requests += 1;
                Ok(0)
            })
            .is_err()
        );
        assert_eq!(requests, 128);
    }
    #[test]
    fn call_frames_align_arguments_bound_arity_and_distinguish_actual_stack_cleanup() {
        for count in 0..=16 {
            let (cdecl, c_return) = call_stack(count, CallingConvention32::Cdecl).unwrap();
            let (stdcall, s_return) = call_stack(count, CallingConvention32::Stdcall).unwrap();
            assert_eq!(cdecl, stdcall);
            assert_eq!((cdecl + 4) % 16, 0);
            assert!(cdecl >= PAGE && cdecl + 4 + count * 4 <= PAGE + STACK);
            assert_eq!(c_return, cdecl + 4);
            assert_eq!(s_return, cdecl + 4 + count * 4);
        }
        for count in [17, usize::MAX] {
            assert!(call_stack(count, CallingConvention32::Cdecl).is_err());
            assert!(call_stack(count, CallingConvention32::Stdcall).is_err());
        }
    }
    #[test]
    fn explicit_dependency_roster_requires_unique_basenames_and_one_actual_cpu_context() {
        let a = std::mem::ManuallyDrop::new(LoadedLibrary32 {
            mapping: std::mem::ManuallyDrop::into_inner(layout()),
            attached: false,
        });
        let mut b = std::mem::ManuallyDrop::new(LoadedLibrary32 {
            mapping: std::mem::ManuallyDrop::into_inner(layout()),
            attached: false,
        });
        assert!(dependency_cpu(&[]).unwrap().is_none());
        let cpu = dependency_cpu(&[("Leaf.DLL", &a)]).unwrap().unwrap();
        assert!(Rc::ptr_eq(&cpu, &a.mapping.cpu));
        for name in ["../leaf.dll", "leaf", "kernel32.dll", "", "/leaf.dll"] {
            assert!(dependency_cpu(&[(name, &a)]).is_err());
        }
        assert!(dependency_cpu(&[("leaf.dll", &a), ("LEAF.DLL", &a)]).is_err());
        assert!(dependency_cpu(&[("leaf.dll", &a), ("other.dll", &b)]).is_err());
        // The synthetic mappings are never dereferenced or freed.
        let _unused = std::mem::ManuallyDrop::new(std::mem::replace(
            &mut b.mapping.cpu,
            Rc::clone(&a.mapping.cpu),
        ));
        assert!(dependency_cpu(&[("leaf.dll", &a), ("other.dll", &b)]).is_ok());
        assert!(dependency_cpu(&vec![("leaf.dll", &*a); 65]).is_err());
    }
    #[test]
    fn dependency_symbol_resolution_keeps_names_ordinals_data_and_checked_addresses() {
        let mut lib = std::mem::ManuallyDrop::new(LoadedLibrary32 {
            mapping: std::mem::ManuallyDrop::into_inner(layout()),
            attached: false,
        });
        lib.mapping.image.base = 0x100000usize as *mut u8;
        assert!(
            lib.import_address(&ImportSymbol::Name("Function".into()))
                .is_err()
        );
        lib.mapping.exports = Some(ExportTable {
            dll_name: "owned.dll".into(),
            ordinal_base: 17,
            entries: vec![
                crate::Export {
                    ordinal: 17,
                    names: vec!["Function".into()],
                    target: ExportTarget::Address { rva: 0x1000 },
                },
                crate::Export {
                    ordinal: 18,
                    names: vec!["Data".into()],
                    target: ExportTarget::Address { rva: 0x2000 },
                },
            ],
        });
        assert_eq!(
            lib.import_address(&ImportSymbol::Name("Function".into()))
                .unwrap(),
            0x101000
        );
        assert_eq!(
            lib.import_address(&ImportSymbol::Name("Data".into()))
                .unwrap(),
            0x102000
        );
        assert_eq!(
            lib.import_address(&ImportSymbol::Ordinal(18)).unwrap(),
            0x102000
        );
        for symbol in [
            ImportSymbol::Name("function".into()),
            ImportSymbol::Name("Missing".into()),
            ImportSymbol::Ordinal(0),
            ImportSymbol::Ordinal(19),
        ] {
            assert!(lib.import_address(&symbol).is_err());
        }
        lib.mapping.exports.as_mut().unwrap().entries[0].ordinal = 0;
        assert!(lib.import_address(&ImportSymbol::Ordinal(0)).is_err());
        lib.mapping.exports.as_mut().unwrap().entries[0].ordinal = 17;
        lib.mapping.exports.as_mut().unwrap().entries[0].target =
            ExportTarget::Forwarder("other.F".into());
        assert!(lib.import_address(&ImportSymbol::Ordinal(17)).is_err());
        lib.mapping.exports.as_mut().unwrap().entries[0].target =
            ExportTarget::Address { rva: 0xffff0000 };
        assert!(lib.import_address(&ImportSymbol::Ordinal(17)).is_err());
    }
}
