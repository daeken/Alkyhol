//! Closed, single-thread PE32 process ownership. No filesystem DLL search.
use super::{CallingConvention32, Cpu32, LoadedImage32, MapContext32, Module32, Phase32};
use crate::{ExportTarget, Image, ImportSymbol, ProcessContext, Result};
use std::{collections::BTreeMap, rc::Rc};

struct Node<'a> {
    name: String,
    image: Image<'a>,
    bytes: &'a [u8],
}
struct Plan<'a> {
    nodes: Vec<Node<'a>>,
    order: Vec<usize>,
    edges: Vec<Vec<usize>>,
    deferred: Vec<(String, String, ImportSymbol)>,
}
fn executable_name(name: &str) -> Result<String> {
    if name.len() > 255
        || !name.to_ascii_lowercase().ends_with(".exe")
        || !name
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err("invalid explicit PE32 executable basename".into());
    }
    Ok(name.to_ascii_lowercase())
}
fn target(table: Option<&crate::ExportTable>, symbol: &ImportSymbol) -> Result<usize> {
    let export = table
        .and_then(|t| {
            t.entries.iter().find(|e| match symbol {
                ImportSymbol::Name(name) => e.names.contains(name),
                ImportSymbol::Ordinal(n) => *n != 0 && e.ordinal == u32::from(*n),
            })
        })
        .ok_or_else(|| format!("missing PE32 process export {symbol:?}"))?;
    match export.target {
        ExportTarget::Address { rva } => Ok(rva),
        ExportTarget::Forwarder(_) => Err("PE32 process export forwarders are unsupported".into()),
    }
}
fn visit(
    index: usize,
    edges: &[Vec<usize>],
    states: &mut [u8],
    order: &mut Vec<usize>,
) -> Result<()> {
    match states[index] {
        2 => return Ok(()),
        1 => return Err("cyclic PE32 process imports are unsupported".into()),
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
impl<'a> Plan<'a> {
    fn new(executable: (&str, &'a [u8]), dlls: &[(&str, &'a [u8])], rebase: bool) -> Result<Self> {
        Self::with_external(executable, dlls, rebase, None)
    }
    fn with_external(
        executable: (&str, &'a [u8]),
        dlls: &[(&str, &'a [u8])],
        rebase: bool,
        external: Option<&[String]>,
    ) -> Result<Self> {
        let mut external_names = std::collections::BTreeSet::new();
        for name in external.unwrap_or(&[]) {
            let n = name.to_ascii_lowercase();
            if n.len() > 255
                || !n.ends_with(".dll")
                || !n
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                || !external_names.insert(n)
            {
                return Err("invalid or duplicate diagnostic external module".into());
            }
        }
        let mut deferred = Vec::new();
        if dlls.len() > 64 {
            return Err("PE32 process exceeds 64 explicit DLLs".into());
        }
        let mut nodes = vec![Node {
            name: executable_name(executable.0)?,
            image: if external.is_some() {
                Image::parse_i386_diagnostic(executable.1, false)?
            } else {
                Image::parse_i386(executable.1)?
            },
            bytes: executable.1,
        }];
        if external.is_some() {
            nodes[0]
                .image
                .check_diagnostic_startup32()
                .map_err(|e| format!("{}: {e}", executable.0))?;
        } else {
            nodes[0].image.check_startup()?;
        }
        let mut indices = BTreeMap::new();
        for &(name, bytes) in dlls {
            let name = crate::runtime::dll_basename(name)?;
            if external_names.contains(&name) {
                return Err(
                    "provided DLL cannot be replaced by diagnostic external imports".into(),
                );
            }
            if indices.insert(name.clone(), nodes.len()).is_some() {
                return Err("duplicate PE32 process DLL basename".into());
            }
            let image = if external.is_some() {
                Image::parse_i386_diagnostic(bytes, true)?
            } else {
                Image::parse_i386_dll(bytes)?
            };
            if external.is_some() {
                image
                    .check_diagnostic_startup32()
                    .map_err(|e| format!("{name}: {e}"))?;
            } else {
                image.check_library_startup()?;
            }
            nodes.push(Node { name, image, bytes });
        }
        let mut exports = Vec::new();
        for node in &nodes {
            let fixups = node.image.relocations()?;
            if rebase && fixups.is_empty() {
                return Err(
                    "forced PE32 process relocation requires HIGHLOW fixups in every module".into(),
                );
            }
            if node
                .image
                .imports()?
                .iter()
                .any(|i| fixups.iter().any(|&r| r < i.iat + 4 && i.iat < r + 4))
            {
                return Err("PE32 process IAT overlaps HIGHLOW fixup".into());
            }
            let table = node.image.exports()?;
            if table.as_ref().is_some_and(|t| {
                t.entries
                    .iter()
                    .any(|e| matches!(e.target, ExportTarget::Forwarder(_)))
            }) {
                return Err("PE32 process export forwarders are unsupported".into());
            }
            exports.push(table);
        }
        let mut edges = vec![Vec::new(); nodes.len()];
        for (index, node) in nodes.iter().enumerate() {
            for import in node.image.imports()? {
                if super::api_id(&import.module, &import.symbol).is_some() {
                    continue;
                }
                if external_names.contains(&import.module.to_ascii_lowercase()) {
                    deferred.push((
                        node.name.clone(),
                        import.module.clone(),
                        import.symbol.clone(),
                    ));
                    continue;
                }
                if crate::abi::is_builtin(&import.module) {
                    return Err(format!(
                        "unsupported Win32 process builtin {}!{:?}",
                        import.module, import.symbol
                    ));
                }
                let dependency = *indices
                    .get(&import.module.to_ascii_lowercase())
                    .ok_or_else(|| format!("missing DLL from PE32 process: {}", import.module))?;
                target(exports[dependency].as_ref(), &import.symbol)?;
                if !edges[index].contains(&dependency) {
                    edges[index].push(dependency);
                }
            }
        }
        let mut order = Vec::new();
        visit(0, &edges, &mut vec![0; nodes.len()], &mut order)?;
        if order.len() != nodes.len() {
            return Err("unreferenced DLL in explicit PE32 process".into());
        }
        Ok(Self {
            nodes,
            order,
            edges,
            deferred,
        })
    }
}

/// Own one executable and its complete explicitly supplied acyclic DLL graph.
/// All modules map and bind before any initializer. A shared guest stack/TEB
/// spans static DLL initialization, EXE entry and reverse DLL process teardown.
/// DLL names identify caller-supplied bytes; no filesystem search or loader API.
/// No DLL TLS/SEH, forwarders, cyclic imports, guest threads or reentrant loading.
///
/// ```compile_fail,E0505
/// # use alkyhol_pe::{ProcessImage32,ProcessContext};
/// # fn example(bytes:&[u8])->alkyhol_pe::Result<()> {
/// let process=ProcessContext::install(&["program.exe".into()],&[])?;
/// let image=unsafe {ProcessImage32::load(("program.exe",bytes),&[],false,&process)}?;
/// drop(process);
/// drop(image);
/// # Ok(()) }
/// ```
pub struct ProcessImage32<'a> {
    // Dependency-first order; the last item is the executable. This vector is
    // private: an individual mapping cannot escape its dependency owner.
    mappings: Vec<LoadedImage32<'a>>,
    attached: usize,
    started: bool,
    finished: bool,
}
/// Explicit backed guest-profile roles. Default supplies no Windows installation
/// or system directory. A configured directory is verified before guest entry;
/// this does not install Windows files or establish side-by-side bindings.
#[derive(Default)]
pub struct ProcessOptions32 {
    pub system_directory: Option<std::path::PathBuf>,
    /// Opt in to scoped, exact unsigned single-file assembly binding from
    /// explicitly supplied manifests and genuine provided module origins.
    /// No search, publisher policy, signed/hash metadata or implicit fallback.
    pub activation_assemblies: Option<Vec<std::path::PathBuf>>,
}
impl<'a> ProcessImage32<'a> {
    /// Check the complete closed graph without mapping or executing any module.
    /// This is metadata/import inspection, not a runtime compatibility result.
    pub fn preflight(
        executable: (&str, &[u8]),
        dlls: &[(&str, &[u8])],
        rebase: bool,
    ) -> Result<()> {
        Plan::new(executable, dlls, rebase).map(|_| ())
    }
    /// Validate the complete graph, map all modules and perform static ATTACH.
    /// A false initializer returns an explicit initialization error and rolls
    /// back acquired initializers; it never becomes successful application entry.
    ///
    /// # Safety
    /// All supplied instructions/initializers/teardown obey LoadedImage32's
    /// trusted integer-only contract. Supplied module identities must be correct.
    /// No threads, retained pointers, direct syscalls or unsupported exceptions.
    pub unsafe fn load(
        executable: (&str, &[u8]),
        dlls: &[(&str, &[u8])],
        rebase: bool,
        process: &'a ProcessContext,
    ) -> Result<Self> {
        let plan = Plan::new(executable, dlls, rebase)?;
        unsafe { Self::load_plan(plan, rebase, process, false, &BTreeMap::new(), None, None) }
    }
    /// Instrument a trusted startup path with guarded external-service imports.
    /// This is diagnostic execution, NOT Windows compatibility: missing service
    /// code/data and every fault stop explicitly; no exception recovery is claimed.
    /// Supplied original DLLs still map, bind and initialize in dependency order.
    ///
    /// # Safety
    /// Same native-code restrictions as load. External module names explicitly
    /// describe host services outside the supplied binary graph, not DLLs whose
    /// original initialization may be skipped. All observations stay diagnostic.
    pub unsafe fn load_diagnostic(
        executable: (&str, &[u8]),
        dlls: &[(&str, &[u8])],
        external: &[String],
        rebase: bool,
        process: &'a ProcessContext,
    ) -> Result<Self> {
        let plan = Plan::with_external(executable, dlls, rebase, Some(external))?;
        unsafe { Self::load_plan(plan, rebase, process, true, &BTreeMap::new(), None, None) }
    }
    /// Load a caller-supplied file graph and retain each actual canonical source
    /// path before initializers run. Z: mirrors the real host root. No DLL search,
    /// system-directory substitution, or paths inferred from basenames/bytes.
    /// `Some(external)` selects the explicit guarded diagnostic profile;
    /// `None` selects strict import preflight. Existing byte loaders stay separate.
    ///
    /// # Safety
    /// The same trusted native-code/initializer contract as load applies to all
    /// supplied files. This reads and executes those files, not a security sandbox.
    ///
    /// ```compile_fail,E0505
    /// # use alkyhol_pe::{ProcessImage32,ProcessContext};
    /// # fn example(path:&std::path::Path)->alkyhol_pe::Result<()> {
    /// let process=ProcessContext::install(&["program.exe".into()],&[])?;
    /// let image=unsafe {ProcessImage32::load_files(path,&[],None,false,&process)}?;
    /// drop(process);
    /// drop(image);
    /// # Ok(()) }
    /// ```
    pub unsafe fn load_files(
        executable: &std::path::Path,
        dlls: &[(&str, &std::path::Path)],
        external: Option<&[String]>,
        rebase: bool,
        process: &'a ProcessContext,
    ) -> Result<Self> {
        unsafe {
            Self::load_files_configured(
                executable,
                dlls,
                external,
                rebase,
                process,
                &ProcessOptions32::default(),
            )
        }
    }
    /// Load the same explicit file graph with verified guest-profile paths.
    /// No directory is guessed from a module filename or silently created.
    ///
    /// # Safety
    /// The same trusted native-code contract as load_files applies. Profile
    /// roles must describe the intended guest setup, not substituted DLL origins.
    ///
    /// ```compile_fail,E0505
    /// # use alkyhol_pe::{ProcessImage32,ProcessContext,ProcessOptions32};
    /// # fn example(path:&std::path::Path)->alkyhol_pe::Result<()> {
    /// let process=ProcessContext::install(&["program.exe".into()],&[])?;
    /// let options=ProcessOptions32::default();
    /// let image=unsafe {ProcessImage32::load_files_configured(path,&[],None,false,&process,&options)}?;
    /// drop(process);
    /// drop(image);
    /// # Ok(()) }
    /// ```
    pub unsafe fn load_files_configured(
        executable: &std::path::Path,
        dlls: &[(&str, &std::path::Path)],
        external: Option<&[String]>,
        rebase: bool,
        process: &'a ProcessContext,
        options: &ProcessOptions32,
    ) -> Result<Self> {
        let system_directory = options
            .system_directory
            .as_deref()
            .map(super::module_path32::directory)
            .transpose()?;
        if dlls.len() > 64 {
            return Err("PE32 process exceeds 64 explicit DLLs".into());
        }
        let (exe_bytes, exe_origin) = super::module_path32::read(executable)?;
        let name = exe_origin
            .host
            .file_name()
            .and_then(|v| v.to_str())
            .ok_or("non-Unicode EXE basename")?;
        let mut paths = BTreeMap::new();
        paths.insert(executable_name(name)?, exe_origin.clone());
        let mut files = Vec::new();
        for &(name, path) in dlls {
            let name = crate::runtime::dll_basename(name)?;
            let (bytes, origin) = super::module_path32::read(path)?;
            if paths.insert(name.clone(), origin).is_some() {
                return Err("duplicate module source name".into());
            }
            files.push((name, bytes));
        }
        let dlls: Vec<_> = files
            .iter()
            .map(|(name, bytes)| (name.as_str(), bytes.as_slice()))
            .collect();
        let plan = Plan::with_external((name, &exe_bytes), &dlls, rebase, external)?;
        unsafe {
            Self::load_plan(
                plan,
                rebase,
                process,
                external.is_some(),
                &paths,
                system_directory,
                options.activation_assemblies.as_deref(),
            )
        }
    }
    unsafe fn load_plan(
        plan: Plan<'_>,
        rebase: bool,
        process: &'a ProcessContext,
        diagnostic: bool,
        origins: &BTreeMap<String, super::module_path32::Origin>,
        system_directory: Option<Vec<u16>>,
        assembly_paths: Option<&[std::path::PathBuf]>,
    ) -> Result<Self> {
        let mut cpu = Cpu32::new()?;
        cpu.system_directory = system_directory;
        // Keep unknown manifest/binding state deferred until an activation query.
        // Existing paths do not gain a fabricated empty/default context.
        *cpu.activation.borrow_mut() = crate::activation_manifest::empty_default(
            &plan.nodes[0].image,
            &plan.nodes[1..].iter().map(|n| &n.image).collect::<Vec<_>>(),
        );
        if diagnostic {
            cpu.diagnostic = Some(super::diagnostic32::Traps::new(plan.deferred.clone())?);
        }
        let cpu = Rc::new(cpu);
        cpu.phase.set(Phase32::Preparing);
        let mut mappings: Vec<LoadedImage32<'a>> = Vec::new();
        let mut modules: Vec<(String, usize)> = Vec::new();
        for &index in &plan.order {
            let node = &plan.nodes[index];
            let resolve = |name: &str, symbol: &ImportSymbol| {
                if let Some(address) = cpu
                    .diagnostic
                    .as_ref()
                    .and_then(|t| t.address(&node.name, name, symbol))
                {
                    return Ok(Some(address));
                }
                let Some((_, i)) = modules.iter().find(|(n, _)| n.eq_ignore_ascii_case(name))
                else {
                    return Ok(None);
                };
                let mapping = &mappings[*i];
                let rva = target(mapping.exports.as_ref(), symbol)?;
                (mapping.image.base as u32)
                    .checked_add(rva as u32)
                    .map(Some)
                    .ok_or_else(|| "PE32 process import address overflow".into())
            };
            let mapping = LoadedImage32::map_resolved(
                node.bytes,
                rebase,
                process,
                index != 0,
                false,
                MapContext32 {
                    cpu: Some(cpu.clone()),
                    resolve: &resolve,
                    regions: &[],
                },
            )?;
            modules.push((node.name.clone(), mappings.len()));
            mappings.push(mapping);
        }
        let regions: Vec<_> = mappings
            .iter()
            .flat_map(|m| m.regions.iter().copied())
            .collect();
        for mapping in &mut mappings {
            mapping.regions = regions.clone();
        }
        cpu.executable
            .set(mappings.last().expect("root mapping").image.base as u32);
        if let Some(paths) = assembly_paths {
            let origins = origins
                .iter()
                .map(|(name, origin)| (name.clone(), origin.host.clone()))
                .collect();
            let inventory = crate::activation_binding::Inventory::new(paths, &origins)?;
            let templates: Vec<_> = plan
                .nodes
                .iter()
                .enumerate()
                .map(|(i, n)| inventory.template(&n.image, i == 0))
                .collect();
            let own: Vec<_> = templates.iter().map(|t| !matches!(t, Ok(None))).collect();
            let owners = scope_owners(&plan.edges, &own)?;
            let templates = plan
                .nodes
                .iter()
                .zip(templates)
                .map(|(node, template)| {
                    let (_, index) = modules
                        .iter()
                        .find(|(name, _)| name == &node.name)
                        .expect("mapped node");
                    (mappings[*index].image.base as u32, template)
                })
                .collect();
            *cpu.bound_activation.borrow_mut() =
                Some(super::activation32::State::new(templates, &owners)?);
        }
        *cpu.modules.borrow_mut() = modules
            .into_iter()
            .map(|(name, index)| Module32 {
                origin: origins.get(&name).cloned(),
                name,
                base: mappings[index].image.base as u32,
                size: mappings[index].image.len,
                exports: mappings[index].exports.clone(),
            })
            .collect();
        let mut result = Self {
            mappings,
            attached: 0,
            started: false,
            finished: false,
        };
        for index in 0..result.mappings.len() - 1 {
            if unsafe { result.dll_entry(index, 1, true) }? == 0 {
                // A failed initializer is not an acquired successful module.
                unsafe { result.dll_entry(index, 0, false) }?;
                return Err("PE32 static DLL PROCESS_ATTACH returned FALSE".into());
            }
            result.attached += 1;
        }
        cpu.phase.set(Phase32::Ready);
        Ok(result)
    }
    unsafe fn dll_entry(&self, index: usize, reason: u32, non_null: bool) -> Result<u32> {
        let image = &self.mappings[index];
        let _scope = super::activation32::enter_scope(&image.cpu, image.image.base as u32);
        if image.entry == 0 {
            return Ok(1);
        }
        // Reserved is opaque; expose only a valid owned non-NULL storage token,
        // not an invented Windows startup structure. Dereferencing is unsupported.
        let reserved = if non_null {
            image.cpu.teb.base as u32
        } else {
            0
        };
        unsafe {
            image.invoke(
                image.entry,
                &[image.image.base as u32, reason, reserved],
                CallingConvention32::Stdcall,
            )
        }
    }
    fn shutdown(&mut self, terminating: bool) -> Result<()> {
        if self.finished {
            return Ok(());
        }
        let cpu = self.mappings.last().expect("root mapping").cpu.clone();
        cpu.phase.set(Phase32::Detaching);
        while self.attached > 0 {
            self.attached -= 1;
            unsafe { self.dll_entry(self.attached, 0, terminating) }?;
        }
        cpu.close_files();
        cpu.clear_heap();
        cpu.phase.set(Phase32::Exited);
        self.finished = true;
        Ok(())
    }
    /// Execute the original EXE once. Imported ExitProcess escapes through the
    /// CPU exit gate, then real DLL DETACH runs before owned file handles close.
    ///
    /// # Safety
    /// Same native-code contract as load; integer entry-result/ExitProcess only.
    pub unsafe fn enter(&mut self) -> Result<u32> {
        if self.started || self.finished {
            return Err("PE32 process cannot be restarted".into());
        }
        self.started = true;
        let image = self.mappings.last().expect("root mapping");
        image.cpu.phase.set(Phase32::Running);
        let result = unsafe { image.enter() };
        if let Ok(status) = result {
            image.cpu.exit.set(Some(status));
        }
        self.shutdown(true)?;
        result
    }
    /// Actual relocation decisions in dependency initialization order, EXE last.
    pub fn relocations(&self) -> Vec<bool> {
        self.mappings.iter().map(LoadedImage32::relocated).collect()
    }
}

// Match the loader's first dependency traversal: a resource-2 module supplies
// its own scope to its initializer and manifest-free imports. A shared module
// keeps its first loading scope; manifests are never globally unioned.
fn scope_owners(edges: &[Vec<usize>], own: &[bool]) -> Result<Vec<usize>> {
    if edges.is_empty() || edges.len() != own.len() || !own[0] {
        return Err("activation scope graph lacks a root context".into());
    }
    let mut owners = vec![usize::MAX; edges.len()];
    fn walk(
        index: usize,
        inherited: usize,
        edges: &[Vec<usize>],
        own: &[bool],
        owners: &mut [usize],
    ) -> Result<()> {
        if index >= edges.len() {
            return Err("activation dependency outside graph".into());
        }
        if owners[index] != usize::MAX {
            return Ok(());
        }
        let scope = if own[index] { index } else { inherited };
        owners[index] = scope;
        for &child in &edges[index] {
            walk(child, scope, edges, own, owners)?;
        }
        Ok(())
    }
    walk(0, 0, edges, own, &mut owners)?;
    if owners.contains(&usize::MAX) {
        return Err("unreferenced activation scope module".into());
    }
    Ok(owners)
}
impl Drop for ProcessImage32<'_> {
    fn drop(&mut self) {
        if let Err(error) = self.shutdown(false) {
            super::unsupported(&error);
        }
        // Reverse order is intentional, including after initialization rollback.
        while let Some(mapping) = self.mappings.pop() {
            drop(mapping);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn activation_scopes_inherit_first_importer_and_restore_other_owners() {
        let edges = vec![vec![1, 3], vec![2], vec![], vec![2]];
        assert_eq!(
            scope_owners(&edges, &[true, true, false, true]).unwrap(),
            [0, 1, 1, 3]
        );
        assert_eq!(
            scope_owners(&edges, &[true, false, false, true]).unwrap(),
            [0, 0, 0, 3]
        );
        assert!(scope_owners(&edges, &[false, true, false, true]).is_err());
        assert!(scope_owners(&[vec![2], vec![]], &[true, false]).is_err());
        assert!(scope_owners(&[vec![], vec![]], &[true, false]).is_err());
    }
    #[test]
    fn graph_preserves_dependency_order_shares_diamonds_and_rejects_cycles() {
        let edges = vec![vec![2, 1], vec![3], vec![3], vec![]];
        let mut order = Vec::new();
        visit(0, &edges, &mut [0; 4], &mut order).unwrap();
        assert_eq!(order, [3, 2, 1, 0]);
        for edges in [
            vec![vec![0]],
            vec![vec![1], vec![0]],
            vec![vec![1], vec![2], vec![1]],
        ] {
            assert!(visit(0, &edges, &mut vec![0; edges.len()], &mut Vec::new()).is_err());
        }
    }
    #[test]
    fn identities_reject_paths_reserved_names_and_ambiguous_executables() {
        assert_eq!(executable_name("APP.ExE").unwrap(), "app.exe");
        for name in [
            "",
            "../app.exe",
            "C:app.exe",
            "a/b.exe",
            "x.dll",
            ".exe",
            "app exe",
        ] {
            assert!(executable_name(name).is_err());
        }
    }

    #[test]
    fn module_context_exposes_the_executable_in_dll_startup_and_checks_export_ownership() {
        let mut image = super::super::tests::layout();
        image.dll = true;
        let table = crate::ExportTable {
            dll_name: "leaf.dll".into(),
            ordinal_base: 17,
            entries: vec![crate::Export {
                ordinal: 17,
                names: vec!["Data".into()],
                target: ExportTarget::Address { rva: 0x1000 },
            }],
        };
        image.cpu.executable.set(0x400000);
        *image.cpu.modules.borrow_mut() = vec![
            Module32 {
                name: "app.exe".into(),
                base: 0x400000,
                size: 0x10000,
                exports: None,
                origin: None,
            },
            Module32 {
                name: "leaf.dll".into(),
                base: 0x200000,
                size: 0x10000,
                exports: Some(table.clone()),
                origin: None,
            },
        ];
        for phase in [Phase32::Preparing, Phase32::Running, Phase32::Detaching] {
            image.cpu.phase.set(phase);
            assert_eq!(image.module_handle(0, false).unwrap(), Some(0x400000));
            assert_eq!(image.module_handle(0, true).unwrap(), Some(0x400000));
            assert_eq!(image.export_address(0x200000, 17).unwrap(), Some(0x201000));
            assert_eq!(image.export_address(0x200000, 18).unwrap(), None);
            assert_eq!(image.export_address(0x400000, 17).unwrap(), None);
            assert!(image.export_address(0xdead0000, 17).is_err());
        }
        assert_eq!(
            target(Some(&table), &ImportSymbol::Name("Data".into())).unwrap(),
            0x1000
        );
        assert!(target(Some(&table), &ImportSymbol::Name("data".into())).is_err());
        assert!(target(Some(&table), &ImportSymbol::Ordinal(0)).is_err());
        assert!(target(None, &ImportSymbol::Ordinal(17)).is_err());
    }
}
