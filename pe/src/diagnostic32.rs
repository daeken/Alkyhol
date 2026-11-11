//! Opt-in startup instrumentation. Unimplemented imports have inaccessible
//! addresses: function execution AND data reads stop, never fabricate values.
use super::{LoadedImage32, LowMapping, Module32, PAGE, STACK};
use crate::{ImportSymbol, Result};
struct Trap {
    owner: String,
    module: String,
    symbol: ImportSymbol,
    label: String,
}
struct FaultModule {
    base: u64,
    end: u64,
    name: String,
}

pub(super) struct Traps {
    memory: LowMapping,
    entries: Vec<Trap>,
    modules: std::sync::OnceLock<Vec<FaultModule>>,
}
impl Traps {
    pub(super) fn new(entries: Vec<(String, String, ImportSymbol)>) -> Result<Self> {
        if entries.len() > 8192 {
            return Err("diagnostic import budget exceeded".into());
        }
        let memory = LowMapping::new(entries.len().max(1) * PAGE, PAGE, 0)?;
        memory.protect(0, memory.len, 0)?;
        let entries = entries
            .into_iter()
            .map(|(owner, module, symbol)| {
                let label = format!("{owner}: {module}!{symbol:?}");
                Trap {
                    owner,
                    module,
                    symbol,
                    label,
                }
            })
            .collect();
        Ok(Self {
            memory,
            entries,
            modules: std::sync::OnceLock::new(),
        })
    }
    pub(super) fn bind_modules(&self, modules: &[Module32]) -> Result<()> {
        let snapshot = modules
            .iter()
            .map(|m| FaultModule {
                base: u64::from(m.base),
                end: u64::from(m.base) + m.size as u64,
                name: m.name.clone(),
            })
            .collect();
        self.modules
            .set(snapshot)
            .map_err(|_| "diagnostic module attribution already initialized".into())
    }
    pub(super) fn address(&self, owner: &str, module: &str, symbol: &ImportSymbol) -> Option<u32> {
        self.entries
            .iter()
            .position(|t| {
                t.owner == owner && t.module.eq_ignore_ascii_case(module) && &t.symbol == symbol
            })
            .map(|i| self.memory.base as u32 + (i * PAGE) as u32)
    }
    pub(super) fn at(&self, address: u64) -> Option<&str> {
        let start = self.memory.base as u64;
        let offset = address.checked_sub(start)?;
        if offset >= self.memory.len as u64 {
            return None;
        }
        self.entries
            .get(offset as usize / PAGE)
            .map(|t| t.label.as_str())
    }
}
unsafe extern "C" {
    fn write(fd: i32, bytes: *const std::ffi::c_void, count: usize) -> isize;
    fn _exit(status: i32) -> !;
}
fn out(bytes: &[u8]) {
    unsafe {
        write(2, bytes.as_ptr().cast(), bytes.len());
    }
}
fn hex(value: u64) {
    let mut bytes = *b"0x0000000000000000";
    for i in 0..16 {
        bytes[17 - i] = b"0123456789abcdef"[((value >> (i * 4)) & 15) as usize];
    }
    out(&bytes);
}
// Called only for the registered thread's compatibility-mode fault. Everything
// below is immutable owned metadata, checked guest-stack reads and write/_exit.
pub(super) fn fault(image: &LoadedImage32<'_>, exception: &[u8; 16], registers: &[u64; 21]) -> ! {
    let pc = registers[16];
    let address = u64::from_le_bytes(exception[8..16].try_into().unwrap());
    let label = image
        .cpu
        .diagnostic
        .as_ref()
        .and_then(|t| t.at(address).or_else(|| t.at(pc)));
    if let Some(label) = label {
        out(b"Alkyhol PE32 diagnostic stop: unimplemented import ");
        out(label.as_bytes());
    } else {
        out(b"Alkyhol PE32 diagnostic stop: unsupported guest fault");
    }
    out(b"; fault_pc=");
    hex(pc);
    out(b"; address=");
    hex(address);
    let mut source = pc;
    if label.is_some()
        && image
            .cpu
            .diagnostic
            .as_ref()
            .and_then(|t| t.at(pc))
            .is_some()
    {
        let sp = registers[7];
        let first = image.cpu.stack().base as u64 + PAGE as u64;
        if sp >= first
            && sp
                .checked_add(4)
                .is_some_and(|end| end <= first + STACK as u64)
        {
            source = u64::from(unsafe { (sp as *const u32).read_unaligned() });
            out(b"; return_pc=");
            hex(source);
            if sp
                .checked_add(16)
                .is_some_and(|end| end <= first + STACK as u64)
            {
                out(b"; entry_stack=");
                for i in 1..=3 {
                    if i > 1 {
                        out(b",");
                    }
                    hex(u64::from(unsafe {
                        ((sp as *const u32).add(i)).read_unaligned()
                    }));
                }
            }
        }
    }
    // The loader may be updating its registry on a different host thread.
    // Only the immutable mapping snapshot is safe on this signal path.
    if let Some(module) = image
        .cpu
        .diagnostic
        .as_ref()
        .and_then(|t| t.modules.get())
        .and_then(|modules| modules.iter().find(|m| source >= m.base && source < m.end))
    {
        out(b"; source_module=");
        out(module.name.as_bytes());
        out(b"+");
        hex(source - module.base);
    }
    out(b"\n");
    unsafe { _exit(78) }
}
