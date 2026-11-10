//! Native ARM CPU adapter for the existing Alkyhol Win32/D3D9 service runtime.
//! All original guest instructions use SharpRetro; no native x86 fallback.
use crate::{Result, compiler, memory::Memory};
use alkyhol_pe::ExecutionContext32;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
#[derive(Clone, Copy, Debug, Default)]
pub struct Stats {
    pub instructions: u64,
    pub services: u64,
    pub compiled: u64,
    pub invocations: u64,
}
pub fn stats() -> Stats {
    STATS.with(Cell::get)
}
struct Cached {
    bytes: Vec<u8>,
    instruction: Rc<compiler::Instruction>,
}
type FpuStates = BTreeMap<(u64, u32, u32), Rc<RefCell<crate::fpu::State>>>;
thread_local! {static STATS:Cell<Stats>=const{Cell::new(Stats{instructions:0,services:0,compiled:0,invocations:0})};static FPUS:RefCell<FpuStates>=const{RefCell::new(BTreeMap::new())};static CACHE:RefCell<BTreeMap<u32,Cached>>=const{RefCell::new(BTreeMap::new())};}
unsafe fn validate(context: *const (), address: u64, length: usize, rights: u8) -> bool {
    let context = unsafe { &*(context as *const ExecutionContext32<'_>) };
    context.permits(address, length, rights)
}
pub fn install() -> Result<()> {
    alkyhol_pe::install_executor32(execute)
}
fn execute(context: &ExecutionContext32<'_>, entry: u32, stack: u32) -> Result<u64> {
    STATS.with(|s| {
        let mut v = s.get();
        v.invocations += 1;
        s.set(v)
    });
    execute_inner(context, entry, stack)
        .map_err(|e| format!("{e}; native ARM totals {:?}", stats()))
}
fn execute_inner(context: &ExecutionContext32<'_>, entry: u32, stack: u32) -> Result<u64> {
    let mut memory = Box::new(unsafe {
        Memory::foreign(
            context as *const _ as *const (),
            context.memory_base(),
            validate,
        )
    });
    memory.fpu = FPUS.with(|fpus| {
        fpus.borrow_mut()
            .entry(context.thread_key())
            .or_default()
            .clone()
    });
    let mut state = [0u64; 91];
    state[4] = stack as u64;
    state[16] = 0x202;
    state[17] = entry as u64;
    state[22] = context.teb() as u64;
    state[88] = context.memory_base();
    state[90] = (&mut *memory as *mut Memory) as u64;
    let mut spill = [0u64; 4096];
    let mut instructions = 0u64;
    let mut services = 0u64;
    let progress = std::env::var_os("ALKYHOL_JIT32_TRACE_PROGRESS").is_some();
    for _ in 0..1_000_000_000u64 {
        let pc = state[17] as u32;
        if pc == context.return_gate() {
            return Ok(state[0] as u32 as u64 | ((state[4] as u32 as u64) << 32));
        }
        if let Some((id, _)) = context.gateway(pc) {
            let sp = state[4] as u32;
            let return_pc = u32::from_le_bytes(context.read(sp, 4, false)?.try_into().unwrap());
            if id == 89 || id == 92 {
                crate::fpu_ops::ci_math(&mut memory, id)?;
                state[4] = sp.checked_add(4).ok_or("numeric service stack overflow")? as u64;
                state[17] = return_pc as u64;
                services += 1;
                STATS.with(|s| {
                    let mut v = s.get();
                    v.services += 1;
                    s.set(v)
                });
                continue;
            }
            let (value, cleanup, exit) = context.dispatch(pc, sp)?;
            services += 1;
            STATS.with(|s| {
                let mut v = s.get();
                v.services += 1;
                s.set(v)
            });
            if exit {
                return Ok(value as u64);
            }
            state[0] = value as u64;
            state[4] = sp
                .checked_add(4 + cleanup as u32)
                .ok_or("service stack overflow")? as u64;
            state[17] = return_pc as u64;
            continue;
        }
        let cached = CACHE.with(|cache| {
            cache
                .borrow()
                .get(&pc)
                .filter(|c| context.code_equals(pc, &c.bytes))
                .map(|c| c.instruction.clone())
        });
        let instruction = if let Some(code) = cached {
            code
        } else {
            let mut bytes = vec![];
            for n in 0..15u32 {
                match context.read(pc.wrapping_add(n), 1, true) {
                    Ok(b) => bytes.push(b[0]),
                    Err(_) if !bytes.is_empty() => break,
                    Err(e) => return Err(e),
                }
            }
            let instruction = Rc::new(
                compiler::compile(pc, &bytes)
                    .map_err(|e| format!("{e}; {} bytes={bytes:02x?}", context.location(pc)))?,
            );
            STATS.with(|s| {
                let mut v = s.get();
                v.compiled += 1;
                s.set(v)
            });
            bytes.truncate(instruction.length);
            CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                if cache.len() >= 65536 {
                    cache.clear();
                }
                cache.insert(
                    pc,
                    Cached {
                        bytes,
                        instruction: instruction.clone(),
                    },
                );
            });
            instruction
        };
        let snapshot = state;
        unsafe {
            instruction.code.enter(&mut state, &mut spill);
        }
        if let Some(reason) = memory.cpu_fault.take() {
            return Err(format!("JIT32 CPU fault pc={pc:#x}: {reason}"));
        }
        if let Some(fault) = memory.fault.take() {
            state = snapshot;
            return Err(format!(
                "JIT32 guest fault pc={pc:#x} {fault:?} registers={:x?}",
                &state[..8]
            ));
        }
        instructions += 1;
        if progress && instructions.is_multiple_of(1_000_000) {
            eprintln!(
                "JIT32 progress: instructions={instructions} services={services} pc={pc:#x} {} registers={:x?}",
                context.location(pc),
                &state[..8]
            );
        }
        STATS.with(|s| {
            let mut v = s.get();
            v.instructions += 1;
            s.set(v)
        });
    }
    Err(format!(
        "native JIT instruction/service budget exhausted at {:#x}; instructions={instructions} services={services}",
        state[17]
    ))
}
