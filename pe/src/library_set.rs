//! A closed, explicitly supplied acyclic DLL bundle. No process-global modules.
use crate::runtime::{dll_basename, forwarder_symbol, resolve_builtin};
use crate::{Export, ExportTarget, Image, Import, ImportSymbol, LoadedLibrary, Result};
use std::collections::{BTreeMap, BTreeSet};

struct Module<'a> {
    name: String,
    bytes: &'a [u8],
    imports: Vec<Import>,
    exports: Vec<Export>,
    names: BTreeMap<String, usize>,
    ordinals: BTreeMap<u32, usize>,
}
impl Module<'_> {
    fn target(&self, symbol: &ImportSymbol) -> Result<&ExportTarget> {
        let index = match symbol {
            ImportSymbol::Name(name) => self.names.get(name),
            ImportSymbol::Ordinal(n) if *n != 0 => self.ordinals.get(&u32::from(*n)),
            ImportSymbol::Ordinal(_) => None,
        }
        .ok_or_else(|| format!("missing bundle export {}!{symbol:?}", self.name))?;
        Ok(&self.exports[*index].target)
    }
}

fn dependency(
    module: &str,
    symbol: &ImportSymbol,
    indices: &BTreeMap<String, usize>,
) -> Result<Option<usize>> {
    if module.eq_ignore_ascii_case("kernel32.dll") {
        if resolve_builtin(module, symbol).is_none() {
            return Err(format!("unsupported builtin import {module}!{symbol:?}"));
        }
        return Ok(None);
    }
    indices
        .get(&module.to_ascii_lowercase())
        .copied()
        .map(Some)
        .ok_or_else(|| format!("missing DLL from closed bundle: {module}"))
}

fn verify_symbol(
    module: usize,
    symbol: &ImportSymbol,
    modules: &[Module<'_>],
    indices: &BTreeMap<String, usize>,
    depth: usize,
) -> Result<()> {
    if depth >= modules.len() {
        return Err("cyclic or excessive bundle export forwarding".into());
    }
    match modules[module].target(symbol)? {
        ExportTarget::Address { .. } => Ok(()),
        ExportTarget::Forwarder(text) => {
            let (name, symbol) = forwarder_symbol(text)?;
            if let Some(target) = dependency(&name, &symbol, indices)? {
                verify_symbol(target, &symbol, modules, indices, depth + 1)?;
            }
            Ok(())
        }
    }
}

fn visit(
    index: usize,
    edges: &[BTreeSet<usize>],
    states: &mut [u8],
    order: &mut Vec<usize>,
) -> Result<()> {
    match states[index] {
        2 => return Ok(()),
        1 => return Err("cyclic DLL dependencies are unsupported in a closed bundle".into()),
        _ => (),
    }
    states[index] = 1;
    for &child in &edges[index] {
        visit(child, edges, states, order)?;
    }
    states[index] = 2;
    order.push(index);
    Ok(())
}

/// Owns a complete acyclic set of 1..64 trusted DLLs. Every provided module is
/// loaded once per normalized name, after its dependencies, and detached in
/// reverse order. Missing/unsupported metadata, imports and forwarders are
/// preflighted for the entire bundle before the first guest initializer.
/// This is not Windows' global module cache, filesystem search or guest loader.
///
/// Dependency borrows from a set also outlive consumer teardown:
/// ```no_run
/// # use alkyhol_pe::{LibrarySet, LinkedLibrary};
/// # fn example(bytes: &[u8]) -> alkyhol_pe::Result<()> {
/// let set = unsafe { LibrarySet::load(&[("leaf.dll", bytes)], false) }?;
/// let consumer = unsafe { LinkedLibrary::load(bytes, false, &[("leaf.dll", set.get("leaf.dll").unwrap())]) }?;
/// drop(consumer);
/// drop(set);
/// # Ok(()) }
/// ```
/// ```compile_fail,E0505
/// # use alkyhol_pe::{LibrarySet, LinkedLibrary};
/// # fn example(bytes: &[u8]) -> alkyhol_pe::Result<()> {
/// let set = unsafe { LibrarySet::load(&[("leaf.dll", bytes)], false) }?;
/// let _consumer = unsafe { LinkedLibrary::load(bytes, false, &[("leaf.dll", set.get("leaf.dll").unwrap())]) }?;
/// drop(set);
/// # Ok(()) }
/// ```
pub struct LibrarySet {
    libraries: Vec<(String, LoadedLibrary)>,
}
impl LibrarySet {
    /// Automatically order and bind a closed DLL bundle supplied in any order.
    /// All supplied DLLs participate, including otherwise unreferenced modules.
    /// The module names use the same restricted basename contract as LinkedLibrary.
    /// Cyclic/self dependencies, static TLS, SEH, reentrancy and threads fail or
    /// remain outside the trusted-code contract. No external DLL is discovered.
    ///
    /// # Safety
    /// Every DLL must obey LoadedLibrary's native-code and context-lifetime
    /// contract. Caller-assigned module identities must be correct. Guest code
    /// cannot retain addresses past this owner or use loader callbacks/threads.
    /// A failed ATTACH detaches that DLL, then rolls back already attached DLLs
    /// in reverse order; external initialization side effects are not undone.
    pub unsafe fn load(inputs: &[(&str, &[u8])], rebase: bool) -> Result<Self> {
        if inputs.is_empty() || inputs.len() > 64 {
            return Err("closed DLL bundle requires 1..64 modules".into());
        }
        let mut modules = Vec::new();
        let mut indices = BTreeMap::new();
        for &(name, bytes) in inputs {
            let name = dll_basename(name)?;
            if indices.insert(name.clone(), modules.len()).is_some() {
                return Err("duplicate case-insensitive bundle DLL name".into());
            }
            let image = Image::parse_dll(bytes)?;
            image.check_library_startup()?;
            let imports = image.imports()?;
            let relocations = image.relocations()?;
            if rebase && relocations.is_empty() {
                return Err("forced bundle rebase requires DIR64 fixups in every DLL".into());
            }
            if imports
                .iter()
                .any(|i| relocations.iter().any(|&r| r < i.iat + 8 && i.iat < r + 8))
            {
                return Err("bundle IAT overlaps relocation fixup".into());
            }
            let exports = image.exports()?.map(|t| t.entries).unwrap_or_default();
            let mut names = BTreeMap::new();
            let mut ordinals = BTreeMap::new();
            for (index, export) in exports.iter().enumerate() {
                ordinals.insert(export.ordinal, index);
                for name in &export.names {
                    names.insert(name.clone(), index);
                }
            }
            modules.push(Module {
                name,
                bytes,
                imports,
                exports,
                names,
                ordinals,
            });
        }
        let mut edges = vec![BTreeSet::new(); modules.len()];
        for (i, module) in modules.iter().enumerate() {
            let mut targets: Vec<_> = module
                .imports
                .iter()
                .map(|v| (v.module.clone(), v.symbol.clone()))
                .collect();
            for export in &module.exports {
                if let ExportTarget::Forwarder(text) = &export.target {
                    targets.push(forwarder_symbol(text)?);
                }
            }
            for (name, symbol) in targets {
                if let Some(target) = dependency(&name, &symbol, &indices)? {
                    edges[i].insert(target);
                }
            }
        }
        let mut order = Vec::new();
        let mut states = vec![0; modules.len()];
        for i in 0..modules.len() {
            visit(i, &edges, &mut states, &mut order)?;
        }
        // The graph is acyclic; recursively validate every forwarded target and
        // every import before mapping or calling any initializer.
        for module in &modules {
            for import in &module.imports {
                if let Some(target) = dependency(&import.module, &import.symbol, &indices)? {
                    verify_symbol(target, &import.symbol, &modules, &indices, 0)?;
                }
            }
            for export in &module.exports {
                if let ExportTarget::Forwarder(text) = &export.target {
                    let (name, symbol) = forwarder_symbol(text)?;
                    if let Some(target) = dependency(&name, &symbol, &indices)? {
                        verify_symbol(target, &symbol, &modules, &indices, 0)?;
                    }
                }
            }
        }
        let mut result = Self {
            libraries: Vec::new(),
        };
        for i in order {
            let resolve = |name: &str, symbol: &ImportSymbol| {
                resolve_builtin(name, symbol).or_else(|| {
                    result.get(name).and_then(|library| {
                        match symbol {
                            ImportSymbol::Name(name) => library.get_name(name),
                            ImportSymbol::Ordinal(n) if *n != 0 => {
                                library.get_ordinal(u32::from(*n))
                            }
                            ImportSymbol::Ordinal(_) => None,
                        }
                        .map(|p| p as usize)
                    })
                })
            };
            // SAFETY: complete metadata/symbol graph is checked above. Earlier
            // entries own every dependency until this set tears down in reverse.
            let library =
                unsafe { LoadedLibrary::load_resolved(modules[i].bytes, rebase, &resolve) }?;
            result.libraries.push((modules[i].name.clone(), library));
        }
        Ok(result)
    }
    /// Borrow an attached DLL by its case-insensitive supplied basename. Raw
    /// symbol addresses remain valid only while this set/context stay alive.
    pub fn get(&self, name: &str) -> Option<&LoadedLibrary> {
        self.libraries
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
            .map(|(_, library)| library)
    }
}
impl Drop for LibrarySet {
    fn drop(&mut self) {
        // Ordinary container destruction does not express reverse dependency
        // order. Explicit popping also handles constructor rollback.
        while let Some((_, library)) = self.libraries.pop() {
            drop(library);
        }
    }
}
