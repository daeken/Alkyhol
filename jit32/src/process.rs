//! Checked single-thread PE32 process startup. No host-pointer guest imports.
use crate::memory::{EXEC, Memory, READ, WRITE};
use crate::{GATE, RETURN, Result, Runner, STACK, STACK_SIZE, Service, Stop};
use alkyhol_pe::{ExportTarget, Image, ImportSymbol, StaticTls32};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
};

pub struct ModuleInput<'a> {
    pub name: &'a str,
    pub bytes: &'a [u8],
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportGate {
    pub module: String,
    pub symbol: ImportSymbol,
}
#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub base: u32,
    pub size: u32,
    pub entry: u32,
    pub tls_index: Option<u32>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Phase {
    TlsAttach(String),
    DllAttach(String),
    Ready,
    Application,
    Finished,
    Failed,
}
#[derive(Clone)]
struct Initializer {
    module: usize,
    entry: u32,
    dll: bool,
}
struct Active {
    saved: [u64; 91],
    initial_sp: u32,
    expected_sp: u32,
}
struct Arena {
    next: u32,
}
impl Arena {
    fn allocate(
        &mut self,
        memory: &mut Memory,
        size: usize,
        alignment: usize,
        initial: &[u8],
    ) -> Result<u32> {
        if !alignment.is_power_of_two() || alignment > 65536 || size > 16 * 1024 * 1024 {
            return Err("startup allocation size/alignment exceeded".into());
        }
        let start = (self.next as u64 + alignment as u64 - 1) & !(alignment as u64 - 1);
        let end = start
            .checked_add(size.max(1) as u64)
            .filter(|e| *e <= 0x71000000)
            .ok_or("startup arena exhausted")?;
        memory.map(start as u32, size.max(1), READ | WRITE, initial)?;
        self.next = end as u32;
        Ok(start as u32)
    }
}
fn name(value: &str) -> Result<String> {
    if value.is_empty()
        || !value.is_ascii()
        || value.len() > 255
        || value.contains(['/', '\\', ':'])
        || value == "."
        || value == ".."
    {
        return Err("invalid explicit module basename".into());
    }
    Ok(value.to_ascii_lowercase())
}
fn system_module(value: &str) -> bool {
    [
        "netapi32.dll",
        "iphlpapi.dll",
        "msacm32.dll",
        "kernel32.dll",
        "kernelbase.dll",
        "ntdll.dll",
        "user32.dll",
        "gdi32.dll",
        "advapi32.dll",
        "ole32.dll",
        "oleaut32.dll",
        "shell32.dll",
        "shlwapi.dll",
        "winmm.dll",
        "ws2_32.dll",
        "wsock32.dll",
        "version.dll",
        "imm32.dll",
        "comdlg32.dll",
        "comctl32.dll",
        "psapi.dll",
        "msvcrt.dll",
        "d3d9.dll",
        "dinput8.dll",
        "dsound.dll",
        "dxgi.dll",
    ]
    .contains(&value)
}
fn order(root: usize, edges: &[Vec<usize>]) -> Result<Vec<usize>> {
    fn visit(
        i: usize,
        edges: &[Vec<usize>],
        visiting: &mut BTreeSet<usize>,
        done: &mut BTreeSet<usize>,
        out: &mut Vec<usize>,
    ) -> Result<()> {
        if done.contains(&i) {
            return Ok(());
        }
        if !visiting.insert(i) {
            return Err("cyclic DLL initialization is not yet admitted".into());
        }
        for &child in &edges[i] {
            visit(child, edges, visiting, done, out)?;
        }
        visiting.remove(&i);
        done.insert(i);
        out.push(i);
        Ok(())
    }
    let mut out = vec![];
    visit(
        root,
        edges,
        &mut BTreeSet::new(),
        &mut BTreeSet::new(),
        &mut out,
    )?;
    Ok(out)
}
fn place(preferred: u64, size: usize, force: bool, used: &[(u32, u32)]) -> Result<u32> {
    if size == 0 || size > 256 * 1024 * 1024 {
        return Err("module size budget exceeded".into());
    }
    let free = |base: u64| {
        base >= 0x10000
            && base & 0xffff == 0
            && base + size as u64 <= 0x70000000
            && used
                .iter()
                .all(|&(b, n)| base + size as u64 <= b as u64 || base >= b as u64 + n as u64)
    };
    if !force && free(preferred) {
        return Ok(preferred as u32);
    }
    let mut at = 0x10000000u64;
    while at + size as u64 <= 0x70000000 {
        if free(at) {
            return Ok(at as u32);
        }
        let end = used
            .iter()
            .filter(|&&(b, n)| at + size as u64 > b as u64 && at < b as u64 + n as u64)
            .map(|&(b, n)| b as u64 + n as u64)
            .max()
            .ok_or("cannot place module")?;
        at = (end + 65535) & !65535;
    }
    Err("module address space exhausted".into())
}
struct Binding<'a> {
    images: &'a [Image<'a>],
    bases: &'a [u32],
    names: &'a BTreeMap<String, usize>,
    gates: BTreeMap<u32, Service>,
    external: BTreeMap<u32, ImportGate>,
    keys: BTreeMap<(String, String), u32>,
}
impl Binding<'_> {
    fn resolve(
        &mut self,
        module: &str,
        symbol: &ImportSymbol,
        seen: &mut BTreeSet<(String, String)>,
    ) -> Result<u32> {
        let module = name(module)?;
        let key = (module.clone(), format!("{symbol:?}"));
        if seen.len() >= 16 || !seen.insert(key.clone()) {
            return Err("cyclic or excessive export forwarding".into());
        }
        if let Some(&i) = self.names.get(&module) {
            let table = self.images[i]
                .exports()?
                .ok_or_else(|| format!("{module} has no export table"))?;
            let export = table
                .entries
                .iter()
                .find(|e| match symbol {
                    ImportSymbol::Name(n) => e.names.contains(n),
                    ImportSymbol::Ordinal(n) => e.ordinal == *n as u32,
                })
                .ok_or_else(|| format!("missing export {module}!{symbol:?}"))?;
            return match &export.target {
                ExportTarget::Address { rva } => self.bases[i]
                    .checked_add(*rva as u32)
                    .ok_or("export address overflow".into()),
                ExportTarget::Forwarder(text) => {
                    let (dll, entry) = text.rsplit_once('.').ok_or("invalid export forwarder")?;
                    let dll = if dll.to_ascii_lowercase().ends_with(".dll") {
                        dll.to_string()
                    } else {
                        format!("{dll}.dll")
                    };
                    let symbol = if let Some(n) = entry.strip_prefix('#') {
                        ImportSymbol::Ordinal(n.parse().map_err(|_| "invalid forwarded ordinal")?)
                    } else {
                        ImportSymbol::Name(entry.into())
                    };
                    self.resolve(&dll, &symbol, seen)
                }
            };
        }
        if !system_module(&module) {
            return Err(format!("required DLL not explicitly supplied: {module}"));
        }
        if let Some(&gate) = self.keys.get(&key) {
            return Ok(gate);
        }
        if self.keys.len() >= 4000 {
            return Err("process import gate budget exceeded".into());
        }
        let gate = GATE + self.keys.len() as u32 * 16;
        match Service::resolve(&module, symbol) {
            Ok(service) => {
                self.gates.insert(gate, service);
            }
            Err(_) => {
                self.external.insert(
                    gate,
                    ImportGate {
                        module: module.clone(),
                        symbol: symbol.clone(),
                    },
                );
            }
        }
        self.keys.insert(key, gate);
        Ok(gate)
    }
}

pub struct Process {
    machine: Runner,
    modules: Vec<Module>,
    startup: Vec<Initializer>,
    next: usize,
    active: Option<Active>,
    phase: Phase,
    entry: u32,
}
impl Process {
    /// The first input is the EXE; subsequent DLLs are explicit available files.
    /// Only statically reachable DLLs are initialized. Unknown OS imports become
    /// explicit exits, while missing original DLLs/exports fail before any call.
    pub fn load(inputs: &[ModuleInput<'_>], rebase: bool, stdout: File) -> Result<Self> {
        if inputs.is_empty() || inputs.len() > 65 {
            return Err("process requires EXE and at most 64 DLLs".into());
        }
        let mut names = BTreeMap::new();
        let mut images = vec![];
        for (i, input) in inputs.iter().enumerate() {
            if names.insert(name(input.name)?, i).is_some() {
                return Err("duplicate module name".into());
            }
            let image = Image::parse_i386_diagnostic(input.bytes, i != 0)?;
            images.push(image);
        }
        let mut edges = vec![vec![]; inputs.len()];
        // Resolve only reachable imports. An available dormant DLL is not a
        // loaded module and must not introduce new startup dependencies.
        let mut pending = vec![0];
        let mut traversed = BTreeSet::new();
        while let Some(i) = pending.pop() {
            if !traversed.insert(i) {
                continue;
            }
            let image = &images[i];
            image.check_emulated_startup32()?;
            for import in image.imports()? {
                let dep = name(&import.module)?;
                if let Some(&j) = names.get(&dep) {
                    if j != i && !edges[i].contains(&j) {
                        edges[i].push(j);
                        pending.push(j);
                    }
                } else if !system_module(&dep) {
                    return Err(format!("required DLL not explicitly supplied: {dep}"));
                }
            }
        }
        let ordered = order(0, &edges)?;
        let loaded: BTreeSet<_> = ordered.iter().copied().collect();
        if ordered.iter().map(|&i| images[i].size as u64).sum::<u64>() > 1024 * 1024 * 1024 {
            return Err("combined mapped image budget exceeded".into());
        }
        let mut bases = vec![0; images.len()];
        let mut used = vec![];
        // Keep the EXE preferred first; relocate conflicting DLLs deterministically.
        for i in std::iter::once(0).chain(ordered.iter().copied().filter(|&i| i != 0)) {
            bases[i] = place(images[i].base, images[i].size, rebase, &used)?;
            used.push((bases[i], images[i].size as u32));
        }
        let active_names = names
            .iter()
            .filter(|(_, i)| loaded.contains(i))
            .map(|(n, &i)| (n.clone(), i))
            .collect();
        let mut binding = Binding {
            images: &images,
            bases: &bases,
            names: &active_names,
            gates: BTreeMap::new(),
            external: BTreeMap::new(),
            keys: BTreeMap::new(),
        };
        let mut mapped = BTreeMap::new();
        let mut tls = BTreeMap::<usize, StaticTls32>::new();
        for &i in &ordered {
            let image = &images[i];
            let base = bases[i];
            let mut data = vec![0; image.size];
            data[..image.headers].copy_from_slice(&image.bytes[..image.headers]);
            for s in &image.sections {
                let n = s.raw_size.min(s.size);
                data[s.rva..s.rva + n].copy_from_slice(&image.bytes[s.raw..s.raw + n]);
            }
            let fixups = image.relocations()?;
            if base as u64 != image.base {
                if fixups.is_empty() {
                    return Err(format!(
                        "cannot relocate {} without HIGHLOW fixups",
                        inputs[i].name
                    ));
                }
                for at in fixups {
                    let v = u32::from_le_bytes(data[at..at + 4].try_into().unwrap())
                        .wrapping_add(base.wrapping_sub(image.base as u32));
                    data[at..at + 4].copy_from_slice(&v.to_le_bytes());
                }
            }
            // Cookie seeding is required for modern load-config structures; this
            // initial legacy profile admits no implicit exception dispatch.
            if let Some(config) = image.legacy_load_config32()?
                && let Some(at) = config.cookie_rva
            {
                let old = u32::from_le_bytes(data[at..at + 4].try_into().unwrap());
                // Preserve the existing frozen native policy: only a
                // declared 72-byte config with the default cookie is seeded.
                if config.declared_size == 72 && old == 0xbb40e64e {
                    let mut entropy = [0u8; 4];
                    let mut source = File::open("/dev/urandom").map_err(|e| e.to_string())?;
                    let mut selected = None;
                    for _ in 0..128 {
                        source.read_exact(&mut entropy).map_err(|e| e.to_string())?;
                        let value = u32::from_le_bytes(entropy);
                        if value != 0 && value != 0xbb40e64e {
                            selected = Some(value);
                            break;
                        }
                    }
                    data[at..at + 4].copy_from_slice(
                        &selected
                            .ok_or("security cookie entropy exhausted")?
                            .to_le_bytes(),
                    );
                }
            }
            for import in image.imports()? {
                let gate = binding.resolve(&import.module, &import.symbol, &mut BTreeSet::new())?;
                data[import.iat..import.iat + 4].copy_from_slice(&gate.to_le_bytes());
            }
            if let Some(info) = image.static_tls32()? {
                // Nonzero SizeOfZeroFill has divergent legacy native evidence;
                // admit only the independently compared zero-fill-zero profile.
                if info.zero_fill != 0 {
                    return Err(
                        "nonzero PE32 TLS zero-fill requires a separate native contract".into(),
                    );
                }
                tls.insert(i, info);
            }
            mapped.insert(i, data);
        }
        let mut memory = Box::new(Memory::new()?);
        let mut arena = Arena { next: 0x70000000 };
        memory.map(STACK, STACK_SIZE, READ | WRITE, &[])?;
        let mut teb = [0u8; 4096];
        for (off, v) in [(0, u32::MAX), (4, STACK + STACK_SIZE as u32), (8, STACK)] {
            teb[off..off + 4].copy_from_slice(&v.to_le_bytes());
        }
        let teb_addr = arena.allocate(&mut memory, 4096, 4096, &teb)?;
        memory.write(teb_addr as u64 + 0x18, &teb_addr.to_le_bytes())?;
        memory.write(teb_addr as u64 + 0x20, &std::process::id().to_le_bytes())?;
        memory.write(
            teb_addr as u64 + 0x24,
            &alkyhol::kernel::current_thread_id().to_le_bytes(),
        )?;
        let mut modules = vec![];
        let mut module_index = BTreeMap::new();
        let mut tls_blocks = vec![];
        // Assign stable EXE-first indices, independent of initializer order.
        for i in (0..inputs.len()).filter(|i| loaded.contains(i)) {
            let data = mapped.get_mut(&i).unwrap();
            let index = if let Some(info) = tls.get(&i) {
                let index = tls_blocks.len() as u32;
                if index >= 64 {
                    return Err("static TLS module budget exceeded".into());
                }
                let block = arena.allocate(
                    &mut memory,
                    info.total_size.max(1),
                    info.alignment.max(16),
                    &data[info.template_rva..info.template_rva + info.template_size],
                )?;
                data[info.index_rva..info.index_rva + 4].copy_from_slice(&index.to_le_bytes());
                tls_blocks.push(block);
                Some(index)
            } else {
                None
            };
            module_index.insert(i, modules.len());
            modules.push(Module {
                name: name(inputs[i].name)?,
                base: bases[i],
                size: images[i].size as u32,
                entry: if images[i].entry == 0 {
                    0
                } else {
                    bases[i] + images[i].entry as u32
                },
                tls_index: index,
            });
        }
        if !tls_blocks.is_empty() {
            let bytes = tls_blocks
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>();
            let vector = arena.allocate(&mut memory, 256, 16, &bytes)?;
            memory.write(teb_addr as u64 + 0x2c, &vector.to_le_bytes())?;
        }
        for &i in &ordered {
            let image = &images[i];
            let data = &mapped[&i];
            memory.map(bases[i], image.headers, READ, &data[..image.headers])?;
            for s in &image.sections {
                if s.size == 0 {
                    continue;
                }
                let rights = (u8::from(s.rights & 0x40000000 != 0) * READ)
                    | (u8::from(s.rights & 0x80000000 != 0) * WRITE)
                    | (u8::from(s.rights & 0x20000000 != 0) * EXEC);
                memory.map(
                    bases[i] + s.rva as u32,
                    s.size,
                    rights,
                    &data[s.rva..s.rva + s.size],
                )?;
            }
        }
        // Static process attach receives a non-NULL reserved indicator, as
        // specified by DllMain; it is not a fabricated dynamic-load callback.
        let mut startup = vec![];
        for &i in &ordered {
            if let Some(info) = tls.get(&i) {
                for &rva in &info.callbacks {
                    startup.push(Initializer {
                        module: module_index[&i],
                        entry: bases[i] + rva as u32,
                        dll: false,
                    });
                }
            }
            if i != 0 && images[i].entry != 0 {
                startup.push(Initializer {
                    module: module_index[&i],
                    entry: bases[i] + images[i].entry as u32,
                    dll: true,
                });
            }
        }
        let mut state = [0; 91];
        state[4] = STACK as u64 + STACK_SIZE as u64;
        state[16] = 0x202;
        state[22] = teb_addr as u64;
        state[17] = RETURN as u64;
        state[88] = memory.base();
        state[90] = (&mut *memory as *mut Memory) as u64;
        let stdout = alkyhol::kernel::kernel()
            .adopt_file(stdout, alkyhol::kernel::GENERIC_WRITE)
            .map_err(|e| format!("stdout: {e:?}"))?;
        let machine = Runner {
            memory,
            state,
            gates: binding.gates,
            external_gates: binding.external,
            teb: Some(teb_addr),
            system: crate::system::State::default(),
            stdout,
            blocks: BTreeMap::new(),
            instructions: 0,
            service_calls: 0,
        };
        Ok(Self {
            machine,
            modules,
            startup,
            next: 0,
            active: None,
            phase: Phase::Ready,
            entry: bases[0] + images[0].entry as u32,
        })
    }
    pub fn use_legacy_nt62_persona(&mut self) -> Result<()> {
        if self.machine.instructions != 0 || self.active.is_some() {
            return Err("guest persona must be selected before execution".into());
        }
        self.machine.system.legacy_nt62 = true;
        Ok(())
    }
    pub fn machine(&self) -> &Runner {
        &self.machine
    }
    pub fn modules(&self) -> &[Module] {
        &self.modules
    }
    pub fn phase(&self) -> &Phase {
        &self.phase
    }
    fn begin(&mut self, entry: u32, args: &[u32]) -> Result<()> {
        if self.active.is_some() {
            return Err("guest invocation already active".into());
        }
        self.machine
            .memory
            .check(entry as u64, 1, EXEC)
            .map_err(|f| format!("initializer entry fault: {f:?}"))?;
        let saved = self.machine.state;
        let initial_sp = saved[4] as u32;
        let size = (args.len() + 1)
            .checked_mul(4)
            .ok_or("guest argument overflow")?;
        let sp = initial_sp
            .checked_sub(size as u32)
            .ok_or("guest call stack underflow")?;
        let frame = std::iter::once(RETURN)
            .chain(args.iter().copied())
            .flat_map(|v| v.to_le_bytes())
            .collect::<Vec<_>>();
        self.machine.memory.write(sp as u64, &frame)?;
        self.machine.state[4] = sp as u64;
        self.machine.state[17] = entry as u64;
        self.active = Some(Active {
            saved,
            initial_sp,
            expected_sp: initial_sp,
        });
        Ok(())
    }
    /// Resume startup without repeating completed initializers or a stopped call.
    /// Ready means every admitted static initializer returned successfully.
    pub fn run_startup(&mut self, mut budget: u64) -> Stop {
        if matches!(
            self.phase,
            Phase::Failed | Phase::Application | Phase::Finished
        ) {
            return Stop::Unsupported {
                pc: self.machine.pc(),
                reason: "startup cannot run in this process phase".into(),
            };
        }
        while self.next < self.startup.len() {
            let call = self.startup[self.next].clone();
            if self.active.is_none() {
                let module = &self.modules[call.module];
                self.phase = if call.dll {
                    Phase::DllAttach(module.name.clone())
                } else {
                    Phase::TlsAttach(module.name.clone())
                };
                let args = [module.base, 1, self.machine.teb.unwrap()];
                if let Err(reason) = self.begin(call.entry, &args) {
                    self.phase = Phase::Failed;
                    return Stop::Unsupported {
                        pc: call.entry,
                        reason,
                    };
                }
            }
            let before = self.machine.instructions + self.machine.service_calls;
            let stop = self.machine.run(budget);
            budget = budget
                .saturating_sub(self.machine.instructions + self.machine.service_calls - before);
            if let Stop::Returned(value) = stop {
                let active = self.active.take().unwrap();
                let preserved = [3, 5, 6, 7]
                    .iter()
                    .all(|&i| self.machine.state[i] == active.saved[i]);
                if self.machine.state[4] != active.expected_sp as u64
                    || !preserved
                    || call.dll && value == 0
                {
                    self.phase = Phase::Failed;
                    return Stop::Unsupported {
                        pc: call.entry,
                        reason: "initializer rejected attach or violated stdcall/nonvolatile state"
                            .into(),
                    };
                }
                self.machine.state = active.saved;
                self.next += 1;
            } else {
                return stop;
            }
        }
        self.phase = Phase::Ready;
        Stop::Returned(0)
    }
    /// Enter original EXE only after all original static initializers succeeded.
    pub fn run_application(&mut self, budget: u64) -> Stop {
        if self.next != self.startup.len()
            || self.phase == Phase::Failed
            || self.phase == Phase::Finished
        {
            return Stop::Unsupported {
                pc: self.machine.pc(),
                reason: "application entry requires completed startup and a live process".into(),
            };
        }
        if self.phase != Phase::Application {
            if let Err(reason) = self.begin(self.entry, &[]) {
                return Stop::Unsupported {
                    pc: self.entry,
                    reason,
                };
            }
            self.phase = Phase::Application;
        }
        let stop = self.machine.run(budget);
        if let Stop::Returned(_) = stop {
            let active = self.active.take().unwrap();
            if self.machine.state[4] != active.initial_sp as u64 {
                return Stop::Unsupported {
                    pc: self.entry,
                    reason: "EXE return stack mismatch".into(),
                };
            }
            self.phase = Phase::Finished;
        }
        stop
    }
}
use std::io::Read;

#[cfg(test)]
#[path = "process_tests.rs"]
mod tests;
