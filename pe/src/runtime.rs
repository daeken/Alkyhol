use crate::{Image, ImportSymbol, Result, abi};
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

pub(crate) fn resolve_builtin(module: &str, symbol: &ImportSymbol) -> Option<usize> {
    match symbol {
        ImportSymbol::Name(name) => abi::resolve(module, name),
        // Only named system imports have a validated bridge contract.
        ImportSymbol::Ordinal(_) => None,
    }
}

fn dependency_modules<'a>(
    dependencies: &[(&str, &'a LoadedLibrary)],
) -> Result<BTreeMap<String, &'a LoadedLibrary>> {
    if dependencies.len() > 64 {
        return Err("more than 64 explicit DLL dependencies".into());
    }
    let mut modules = BTreeMap::new();
    for &(name, library) in dependencies {
        let name = dll_basename(name)?;
        if modules.insert(name, library).is_some() {
            return Err("duplicate case-insensitive DLL dependency".into());
        }
    }
    Ok(modules)
}
fn resolve_dependency(
    modules: &BTreeMap<String, &LoadedLibrary>,
    module: &str,
    symbol: &ImportSymbol,
) -> Option<usize> {
    resolve_builtin(module, symbol).or_else(|| {
        modules
            .get(&module.to_ascii_lowercase())
            .and_then(|library| match symbol {
                ImportSymbol::Name(name) => library.get_name(name),
                ImportSymbol::Ordinal(ordinal) if *ordinal != 0 => {
                    library.get_ordinal(u32::from(*ordinal))
                }
                ImportSymbol::Ordinal(_) => None,
            })
            .map(|p| p as usize)
    })
}

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
    /// builtin Win64 whitelists. `rebase` deliberately avoids the preferred address.
    pub fn load(bytes: &[u8], rebase: bool) -> Result<Self> {
        let image = Image::parse(bytes)?;
        image.check_startup()?;
        Self::map(&image, rebase, &resolve_builtin)
    }
    fn map(
        image: &Image<'_>,
        rebase: bool,
        resolve: &impl Fn(&str, &ImportSymbol) -> Option<usize>,
    ) -> Result<Self> {
        let bytes = image.bytes;
        let imports = image.imports()?;
        let relocations = image.relocations()?;
        let mut bindings = Vec::new();
        for import in &imports {
            let address = resolve(&import.module, &import.symbol).ok_or_else(|| {
                format!("unsupported import {}!{:?}", import.module, import.symbol)
            })?;
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

/// Executable whose validated static function records are registered until drop.
/// The borrowed process guard must remain alive through registry cleanup/unmapping.
/// Exception dispatch and epilogue unwinding remain unsupported; DLLs use RegisteredLibrary.
///
/// ```compile_fail,E0505
/// # use alkyhol_pe::{ProcessContext,RegisteredImage};
/// # fn example(bytes:&[u8])->alkyhol_pe::Result<()> {
/// let process=ProcessContext::install(&["app.exe".into()],&[])?;
/// let image=RegisteredImage::load(bytes,false,&process)?;
/// drop(process);
/// drop(image);
/// # Ok(()) }
/// ```
pub struct RegisteredImage<'a> {
    image: LoadedImage,
    table: Option<*const crate::RuntimeFunction>,
    _process: &'a crate::ProcessContext,
}
fn register_static_functions(
    image: &LoadedImage,
    functions: Option<crate::RuntimeFunctionTable>,
) -> Result<Option<*const crate::RuntimeFunction>> {
    let Some(functions) = functions else {
        return Ok(None);
    };
    let records = unsafe {
        image
            .base
            .add(functions.rva)
            .cast::<crate::RuntimeFunction>()
    };
    let added = crate::process::with_function_tables(|registry| {
        registry.map(|r| unsafe {
            r.add_static(records, functions.entries.len() as u32, image.base as u64)
        })
    });
    if added != Some(1) {
        return Err("static function registration requires an active process scope and available table capacity".into());
    }
    Ok(Some(records))
}
fn unregister_static_functions(table: &mut Option<*const crate::RuntimeFunction>) {
    if let Some(table) = table.take() {
        crate::process::with_function_tables(|registry| {
            if let Some(r) = registry {
                r.delete_static(table);
            }
        });
    }
}
impl<'a> RegisteredImage<'a> {
    /// Check/map an EXE and register its supported static table before entry.
    /// Unknown recipes/startup features/imports fail before guest code executes.
    pub fn load(bytes: &[u8], rebase: bool, process: &'a crate::ProcessContext) -> Result<Self> {
        let parsed = Image::parse(bytes)?;
        parsed.check_registered_startup()?;
        let functions = parsed.runtime_functions()?;
        let image = LoadedImage::map(&parsed, rebase, &resolve_builtin)?;
        let table = register_static_functions(&image, functions)?;
        Ok(Self {
            image,
            table,
            _process: process,
        })
    }
    /// Whether this checked mapping differs from the PE preferred base.
    pub fn relocated(&self) -> bool {
        self.image.relocated()
    }
    /// Enter the mapped executable.
    ///
    /// # Safety
    /// Same trusted native-code restrictions as LoadedImage::enter. Static table
    /// lookup/body restoration does not install Windows exception dispatch.
    pub unsafe fn enter(&self) -> u32 {
        unsafe { self.image.enter() }
    }
}
impl Drop for RegisteredImage<'_> {
    fn drop(&mut self) {
        unregister_static_functions(&mut self.table);
        // image field drops/unmaps only after its registry entry has been removed.
    }
}

/// One executable linked to explicitly supplied, already attached DLL instances.
/// Loading only checks/maps the EXE; it does not enter or initialize guest code.
/// DLL ownership stays with the caller, with borrows retained through unmapping.
/// There is no filesystem discovery, guest module API or process-global cache.
///
/// ```no_run
/// # use alkyhol_pe::{LinkedImage, LoadedLibrary};
/// # fn example(exe: &[u8], dll: &[u8]) -> alkyhol_pe::Result<()> {
/// let dependency = unsafe { LoadedLibrary::load(dll, false) }?;
/// let program = LinkedImage::load(exe, false, &[("dependency.dll", &dependency)])?;
/// drop(program);
/// drop(dependency);
/// # Ok(()) }
/// ```
/// ```compile_fail,E0505
/// # use alkyhol_pe::{LinkedImage, LoadedLibrary};
/// # fn example(exe: &[u8], dll: &[u8]) -> alkyhol_pe::Result<()> {
/// let dependency = unsafe { LoadedLibrary::load(dll, false) }?;
/// let _program = LinkedImage::load(exe, false, &[("dependency.dll", &dependency)])?;
/// drop(dependency);
/// # Ok(()) }
/// ```
pub struct LinkedImage<'a> {
    image: Option<LoadedImage>,
    _dependencies: BTreeMap<String, &'a LoadedLibrary>,
}
impl<'a> LinkedImage<'a> {
    /// Bind named/ordinal code/data imports against at most 64 explicit DLLs.
    /// DLL basenames are case-insensitive; symbol names remain case-sensitive.
    /// Builtin names are reserved. No DLL is loaded or reinitialized here.
    /// Missing, sparse, zero/system ordinals and unsupported startup fail before
    /// EXE entry. The caller supplies correct dependency identities; entering
    /// the borrowed image requires LoadedImage::enter's trusted-code contract.
    pub fn load(
        bytes: &[u8],
        rebase: bool,
        dependencies: &[(&str, &'a LoadedLibrary)],
    ) -> Result<Self> {
        let modules = dependency_modules(dependencies)?;
        let image = Image::parse(bytes)?;
        image.check_startup()?;
        let mapped = LoadedImage::map(&image, rebase, &|name, symbol| {
            resolve_dependency(&modules, name, symbol)
        })?;
        Ok(Self {
            image: Some(mapped),
            _dependencies: modules,
        })
    }
    /// Borrow the checked mapping; its lifetime cannot escape this owner or its
    /// transitive dependency owners. Calling enter remains explicitly unsafe.
    pub fn image(&self) -> &LoadedImage {
        self.image.as_ref().expect("mapped linked EXE")
    }
}
impl Drop for LinkedImage<'_> {
    fn drop(&mut self) {
        // Explicit destruction enforces Rust drop-checking of dependency
        // borrows even when the owner is implicitly dropped without later use.
        drop(self.image.take());
    }
}

/// An isolated, explicitly loaded trusted DLL. This is not a process module
/// cache or guest LoadLibrary/GetProcAddress implementation. Imports/forwarders
/// can target only the supported builtin ABI with the isolated load constructor;
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
        unsafe { Self::load_resolved(bytes, rebase, &resolve_builtin) }
    }
    pub(crate) unsafe fn load_resolved(
        bytes: &[u8],
        rebase: bool,
        resolve: &impl Fn(&str, &ImportSymbol) -> Option<usize>,
    ) -> Result<Self> {
        let image = Image::parse_dll(bytes)?;
        image.check_library_startup()?;
        let mut library = Self::map_preflighted(&image, rebase, resolve)?;
        unsafe {
            library.attach()?;
        }
        Ok(library)
    }
    // All constructor-specific startup gates must pass before this mapping step.
    fn map_preflighted(
        image: &Image<'_>,
        rebase: bool,
        resolve: &impl Fn(&str, &ImportSymbol) -> Option<usize>,
    ) -> Result<Self> {
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
        let mapping = LoadedImage::map(image, rebase, resolve)?;
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
        Ok(Self {
            mapping,
            symbols,
            attached: false,
        })
    }
    unsafe fn attach(&mut self) -> Result<()> {
        if self.mapping.entry != 0 {
            if unsafe { self.entry(1) } == 0 {
                // Windows calls DETACH after a failed dynamic ATTACH. The
                // attached flag stays false, so Drop cannot call it twice.
                unsafe {
                    self.entry(0);
                }
                return Err("DLL PROCESS_ATTACH returned FALSE".into());
            }
            self.attached = true;
        }
        Ok(())
    }
    fn detach(&mut self) {
        if self.attached {
            self.attached = false;
            unsafe {
                self.entry(0);
            }
        }
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
    pub(crate) fn export_bindings(&self) -> impl Iterator<Item = (u32, &[String], usize)> {
        self.symbols
            .iter()
            .map(|(ordinal, names, address)| (*ordinal, names.as_slice(), *address))
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
        self.detach();
        // The owned mapping drops after DETACH returns.
    }
}
/// Isolated DLL with static function records available throughout attach/detach.
/// Borrows the process scope through detach, registry removal and unmapping.
///
/// ```compile_fail,E0505
/// # use alkyhol_pe::{ProcessContext,RegisteredLibrary};
/// # fn example(bytes:&[u8])->alkyhol_pe::Result<()> {
/// let process=ProcessContext::install(&["host".into()],&[])?;
/// let dll=unsafe { RegisteredLibrary::load(bytes,false,&process) }?;
/// drop(process);
/// drop(dll);
/// # Ok(()) }
/// ```
/// ```no_run
/// # use alkyhol_pe::{ProcessContext,RegisteredLibrary};
/// # fn example(bytes:&[u8])->alkyhol_pe::Result<()> {
/// let process=ProcessContext::install(&["host".into()],&[])?;
/// let dll=unsafe { RegisteredLibrary::load(bytes,false,&process) }?;
/// let _export=dll.library().get_name("Probe");
/// drop(dll);
/// drop(process);
/// # Ok(()) }
/// ```
pub struct RegisteredLibrary<'a> {
    library: Option<LoadedLibrary>,
    table: Option<*const crate::RuntimeFunction>,
    _process: &'a crate::ProcessContext,
}
impl<'a> RegisteredLibrary<'a> {
    /// Preflight/map/register an isolated DLL, then call dynamic PROCESS_ATTACH.
    ///
    /// # Safety
    /// LoadedLibrary's trusted-code contract applies. Only builtin imports and
    /// forwarders are supported. No guest threads, reentrancy, TLS or SEH dispatch.
    pub unsafe fn load(
        bytes: &[u8],
        rebase: bool,
        process: &'a crate::ProcessContext,
    ) -> Result<Self> {
        let image = Image::parse_dll(bytes)?;
        image.check_registered_library_startup()?;
        let functions = image.runtime_functions()?;
        let library = LoadedLibrary::map_preflighted(&image, rebase, &resolve_builtin)?;
        let table = register_static_functions(&library.mapping, functions)?;
        let mut registered = Self {
            library: Some(library),
            table,
            _process: process,
        };
        // On failed attach, its cleanup DETACH runs while static entries remain
        // registered. The local owner's Drop then removes them before unmapping.
        unsafe {
            registered.library.as_mut().unwrap().attach()?;
        }
        Ok(registered)
    }
    /// Borrow exports/base/rebase information; this view cannot outlive its owner.
    pub fn library(&self) -> &LoadedLibrary {
        self.library.as_ref().unwrap()
    }
}
impl Drop for RegisteredLibrary<'_> {
    fn drop(&mut self) {
        if let Some(mut library) = self.library.take() {
            library.detach();
            unregister_static_functions(&mut self.table);
            drop(library);
        }
    }
}
/// One trusted DLL linked against explicitly supplied, already attached DLLs.
/// Borrowing retains every dependency until this DLL has detached and unmapped.
/// There is no filesystem search, automatic graph loading or global module cache.
///
/// Correct teardown order is accepted without executing these example bytes:
/// ```no_run
/// # use alkyhol_pe::{LinkedLibrary, LoadedLibrary};
/// # fn example(bytes: &[u8]) -> alkyhol_pe::Result<()> {
/// let dependency = unsafe { LoadedLibrary::load(bytes, false) }?;
/// let consumer = unsafe { LinkedLibrary::load(bytes, false, &[("dependency.dll", &dependency)]) }?;
/// drop(consumer);
/// drop(dependency);
/// # Ok(()) }
/// ```
/// Dropping a dependency before an implicitly dropped consumer must be rejected:
/// ```compile_fail,E0505
/// # use alkyhol_pe::{LinkedLibrary, LoadedLibrary};
/// # fn example(bytes: &[u8]) -> alkyhol_pe::Result<()> {
/// let dependency = unsafe { LoadedLibrary::load(bytes, false) }?;
/// let _consumer = unsafe { LinkedLibrary::load(bytes, false, &[("dependency.dll", &dependency)]) }?;
/// drop(dependency);
/// # Ok(()) }
/// ```
pub struct LinkedLibrary<'a> {
    library: Option<LoadedLibrary>,
    _dependencies: BTreeMap<String, &'a LoadedLibrary>,
}
impl<'a> LinkedLibrary<'a> {
    /// Bind named/ordinal imports and forwarders to explicit dependency instances.
    /// Names are ASCII DLL basenames, matched case-insensitively. Symbols remain
    /// case-sensitive. kernel32 is reserved for the explicit native ABI bridges.
    /// At most 64 dependencies are accepted. None are loaded or reinitialized.
    ///
    /// # Safety
    /// LoadedLibrary's trusted-code/context rules apply to this DLL and every
    /// dependency. The caller must supply the intended module identities and a
    /// correctly initialized acyclic dependency order. Reentrancy, threads, SEH,
    /// static TLS and system-DLL/zero ordinals remain unsupported. Missing and
    /// sparse ordinal slots fail before entry, without guessing Windows quirks.
    pub unsafe fn load(
        bytes: &[u8],
        rebase: bool,
        dependencies: &[(&str, &'a LoadedLibrary)],
    ) -> Result<Self> {
        let modules = dependency_modules(dependencies)?;
        let resolve =
            |module: &str, symbol: &ImportSymbol| resolve_dependency(&modules, module, symbol);
        let library = unsafe { LoadedLibrary::load_resolved(bytes, rebase, &resolve) }?;
        Ok(Self {
            library: Some(library),
            _dependencies: modules,
        })
    }
    /// Borrow the attached DLL for lookup or as another explicit dependency.
    /// Its lifetime cannot escape this owner and its transitive dependency borrows.
    pub fn library(&self) -> &LoadedLibrary {
        self.library.as_ref().expect("attached linked DLL")
    }
}
impl Drop for LinkedLibrary<'_> {
    fn drop(&mut self) {
        // An explicit destructor also enforces drop-checking of the borrowed
        // lifetime. Container fields holding references alone may allow NLL to
        // end a borrow before implicit field destruction. Detach/unmap now,
        // while dependency owners are still statically required to be alive.
        drop(self.library.take());
    }
}

fn resolve_forwarder(
    value: &str,
    resolve: &impl Fn(&str, &ImportSymbol) -> Option<usize>,
) -> Result<usize> {
    let (module, symbol) = forwarder_symbol(value)?;
    resolve(&module, &symbol).ok_or_else(|| format!("unsupported export forwarder {value}"))
}
pub(crate) fn dll_basename(name: &str) -> Result<String> {
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
        || abi::is_builtin(&name)
    {
        return Err("invalid or reserved explicit DLL basename".into());
    }
    Ok(name)
}
pub(crate) fn forwarder_symbol(value: &str) -> Result<(String, ImportSymbol)> {
    let (module, name) = value.rsplit_once('.').ok_or("invalid export forwarder")?;
    let module = if module.to_ascii_lowercase().ends_with(".dll") {
        module.to_owned()
    } else {
        format!("{module}.dll")
    };
    let symbol = if let Some(digits) = name.strip_prefix('#') {
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err("invalid decimal ordinal forwarder".into());
        }
        let ordinal = digits
            .parse::<u16>()
            .map_err(|_| "ordinal forwarder exceeds 16 bits")?;
        if ordinal == 0 {
            return Err("zero ordinal forwarders are unsupported".into());
        }
        ImportSymbol::Ordinal(ordinal)
    } else {
        ImportSymbol::Name(name.to_owned())
    };
    Ok((module, symbol))
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

    fn static_fixture() -> Vec<u8> {
        let mut bytes = permission_fixture();
        let opt = 0x98;
        bytes[opt + 112 + 3 * 8..opt + 116 + 3 * 8].copy_from_slice(&0x2040u32.to_le_bytes());
        bytes[opt + 116 + 3 * 8..opt + 120 + 3 * 8].copy_from_slice(&12u32.to_le_bytes());
        for (at, value) in [(0x640, 0x1000u32), (0x644, 0x1006), (0x648, 0x2060)] {
            bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[0x660..0x664].copy_from_slice(&[1, 0, 0, 0]);
        bytes
    }
    #[test]
    fn registered_exe_exposes_original_static_records_and_removes_them_before_unmapping() {
        let process = crate::ProcessContext::install(&["static-unit".into()], &[]).unwrap();
        let bytes = static_fixture();
        assert!(LoadedImage::load(&bytes, true).is_err());
        let image = RegisteredImage::load(&bytes, true, &process).unwrap();
        assert!(image.relocated());
        let base = image.image.base as u64;
        let table = image.table.unwrap();
        let lookup: unsafe extern "win64" fn(
            u64,
            *mut u64,
            *mut c_void,
        ) -> *const crate::RuntimeFunction = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlLookupFunctionEntry").unwrap())
        };
        let delete: unsafe extern "win64" fn(*const crate::RuntimeFunction) -> u8 = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlDeleteFunctionTable").unwrap())
        };
        let mut returned = 0;
        assert_eq!(
            unsafe { lookup(base + 0x1000, &mut returned, ptr::null_mut()) },
            table
        );
        assert_eq!(returned, base);
        assert_eq!(table as u64 - base, 0x2040);
        assert_eq!(unsafe { (*table).unwind }, 0x2060);
        assert_eq!(unsafe { delete(table) }, 0);
        assert_eq!(
            unsafe { lookup(base + 0x1005, &mut returned, ptr::null_mut()) },
            table
        );
        assert!(unsafe { lookup(base + 0x1006, &mut returned, ptr::null_mut()) }.is_null());
        assert_eq!(unsafe { image.enter() }, 42);
        drop(image);
        assert!(unsafe { lookup(base + 0x1000, &mut returned, ptr::null_mut()) }.is_null());
        let ordinary = RegisteredImage::load(&permission_fixture(), true, &process).unwrap();
        assert_eq!(unsafe { ordinary.enter() }, 42);
    }
    #[test]
    fn static_registration_failure_preserves_dynamic_tables_and_releases_capacity_on_drop() {
        let process = crate::ProcessContext::install(&["static-capacity".into()], &[]).unwrap();
        let rows = vec![
            crate::RuntimeFunction {
                begin: 16,
                end: 32,
                unwind: 0x200
            };
            64
        ];
        let add: unsafe extern "win64" fn(*const crate::RuntimeFunction, u32, u64) -> u8 = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlAddFunctionTable").unwrap())
        };
        let delete: unsafe extern "win64" fn(*const crate::RuntimeFunction) -> u8 = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlDeleteFunctionTable").unwrap())
        };
        for i in 0..64 {
            assert_eq!(
                unsafe { add(rows.as_ptr().add(i), 1, 0x123400000000 + i as u64 * 0x10000) },
                1
            );
        }
        assert!(RegisteredImage::load(&static_fixture(), true, &process).is_err());
        assert_eq!(
            crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
            Some(64)
        );
        assert_eq!(unsafe { delete(rows.as_ptr()) }, 1);
        let image = RegisteredImage::load(&static_fixture(), true, &process).unwrap();
        assert_eq!(
            crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
            Some(64)
        );
        drop(image);
        assert_eq!(
            crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
            Some(63)
        );
        for i in 1..64 {
            assert_eq!(unsafe { delete(rows.as_ptr().add(i)) }, 1);
        }
    }
    #[test]
    fn large_sparse_image_has_real_zeroed_writable_end_pages() {
        let mut bytes = permission_fixture();
        let size = 0x20008000u32;
        bytes[0x98 + 56..0x98 + 60].copy_from_slice(&size.to_le_bytes());
        let section = 0x98 + 240 + 2 * 40;
        bytes[section + 8..section + 12].copy_from_slice(&(size - 0x3000).to_le_bytes());
        let mapped = LoadedImage::load(&bytes, true).unwrap();
        assert_eq!(mapped.size, size as usize);
        assert!(mapped.relocated());
        unsafe {
            assert_eq!(mapped.enter(), 42);
            let last = mapped.base.add(size as usize - 1);
            assert_eq!(last.read_volatile(), 0);
            last.write_volatile(0x5a);
            assert_eq!(last.read_volatile(), 0x5a);
            assert_eq!(mapped.base.add(0x300000).read_volatile(), 0);
        }
        drop(mapped);
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
    struct RegisteredLog {
        calls: u32,
        module: usize,
        visible: [bool; 2],
        accept: bool,
    }
    unsafe extern "win64" fn registered_lifecycle(
        module: usize,
        reason: u32,
        reserved: usize,
        log: *mut RegisteredLog,
    ) -> i32 {
        let log = unsafe { &mut *log };
        log.calls += 1;
        log.module = module;
        let lookup: unsafe extern "win64" fn(
            u64,
            *mut u64,
            *mut c_void,
        ) -> *const crate::RuntimeFunction = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlLookupFunctionEntry").unwrap())
        };
        let delete: unsafe extern "win64" fn(*const crate::RuntimeFunction) -> u8 = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlDeleteFunctionTable").unwrap())
        };
        let mut base = 0;
        let row = unsafe { lookup(module as u64 + 0x1000, &mut base, ptr::null_mut()) };
        let retained = unsafe { delete(row) } == 0
            && unsafe { lookup(module as u64 + 0x1000, &mut base, ptr::null_mut()) } == row;
        if reason <= 1 {
            log.visible[(reason == 0) as usize] = reserved == 0
                && base == module as u64
                && row as usize == module + 0x2040
                && retained;
        }
        i32::from(reason != 1 || log.accept)
    }
    fn registered_library_fixture(log: *mut RegisteredLog) -> Vec<u8> {
        let mut bytes = lifecycle_fixture(ptr::null_mut(), true, None);
        let template = static_fixture();
        bytes[0x98 + 136..0x98 + 144].copy_from_slice(&template[0x98 + 136..0x98 + 144]);
        bytes[0x640..0x668].copy_from_slice(&template[0x640..0x668]);
        bytes[0x644..0x648].copy_from_slice(&0x1016u32.to_le_bytes());
        bytes[0x660..0x668].copy_from_slice(&[1, 4, 1, 0, 4, 0x42, 0, 0]);
        bytes[0x810..0x818].copy_from_slice(&(log as usize as u64).to_le_bytes());
        bytes[0x818..0x820]
            .copy_from_slice(&(registered_lifecycle as *const () as u64).to_le_bytes());
        bytes[0x400..0x416].copy_from_slice(&[
            0x48, 0x83, 0xec, 40, // sub rsp,40
            0x4c, 0x8b, 0x0d, 5, 0x20, 0, 0, // mov r9,[rip+0x2005]
            0xff, 0x15, 7, 0x20, 0, 0, // call [rip+0x2007]
            0x48, 0x83, 0xc4, 40, 0xc3,
        ]);
        bytes
    }
    #[test]
    fn registered_library_tables_survive_attach_detach_and_are_removed_before_unmapping() {
        let process = crate::ProcessContext::install(&["registered-dll-unit".into()], &[]).unwrap();
        let mut log = RegisteredLog {
            calls: 0,
            module: 0,
            visible: [false; 2],
            accept: true,
        };
        let bytes = registered_library_fixture(&mut log);
        assert!(unsafe { LoadedLibrary::load(&bytes, true) }.is_err());
        let dll = unsafe { RegisteredLibrary::load(&bytes, true, &process) }.unwrap();
        assert!(dll.library().relocated());
        assert_eq!(log.calls, 1);
        assert!(log.visible[0]);
        let base = dll.library().base_address();
        assert_eq!(
            dll.library().get_name("Entry"),
            dll.library().get_ordinal(17)
        );
        assert_eq!(
            dll.library().get_name("Entry").unwrap() as usize,
            base + 0x1000
        );
        drop(dll);
        assert_eq!(log.calls, 2);
        assert_eq!(log.visible, [true, true]);
        assert_eq!(
            crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
            Some(0)
        );
        // Lookup after drop must not dereference stale unmapped records.
        let lookup: unsafe extern "win64" fn(u64, *mut u64, *mut c_void) -> usize = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlLookupFunctionEntry").unwrap())
        };
        let mut returned = 0;
        assert_eq!(
            unsafe { lookup(base as u64 + 0x1000, &mut returned, ptr::null_mut()) },
            0
        );
        // A table-bearing DLL with no DllMain still owns lookup until drop.
        log.calls = 0;
        let mut bytes = bytes.clone();
        bytes[0x98 + 16..0x98 + 20].fill(0);
        let dll = unsafe { RegisteredLibrary::load(&bytes, true, &process) }.unwrap();
        assert_eq!(log.calls, 0);
        assert_eq!(
            crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
            Some(1)
        );
        drop(dll);
        assert_eq!(log.calls, 0);
        assert_eq!(
            crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
            Some(0)
        );
        // DLLs without static tables retain the same attach/detach semantics.
        let ordinary_log = lifecycle_log();
        let dll = unsafe {
            RegisteredLibrary::load(
                &lifecycle_fixture(ordinary_log.get(), true, None),
                true,
                &process,
            )
        }
        .unwrap();
        drop(dll);
        assert_eq!(unsafe { (*ordinary_log.get()).calls }, 2);
    }
    #[test]
    fn rejected_registered_library_cleans_up_with_live_metadata_and_preflight_failure_never_attaches()
     {
        let process =
            crate::ProcessContext::install(&["registered-dll-failure".into()], &[]).unwrap();
        let mut log = RegisteredLog {
            calls: 0,
            module: 0,
            visible: [false; 2],
            accept: false,
        };
        let bytes = registered_library_fixture(&mut log);
        let error = unsafe { RegisteredLibrary::load(&bytes, true, &process) }
            .err()
            .unwrap();
        assert!(error.contains("PROCESS_ATTACH returned FALSE"));
        assert_eq!(log.calls, 2);
        assert_eq!(log.visible, [true, true]);
        assert_eq!(
            crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
            Some(0)
        );
        for change in [0, 1, 2] {
            log.calls = 0;
            let mut bad = bytes.clone();
            match change {
                0 => bad[0x660] = 9, // unsupported handler recipe
                1 => bad[0x98 + 112 + 9 * 8..0x98 + 116 + 9 * 8]
                    .copy_from_slice(&0x2000u32.to_le_bytes()),
                _ => {
                    // unresolved forwarder
                    bad[0x6d0..0x6d4].copy_from_slice(&0x20f0u32.to_le_bytes());
                    bad[0x6f0..0x6fb].copy_from_slice(b"bad.Entry\0\0");
                }
            }
            assert!(unsafe { RegisteredLibrary::load(&bad, true, &process) }.is_err());
            assert_eq!(log.calls, 0);
            assert_eq!(
                crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
                Some(0)
            );
        }
        // Registration capacity must be checked BEFORE the first initializer.
        log.calls = 0;
        let rows = vec![
            crate::RuntimeFunction {
                begin: 16,
                end: 32,
                unwind: 256
            };
            64
        ];
        let add: unsafe extern "win64" fn(*const crate::RuntimeFunction, u32, u64) -> u8 = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlAddFunctionTable").unwrap())
        };
        let delete: unsafe extern "win64" fn(*const crate::RuntimeFunction) -> u8 = unsafe {
            std::mem::transmute(abi::resolve("kernel32.dll", "RtlDeleteFunctionTable").unwrap())
        };
        for i in 0..64 {
            assert_eq!(
                unsafe { add(rows.as_ptr().add(i), 1, 0x123400000000 + i as u64 * 0x10000) },
                1
            );
        }
        assert!(unsafe { RegisteredLibrary::load(&bytes, true, &process) }.is_err());
        assert_eq!(log.calls, 0);
        assert_eq!(
            crate::process::with_function_tables(|r| r.map(|r| r.entry_count_for_test())),
            Some(64)
        );
        for i in 0..64 {
            assert_eq!(unsafe { delete(rows.as_ptr().add(i)) }, 1);
        }
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
    fn lookup_bridges() -> (
        unsafe extern "win64" fn(*const u16) -> usize,
        unsafe extern "win64" fn(usize, usize) -> usize,
    ) {
        unsafe {
            (
                std::mem::transmute::<usize, unsafe extern "win64" fn(*const u16) -> usize>(
                    abi::resolve("kernel32.dll", "GetModuleHandleW").unwrap(),
                ),
                std::mem::transmute::<usize, unsafe extern "win64" fn(usize, usize) -> usize>(
                    abi::resolve("kernel32.dll", "GetProcAddress").unwrap(),
                ),
            )
        }
    }
    #[test]
    fn guest_module_lookup_returns_real_bases_exports_ordinals_and_forwarded_calls() {
        let (module, symbol) = lookup_bridges();
        let log = lifecycle_log();
        let mut bytes = lifecycle_fixture(log.get(), true, None);
        bytes[0x6d0..0x6d4].copy_from_slice(&0x1100u32.to_le_bytes());
        bytes[0x500..0x506].copy_from_slice(&[0xb8, 42, 0, 0, 0, 0xc3]);
        let library = unsafe { LoadedLibrary::load(&bytes, true) }.unwrap();
        let forward_log = lifecycle_log();
        let forward = unsafe {
            LoadedLibrary::load(
                &lifecycle_fixture(forward_log.get(), true, Some("kernel32.GetLastError")),
                true,
            )
        }
        .unwrap();
        let context =
            crate::ModuleContext::install(&[("custom.dll", &library), ("forward.dll", &forward)])
                .unwrap();
        for name in ["CUSTOM.DLL", "custom", "custom.dll."] {
            let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
            assert_eq!(unsafe { module(wide.as_ptr()) }, library.base_address());
        }
        assert_eq!(
            unsafe { (library.base_address() as *const u16).read() },
            0x5a4d
        );
        let name = b"Entry\0";
        let address = unsafe { symbol(library.base_address(), name.as_ptr() as usize) };
        assert_eq!(address, library.get_name("Entry").unwrap() as usize);
        assert_eq!(unsafe { symbol(library.base_address(), 17) }, address);
        let call: unsafe extern "win64" fn() -> u32 = unsafe { std::mem::transmute(address) };
        assert_eq!(unsafe { call() }, 42);
        let forwarded: unsafe extern "win64" fn() -> u32 =
            unsafe { std::mem::transmute(symbol(forward.base_address(), name.as_ptr() as usize)) };
        alkyhol::ffi::SetLastError(0x3456);
        assert_eq!(unsafe { forwarded() }, 0x3456);
        assert_eq!(
            unsafe { ((*log.get()).calls, (*forward_log.get()).calls) },
            (1, 1)
        );
        drop(context);
        assert_eq!(unsafe { call() }, 42);
        assert_eq!(unsafe { (*log.get()).calls }, 1);
        drop(forward);
        drop(library);
        assert_eq!(
            unsafe { ((*log.get()).calls, (*forward_log.get()).calls) },
            (2, 2)
        );
    }
    #[test]
    fn guest_module_lookup_scope_errors_and_pointer_width_are_explicit() {
        let (module, symbol) = lookup_bridges();
        assert_eq!(unsafe { module(ptr::null()) }, 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 50);
        assert_eq!(unsafe { symbol(0, 0) }, 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 50);
        assert!(crate::ModuleContext::install(&[]).is_err());
        let log = lifecycle_log();
        let library =
            unsafe { LoadedLibrary::load(&lifecycle_fixture(log.get(), true, None), true) }
                .unwrap();
        for inputs in [
            vec![("custom.dll", &library); 65],
            vec![("custom.dll", &library), ("CUSTOM.DLL", &library)],
            vec![("one.dll", &library), ("two.dll", &library)],
        ] {
            assert!(crate::ModuleContext::install(&inputs).is_err());
        }
        for name in [
            "../custom.dll",
            "",
            "kernel32.dll",
            "d3d12.dll",
            "d3dcompiler_47.dll",
        ] {
            assert!(crate::ModuleContext::install(&[(name, &library)]).is_err());
        }
        let context = crate::ModuleContext::install(&[("custom.dll", &library)]).unwrap();
        assert!(crate::ModuleContext::install(&[("custom.dll", &library)]).is_err());
        let base = library.base_address();
        assert_ne!(base >> 32, 0);
        for bad in [0, base ^ 0x100000000, base as u32 as usize] {
            assert_eq!(unsafe { symbol(bad, 17) }, 0);
            assert_eq!(alkyhol::ffi::GetLastError(), 6);
        }
        for ordinal in [0, 16, 18, 65535] {
            assert_eq!(unsafe { symbol(base, ordinal) }, 0);
            assert_eq!(alkyhol::ffi::GetLastError(), 127);
        }
        for name in [b"entry\0".as_slice(), b"Missing\0"] {
            assert_eq!(unsafe { symbol(base, name.as_ptr() as usize) }, 0);
            assert_eq!(alkyhol::ffi::GetLastError(), 127);
        }
        for (name, error) in [
            ("unknown.dll", 126),
            ("../custom.dll", 50),
            ("kernel32", 50),
            ("🌸.dll", 50),
        ] {
            let wide: Vec<u16> = name.encode_utf16().chain([0]).collect();
            assert_eq!(unsafe { module(wide.as_ptr()) }, 0);
            assert_eq!(alkyhol::ffi::GetLastError(), error);
        }
        assert_eq!(unsafe { module([0xd800u16, 0].as_ptr()) }, 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 1113);
        let bad = vec![b'x'; 4096];
        assert_eq!(unsafe { symbol(base, bad.as_ptr() as usize) }, 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 87);
        assert_eq!(unsafe { symbol(base, c"bad\xff".as_ptr() as usize) }, 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 87);
        assert_eq!(
            unsafe { symbol(base, 17) },
            library.get_ordinal(17).unwrap() as usize
        );
        drop(context);
        assert_eq!(unsafe { symbol(base, 17) }, 0);
        assert_eq!(alkyhol::ffi::GetLastError(), 50);
        drop(library);
        assert_eq!(unsafe { (*log.get()).calls }, 2);
    }
    #[test]
    fn guest_module_views_do_not_escape_to_other_host_threads() {
        let log = lifecycle_log();
        let library =
            unsafe { LoadedLibrary::load(&lifecycle_fixture(log.get(), true, None), true) }
                .unwrap();
        let context = crate::ModuleContext::install(&[("custom.dll", &library)]).unwrap();
        let base = library.base_address();
        alkyhol::ffi::SetLastError(0x1234);
        std::thread::spawn(move || {
            let (_, symbol) = lookup_bridges();
            assert_eq!(unsafe { symbol(base, 17) }, 0);
            assert_eq!(alkyhol::ffi::GetLastError(), 50);
        })
        .join()
        .unwrap();
        assert_eq!(alkyhol::ffi::GetLastError(), 0x1234);
        let (_, symbol) = lookup_bridges();
        assert_eq!(
            unsafe { symbol(base, 17) },
            library.get_ordinal(17).unwrap() as usize
        );
        drop(context);
        drop(library);
    }
    #[test]
    fn linked_executable_calls_named_ordinal_and_forwarded_targets_without_reinitializing_dll() {
        for forwarded in [false, true] {
            for ordinal in [false, true] {
                let log = lifecycle_log();
                let mut dependency = lifecycle_fixture(
                    log.get(),
                    true,
                    forwarded.then_some("kernel32.GetLastError"),
                );
                if !forwarded {
                    dependency[0x6d0..0x6d4].copy_from_slice(&0x1100u32.to_le_bytes());
                    dependency[0x500..0x506].copy_from_slice(&[0xb8, 42, 0, 0, 0, 0xc3]);
                }
                let library = unsafe { LoadedLibrary::load(&dependency, true) }.unwrap();
                let parent_log = lifecycle_log();
                let mut exe = dependent_fixture(parent_log.get(), true);
                exe[0x96..0x98].copy_from_slice(&0x22u16.to_le_bytes());
                exe[0x98 + 112..0x98 + 120].fill(0);
                // Win64 entry: allocate shadow/alignment space, call the actual
                // imported target at RVA 0x3020, restore the stack, return EAX.
                exe[0x400..0x40f].copy_from_slice(&[
                    0x48, 0x83, 0xec, 0x28, 0xff, 0x15, 0x16, 0x20, 0, 0, 0x48, 0x83, 0xc4, 0x28,
                    0xc3,
                ]);
                if ordinal {
                    exe[0x7a0..0x7a8].copy_from_slice(&0x8000000000000011u64.to_le_bytes());
                }
                assert!(LoadedImage::load(&exe, true).is_err());
                let program = LinkedImage::load(&exe, true, &[("CuStOm.DlL", &library)]).unwrap();
                assert!(program.image().relocated());
                assert_eq!(
                    unsafe { program.image().base.add(0x3020).cast::<usize>().read() },
                    library.get_name("Entry").unwrap() as usize
                );
                assert_eq!(unsafe { (*log.get()).calls }, 1);
                alkyhol::ffi::SetLastError(0x1234);
                assert_eq!(
                    unsafe { program.image().enter() },
                    if forwarded { 0x1234 } else { 42 }
                );
                drop(program);
                assert_eq!(unsafe { (*log.get()).calls }, 1);
                let call: unsafe extern "win64" fn() -> u32 =
                    unsafe { std::mem::transmute(library.get_ordinal(17).unwrap()) };
                alkyhol::ffi::SetLastError(0x1234);
                assert_eq!(unsafe { call() }, if forwarded { 0x1234 } else { 42 });
                drop(library);
                assert_eq!(unsafe { (*log.get()).calls }, 2);
                assert_eq!(unsafe { (*parent_log.get()).calls }, 0);
            }
        }
        let program = LinkedImage::load(&permission_fixture(), true, &[]).unwrap();
        assert_eq!(unsafe { program.image().enter() }, 42);
    }
    #[test]
    fn linked_executable_rejects_missing_invalid_and_unsupported_inputs_before_entry() {
        let log = lifecycle_log();
        let dependency = lifecycle_fixture(log.get(), true, None);
        let library = unsafe { LoadedLibrary::load(&dependency, true) }.unwrap();
        let parent_log = lifecycle_log();
        let dll = dependent_fixture(parent_log.get(), true);
        assert!(LinkedImage::load(&dll, true, &[("custom.dll", &library)]).is_err());
        let mut exe = dll;
        exe[0x96..0x98].copy_from_slice(&0x22u16.to_le_bytes());
        exe[0x98 + 112..0x98 + 120].fill(0);
        assert!(LinkedImage::load(&exe, true, &[]).is_err());
        for module in [
            "../custom.dll",
            "custom",
            "",
            "kernel32.dll",
            "D3D12.DLL",
            "d3dcompiler_47.dll",
        ] {
            assert!(LinkedImage::load(&exe, true, &[(module, &library)]).is_err());
        }
        assert!(
            LinkedImage::load(
                &exe,
                true,
                &[("custom.dll", &library), ("CUSTOM.DLL", &library)]
            )
            .is_err()
        );
        assert!(LinkedImage::load(&exe, true, &vec![("custom.dll", &library); 65]).is_err());
        for mode in [
            "symbol-case",
            "module",
            "ordinal-zero",
            "ordinal-hole",
            "system-ordinal",
            "tls",
            "iat-reloc",
        ] {
            let mut invalid = exe.clone();
            match mode {
                "symbol-case" => invalid[0x7d2] = b'e',
                "module" => invalid[0x7c0] = b'x',
                "ordinal-zero" | "ordinal-hole" => invalid[0x7a0..0x7a8].copy_from_slice(
                    &(0x8000000000000000u64 | if mode == "ordinal-zero" { 0 } else { 18 })
                        .to_le_bytes(),
                ),
                "system-ordinal" => {
                    invalid[0x7c0..0x7cd].copy_from_slice(b"kernel32.dll\0");
                    invalid[0x7a0..0x7a8].copy_from_slice(&0x8000000000000011u64.to_le_bytes());
                }
                "tls" => invalid[0x98 + 112 + 9 * 8..0x98 + 120 + 9 * 8]
                    .copy_from_slice(&[0, 0x20, 0, 0, 16, 0, 0, 0]),
                "iat-reloc" => invalid[0x608..0x60a].copy_from_slice(&0xa020u16.to_le_bytes()),
                _ => unreachable!(),
            }
            assert!(
                LinkedImage::load(&invalid, true, &[("custom.dll", &library)]).is_err(),
                "{mode}"
            );
        }
        assert_eq!(unsafe { (*parent_log.get()).calls }, 0);
        assert_eq!(unsafe { (*log.get()).calls }, 1);
        drop(library);
        assert_eq!(unsafe { (*log.get()).calls }, 2);
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
                    .library()
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
            "d3dcompiler_47.dll",
            "D3D12.dll",
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
                    invalid[0x7a0..0x7a8].copy_from_slice(&0x8000000000000012u64.to_le_bytes())
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
    fn explicit_ordinal_iat_and_forwarders_use_actual_biased_export_slots() {
        for forwarder in ["custom.#17", "custom.dll.#00017"] {
            let child_log = lifecycle_log();
            let child_bytes = lifecycle_fixture(child_log.get(), true, None);
            let child = unsafe { LoadedLibrary::load(&child_bytes, true) }.unwrap();
            let log = lifecycle_log();
            let mut bytes = dependent_fixture(log.get(), true);
            bytes[0x6f0..0x710].fill(0);
            bytes[0x6f0..0x6f0 + forwarder.len()].copy_from_slice(forwarder.as_bytes());
            bytes[0x7a0..0x7a8].copy_from_slice(&0x8000000000000011u64.to_le_bytes());
            let parent =
                unsafe { LinkedLibrary::load(&bytes, true, &[("CUSTOM.dll", &child)]) }.unwrap();
            let address = child.get_ordinal(17).unwrap();
            assert_eq!(parent.library().get_name("Entry"), Some(address));
            assert_eq!(
                unsafe {
                    parent
                        .library()
                        .mapping
                        .base
                        .add(0x3020)
                        .cast::<usize>()
                        .read()
                },
                address as usize
            );
            drop(parent);
            assert_eq!(
                unsafe { ((*log.get()).calls, (*child_log.get()).calls) },
                (2, 1)
            );
        }
    }
    #[test]
    fn unsupported_or_malformed_ordinals_fail_before_consumer_entry() {
        let child_log = lifecycle_log();
        let child_bytes = lifecycle_fixture(child_log.get(), true, None);
        let child = unsafe { LoadedLibrary::load(&child_bytes, true) }.unwrap();
        let log = lifecycle_log();
        for forwarder in [
            "custom.#",
            "custom.#+17",
            "custom.#-1",
            "custom.#65536",
            "custom.#0x11",
            "custom.#0",
            "custom.#18",
            "kernel32.#17",
        ] {
            let mut bytes = dependent_fixture(log.get(), true);
            bytes[0x6f0..0x710].fill(0);
            bytes[0x6f0..0x6f0 + forwarder.len()].copy_from_slice(forwarder.as_bytes());
            assert!(
                unsafe { LinkedLibrary::load(&bytes, true, &[("custom.dll", &child)]) }.is_err(),
                "{forwarder}"
            );
        }
        for ordinal in [0, 16, 18, 65535] {
            let mut bytes = dependent_fixture(log.get(), true);
            bytes[0x7a0..0x7a8].copy_from_slice(&(0x8000000000000000u64 | ordinal).to_le_bytes());
            assert!(
                unsafe { LinkedLibrary::load(&bytes, true, &[("custom.dll", &child)]) }.is_err()
            );
        }
        let mut system = dependent_fixture(log.get(), true);
        system[0x7c0..0x7cd].copy_from_slice(b"kernel32.dll\0");
        system[0x7a0..0x7a8].copy_from_slice(&0x8000000000000011u64.to_le_bytes());
        assert!(unsafe { LinkedLibrary::load(&system, true, &[("custom.dll", &child)]) }.is_err());
        assert_eq!(unsafe { (*log.get()).calls }, 0);
    }
    fn graph_fixture(
        log: *mut LifecycleLog,
        accept: bool,
        import: &str,
        forwarder: &str,
    ) -> Vec<u8> {
        let mut bytes = dependent_fixture(log, accept);
        bytes[0x7c0..0x7d0].fill(0);
        bytes[0x7c0..0x7c0 + import.len()].copy_from_slice(import.as_bytes());
        bytes[0x6f0..0x730].fill(0);
        bytes[0x6f0..0x6f0 + forwarder.len()].copy_from_slice(forwarder.as_bytes());
        bytes
    }
    #[test]
    fn closed_bundle_orders_a_diamond_and_loads_its_shared_leaf_once() {
        let logs: Vec<_> = (0..4).map(|_| lifecycle_log()).collect();
        let root = graph_fixture(logs[0].get(), true, "left.dll", "right.Entry");
        let right = graph_fixture(logs[1].get(), true, "leaf.dll", "leaf.Entry");
        let leaf = lifecycle_fixture(logs[2].get(), true, None);
        let left = graph_fixture(logs[3].get(), true, "leaf.dll", "leaf.Entry");
        let set = unsafe {
            crate::LibrarySet::load(
                &[
                    ("ROOT.dll", &root),
                    ("right.dll", &right),
                    ("leaf.dll", &leaf),
                    ("left.dll", &left),
                ],
                true,
            )
        }
        .unwrap();
        assert!(set.get("unknown.dll").is_none());
        let address = set.get("LEAF.DLL").unwrap().get_name("Entry").unwrap();
        for name in ["root.dll", "left.dll", "right.dll"] {
            let library = set.get(name).unwrap();
            assert_eq!(library.get_name("Entry"), Some(address));
            assert_eq!(
                unsafe { library.mapping.base.add(0x3020).cast::<usize>().read() },
                address as usize
            );
        }
        for log in &logs {
            assert_eq!(unsafe { ((*log.get()).reason, (*log.get()).calls) }, (1, 1));
        }
        drop(set);
        for log in &logs {
            assert_eq!(unsafe { ((*log.get()).reason, (*log.get()).calls) }, (0, 2));
        }
    }
    #[test]
    fn closed_bundle_rejects_graph_and_symbol_errors_before_any_initializer() {
        let root_log = lifecycle_log();
        let leaf_log = lifecycle_log();
        let root = graph_fixture(root_log.get(), true, "leaf.dll", "leaf.Entry");
        let leaf = lifecycle_fixture(leaf_log.get(), true, None);
        assert!(unsafe { crate::LibrarySet::load(&[], true) }.is_err());
        let excessive = vec![("leaf.dll", leaf.as_slice()); 65];
        assert!(unsafe { crate::LibrarySet::load(&excessive, true) }.is_err());
        assert!(
            unsafe { crate::LibrarySet::load(&[("ROOT.dll", &root), ("root.DLL", &leaf)], true) }
                .is_err()
        );
        for name in [
            "../root.dll",
            "kernel32.dll",
            "d3dcompiler_47.dll",
            "D3D12.dll",
            "",
            "root",
        ] {
            assert!(unsafe { crate::LibrarySet::load(&[(name, &root)], true) }.is_err());
        }
        assert!(unsafe { crate::LibrarySet::load(&[("root.dll", &root)], true) }.is_err());
        for mode in [
            "cycle",
            "self-cycle",
            "missing-forward",
            "missing-import",
            "zero-ordinal",
            "malformed-forward",
            "unsupported-builtin",
            "static-tls",
            "relocation-overlap",
            "missing-fixups",
        ] {
            let mut bad_leaf = leaf.clone();
            let mut bad_root = root.clone();
            match mode {
                "cycle" => bad_leaf = graph_fixture(leaf_log.get(), true, "root.dll", "root.Entry"),
                "self-cycle" => {
                    bad_root = graph_fixture(root_log.get(), true, "root.dll", "leaf.Entry")
                }
                "missing-forward" => {
                    bad_root = graph_fixture(root_log.get(), true, "leaf.dll", "leaf.Missing")
                }
                "missing-import" => bad_root[0x7d2] = b'e',
                "zero-ordinal" => {
                    bad_root[0x7a0..0x7a8].copy_from_slice(&0x8000000000000000u64.to_le_bytes())
                }
                "malformed-forward" => {
                    bad_root = graph_fixture(root_log.get(), true, "leaf.dll", "leaf.#+17")
                }
                "unsupported-builtin" => {
                    bad_root[0x7c0..0x7d0].fill(0);
                    bad_root[0x7c0..0x7cd].copy_from_slice(b"kernel32.dll\0");
                }
                "static-tls" => bad_leaf[0x98 + 112 + 9 * 8..0x98 + 120 + 9 * 8]
                    .copy_from_slice(&[0, 0x20, 0, 0, 16, 0, 0, 0]),
                "relocation-overlap" => {
                    bad_root[0x608..0x60a].copy_from_slice(&0xa020u16.to_le_bytes())
                }
                "missing-fixups" => bad_leaf[0x98 + 112 + 5 * 8..0x98 + 120 + 5 * 8].fill(0),
                _ => unreachable!(),
            }
            assert!(
                unsafe {
                    crate::LibrarySet::load(
                        &[("root.dll", &bad_root), ("leaf.dll", &bad_leaf)],
                        true,
                    )
                }
                .is_err(),
                "{mode}"
            );
        }
        assert_eq!(
            unsafe { ((*root_log.get()).calls, (*leaf_log.get()).calls) },
            (0, 0)
        );
    }
    #[test]
    fn closed_bundle_failed_attach_rolls_back_dependencies_and_never_enters_later_modules() {
        let logs: Vec<_> = (0..3).map(|_| lifecycle_log()).collect();
        let root = graph_fixture(logs[0].get(), false, "leaf.dll", "leaf.Entry");
        let leaf = lifecycle_fixture(logs[1].get(), true, None);
        let later = lifecycle_fixture(logs[2].get(), true, None);
        let error = unsafe {
            crate::LibrarySet::load(
                &[
                    ("root.dll", &root),
                    ("leaf.dll", &leaf),
                    ("later.dll", &later),
                ],
                true,
            )
        }
        .err()
        .unwrap();
        assert!(error.contains("ATTACH returned FALSE"));
        for log in &logs[..2] {
            assert_eq!(unsafe { ((*log.get()).reason, (*log.get()).calls) }, (0, 2));
        }
        assert_eq!(unsafe { (*logs[2].get()).calls }, 0);
    }
    #[test]
    fn closed_bundle_mapping_failure_rolls_back_already_attached_dependencies() {
        let root_log = lifecycle_log();
        let leaf_log = lifecycle_log();
        let mut root = graph_fixture(root_log.get(), true, "leaf.dll", "leaf.Entry");
        let mut leaf = lifecycle_fixture(leaf_log.get(), true, None);
        const BASE: u64 = 0x1_8000_0000;
        for bytes in [&mut root, &mut leaf] {
            bytes[0xb0..0xb8].copy_from_slice(&BASE.to_le_bytes());
            bytes[0x800..0x808].copy_from_slice(&(BASE + 0x1000).to_le_bytes());
        }
        // A preferred-base-only parent cannot occupy its leaf's live mapping.
        // Placement fails after leaf ATTACH, before parent ATTACH. MAP_FIXED
        // replacement is never allowed, and rollback must detach the leaf.
        root[0x98 + 112 + 5 * 8..0x98 + 120 + 5 * 8].fill(0);
        let error =
            unsafe { crate::LibrarySet::load(&[("root.dll", &root), ("leaf.dll", &leaf)], false) }
                .err()
                .unwrap();
        assert!(error.contains("no DIR64 relocation targets"), "{error}");
        assert_eq!(unsafe { (*leaf_log.get()).module as u64 }, BASE);
        assert_eq!(
            unsafe {
                (
                    (*leaf_log.get()).calls,
                    (*leaf_log.get()).reason,
                    (*root_log.get()).calls,
                )
            },
            (2, 0, 0)
        );
    }
    #[test]
    fn closed_bundle_binds_com_builtins_without_supplied_system_dlls() {
        for (module, symbol) in [
            ("d3dcompiler_47.dll", "D3DCreateBlob"),
            ("d3d12.dll", "D3D12SerializeRootSignature"),
            ("d3d12.dll", "D3D12SerializeVersionedRootSignature"),
        ] {
            let log = lifecycle_log();
            let mut bytes = dependent_fixture(log.get(), true);
            bytes[0x6d0..0x6d4].copy_from_slice(&0x1000u32.to_le_bytes());
            bytes[0x7c0..0x800].fill(0);
            bytes[0x7c0..0x7c0 + module.len()].copy_from_slice(module.as_bytes());
            bytes[0x7a0..0x7a8].copy_from_slice(&0x21d8u64.to_le_bytes());
            bytes[0x7da..0x7da + symbol.len()].copy_from_slice(symbol.as_bytes());
            let set = unsafe { crate::LibrarySet::load(&[("client.dll", &bytes)], true) }.unwrap();
            let client = set.get("client.dll").unwrap();
            assert_eq!(
                unsafe { client.mapping.base.add(0x3020).cast::<usize>().read() },
                abi::resolve(module, symbol).unwrap()
            );
            assert!(set.get(module).is_none());
            drop(set);
            assert_eq!(unsafe { (*log.get()).calls }, 2);
        }
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
            let mut bytes = lifecycle_fixture(log.get(), true, None);
            // Use an isolated preferred address, outside the normal mmap/JIT
            // allocation area. Otherwise Rosetta/allocator activity can reuse
            // an unmapped address before a later VM-region snapshot observes it.
            // This is a non-replacing hint, and exact placement is asserted.
            const PREFERRED: u64 = 0x1_8000_0000;
            bytes[0xb0..0xb8].copy_from_slice(&PREFERRED.to_le_bytes());
            bytes[0x800..0x808].copy_from_slice(&(PREFERRED + 0x1000).to_le_bytes());
            let lib = unsafe { LoadedLibrary::load(&bytes, false) }.unwrap();
            let base = lib.base_address();
            assert_eq!(base as u64, PREFERRED);
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
