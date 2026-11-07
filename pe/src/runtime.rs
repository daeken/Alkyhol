use crate::{Image, Result, abi};
use std::{collections::BTreeMap, ffi::c_void, ptr};

unsafe extern "C" {
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
    fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
    fn getpagesize() -> i32;
}
const READ: i32 = 1;
const WRITE: i32 = 2;
const EXEC: i32 = 4;
const PRIVATE_ANON: i32 = 2 | 0x1000; // Darwin MAP_PRIVATE | MAP_ANON; never MAP_FIXED.

/// Owned executable mapping. Loading performs all metadata/import checks before
/// allocation and never enters guest code. Dropping unmaps it; code may not retain
/// pointers or start threads beyond the mapping's lifetime.
pub struct LoadedImage {
    base: *mut u8,
    size: usize,
    entry: usize,
    relocated: bool,
}
impl LoadedImage {
    /// Map checked AMD64 PE data, apply DIR64 relocations and bind an explicit
    /// kernel32 whitelist. `rebase` deliberately avoids the preferred address.
    pub fn load(bytes: &[u8], rebase: bool) -> Result<Self> {
        let image = Image::parse(bytes)?;
        image.check_startup()?;
        Self::map(&image, rebase, &abi::resolve)
    }
    fn map(
        image: &Image<'_>,
        rebase: bool,
        resolve: &impl Fn(&str, &str) -> Option<usize>,
    ) -> Result<Self> {
        let bytes = image.bytes;
        let imports = image.imports()?;
        let relocations = image.relocations()?;
        let mut bindings = Vec::new();
        for import in &imports {
            let address = resolve(&import.module, &import.symbol)
                .ok_or_else(|| format!("unsupported import {}!{}", import.module, import.symbol))?;
            if relocations
                .iter()
                .any(|&r| r < import.iat + 8 && import.iat < r + 8)
            {
                return Err("IAT overlaps relocation fixup".into());
            }
            bindings.push((import.iat, address));
        }
        // This contract uses independent 4K section pages. Do not accidentally
        // grant neighbouring sections RWX on a different host page geometry.
        if unsafe { getpagesize() } != 4096 {
            return Err("PE runtime requires 4K x86 process pages".into());
        }
        let hint = if rebase {
            ptr::null_mut()
        } else {
            image.base as *mut c_void
        };
        // Reserve extra space and trim to Windows' 64K image-base granularity.
        // MAP_FIXED is never used. Update ownership after each successful trim
        // so failure cleanup never unmaps a gap another thread could reuse.
        let reserved = image.size + 65536;
        let raw = unsafe { mmap(hint, reserved, READ | WRITE, PRIVATE_ANON, -1, 0) };
        if raw as isize == -1 {
            return Err(format!(
                "image mmap failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        let mut mapped = Self {
            base: raw.cast(),
            size: reserved,
            entry: image.entry,
            relocated: false,
        };
        let aligned = (raw as usize)
            .checked_add(65535)
            .ok_or("image alignment overflow")?
            & !65535;
        let prefix = aligned - raw as usize;
        if prefix != 0 {
            if unsafe { munmap(raw, prefix) } != 0 {
                return Err("image prefix trim failed".into());
            }
            mapped.base = aligned as *mut u8;
            mapped.size -= prefix;
        }
        let suffix = mapped.size - image.size;
        if suffix != 0 {
            if unsafe { munmap(mapped.base.add(image.size).cast(), suffix) } != 0 {
                return Err("image suffix trim failed".into());
            }
            mapped.size = image.size;
        }
        mapped.relocated = mapped.base as u64 != image.base;
        if rebase && !mapped.relocated {
            return Err("forced rebasing did not obtain a different address".into());
        }
        if mapped.relocated && relocations.is_empty() {
            return Err("preferred image base unavailable and no DIR64 relocation targets".into());
        }
        // SAFETY: checked sections fit, don't overlap and all writes precede W^X protections.
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), mapped.base, image.headers);
            for s in &image.sections {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(s.raw),
                    mapped.base.add(s.rva),
                    s.raw_size,
                );
            }
            if mapped.relocated {
                let delta = (mapped.base as u64).wrapping_sub(image.base);
                for rva in relocations {
                    let p = mapped.base.add(rva).cast::<u64>();
                    ptr::write_unaligned(p, ptr::read_unaligned(p).wrapping_add(delta));
                }
            }
            for (iat, address) in bindings {
                ptr::write_unaligned(mapped.base.add(iat).cast::<usize>(), address);
            }
        }
        mapped.protect(0, mapped.size, 0)?;
        mapped.protect(0, image.headers, READ)?;
        for s in &image.sections {
            let mut rights = 0;
            if s.rights & 0x40000000 != 0 {
                rights |= READ;
            }
            if s.rights & 0x80000000 != 0 {
                rights |= WRITE;
            }
            if s.rights & 0x20000000 != 0 {
                rights |= EXEC;
            }
            mapped.protect(s.rva, s.size, rights)?;
        }
        Ok(mapped)
    }
    fn protect(&self, rva: usize, len: usize, rights: i32) -> Result<()> {
        // SAFETY: parser ensures aligned, disjoint, in-mapping ranges. mprotect
        // rounds the ending page; the header/section page extents never overlap.
        if unsafe { mprotect(self.base.add(rva).cast(), len, rights) } != 0 {
            return Err(format!(
                "image mprotect failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }
    pub fn relocated(&self) -> bool {
        self.relocated
    }
    /// Enter trusted Windows machine code on the current thread using Win64 ABI.
    ///
    /// # Safety
    /// Caller must trust all executable bytes to obey this runtime's bounded
    /// contract (no direct syscalls, GS/TEB accesses, SEH, guest callbacks/threads,
    /// arbitrary host memory access or pointers escaping this mapping). This is
    /// native execution, not validation of code safety or a security sandbox.
    pub unsafe fn enter(&self) -> u32 {
        let entry: unsafe extern "win64" fn() -> u32 =
            unsafe { std::mem::transmute(self.base.add(self.entry)) };
        unsafe { entry() }
    }
}
impl Drop for LoadedImage {
    fn drop(&mut self) {
        unsafe {
            munmap(self.base.cast(), self.size);
        }
    }
}

/// An isolated, explicitly loaded trusted DLL. This is not a process module
/// cache or guest LoadLibrary/GetProcAddress implementation. Imports/forwarders
/// can target only the supported kernel32 ABI with the isolated load constructor;
/// see LinkedLibrary for explicitly borrowed DLL dependencies.
pub struct LoadedLibrary {
    mapping: LoadedImage,
    symbols: Vec<(u32, Vec<String>, usize)>,
    attached: bool,
}
impl LoadedLibrary {
    /// Map, relocate, bind and dynamically attach one trusted DLL.
    ///
    /// # Safety
    /// All DLL instructions, DllMain and exported calls must obey LoadedImage's
    /// native-code contract. No threads, escaping pointers, TEB/SEH or reentrant
    /// loader operations are supported. Any ProcessContext used by the DLL must
    /// outlive this object. Drop executes PROCESS_DETACH on this same thread.
    pub unsafe fn load(bytes: &[u8], rebase: bool) -> Result<Self> {
        unsafe { Self::load_resolved(bytes, rebase, &abi::resolve) }
    }
    unsafe fn load_resolved(
        bytes: &[u8],
        rebase: bool,
        resolve: &impl Fn(&str, &str) -> Option<usize>,
    ) -> Result<Self> {
        let image = Image::parse_dll(bytes)?;
        image.check_library_startup()?;
        let exports = image.exports()?.map(|t| t.entries).unwrap_or_default();
        // Resolve every forwarder before any guest initializer can run.
        let mut pending = Vec::new();
        for export in exports {
            let external = match &export.target {
                crate::ExportTarget::Forwarder(value) => Some(resolve_forwarder(value, resolve)?),
                crate::ExportTarget::Address { .. } => None,
            };
            pending.push((export, external));
        }
        let mapping = LoadedImage::map(&image, rebase, resolve)?;
        let symbols = pending
            .into_iter()
            .map(|(e, external)| {
                let address = match e.target {
                    crate::ExportTarget::Address { rva } => mapping.base as usize + rva,
                    crate::ExportTarget::Forwarder(_) => external.unwrap(),
                };
                (e.ordinal, e.names, address)
            })
            .collect();
        let mut library = Self {
            mapping,
            symbols,
            attached: false,
        };
        if library.mapping.entry != 0 {
            if unsafe { library.entry(1) } == 0 {
                // Windows calls DETACH after a failed dynamic ATTACH. The
                // attached flag stays false, so Drop cannot call it twice.
                unsafe {
                    library.entry(0);
                }
                return Err("DLL PROCESS_ATTACH returned FALSE".into());
            }
            library.attached = true;
        }
        Ok(library)
    }
    unsafe fn entry(&self, reason: u32) -> i32 {
        let entry: unsafe extern "win64" fn(*mut c_void, u32, *mut c_void) -> i32 =
            unsafe { std::mem::transmute(self.mapping.base.add(self.mapping.entry)) };
        unsafe { entry(self.mapping.base.cast(), reason, ptr::null_mut()) }
    }
    /// Address of the owned image, not a global Windows module handle.
    pub fn base_address(&self) -> usize {
        self.mapping.base as usize
    }
    pub fn relocated(&self) -> bool {
        self.mapping.relocated
    }
    /// Case-sensitive exported name lookup. The raw pointer can address code or
    /// data and is valid only while this library and its required context live.
    pub fn get_name(&self, name: &str) -> Option<*mut c_void> {
        self.symbols
            .iter()
            .find(|(_, names, _)| names.iter().any(|n| n == name))
            .map(|(_, _, p)| *p as *mut c_void)
    }
    /// Explicit biased ordinal lookup; sparse slots return None. This is a Rust
    /// lookup API, not a promise about every Windows GetProcAddress error case.
    pub fn get_ordinal(&self, ordinal: u32) -> Option<*mut c_void> {
        self.symbols
            .iter()
            .find(|(o, _, _)| *o == ordinal)
            .map(|(_, _, p)| *p as *mut c_void)
    }
}
impl Drop for LoadedLibrary {
    fn drop(&mut self) {
        if self.attached {
            self.attached = false;
            unsafe {
                self.entry(0);
            }
        }
        // The owned mapping drops after DETACH returns.
    }
}
/// One trusted DLL linked against explicitly supplied, already attached DLLs.
/// Borrowing retains every dependency until this DLL has detached and unmapped.
/// There is no filesystem search, automatic graph loading or global module cache.
pub struct LinkedLibrary<'a> {
    library: LoadedLibrary,
    _dependencies: BTreeMap<String, &'a LoadedLibrary>,
}
impl<'a> LinkedLibrary<'a> {
    /// Bind named imports and named forwarders to explicit dependency instances.
    /// Names are ASCII DLL basenames, matched case-insensitively. Symbols remain
    /// case-sensitive. kernel32 is reserved for the explicit native ABI bridges.
    /// At most 64 dependencies are accepted. None are loaded or reinitialized.
    ///
    /// # Safety
    /// LoadedLibrary's trusted-code/context rules apply to this DLL and every
    /// dependency. The caller must supply the intended module identities and a
    /// correctly initialized acyclic dependency order. Reentrancy, threads, SEH,
    /// static TLS and ordinal imports/forwarders remain unsupported.
    pub unsafe fn load(
        bytes: &[u8],
        rebase: bool,
        dependencies: &[(&str, &'a LoadedLibrary)],
    ) -> Result<Self> {
        if dependencies.len() > 64 {
            return Err("more than 64 explicit DLL dependencies".into());
        }
        let mut modules = BTreeMap::new();
        for &(name, library) in dependencies {
            let name = name.to_ascii_lowercase();
            if name.len() > 255
                || !name.ends_with(".dll")
                || !name
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                || !name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                || name == "kernel32.dll"
            {
                return Err("invalid or reserved explicit DLL basename".into());
            }
            if modules.insert(name, library).is_some() {
                return Err("duplicate case-insensitive DLL dependency".into());
            }
        }
        let resolve = |module: &str, symbol: &str| {
            abi::resolve(module, symbol).or_else(|| {
                modules
                    .get(&module.to_ascii_lowercase())
                    .and_then(|library| library.get_name(symbol))
                    .map(|p| p as usize)
            })
        };
        let library = unsafe { LoadedLibrary::load_resolved(bytes, rebase, &resolve) }?;
        Ok(Self {
            library,
            _dependencies: modules,
        })
    }
    /// Borrow the attached DLL for lookup or as another explicit dependency.
    /// Its lifetime cannot escape this owner and its transitive dependency borrows.
    pub fn library(&self) -> &LoadedLibrary {
        &self.library
    }
}

fn resolve_forwarder(value: &str, resolve: &impl Fn(&str, &str) -> Option<usize>) -> Result<usize> {
    let (module, name) = value.rsplit_once('.').ok_or("invalid export forwarder")?;
    let module = if module.to_ascii_lowercase().ends_with(".dll") {
        module.to_owned()
    } else {
        format!("{module}.dll")
    };
    if name.starts_with('#') {
        return Err("ordinal export forwarders are unsupported".into());
    }
    resolve(&module, name).ok_or_else(|| format!("unsupported export forwarder {value}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::process::ExitStatusExt;

    // Small conventional PE: RX code returns 42, RO data, RW zero-filled data.
    fn permission_fixture() -> Vec<u8> {
        let mut b = vec![0; 0xa00];
        b[..2].copy_from_slice(b"MZ");
        let p16 = |b: &mut [u8], o, n: u16| b[o..o + 2].copy_from_slice(&n.to_le_bytes());
        let p32 = |b: &mut [u8], o, n: u32| b[o..o + 4].copy_from_slice(&n.to_le_bytes());
        p32(&mut b, 0x3c, 0x80);
        b[0x80..0x84].copy_from_slice(b"PE\0\0");
        p16(&mut b, 0x84, 0x8664);
        p16(&mut b, 0x86, 3);
        p16(&mut b, 0x94, 240);
        p16(&mut b, 0x96, 0x22);
        let o = 0x98;
        p16(&mut b, o, 0x20b);
        p32(&mut b, o + 16, 0x1000);
        b[o + 24..o + 32].copy_from_slice(&0x140000000u64.to_le_bytes());
        p32(&mut b, o + 32, 4096);
        p32(&mut b, o + 36, 512);
        p32(&mut b, o + 56, 0x4000);
        p32(&mut b, o + 60, 0x400);
        p16(&mut b, o + 68, 3);
        p32(&mut b, o + 108, 16);
        for (i, rva, raw, rights) in [
            (0, 0x1000, 0x400, 0x60000020),
            (1, 0x2000, 0x600, 0x40000040),
            (2, 0x3000, 0x800, 0xc0000040),
        ] {
            let s = o + 240 + i * 40;
            p32(&mut b, s + 8, 512);
            p32(&mut b, s + 12, rva);
            p32(&mut b, s + 16, 512);
            p32(&mut b, s + 20, raw);
            p32(&mut b, s + 36, rights);
        }
        b[0x400..0x406].copy_from_slice(&[0xb8, 42, 0, 0, 0, 0xc3]);
        // One DIR64 entry in RO data, fixing a pointer in RW data.
        p32(&mut b, o + 112 + 5 * 8, 0x2000);
        p32(&mut b, o + 116 + 5 * 8, 12);
        p32(&mut b, 0x600, 0x3000);
        p32(&mut b, 0x604, 12);
        p16(&mut b, 0x608, 0xa000);
        b[0x800..0x808].copy_from_slice(&0x140001000u64.to_le_bytes());
        b
    }

    #[test]
    fn image_permissions_are_enforced_by_the_host() {
        const CHILD: &str = "ALKYHOL_PE_PERMISSION_CHILD";
        if let Ok(mode) = std::env::var(CHILD) {
            let bytes = permission_fixture();
            let mapped = LoadedImage::load(&bytes, true).unwrap();
            unsafe {
                assert_eq!(
                    mapped.base.add(0x3000).cast::<usize>().read(),
                    mapped.base.add(0x1000) as usize
                );
                match mode.as_str() {
                    "header-write" => mapped.base.write_volatile(0),
                    "code-write" => mapped.base.add(0x1000).write_volatile(0),
                    "rodata-write" => mapped.base.add(0x2000).write_volatile(0),
                    "data-execute" => {
                        mapped.base.add(0x3010).write_volatile(0xc3);
                        let f: unsafe extern "win64" fn() =
                            std::mem::transmute(mapped.base.add(0x3010));
                        f();
                    }
                    "allowed" => {
                        mapped.base.add(0x3010).write_volatile(0x55);
                        assert_eq!(mapped.base.add(0x3010).read_volatile(), 0x55);
                        assert_eq!(mapped.enter(), 42);
                        return;
                    }
                    _ => panic!("unknown permission case"),
                }
            }
            panic!("forbidden mapping access unexpectedly succeeded");
        }
        let current = std::env::current_exe().unwrap();
        for mode in [
            "allowed",
            "header-write",
            "code-write",
            "rodata-write",
            "data-execute",
        ] {
            let child = std::process::Command::new(&current)
                .args([
                    "--exact",
                    "runtime::tests::image_permissions_are_enforced_by_the_host",
                    "--nocapture",
                ])
                .env(CHILD, mode)
                .output()
                .unwrap();
            if mode == "allowed" {
                assert!(child.status.success(), "{child:?}");
            } else {
                assert!(
                    matches!(child.status.signal(), Some(10 | 11)),
                    "{mode}: {child:?}"
                );
            }
        }
    }
    #[repr(C)]
    #[derive(Clone, Copy, Debug)]
    struct LifecycleLog {
        reason: u32,
        calls: u32,
        module: usize,
        reserved: usize,
    }
    fn lifecycle_log() -> Box<std::cell::UnsafeCell<LifecycleLog>> {
        Box::new(std::cell::UnsafeCell::new(LifecycleLog {
            reason: 0xdead,
            calls: 0,
            module: 0,
            reserved: usize::MAX,
        }))
    }
    fn lifecycle_fixture(log: *mut LifecycleLog, accept: bool, forwarder: Option<&str>) -> Vec<u8> {
        let mut b = permission_fixture();
        let put32 = |b: &mut [u8], o, n: u32| b[o..o + 4].copy_from_slice(&n.to_le_bytes());
        b[0x96..0x98].copy_from_slice(&0x2022u16.to_le_bytes());
        b[0x810..0x818].copy_from_slice(&(log as usize as u64).to_le_bytes());
        // Win64 DllMain: capture EDX reason, RCX module and R8 reserved through
        // an external UnsafeCell. The pointer is data, not an executable fixup.
        let mut code = vec![0x48, 0x8b, 0x05];
        code.extend(0x2009u32.to_le_bytes());
        code.extend([
            0x89, 0x10, 0xff, 0x40, 0x04, 0x48, 0x89, 0x48, 0x08, 0x4c, 0x89, 0x40, 0x10, 0xb8,
        ]);
        code.extend(u32::from(accept).to_le_bytes());
        code.push(0xc3);
        b[0x400..0x400 + code.len()].copy_from_slice(&code);
        let o = 0x98;
        put32(&mut b, o + 112, 0x2080);
        put32(&mut b, o + 116, 0x100);
        put32(&mut b, 0x68c, 0x20c0);
        put32(&mut b, 0x690, 17);
        put32(&mut b, 0x694, 1);
        put32(&mut b, 0x698, 1);
        put32(&mut b, 0x69c, 0x20d0);
        put32(&mut b, 0x6a0, 0x20d8);
        put32(&mut b, 0x6a4, 0x20dc);
        b[0x6c0..0x6ce].copy_from_slice(b"lifecycle.dll\0");
        put32(
            &mut b,
            0x6d0,
            if forwarder.is_some() { 0x20f0 } else { 0x1000 },
        );
        put32(&mut b, 0x6d8, 0x20e0);
        b[0x6dc..0x6de].fill(0);
        b[0x6e0..0x6e6].copy_from_slice(b"Entry\0");
        if let Some(text) = forwarder {
            b[0x6f0..0x6f0 + text.len()].copy_from_slice(text.as_bytes());
            b[0x6f0 + text.len()] = 0;
        }
        b
    }
    #[test]
    fn library_attach_lookup_detach_and_win64_arguments_are_real() {
        let log = lifecycle_log();
        let bytes = lifecycle_fixture(log.get(), true, None);
        let lib = unsafe { LoadedLibrary::load(&bytes, true) }.unwrap();
        assert!(lib.relocated());
        assert_eq!(lib.base_address() % 65536, 0);
        let first = unsafe { *log.get() };
        assert_eq!(
            (first.reason, first.calls, first.module, first.reserved),
            (1, 1, lib.base_address(), 0)
        );
        assert_eq!(
            lib.get_name("Entry").unwrap() as usize,
            lib.base_address() + 0x1000
        );
        assert_eq!(lib.get_name("Entry"), lib.get_ordinal(17));
        assert!(lib.get_name("entry").is_none());
        assert!(lib.get_name("missing").is_none());
        assert!(lib.get_ordinal(16).is_none());
        assert!(lib.get_ordinal(18).is_none());
        let base = lib.base_address();
        drop(lib);
        let last = unsafe { *log.get() };
        assert_eq!(
            (last.reason, last.calls, last.module, last.reserved),
            (0, 2, base, 0)
        );
    }
    #[test]
    fn failed_library_attach_detaches_once_and_returns_an_error() {
        let log = lifecycle_log();
        let bytes = lifecycle_fixture(log.get(), false, None);
        let error = unsafe { LoadedLibrary::load(&bytes, true) }.err().unwrap();
        assert!(error.contains("ATTACH returned FALSE"));
        let record = unsafe { *log.get() };
        assert_eq!((record.reason, record.calls, record.reserved), (0, 2, 0));
    }
    fn dependent_fixture(log: *mut LifecycleLog, accept: bool) -> Vec<u8> {
        let mut bytes = lifecycle_fixture(log, accept, Some("custom.Entry"));
        let p32 = |b: &mut [u8], o, n: u32| b[o..o + 4].copy_from_slice(&n.to_le_bytes());
        p32(&mut bytes, 0x98 + 120, 0x2020);
        p32(&mut bytes, 0x98 + 124, 40);
        p32(&mut bytes, 0x620, 0x21a0);
        p32(&mut bytes, 0x62c, 0x21c0);
        p32(&mut bytes, 0x630, 0x3020);
        bytes[0x7a0..0x7a8].copy_from_slice(&0x21d0u64.to_le_bytes());
        bytes[0x7c0..0x7cb].copy_from_slice(b"custom.dll\0");
        bytes[0x7d2..0x7d8].copy_from_slice(b"Entry\0");
        bytes
    }
    #[test]
    fn explicit_dependencies_bind_actual_iat_forwarders_and_detach_before_borrowed_dll() {
        let child_log = lifecycle_log();
        let child_bytes = lifecycle_fixture(child_log.get(), true, None);
        let child = unsafe { LoadedLibrary::load(&child_bytes, true) }.unwrap();
        let parent_log = lifecycle_log();
        let parent_bytes = dependent_fixture(parent_log.get(), true);
        // The isolated API must keep rejecting foreign dependencies.
        assert!(unsafe { LoadedLibrary::load(&parent_bytes, true) }.is_err());
        assert_eq!(unsafe { (*parent_log.get()).calls }, 0);
        let parent =
            unsafe { LinkedLibrary::load(&parent_bytes, true, &[("CuStOm.DlL", &child)]) }.unwrap();
        assert!(parent.library().relocated());
        let address = child.get_name("Entry").unwrap();
        assert_eq!(parent.library().get_name("Entry"), Some(address));
        assert_eq!(
            unsafe {
                parent
                    .library
                    .mapping
                    .base
                    .add(0x3020)
                    .cast::<usize>()
                    .read()
            },
            address as usize
        );
        assert_eq!(
            unsafe { ((*child_log.get()).calls, (*parent_log.get()).calls) },
            (1, 1)
        );
        drop(parent);
        assert_eq!(
            unsafe { ((*child_log.get()).calls, (*parent_log.get()).calls) },
            (1, 2)
        );
        drop(child);
        assert_eq!(unsafe { (*child_log.get()).calls }, 2);
    }
    #[test]
    fn explicit_dependency_errors_are_pre_entry_and_failed_parent_leaves_child_attached() {
        let child_log = lifecycle_log();
        let child_bytes = lifecycle_fixture(child_log.get(), true, None);
        let child = unsafe { LoadedLibrary::load(&child_bytes, true) }.unwrap();
        let log = lifecycle_log();
        let bytes = dependent_fixture(log.get(), true);
        for name in [
            "kernel32.dll",
            "../custom.dll",
            "custom",
            "",
            "💮.dll",
            "/custom.dll",
        ] {
            assert!(unsafe { LinkedLibrary::load(&bytes, true, &[(name, &child)]) }.is_err());
        }
        let overlong = format!("{}.dll", "a".repeat(252));
        assert!(unsafe { LinkedLibrary::load(&bytes, true, &[(&overlong, &child)]) }.is_err());
        assert!(unsafe { LinkedLibrary::load(&bytes, true, &[]) }.is_err());
        assert!(
            unsafe {
                LinkedLibrary::load(
                    &bytes,
                    true,
                    &[("custom.dll", &child), ("CUSTOM.DLL", &child)],
                )
            }
            .is_err()
        );
        let too_many = vec![("custom.dll", &child); 65];
        assert!(unsafe { LinkedLibrary::load(&bytes, true, &too_many) }.is_err());
        for mode in [
            "wrong-symbol",
            "wrong-module",
            "ordinal",
            "overlapping-relocation",
        ] {
            let mut invalid = bytes.clone();
            match mode {
                "wrong-symbol" => invalid[0x7d2] = b'e',
                "wrong-module" => invalid[0x7c0] = b'x',
                "ordinal" => {
                    invalid[0x7a0..0x7a8].copy_from_slice(&0x8000000000000011u64.to_le_bytes())
                }
                "overlapping-relocation" => {
                    invalid[0x608..0x60a].copy_from_slice(&0xa020u16.to_le_bytes())
                }
                _ => unreachable!(),
            }
            assert!(
                unsafe { LinkedLibrary::load(&invalid, true, &[("custom.dll", &child)]) }.is_err(),
                "{mode}"
            );
        }
        assert_eq!(unsafe { (*log.get()).calls }, 0);
        let rejected = dependent_fixture(log.get(), false);
        assert!(
            unsafe { LinkedLibrary::load(&rejected, true, &[("custom.dll", &child)]) }.is_err()
        );
        assert_eq!(
            unsafe { ((*log.get()).calls, (*child_log.get()).calls) },
            (2, 1)
        );
    }
    #[test]
    fn explicit_transitive_dependency_borrows_preserve_resolved_export_targets() {
        let leaf_log = lifecycle_log();
        let leaf_bytes = lifecycle_fixture(leaf_log.get(), true, None);
        let leaf = unsafe { LoadedLibrary::load(&leaf_bytes, true) }.unwrap();
        let middle_log = lifecycle_log();
        let middle_bytes = dependent_fixture(middle_log.get(), true);
        let middle =
            unsafe { LinkedLibrary::load(&middle_bytes, true, &[("custom.dll", &leaf)]) }.unwrap();
        let root_log = lifecycle_log();
        let root_bytes = dependent_fixture(root_log.get(), true);
        let root =
            unsafe { LinkedLibrary::load(&root_bytes, true, &[("custom.dll", middle.library())]) }
                .unwrap();
        assert_eq!(root.library().get_name("Entry"), leaf.get_name("Entry"));
        drop(root);
        assert_eq!(
            unsafe {
                (
                    (*root_log.get()).calls,
                    (*middle_log.get()).calls,
                    (*leaf_log.get()).calls,
                )
            },
            (2, 1, 1)
        );
        drop(middle);
        assert_eq!(unsafe { (*middle_log.get()).calls }, 2);
        drop(leaf);
        assert_eq!(unsafe { (*leaf_log.get()).calls }, 2);
    }
    #[test]
    fn unsupported_library_features_fail_before_any_guest_initializer() {
        for forwarder in ["vendor.Missing", "kernel32.#7", "Malformed"] {
            let log = lifecycle_log();
            let bytes = lifecycle_fixture(log.get(), true, Some(forwarder));
            assert!(unsafe { LoadedLibrary::load(&bytes, true) }.is_err());
            assert_eq!(unsafe { (*log.get()).calls }, 0);
        }
        for dll in [true, false] {
            let log = lifecycle_log();
            let mut bytes = lifecycle_fixture(log.get(), true, None);
            if dll {
                bytes[0x98 + 112 + 9 * 8..0x98 + 120 + 9 * 8]
                    .copy_from_slice(&[0, 0x20, 0, 0, 16, 0, 0, 0]);
            } else {
                bytes[0x96..0x98].copy_from_slice(&0x22u16.to_le_bytes());
            }
            assert!(unsafe { LoadedLibrary::load(&bytes, true) }.is_err());
            assert_eq!(unsafe { (*log.get()).calls }, 0);
        }
    }
    #[test]
    fn zero_entry_and_actual_kernel32_forwarding_have_distinct_behavior() {
        let log = lifecycle_log();
        let mut bytes = lifecycle_fixture(log.get(), true, None);
        bytes[0xa8..0xac].fill(0);
        let lib = unsafe { LoadedLibrary::load(&bytes, true) }.unwrap();
        drop(lib);
        assert_eq!(unsafe { (*log.get()).calls }, 0);
        let bytes = lifecycle_fixture(log.get(), true, Some("kernel32.dll.GetLastError"));
        let lib = unsafe { LoadedLibrary::load(&bytes, true) }.unwrap();
        let forward: unsafe extern "win64" fn() -> u32 =
            unsafe { std::mem::transmute(lib.get_name("Entry").unwrap()) };
        alkyhol::ffi::SetLastError(0x1234);
        assert_eq!(unsafe { forward() }, 0x1234);
        drop(lib);
        assert_eq!(unsafe { (*log.get()).calls }, 2);
    }

    #[test]
    fn library_drop_unmaps_the_owned_image_after_detach() {
        const CHILD: &str = "ALKYHOL_PE_DLL_UNMAP_CHILD";
        if std::env::var_os(CHILD).is_some() {
            // Query the actual task map. Darwin mincore returns residency bits
            // for holes, and dereferencing a dangling pointer is not a valid
            // Rust oracle for whether the owned VM mapping was removed.
            unsafe extern "C" {
                static mach_task_self_: u32;
                fn mach_vm_region(
                    task: u32,
                    address: *mut u64,
                    size: *mut u64,
                    flavor: i32,
                    info: *mut i32,
                    count: *mut u32,
                    object: *mut u32,
                ) -> i32;
                fn mach_port_deallocate(task: u32, port: u32) -> i32;
            }
            fn mapped(address: usize) -> bool {
                let mut start = address as u64;
                let mut size = 0;
                let mut info = [0i32; 9];
                let mut count = 9;
                let mut object = 0;
                let task = unsafe { mach_task_self_ };
                let result = unsafe {
                    mach_vm_region(
                        task,
                        &mut start,
                        &mut size,
                        9,
                        info.as_mut_ptr(),
                        &mut count,
                        &mut object,
                    )
                };
                assert_eq!(result, 0);
                if object != 0 {
                    assert_eq!(unsafe { mach_port_deallocate(task, object) }, 0);
                }
                start <= address as u64 && (address as u64) < start + size
            }
            let log = lifecycle_log();
            let bytes = lifecycle_fixture(log.get(), true, None);
            let lib = unsafe { LoadedLibrary::load(&bytes, true) }.unwrap();
            let base = lib.base_address();
            assert!(mapped(base));
            drop(lib);
            assert_eq!(unsafe { (*log.get()).calls }, 2);
            assert!(!mapped(base));
            return;
        }
        let child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "runtime::tests::library_drop_unmaps_the_owned_image_after_detach",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(child.status.success(), "{child:?}");
    }
}
