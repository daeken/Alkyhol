//! Scoped guest lookups of explicitly borrowed, already attached PE DLLs.
#![allow(non_snake_case)]
use crate::{LoadedLibrary, Result, abi, runtime::dll_basename};
use alkyhol::ffi;
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

const NOT_SUPPORTED: u32 = 50;
const INVALID_PARAMETER: u32 = 87;
const MOD_NOT_FOUND: u32 = 126;
const PROC_NOT_FOUND: u32 = 127;
struct Module {
    name: String,
    base: usize,
    names: BTreeMap<String, usize>,
    ordinals: BTreeMap<u32, usize>,
}
thread_local! { static MODULES: RefCell<Option<Vec<Module>>> = const { RefCell::new(None) }; }

/// Makes 1..64 explicitly supplied attached DLLs visible to the bounded guest
/// GetModuleHandleW/GetProcAddress providers on this thread, until drop. Borrows
/// retain every module owner; installing/dropping this view does not initialize,
/// detach, load, unload or increment native module references. No nested view,
/// main EXE, builtin module image, filesystem search or guest loader is supplied.
///
/// ```no_run
/// # use alkyhol_pe::{LoadedLibrary, ModuleContext};
/// # fn example(bytes: &[u8]) -> alkyhol_pe::Result<()> {
/// let dll = unsafe { LoadedLibrary::load(bytes, false) }?;
/// let modules = ModuleContext::install(&[("test.dll", &dll)])?;
/// drop(modules);
/// drop(dll);
/// # Ok(()) }
/// ```
/// ```compile_fail,E0505
/// # use alkyhol_pe::{LoadedLibrary, ModuleContext};
/// # fn example(bytes: &[u8]) -> alkyhol_pe::Result<()> {
/// let dll = unsafe { LoadedLibrary::load(bytes, false) }?;
/// let _modules = ModuleContext::install(&[("test.dll", &dll)])?;
/// drop(dll);
/// # Ok(()) }
/// ```
pub struct ModuleContext<'a> {
    _owners: PhantomData<&'a LoadedLibrary>,
    _thread: PhantomData<*mut ()>,
}
impl<'a> ModuleContext<'a> {
    /// Caller-supplied names follow the explicit DLL basename contract. Names
    /// and actual image bases must be unique. Already resolved forwarders retain
    /// their real targets and transitive ownership through the borrowed DLLs.
    pub fn install(inputs: &[(&str, &'a LoadedLibrary)]) -> Result<Self> {
        if inputs.is_empty() || inputs.len() > 64 {
            return Err("module context requires 1..64 attached DLLs".into());
        }
        let mut modules = Vec::new();
        let mut names = BTreeSet::new();
        let mut bases = BTreeSet::new();
        for &(name, library) in inputs {
            let name = dll_basename(name)?;
            let base = library.base_address();
            if !names.insert(name.clone()) || !bases.insert(base) {
                return Err("duplicate module name or actual DLL image base".into());
            }
            let mut module = Module {
                name,
                base,
                names: BTreeMap::new(),
                ordinals: BTreeMap::new(),
            };
            for (ordinal, names, address) in library.export_bindings() {
                module.ordinals.insert(ordinal, address);
                for name in names {
                    module.names.insert(name.clone(), address);
                }
            }
            modules.push(module);
        }
        MODULES.with(|slot| {
            let mut slot = slot.borrow_mut();
            if slot.is_some() {
                return Err("module context already installed on this thread".into());
            }
            *slot = Some(modules);
            Ok(Self {
                _owners: PhantomData,
                _thread: PhantomData,
            })
        })
    }
}
impl Drop for ModuleContext<'_> {
    fn drop(&mut self) {
        MODULES.with(|slot| {
            slot.borrow_mut().take();
        });
    }
}
fn fail(error: u32) -> usize {
    ffi::SetLastError(error);
    0
}
fn module_name(mut name: String) -> std::result::Result<String, u32> {
    if !name.is_ascii() || name.contains(['/', '\\', ':']) {
        return Err(NOT_SUPPORTED);
    }
    if name.len() > 255
        || name
            .bytes()
            .any(|b| !b.is_ascii_alphanumeric() && !b"._-".contains(&b))
    {
        return Err(INVALID_PARAMETER);
    }
    if name.ends_with('.') {
        name.pop();
    } else if !name.contains('.') {
        name.push_str(".dll");
    }
    if abi::is_builtin(&name) {
        return Err(NOT_SUPPORTED);
    }
    Ok(name.to_ascii_lowercase())
}
unsafe extern "win64" fn GetModuleHandleW(name: *const u16) -> usize {
    MODULES.with(|slot| {
        let slot = slot.borrow();
        let Some(modules) = slot.as_ref() else {
            return fail(NOT_SUPPORTED);
        };
        if name.is_null() {
            return fail(NOT_SUPPORTED);
        }
        let name = match unsafe { crate::process::wide(name) }.and_then(module_name) {
            Ok(name) => name,
            Err(error) => return fail(error),
        };
        modules
            .iter()
            .find(|m| m.name == name)
            .map_or_else(|| fail(MOD_NOT_FOUND), |m| m.base)
    })
}
unsafe extern "win64" fn GetProcAddress(module: usize, symbol: usize) -> usize {
    MODULES.with(|slot| {
        let slot = slot.borrow();
        let Some(modules) = slot.as_ref() else {
            return fail(NOT_SUPPORTED);
        };
        // HMODULE is an actual full-width image base, never a kernel handle token.
        let Some(module) = modules.iter().find(|m| m.base == module) else {
            return fail(6);
        };
        if symbol <= u16::MAX as usize {
            return module
                .ordinals
                .get(&(symbol as u32))
                .copied()
                .unwrap_or_else(|| fail(PROC_NOT_FOUND));
        }
        for len in 0..4096 {
            if unsafe { (symbol as *const u8).add(len).read() } == 0 {
                let bytes = unsafe { std::slice::from_raw_parts(symbol as *const u8, len) };
                let Ok(name) = std::str::from_utf8(bytes) else {
                    return fail(INVALID_PARAMETER);
                };
                if !name.is_ascii() {
                    return fail(INVALID_PARAMETER);
                }
                return module
                    .names
                    .get(name)
                    .copied()
                    .unwrap_or_else(|| fail(PROC_NOT_FOUND));
            }
        }
        fail(INVALID_PARAMETER)
    })
}
pub(crate) fn address(name: &str) -> Option<usize> {
    match name {
        "GetModuleHandleW" => Some(GetModuleHandleW as *const () as usize),
        "GetProcAddress" => Some(GetProcAddress as *const () as usize),
        _ => None,
    }
}
