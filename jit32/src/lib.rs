//! Experimental checked PE32 -> native ARM64 runner; no Rosetta or interpreter fallback.
#![cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[cfg(test)]
mod atan2_tests;
#[cfg(test)]
mod bit_tests;
mod compiler;
mod fpu;
mod fpu_atan;
mod fpu_atan2;
mod fpu_math;
mod fpu_ops;
mod fpu_trig;
#[cfg(feature = "shared-runtime")]
pub mod shared;
mod sse;
mod sse_arithmetic;
#[cfg(test)]
mod sse_bits_tests;
mod sse_compare;
mod sse_estimate;
mod stack_bank;
#[cfg(test)]
mod string_tests;
#[cfg(feature = "shared-runtime")]
pub use alkyhol_pe as shared_pe;
pub mod memory;
mod native;
mod process;
mod system;
use alkyhol_pe::{Image, ImportSymbol};
use memory::{EXEC, Fault, Memory, READ, WRITE};
pub use process::{ImportGate, Module, ModuleInput, Phase, Process};
use std::{collections::BTreeMap, fs::File};
pub type Result<T> = std::result::Result<T, String>;
const RETURN: u32 = 0x7fff_ff00;
const GATE: u32 = 0x7fff_0000;
const STACK: u32 = 0x7f00_0000;
const STACK_SIZE: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Stop {
    Returned(u32),
    Import { pc: u32, gate: ImportGate },
    Fault { pc: u32, fault: Fault },
    Unsupported { pc: u32, reason: String },
    Budget { pc: u32 },
    CpuFault { pc: u32, reason: String },
}
#[derive(Debug, Clone, Copy)]
enum Service {
    GetLastError,
    SetLastError,
    GetStdHandle,
    WriteFile,
    Kernel(system::Call),
}
impl Service {
    fn args(self) -> u32 {
        match self {
            Self::GetLastError => 0,
            Self::SetLastError | Self::GetStdHandle => 1,
            Self::WriteFile => 5,
            Self::Kernel(call) => call.args(),
        }
    }
    fn resolve(module: &str, symbol: &ImportSymbol) -> Result<Self> {
        if !module.eq_ignore_ascii_case("kernel32.dll") {
            return Err(format!("unsupported DLL: {module}"));
        }
        match symbol {
            ImportSymbol::Name(n) => match n.as_str() {
                "GetLastError" => Ok(Self::GetLastError),
                "SetLastError" => Ok(Self::SetLastError),
                "GetStdHandle" => Ok(Self::GetStdHandle),
                "WriteFile" => Ok(Self::WriteFile),
                _ => system::Call::resolve(n)
                    .map(Self::Kernel)
                    .ok_or_else(|| format!("unsupported import: {module}!{n}")),
            },
            _ => Err(format!("unsupported ordinal: {module}!{symbol:?}")),
        }
    }
}

pub struct Runner {
    memory: Box<Memory>,
    state: [u64; 91],
    gates: BTreeMap<u32, Service>,
    external_gates: BTreeMap<u32, ImportGate>,
    teb: Option<u32>,
    system: system::State,
    stdout: alkyhol::kernel::Handle,
    blocks: BTreeMap<u32, compiler::Instruction>,
    pub instructions: u64,
    pub service_calls: u64,
}
impl Runner {
    /// Admit a checked no-TLS/no-CRT image with an explicit stdout stream. Import
    /// support is validated before execution. Original instruction bytes stay intact.
    pub fn load(bytes: &[u8], base: Option<u32>, stdout: File) -> Result<Self> {
        let image = Image::parse_i386(bytes)?;
        if image.static_tls32()?.is_some() {
            return Err("static TLS initialization is not implemented by JIT32".into());
        }
        if image.legacy_load_config32()?.is_some() {
            return Err("load-configuration startup is not implemented by JIT32".into());
        }
        let imports = image.imports()?;
        if imports.len() > 1024 {
            return Err("JIT import budget exceeded".into());
        }
        let mut gates = BTreeMap::new();
        for (i, import) in imports.iter().enumerate() {
            gates.insert(
                GATE + i as u32 * 16,
                Service::resolve(&import.module, &import.symbol)?,
            );
        }
        let base = base.unwrap_or(
            u32::try_from(image.base).map_err(|_| "PE32 preferred base exceeds 32 bits")?,
        );
        if base < 0x10000
            || base & 0xffff != 0
            || base as u64 + image.size as u64 > STACK as u64
            || image.size > 256 * 1024 * 1024
        {
            return Err("JIT image address/size budget exceeded".into());
        }
        let mut data = vec![0u8; image.size];
        data[..image.headers].copy_from_slice(&bytes[..image.headers]);
        for s in &image.sections {
            let n = s.raw_size.min(s.size);
            data[s.rva..s.rva + n].copy_from_slice(&bytes[s.raw..s.raw + n]);
        }
        let fixups = image.relocations()?;
        if base as u64 != image.base {
            if fixups.is_empty() {
                return Err("image has no relocations".into());
            }
            let delta = base.wrapping_sub(image.base as u32);
            for rva in fixups {
                let v =
                    u32::from_le_bytes(data[rva..rva + 4].try_into().unwrap()).wrapping_add(delta);
                data[rva..rva + 4].copy_from_slice(&v.to_le_bytes());
            }
        }
        for (i, import) in imports.iter().enumerate() {
            data[import.iat..import.iat + 4].copy_from_slice(&(GATE + i as u32 * 16).to_le_bytes());
        }
        let mut memory = Box::new(Memory::new()?);
        memory.map(base, image.headers, READ, &data[..image.headers])?;
        for s in &image.sections {
            let mut rights = 0;
            if s.rights & 0x4000_0000 != 0 {
                rights |= READ;
            }
            if s.rights & 0x8000_0000 != 0 {
                rights |= WRITE;
            }
            if s.rights & 0x2000_0000 != 0 {
                rights |= EXEC;
            }
            if s.size != 0 {
                memory.map(
                    base + s.rva as u32,
                    s.size,
                    rights,
                    &data[s.rva..s.rva + s.size],
                )?;
            }
        }
        memory.map(STACK, STACK_SIZE, READ | WRITE, &[])?;
        let mut state = [0; 91];
        state[4] = STACK as u64 + STACK_SIZE as u64 - 4;
        memory.write(state[4], &RETURN.to_le_bytes())?;
        state[16] = 0x202;
        state[17] = base as u64 + image.entry as u64;
        state[88] = memory.base();
        state[90] = (&mut *memory as *mut Memory) as u64;
        let stdout = alkyhol::kernel::kernel()
            .adopt_file(stdout, alkyhol::kernel::GENERIC_WRITE)
            .map_err(|e| format!("stdout: {e:?}"))?;
        Ok(Self {
            memory,
            state,
            gates,
            external_gates: BTreeMap::new(),
            teb: None,
            system: system::State::default(),
            stdout,
            blocks: BTreeMap::new(),
            instructions: 0,
            service_calls: 0,
        })
    }
    pub fn pc(&self) -> u32 {
        self.state[17] as u32
    }
    pub fn registers(&self) -> [u32; 8] {
        std::array::from_fn(|i| self.state[i] as u32)
    }
    pub fn flags(&self) -> u32 {
        self.state[16] as u32
    }
    pub fn compiled_blocks(&self) -> usize {
        self.blocks.len()
    }
    pub fn memory(&self) -> &Memory {
        &self.memory
    }
    fn service(&mut self, service: Service) -> Result<()> {
        let sp = self.state[4];
        let ret = self.memory.word(sp)?;
        let mut args = [0u32; 5];
        for i in 0..service.args() {
            args[i as usize] = self.memory.word(sp + 4 + i as u64 * 4)?;
        }
        let next_sp = sp
            .checked_add(4 + service.args() as u64 * 4)
            .filter(|v| *v <= u32::MAX as u64)
            .ok_or("stdcall stack overflow")?;
        if let Some(teb) = self.teb {
            alkyhol::kernel::set_last_error(self.memory.word(teb as u64 + 0x34)?);
        }
        let result = match service {
            Service::Kernel(call) => {
                self.system
                    .dispatch(call, &args, &mut self.memory, self.teb)?
            }
            Service::GetLastError => {
                if let Some(teb) = self.teb {
                    alkyhol::kernel::set_last_error(self.memory.word(teb as u64 + 0x34)?);
                }
                alkyhol::kernel::get_last_error()
            }
            Service::SetLastError => {
                alkyhol::kernel::set_last_error(args[0]);
                self.state[0] as u32
            }
            Service::GetStdHandle => {
                if args[0] != (-11i32) as u32 {
                    return Err("JIT runner currently admits inherited stdout only".into());
                }
                0x100 // guest token, not a host handle or address
            }
            Service::WriteFile => {
                if args[4] != 0 {
                    return Err("overlapped WriteFile is unsupported".into());
                }
                if args[2] > 16 * 1024 * 1024 {
                    return Err("WriteFile transfer budget exceeded".into());
                }
                self.memory
                    .check(args[3] as u64, 4, WRITE)
                    .map_err(|f| format!("output fault: {f:?}"))?;
                let data = if args[2] == 0 {
                    vec![]
                } else {
                    self.memory
                        .read(args[1] as u64, args[2] as usize, READ)?
                        .to_vec()
                };
                let mut count = 0;
                // Real Alkyhol kernel I/O, with owned host bytes/local output only.
                let handle = if args[0] == 0x100 { self.stdout } else { 0 };
                let success = unsafe {
                    alkyhol::ffi::WriteFile(
                        handle,
                        data.as_ptr().cast(),
                        args[2],
                        &mut count,
                        std::ptr::null_mut(),
                    )
                };
                self.memory.write(args[3] as u64, &count.to_le_bytes())?;
                success as u32
            }
        };
        if let Some(teb) = self.teb {
            self.memory.write(
                teb as u64 + 0x34,
                &alkyhol::kernel::get_last_error().to_le_bytes(),
            )?;
        }
        self.state[0] = result as u64;
        self.state[4] = next_sp;
        self.state[17] = ret as u64;
        self.service_calls += 1;
        Ok(())
    }
    /// Bounded cumulative execution. Budgets count both guest instructions and
    /// import transitions. Faults retain the pre-instruction register snapshot.
    pub fn run(&mut self, budget: u64) -> Stop {
        let mut spill = [0; 4096];
        for _ in 0..budget {
            let pc = self.pc();
            if pc == RETURN {
                return Stop::Returned(self.state[0] as u32);
            }
            if let Some(gate) = self.external_gates.get(&pc) {
                return Stop::Import {
                    pc,
                    gate: gate.clone(),
                };
            }
            if let Some(&service) = self.gates.get(&pc) {
                if let Err(reason) = self.service(service) {
                    return Stop::Unsupported { pc, reason };
                }
                continue;
            }
            if let Err(fault) = self.memory.check(pc as u64, 1, EXEC) {
                return Stop::Fault { pc, fault };
            }
            if !self.blocks.contains_key(&pc) {
                if self.blocks.len() >= 4096 {
                    return Stop::Unsupported {
                        pc,
                        reason: "compiled block budget exceeded".into(),
                    };
                }
                let mut bytes = Vec::new();
                for off in 0..15u64 {
                    match self.memory.read(pc as u64 + off, 1, EXEC) {
                        Ok(b) => bytes.push(b[0]),
                        Err(_) => break,
                    }
                }
                match compiler::compile(pc, &bytes) {
                    Ok(block) => {
                        self.blocks.insert(pc, block);
                    }
                    Err(reason) => return Stop::Unsupported { pc, reason },
                }
            }
            let snapshot = self.state;
            let block = &self.blocks[&pc];
            // Fetch-check the entire decoded instruction, including cache hits.
            if let Err(fault) = self.memory.check(pc as u64, block.length, EXEC) {
                return Stop::Fault { pc, fault };
            }
            unsafe {
                block.code.enter(&mut self.state, &mut spill);
            }
            if let Some(reason) = self.memory.cpu_fault.take() {
                self.state = snapshot;
                return Stop::CpuFault {
                    pc,
                    reason: reason.into(),
                };
            }
            if let Some(fault) = self.memory.fault.take() {
                self.state = snapshot;
                return Stop::Fault { pc, fault };
            }
            self.instructions += 1;
        }
        if self.pc() == RETURN {
            Stop::Returned(self.state[0] as u32)
        } else {
            Stop::Budget { pc: self.pc() }
        }
    }
}
impl Drop for Runner {
    fn drop(&mut self) {
        let _ = alkyhol::kernel::kernel().close_handle(self.stdout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn cpu(code: &[u8]) -> Runner {
        let mut memory = Box::new(Memory::new().unwrap());
        memory.map(0x10000, 4096, READ | EXEC, code).unwrap();
        memory
            .map(STACK, 4096, READ | WRITE, &[0x5a; 4096])
            .unwrap();
        let mut state = [0; 91];
        state[4] = STACK as u64 + 4092;
        memory.write(state[4], &RETURN.to_le_bytes()).unwrap();
        state[16] = 0x202;
        state[17] = 0x10000;
        state[88] = memory.base();
        state[90] = (&mut *memory as *mut Memory) as u64;
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "alkyhol-jit32-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let file = File::options()
            .create_new(true)
            .write(true)
            .open(&path)
            .unwrap();
        std::fs::remove_file(path).unwrap();
        let stdout = alkyhol::kernel::kernel()
            .adopt_file(file, alkyhol::kernel::GENERIC_WRITE)
            .unwrap();
        Runner {
            memory,
            state,
            gates: BTreeMap::new(),
            external_gates: BTreeMap::new(),
            teb: None,
            system: system::State::default(),
            stdout,
            blocks: BTreeMap::new(),
            instructions: 0,
            service_calls: 0,
        }
    }
    #[test]
    fn memory_shifts_reload_the_committed_value_without_an_extra_access() {
        for group in [4u8, 5, 7] {
            for input in [0u32, 1, 0x7fffffff, 0x80000000, u32::MAX] {
                for encoded in [0u8, 1, 3, 7, 31, 32, 33, 63] {
                    let mut r = cpu(&[0xc1, (group << 3) | 5, 0, 0, 2, 0, encoded, 0xc3]);
                    r.memory
                        .map(0x20000, 4, READ | WRITE, &input.to_le_bytes())
                        .unwrap();
                    r.state[16] = 0xe57;
                    let count = encoded as u32 & 31;
                    let expected = match group {
                        4 => input.wrapping_shl(count),
                        5 => input >> count,
                        _ => ((input as i32) >> count) as u32,
                    };
                    assert_eq!(r.run(4), Stop::Returned(0));
                    assert_eq!(
                        r.memory.word(0x20000).unwrap(),
                        expected,
                        "group {group} input {input:x} count {encoded}"
                    );
                    if count == 0 {
                        assert_eq!(r.flags(), 0xe57);
                    } else {
                        let carry = if group == 4 {
                            input >> (32 - count) & 1
                        } else {
                            input >> (count - 1) & 1
                        };
                        let flags = carry
                            | if expected == 0 { 0x40 } else { 0 }
                            | if expected >> 31 != 0 { 0x80 } else { 0 }
                            | if (expected as u8).count_ones().is_multiple_of(2) {
                                4
                            } else {
                                0
                            };
                        assert_eq!(r.flags() & 0xc5, flags);
                        assert_eq!(r.flags() & 0x600, 0x600);
                        if count == 1 {
                            let overflow = match group {
                                4 => (expected >> 31) ^ carry,
                                5 => input >> 31,
                                _ => 0,
                            };
                            assert_eq!((r.flags() >> 11) & 1, overflow);
                        }
                    }
                }
            }
        }
        let mut r = cpu(&[0xc1, 0x7d, 0xfc, 3, 0xc3]);
        r.state[5] = 0x20004;
        r.memory
            .map(0x20000, 4, READ | WRITE, &0x80000000u32.to_le_bytes())
            .unwrap();
        assert_eq!(r.run(4), Stop::Returned(0));
        assert_eq!(r.memory.word(0x20000).unwrap(), 0xf0000000);
        let mut r = cpu(&[0xc1, 0x3d, 0, 0, 2, 0, 3]);
        r.memory
            .map(0x20000, 4, READ, &0x80000000u32.to_le_bytes())
            .unwrap();
        let before = r.state;
        assert!(matches!(r.run(1), Stop::Fault { pc: 0x10000, .. }));
        assert_eq!(r.state, before);
        assert_eq!(r.memory.word(0x20000).unwrap(), 0x80000000);
    }
    #[test]
    fn reached_sincos_float_stores_match_physical_x86_observations() {
        for (input, sine, cosine, status) in [
            (0u32, 0u32, 0x3f800000u32, 0u16),
            (0x80000000, 0x80000000, 0x3f800000, 0),
            (0x3f800000, 0x3f576aa4, 0x3f0a5140, 0x20),
            (0xbf800000, 0xbf576aa4, 0x3f0a5140, 0x20),
            (0x40490fdb, 0xb3bbbd2e, 0xbf800000, 0x20),
            (0x3fc90fdb, 0x3f800000, 0xb33bbd2e, 0x220),
            (0x40c90fdb, 0x343bbd2e, 0x3f800000, 0x20),
            (0x3e000000, 0x3dff5577, 0x3f7e00ab, 0x20),
            (0xc0c90fdb, 0xb43bbd2e, 0x3f800000, 0x20),
            (0x44800000, 0xbe225693, 0x3f7cc335, 0x20),
            (0x1e3ce508, 0x1e3ce508, 0x3f800000, 0x20),
        ] {
            let mut r = cpu(&[
                0xd9, 0x05, 0, 0, 2, 0, 0xd9, 0xfb, 0xd9, 0x1d, 4, 0, 2, 0, 0xd9, 0x1d, 8, 0, 2, 0,
                0xc3,
            ]);
            r.memory
                .map(0x20000, 12, READ | WRITE, &input.to_le_bytes())
                .unwrap();
            assert_eq!(r.run(10), Stop::Returned(0), "input {input:x}");
            assert_eq!(r.memory.word(0x20004).unwrap(), cosine, "cos {input:x}");
            assert_eq!(r.memory.word(0x20008).unwrap(), sine, "sin {input:x}");
            assert_eq!(r.memory.fpu.borrow().status, status, "status {input:x}");
            assert_eq!(r.memory.fpu.borrow().tag, 0xffff);
        }
    }
    #[test]
    fn legacy_vector_moves_and_logic_preserve_bits_and_check_alignment() {
        let mut r = cpu(&[
            0x0f, 0x28, 0x05, 0, 0, 2, 0, 0x0f, 0x28, 0xc8, 0x66, 0x0f, 0xef, 0xc8, 0x0f, 0x11,
            0x05, 0x31, 0, 2, 0, 0x0f, 0x29, 0x0d, 0x20, 0, 2, 0, 0xc3,
        ]);
        let mut bytes = vec![0xcc; 96];
        for (i, b) in bytes[..16].iter_mut().enumerate() {
            *b = (i * 17) as u8;
        }
        r.memory.map(0x20000, 96, READ | WRITE, &bytes).unwrap();
        r.memory.fpu.borrow_mut().mxcsr = 0x5f80;
        assert_eq!(r.run(10), Stop::Returned(0));
        assert_eq!(r.memory.read(0x20031, 16, READ).unwrap(), &bytes[..16]);
        assert_eq!(r.memory.read(0x20020, 16, READ).unwrap(), [0; 16]);
        assert_eq!(
            [r.state[24], r.state[25]],
            [
                u64::from_le_bytes(bytes[..8].try_into().unwrap()),
                u64::from_le_bytes(bytes[8..16].try_into().unwrap())
            ]
        );
        assert_eq!([r.state[26], r.state[27]], [0, 0]);
        assert_eq!(r.memory.fpu.borrow().mxcsr, 0x5f80);
        let mut r = cpu(&[0x0f, 0x28, 0x05, 1, 0, 2, 0]);
        r.memory.map(0x20000, 32, READ, &[0x5a; 32]).unwrap();
        let before = r.state;
        assert!(matches!(r.run(1), Stop::CpuFault { pc: 0x10000, .. }));
        assert_eq!(r.state, before);
        let mut r = cpu(&[0x0f, 0x11, 0x05, 0, 0, 2, 0]);
        r.memory.map(0x20000, 8, READ | WRITE, &[0x5a; 8]).unwrap();
        let before = r.state;
        assert!(matches!(r.run(1), Stop::Fault { pc: 0x10000, .. }));
        assert_eq!(r.state, before);
        assert_eq!(r.memory.read(0x20000, 8, READ).unwrap(), [0x5a; 8]);
        // MOVSS register merge; MOVSS memory load clears the upper 96 bits.
        let mut r = cpu(&[
            0xf3, 0x0f, 0x10, 0xc1, 0xf3, 0x0f, 0x10, 0x05, 0, 0, 2, 0, 0xc3,
        ]);
        r.memory
            .map(0x20000, 4, READ, &0x7fa12345u32.to_le_bytes())
            .unwrap();
        r.state[24] = 0xaabbccdd11223344;
        r.state[25] = 0xfeedface12345678;
        r.state[26] = 0x5566778887654321;
        r.state[27] = u64::MAX;
        assert!(matches!(r.run(1), Stop::Budget { .. }));
        assert_eq!(r.state[24], 0xaabbccdd87654321);
        assert_eq!(r.state[25], 0xfeedface12345678);
        assert!(matches!(r.run(3), Stop::Returned(_)));
        assert_eq!(r.state[24], 0x7fa12345);
        assert_eq!(r.state[25], 0);
        assert_eq!(r.memory.fpu.borrow().mxcsr, 0x1f80);
    }
    #[test]
    fn register_banks_preserve_word_upper_halves_and_skip_saved_stack_pointer() {
        for word in [false, true] {
            let mut r = cpu(if word {
                &[0x66, 0x60, 0x66, 0x61, 0xc3]
            } else {
                &[0x60, 0x61, 0xc3]
            });
            let sp = r.state[4];
            for i in [0, 1, 2, 3, 5, 6, 7] {
                r.state[i] = 0xa5000100 + i as u64;
            }
            let before = r.state;
            let length = if word { 16 } else { 32 };
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            assert_eq!(r.state[4], sp - length);
            let frame = r.memory.read(sp - length, length as usize, READ).unwrap();
            let width = if word { 2 } else { 4 };
            for (i, reg) in [7, 6, 5, 4, 3, 2, 1, 0].into_iter().enumerate() {
                assert_eq!(
                    &frame[i * width..(i + 1) * width],
                    &(before[reg] as u32).to_le_bytes()[..width]
                );
            }
            for i in [0, 1, 2, 3, 5, 6, 7] {
                r.state[i] = 0x77000200 + i as u64;
            }
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            assert_eq!(r.state[4], sp);
            for i in [0, 1, 2, 3, 5, 6, 7] {
                assert_eq!(
                    r.state[i],
                    if word {
                        0x77000000 | (before[i] & 0xffff)
                    } else {
                        before[i]
                    }
                );
            }
            assert!(matches!(r.run(1), Stop::Returned(_)));
        }
        let mut r = cpu(&[0x60]);
        r.state[4] = STACK as u64 + 16;
        let before = r.state;
        assert!(matches!(r.run(1), Stop::CpuFault { .. }));
        assert_eq!(r.state, before);
    }
    #[test]
    fn reached_multiply_word_flags_and_real_timestamp_contracts() {
        for (op, a, b, lo, hi, overflow) in [
            (0xe1u8, u32::MAX, 2, 0xfffffffe, 1, true),
            (0xe9, 0x80000000, 2, 0, 0xffffffff, true),
            (0xe9, (-17i32) as u32, 19, (-323i32) as u32, u32::MAX, false),
        ] {
            let mut r = cpu(&[0xf7, op, 0xc3]);
            r.state[0] = a as u64;
            r.state[1] = b as u64;
            assert_eq!(r.run(5), Stop::Returned(lo));
            assert_eq!(r.registers()[2], hi);
            assert_eq!(r.flags() & 0x801, if overflow { 0x801 } else { 0 });
        }
        let mut r = cpu(&[0x66, 0x9c, 0x66, 0x9d, 0xc3]);
        r.state[16] = 0x200ed7;
        let sp = r.state[4];
        assert_eq!(r.run(1), Stop::Budget { pc: 0x10002 });
        assert_eq!(r.state[4], sp - 2);
        assert_eq!(
            r.memory.read(sp - 2, 2, READ).unwrap(),
            0xed7u16.to_le_bytes()
        );
        assert_eq!(r.memory.word(sp).unwrap(), RETURN);
        assert_eq!(r.run(1), Stop::Budget { pc: 0x10004 });
        assert_eq!(r.state[4], sp);
        assert_eq!(r.flags(), 0x200ed7);
        assert_eq!(r.run(1), Stop::Returned(0));
        let mut r = cpu(&[0x0f, 0x31, 0xc3]);
        r.state[16] = 0xad7;
        r.state[1] = 0x1234;
        r.state[3] = 0x5678;
        let before = unsafe { crate::fpu::timestamp() };
        let stop = r.run(3);
        let after = unsafe { crate::fpu::timestamp() };
        assert!(matches!(stop, Stop::Returned(_)));
        let measured = (r.registers()[2] as u64) << 32 | r.registers()[0] as u64;
        assert!((before..=after).contains(&measured));
        assert_eq!(r.flags(), 0xad7);
        assert_eq!(r.registers()[1], 0x1234);
        assert_eq!(r.registers()[3], 0x5678);
    }
    #[test]
    fn x87_compare_pop_and_eflags_paths_preserve_their_separate_flags() {
        for (op, expected, top) in [(0xd8u8, 0x3100, 6), (0xdc, 0x3900, 7)] {
            // FLD1; FLDZ; FCOM[P] ST1; FNSTSW AX
            let code = [
                0xd9,
                0xe8,
                0xd9,
                0xee,
                0xd8,
                if op == 0xd8 { 0xd1 } else { 0xd9 },
                0xdf,
                0xe0,
                0xc3,
            ];
            let mut r = cpu(&code);
            assert_eq!(r.run(10), Stop::Returned(expected));
            assert_eq!(r.memory.fpu.borrow().top(), top);
        }
        let mut r = cpu(&[0xd9, 0xe8, 0xd9, 0xee, 0xdb, 0xf1, 0xc3]); // FCOMI ST0,ST1
        r.state[16] = 0xed7;
        assert_eq!(r.run(10), Stop::Returned(0));
        assert_eq!(r.flags() & 0x8d5, 1);
        assert_ne!(r.flags() & 0x400, 0);
        assert_eq!(r.memory.fpu.borrow().status & 0x4500, 0);
        let mut r = cpu(&[0x9e, 0x9f, 0xc3]);
        r.state[0] = 0xd500;
        r.state[16] = 0xa02;
        assert_eq!(r.run(5), Stop::Returned(0xd700));
        assert_eq!(r.flags() & 0x8d5, 0x8d5);
    }
    #[test]
    fn x87_reduced_precision_avoids_both_double_rounding_directions() {
        for (sig, sign) in [(0x8000000000000400u64, 0u16), (0x8000000000000c00, 0x8000)] {
            let mut input = sig.to_le_bytes().to_vec();
            input.extend(0x3fffu16.to_le_bytes());
            input.resize(16, 0);
            input.extend(0x8000000000000000u64.to_le_bytes());
            input.extend((0x3f9bu16 | sign).to_le_bytes());
            input.resize(48, 0);
            // FLD tbyte a; FLD tbyte b; FADDP; FSTP tbyte result
            let mut r = cpu(&[
                0xdb, 0x2d, 0, 0, 2, 0, 0xdb, 0x2d, 0x10, 0, 2, 0, 0xde, 0xc1, 0xdb, 0x3d, 0x20, 0,
                2, 0, 0xc3,
            ]);
            r.memory.map(0x20000, 48, READ | WRITE, &input).unwrap();
            r.memory.fpu.borrow_mut().control = 0x27f;
            assert_eq!(r.run(10), Stop::Returned(0));
            assert_eq!(
                r.memory.read(0x20020, 10, READ).unwrap(),
                [0, 8, 0, 0, 0, 0, 0, 0x80, 0xff, 0x3f]
            );
            assert_eq!(r.memory.fpu.borrow().status, 0x20);
        }
        let mut r = cpu(&[0x9b, 0xc3]);
        r.memory.fpu.borrow_mut().status = 0x21;
        assert_eq!(r.run(3), Stop::Returned(0));
        assert_eq!(r.memory.fpu.borrow().status, 0x21);
        let mut r = cpu(&[0x9b]);
        r.memory.fpu.borrow_mut().status = 1;
        r.memory.fpu.borrow_mut().control &= !1;
        assert!(matches!(r.run(1), Stop::CpuFault { pc: 0x10000, .. }));
        let mut r = cpu(&[0x9d]);
        r.memory.write(r.state[4], &0x302u32.to_le_bytes()).unwrap();
        let before = r.state;
        assert!(matches!(r.run(1), Stop::CpuFault { pc: 0x10000, .. }));
        assert_eq!(r.state, before);
    }
    #[test]
    fn x87_transfers_precision_and_integer_rounding_match_x86_observations() {
        // Independent original x86 assembly lives in tests/x87_reference.c.
        // The same finite rows were executed under actual Rosetta on this M4.
        for (control, significand, status) in [
            (0x27fu16, 0x8000000000000000u64, 0x20),
            (0x37f, 0x8000000000000400, 0),
        ] {
            // FILD qword [0x20000]; FLD1; FADDP ST1; FSTP tbyte [0x20010]
            let mut r = cpu(&[
                0xdf, 0x2d, 0, 0, 2, 0, 0xd9, 0xe8, 0xde, 0xc1, 0xdb, 0x3d, 0x10, 0, 2, 0, 0xc3,
            ]);
            r.memory
                .map(
                    0x20000,
                    32,
                    READ | WRITE,
                    &9007199254740992i64.to_le_bytes(),
                )
                .unwrap();
            r.memory.fpu.borrow_mut().control = control;
            assert_eq!(r.run(10), Stop::Returned(0));
            let mut expected = significand.to_le_bytes().to_vec();
            expected.extend(0x4034u16.to_le_bytes());
            assert_eq!(r.memory.read(0x20010, 10, READ).unwrap(), expected);
            assert_eq!(r.memory.fpu.borrow().status, status);
            assert_eq!(r.memory.fpu.borrow().tag, 0xffff);
        }
        for (mode, result, status) in [
            (0, -2i64, 0x220),
            (1, -2, 0x220),
            (2, -1, 0x20),
            (3, -1, 0x20),
        ] {
            let mut r = cpu(&[0xdd, 0x05, 0, 0, 2, 0, 0xdf, 0x3d, 8, 0, 2, 0, 0xc3]); // FLD m64; FISTP m64
            r.memory
                .map(
                    0x20000,
                    16,
                    READ | WRITE,
                    &(-1.5f64).to_bits().to_le_bytes(),
                )
                .unwrap();
            r.memory.fpu.borrow_mut().control = 0x37f | (mode << 10);
            assert_eq!(r.run(10), Stop::Returned(0));
            assert_eq!(
                r.memory.read(0x20008, 8, READ).unwrap(),
                result.to_le_bytes()
            );
            assert_eq!(r.memory.fpu.borrow().status, status);
        }
        let mut r = cpu(&[0xd9, 0xe8, 0xdb, 0x3d, 0, 0, 0, 0]); // FLD1; denied FSTP m80
        assert!(matches!(r.run(5), Stop::Fault { pc: 0x10002, .. }));
        assert_eq!(r.memory.fpu.borrow().top(), 7);
        assert_eq!(r.memory.fpu.borrow().tag, 0x3fff);
        assert_eq!(r.memory.fpu.borrow().registers[7], 0x3fff8000000000000000);
        let mut r = cpu(&[0xd9, 0x05, 0, 0, 0, 0]);
        assert!(matches!(r.run(2), Stop::Fault { pc: 0x10000, .. }));
        assert_eq!(r.memory.fpu.borrow().tag, 0xffff);
    }
    #[test]
    fn flag_stack_cpuid_xchg_and_x87_environment_are_real_state() {
        let mut r = cpu(&[
            0x9c, 0x58, 0x35, 0, 0, 0x20, 0, 0x50, 0x9d, 0x9c, 0x58, 0xc3,
        ]);
        r.state[16] = 0xad7;
        let before = r.memory.word(STACK as u64 + 4092).unwrap();
        assert_eq!(r.run(20), Stop::Returned(0x200ad7));
        assert_eq!(r.flags(), 0x200ad7);
        assert_eq!(r.memory.word(STACK as u64 + 4092).unwrap(), before);
        let mut r = cpu(&[0x0f, 0xa2, 0xc3]);
        assert_eq!(r.run(3), Stop::Returned(1));
        assert_eq!(r.registers()[3], 0x756e6547);
        assert_eq!(r.registers()[1], 0x6c65746e);
        assert_eq!(r.registers()[2], 0x49656e69);
        let mut r = cpu(&[0x87, 0x05, 0, 0, 2, 0, 0x99, 0xc3]);
        r.memory
            .map(0x20000, 4, READ | WRITE, &0x80000000u32.to_le_bytes())
            .unwrap();
        r.state[0] = 0x12345678;
        assert_eq!(r.run(4), Stop::Returned(0x80000000));
        assert_eq!(r.memory.word(0x20000).unwrap(), 0x12345678);
        assert_eq!(r.registers()[2], u32::MAX);
        // FLDCW [0x20000]; FNCLEX; FNSTCW [0x20002]; FNSTSW AX
        let mut r = cpu(&[
            0xd9, 0x2d, 0, 0, 2, 0, 0xdb, 0xe2, 0xd9, 0x3d, 2, 0, 2, 0, 0xdf, 0xe0, 0xc3,
        ]);
        r.memory
            .map(0x20000, 4, READ | WRITE, &[0x34, 0x12, 0, 0])
            .unwrap();
        r.memory.fpu.borrow_mut().status = 0x85ff;
        assert_eq!(r.run(10), Stop::Returned(0x500));
        assert_eq!(&r.memory.read(0x20002, 2, READ).unwrap(), &[0x74, 0x12]);
        assert_eq!(r.memory.fpu.borrow().control, 0x1274);
        assert_eq!(r.memory.fpu.borrow().status, 0x500);
        let mut r = cpu(&[0xdb, 0xe3, 0xc3]);
        r.memory.fpu.borrow_mut().mxcsr = 0x5f80;
        r.memory.fpu.borrow_mut().control = 0x1274;
        assert_eq!(r.run(3), Stop::Returned(0));
        assert_eq!(r.memory.fpu.borrow().control, 0x37f);
        assert_eq!(r.memory.fpu.borrow().tag, 0xffff);
        assert_eq!(r.memory.fpu.borrow().mxcsr, 0x5f80);
    }
    #[test]
    fn repeated_strings_commit_exactly_completed_iterations() {
        let mut r = cpu(&[0xf3, 0xa5, 0xc3]); // REP MOVSD
        r.memory
            .map(0x20000, 8, READ, &[1, 2, 3, 4, 5, 6, 7, 8])
            .unwrap();
        r.memory.map(0x30000, 4, READ | WRITE, &[]).unwrap();
        r.state[1] = 2;
        r.state[6] = 0x20000;
        r.state[7] = 0x30000;
        let flags = r.flags();
        assert!(matches!(r.run(5), Stop::Fault { pc: 0x10000, .. }));
        assert_eq!(r.registers()[1], 1);
        assert_eq!(r.registers()[6], 0x20004);
        assert_eq!(r.registers()[7], 0x30004);
        assert_eq!(r.memory.word(0x30000).unwrap(), 0x04030201);
        assert_eq!(r.flags(), flags);
        let mut r = cpu(&[0xf3, 0xa4, 0xc3]);
        r.state[1] = 0;
        r.state[6] = u32::MAX as u64;
        r.state[7] = 0;
        assert_eq!(r.run(3), Stop::Returned(0));
        assert_eq!(r.registers()[6], u32::MAX);
        let mut r = cpu(&[0xf2, 0xae, 0xc3]); // REPNE SCASB, stop on first equal
        r.memory.map(0x20000, 4, READ, &[1, 2, 0x7f, 4]).unwrap();
        r.state[0] = 0x7f;
        r.state[1] = 4;
        r.state[7] = 0x20000;
        assert_eq!(r.run(10), Stop::Returned(0x7f));
        assert_eq!(r.registers()[1], 1);
        assert_eq!(r.registers()[7], 0x20003);
        assert_ne!(r.flags() & 0x40, 0);
        let mut r = cpu(&[0xfd, 0xf3, 0xab, 0xfc, 0xc3]); // STD; REP STOSD; CLD
        r.memory.map(0x20000, 8, READ | WRITE, &[]).unwrap();
        r.state[0] = 0x12345678;
        r.state[1] = 2;
        r.state[7] = 0x20004;
        assert_eq!(r.run(10), Stop::Returned(0x12345678));
        assert_eq!(r.memory.word(0x20000).unwrap(), 0x12345678);
        assert_eq!(r.memory.word(0x20004).unwrap(), 0x12345678);
        assert_eq!(r.registers()[7], 0x1fffc);
        assert_eq!(r.flags() & 0x400, 0);
    }
    #[test]
    fn integer_division_results_and_precise_de_faults() {
        for signed in [false, true] {
            for (n, d) in if signed {
                vec![
                    (-12345i64 as u64, -97i32 as u32),
                    (i64::from(i32::MIN) as u64, 1),
                    (987654321, 123),
                ]
            } else {
                vec![(0xffffffff, 1), (0x1ffffffff, 3), (0xabcdef1234, 0x1000)]
            } {
                let mut r = cpu(&[0xf7, if signed { 0xf9 } else { 0xf1 }, 0xc3]); // (I)DIV ECX
                r.state[0] = n as u32 as u64;
                r.state[2] = n >> 32;
                r.state[1] = d as u64;
                let q = if signed {
                    ((n as i64) / (d as i32 as i64)) as u32
                } else {
                    (n / d as u64) as u32
                };
                let rem = if signed {
                    ((n as i64) % (d as i32 as i64)) as u32
                } else {
                    (n % d as u64) as u32
                };
                assert_eq!(r.run(2), Stop::Returned(q));
                assert_eq!(r.registers()[2], rem);
            }
        }
        for (op, n, d) in [
            (0xf1, 123, 0),
            (0xf1, 0x100000000, 1),
            (0xf9, i64::from(i32::MIN) as u64, u32::MAX),
            (0xf9, i64::MIN as u64, u32::MAX),
        ] {
            let mut r = cpu(&[0xf7, op]);
            r.state[0] = n as u32 as u64;
            r.state[2] = n >> 32;
            r.state[1] = d as u64;
            let before = r.state;
            assert!(matches!(r.run(1), Stop::CpuFault { pc: 0x10000, .. }));
            assert_eq!(r.state, before);
            assert_eq!(r.instructions, 0);
        }
        let mut r = cpu(&[0xf7, 0x35, 0, 0, 0, 0]); // DIV [0], fault before divide
        assert!(matches!(r.run(1), Stop::Fault { pc: 0x10000, .. }));
    }
    #[test]
    #[ignore = "requires independent actual x86 observations from tools/check_jit32_immediates.py"]
    fn actual_x86_immediate_observations() {
        let data = std::fs::read(std::env::var_os("ALKYHOL_INTEGER_IMMEDIATE_REFERENCE").unwrap())
            .unwrap();
        assert_eq!(data.len(), 960 * 28);
        for (i, row) in data.chunks_exact(28).enumerate() {
            let w: Vec<u32> = row
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            let [width, op, immediate, input, result, flags, carry] = w[..] else {
                unreachable!()
            };
            // Full-width group immediates, accumulator forms, and sign-extended
            // imm8 forms must agree, over both a GPR and checked memory.
            for form in 0..4 {
                if form == 3 && (width == 8 || immediate as i32 != immediate as i8 as i32) {
                    continue;
                }
                let mut bytes = Vec::new();
                if width == 16 {
                    bytes.push(0x66);
                }
                let group = [7, 5, 0, 2, 3][op as usize];
                if form == 0 {
                    bytes.push([0x3c, 0x2c, 0x04, 0x14, 0x1c][op as usize] + u8::from(width != 8));
                } else {
                    bytes.push(if form == 3 {
                        0x83
                    } else if width == 8 {
                        0x80
                    } else {
                        0x81
                    });
                    bytes.push((if form == 2 { 0 } else { 0xc0 }) | (group << 3));
                }
                let n = if form == 3 { 1 } else { width as usize / 8 };
                bytes.extend_from_slice(&immediate.to_le_bytes()[..n]);
                let mut r = cpu(&bytes);
                r.state[0] = if form == 2 {
                    STACK as u64
                } else {
                    input as u64
                };
                r.state[16] = 0x202 | carry as u64;
                r.memory.write(STACK as u64, &input.to_le_bytes()).unwrap();
                assert!(
                    matches!(r.run(1), Stop::Budget { .. }),
                    "row {i} form {form}"
                );
                let actual = if form == 2 {
                    u32::from_le_bytes(
                        r.memory
                            .read(STACK as u64, 4, READ)
                            .unwrap()
                            .try_into()
                            .unwrap(),
                    )
                } else {
                    r.state[0] as u32
                };
                assert_eq!(actual, result, "row {i} form {form}, {w:x?}");
                assert_eq!(
                    r.flags() & 0x8d5,
                    flags & 0x8d5,
                    "row {i} form {form}, {w:x?}"
                );
            }
        }
    }
    #[test]
    fn bit_scans_cover_operand_widths_aliases_and_checked_memory() {
        for width in [16, 32] {
            for reverse in [false, true] {
                for memory in [false, true] {
                    for bit in 0..width {
                        let input = (1u32 << bit) | if reverse { 1 } else { 1u32 << (width - 1) };
                        let mut bytes = if width == 16 { vec![0x66] } else { vec![] };
                        bytes.extend([
                            0x0f,
                            if reverse { 0xbd } else { 0xbc },
                            if memory { 0x01 } else { 0xc1 },
                        ]);
                        let mut r = cpu(&bytes);
                        r.state[0] = 0xa5a55a5a;
                        r.state[1] = if memory { STACK as u64 } else { input as u64 };
                        r.memory.write(STACK as u64, &input.to_le_bytes()).unwrap();
                        r.state[16] = 0xad7;
                        assert!(matches!(r.run(1), Stop::Budget { .. }));
                        let answer = bit;
                        assert_eq!(
                            r.state[0],
                            if width == 16 {
                                0xa5a50000 | answer as u64
                            } else {
                                answer as u64
                            }
                        );
                        assert_eq!(r.flags() & 0x40, 0);
                        // Other arithmetic flags and zero-source destination are
                        // architecturally undefined: do not assert them as x86 facts.
                    }
                    let mut bytes = if width == 16 { vec![0x66] } else { vec![] };
                    bytes.extend([
                        0x0f,
                        if reverse { 0xbd } else { 0xbc },
                        if memory { 0x01 } else { 0xc0 },
                    ]);
                    let mut r = cpu(&bytes);
                    r.state[0] = 0;
                    r.state[1] = if memory { STACK as u64 } else { 0 };
                    r.memory.write(STACK as u64, &[0; 4]).unwrap();
                    assert!(matches!(r.run(1), Stop::Budget { .. }));
                    assert_eq!(r.flags() & 0x40, 0x40);
                }
                let mut r = cpu(&[0x0f, if reverse { 0xbd } else { 0xbc }, 0x00]);
                r.state[0] = 0; // Bad source pointer, not a zero-valued source.
                let before = r.state;
                assert!(matches!(r.run(1), Stop::Fault { .. }));
                assert_eq!(r.state, before);
            }
        }
    }
    #[test]
    fn high_bit_immediates_compare_at_their_declared_width() {
        for immediate in [0xfffe0200u32, 0xffff0200, 0x80000000, u32::MAX] {
            for value in [
                0,
                immediate.wrapping_sub(1),
                immediate,
                immediate.wrapping_add(1),
            ] {
                // CMP EAX, imm32. Decoder immediates are signed i64 values;
                // CF must compare the actual DWORDs, not their host containers.
                let mut bytes = vec![0x3d];
                bytes.extend(immediate.to_le_bytes());
                let mut r = cpu(&bytes);
                r.state[0] = value as u64;
                assert!(matches!(r.run(1), Stop::Budget { .. }));
                assert_eq!(
                    r.flags() & 1,
                    u32::from(value < immediate),
                    "{value:x} - {immediate:x}"
                );
                assert_eq!((r.flags() >> 6) & 1, u32::from(value == immediate));
                assert_eq!(r.state[0], value as u64);
            }
        }
    }
    #[test]
    fn actual_bits32_increment_wrap_flags_and_return() {
        // B8FFFFFFFF 40 (one-byte INC, not REX) C3.
        let mut r = cpu(&[0xb8, 0xff, 0xff, 0xff, 0xff, 0x40, 0xc3]);
        r.state[16] |= 1; // INC must preserve carry.
        assert_eq!(r.run(10), Stop::Returned(0));
        assert_eq!(r.flags() & 0xc5, 0x45); // CF/PF/ZF set, SF clear
        assert_eq!(r.instructions, 3);
        assert_eq!(r.registers()[4], STACK + 4096);
        assert_eq!(r.compiled_blocks(), 3);
    }
    #[test]
    fn call_push_immediate_and_return_do_not_corrupt_adjacent_stack() {
        // CALL +1 ; RET ; PUSH -11 ; POP EAX ; RET
        let mut r = cpu(&[0xe8, 1, 0, 0, 0, 0xc3, 0x6a, 0xf5, 0x58, 0xc3]);
        let before = r.state[4];
        assert_eq!(r.run(10), Stop::Returned(0xfffffff5));
        assert_eq!(r.memory.word(before).unwrap(), RETURN);
        assert_eq!(r.memory.word(before - 4).unwrap(), 0x10005);
        assert_eq!(r.memory.word(before - 8).unwrap(), 0xfffffff5);
        assert_eq!(r.registers()[4], STACK + 4096);
    }
    #[test]
    fn stack_pointer_wrap_is_32_bits_and_memory_footprint_stays_four_bytes() {
        let mut r = cpu(&[0x50, 0x58]);
        r.memory.map(0xffff_fffc, 4, READ | WRITE, &[0; 4]).unwrap();
        r.state[4] = 0;
        r.state[0] = 0x89abcdef;
        assert_eq!(r.run(1), Stop::Budget { pc: 0x10001 });
        assert_eq!(r.registers()[4], 0xfffffffc);
        assert_eq!(r.memory.word(0xfffffffc).unwrap(), 0x89abcdef);
        r.state[0] = 0;
        assert_eq!(r.run(1), Stop::Budget { pc: 0x10002 });
        assert_eq!(r.registers()[0], 0x89abcdef);
        assert_eq!(r.registers()[4], 0);
    }
    #[test]
    fn denied_push_rolls_back_stack_and_registers() {
        let mut r = cpu(&[0x50]);
        r.state[4] = STACK as u64;
        let before = r.registers();
        assert_eq!(
            r.run(1),
            Stop::Fault {
                pc: 0x10000,
                fault: Fault {
                    address: STACK as u64 - 4,
                    length: 4,
                    access: WRITE
                }
            }
        );
        assert_eq!(r.registers(), before);
        assert_eq!(r.instructions, 0);
        assert_eq!(r.memory.word(STACK as u64).unwrap(), 0x5a5a5a5a);
    }
    #[test]
    fn denied_call_rolls_back_and_unmapped_load_never_dereferences_host() {
        let mut r = cpu(&[0xe8, 0, 0, 0, 0]);
        r.state[4] = STACK as u64;
        let before = r.registers();
        assert!(matches!(
            r.run(1),
            Stop::Fault {
                fault: Fault {
                    length: 4,
                    access: WRITE,
                    ..
                },
                ..
            }
        ));
        assert_eq!(r.registers(), before);
        let mut r = cpu(&[0xa1, 0xff, 0xff, 0xff, 0xff]);
        r.state[0] = 0x1234;
        assert_eq!(
            r.run(1),
            Stop::Fault {
                pc: 0x10000,
                fault: Fault {
                    address: 0xffffffff,
                    length: 4,
                    access: READ
                }
            }
        );
        assert_eq!(r.registers()[0], 0x1234);
    }
    #[test]
    fn read_modify_write_to_code_faults_before_committing_flags() {
        // INC DWORD [0x10000]: reading succeeds, writing code is denied.
        let mut r = cpu(&[0xff, 0x05, 0, 0, 1, 0]);
        let before = r.state;
        assert_eq!(
            r.run(1),
            Stop::Fault {
                pc: 0x10000,
                fault: Fault {
                    address: 0x10000,
                    length: 4,
                    access: WRITE
                }
            }
        );
        assert_eq!(r.state, before);
        assert_eq!(
            r.memory.read(0x10000, 6, READ).unwrap(),
            [0xff, 0x05, 0, 0, 1, 0]
        );
    }
    #[test]
    fn exact_budget_resume_and_cached_branch_execution() {
        let mut r = cpu(&[0xeb, 0xfe]);
        assert_eq!(r.run(0), Stop::Budget { pc: 0x10000 });
        assert_eq!(r.compiled_blocks(), 0);
        assert_eq!(r.run(12), Stop::Budget { pc: 0x10000 });
        assert_eq!(r.run(8), Stop::Budget { pc: 0x10000 });
        assert_eq!(r.instructions, 20);
        assert_eq!(r.compiled_blocks(), 1);
    }
    #[test]
    fn unsupported_instructions_and_fetch_failures_are_explicit() {
        for code in [
            &[0x0f, 0x0b][..],
            &[0xd9, 0xf0],
            &[0xf0, 0x90],
            &[0x66, 0x50],
        ] {
            let mut r = cpu(code);
            assert!(matches!(r.run(1), Stop::Unsupported { pc: 0x10000, .. }));
            assert_eq!(r.instructions, 0);
        }
        let mut r = cpu(&[0x90]);
        r.state[17] = STACK as u64;
        assert_eq!(
            r.run(1),
            Stop::Fault {
                pc: STACK,
                fault: Fault {
                    address: STACK as u64,
                    length: 1,
                    access: EXEC
                }
            }
        );
    }
    #[test]
    fn import_policy_and_real_last_error_stdcall_gate() {
        assert!(Service::resolve("wrong.dll", &ImportSymbol::Name("GetLastError".into())).is_err());
        assert!(Service::resolve("kernel32.dll", &ImportSymbol::Ordinal(1)).is_err());
        assert!(Service::resolve("kernel32.dll", &ImportSymbol::Name("Missing".into())).is_err());
        assert!(
            Service::resolve("KERNEL32.DLL", &ImportSymbol::Name("GetLastError".into())).is_ok()
        );
        let mut r = cpu(&[0x90]);
        r.state[17] = GATE as u64;
        r.gates.insert(GATE, Service::SetLastError);
        r.memory
            .write(r.state[4] + 4, &0x13572468u32.to_le_bytes())
            .unwrap_err(); // beyond stack
        r.state[4] -= 4;
        r.memory.write(r.state[4], &RETURN.to_le_bytes()).unwrap();
        r.memory
            .write(r.state[4] + 4, &0x13572468u32.to_le_bytes())
            .unwrap();
        assert_eq!(r.run(1), Stop::Returned(0));
        assert_eq!(alkyhol::kernel::get_last_error(), 0x13572468);
        assert_eq!(r.service_calls, 1);
        assert_eq!(r.instructions, 0);
        assert_eq!(r.registers()[4], STACK + 4096);
    }
    #[test]
    fn checked_writefile_marshalling_uses_real_kernel_and_preserves_nonvolatiles() {
        let mut r = cpu(&[0x90]);
        let sp = STACK as u64 + 64;
        r.state[4] = sp;
        for (i, v) in [RETURN, 0x100, STACK + 128, 4, STACK + 132, 0]
            .iter()
            .enumerate()
        {
            r.memory.write(sp + i as u64 * 4, &v.to_le_bytes()).unwrap();
        }
        r.memory
            .write(STACK as u64 + 128, &0xdeadbeefu32.to_le_bytes())
            .unwrap();
        r.state[3] = 0x11223344;
        r.state[5] = 0x55667788;
        r.state[6] = 0x99aabbcc;
        r.state[7] = 0xddeeff00;
        r.service(Service::WriteFile).unwrap();
        assert_eq!(
            r.registers(),
            [
                1,
                0,
                0,
                0x11223344,
                sp as u32 + 24,
                0x55667788,
                0x99aabbcc,
                0xddeeff00
            ]
        );
        assert_eq!(r.memory.word(STACK as u64 + 132).unwrap(), 4);
        assert_eq!(r.pc(), RETURN);
        // Invalid guest output fails before performing any I/O or changing state.
        r.state[4] = sp;
        r.memory.write(sp + 16, &0u32.to_le_bytes()).unwrap();
        let before = r.state;
        assert!(r.service(Service::WriteFile).is_err());
        assert_eq!(r.state, before);
        r.memory
            .write(sp + 16, &(STACK + 132).to_le_bytes())
            .unwrap();
        r.memory.write(sp + 4, &0x999u32.to_le_bytes()).unwrap();
        r.service(Service::WriteFile).unwrap();
        assert_eq!(r.registers()[0], 0);
        assert_eq!(
            alkyhol::kernel::get_last_error(),
            alkyhol::kernel::ERROR_INVALID_HANDLE
        );
        assert_eq!(r.memory.word(STACK as u64 + 132).unwrap(), 0);
    }
}

#[cfg(test)]
mod extended_cpu_tests {
    use super::*;
    #[test]
    fn segment_override_absolute_modrm_wrap_and_lea_use_distinct_address_rules() {
        let mut r = tests::cpu(&[
            0x64, 0xa1, 0x2c, 0, 0, 0, // MOV EAX,FS:[2c]
            0x64, 0xa3, 0x30, 0, 0, 0, // MOV FS:[30],EAX
            0x64, 0x8d, 0x0d, 0x2c, 0, 0, 0, // LEA ECX,[2c], ignores FS
            0x64, 0x8b, 0x15, 0x30, 0, 0, 0, // MOV EDX,FS:[30]
            0xc3,
        ]);
        r.memory.map(0x30000, 4096, READ | WRITE, &[]).unwrap();
        r.state[22] = 0x30000;
        r.memory
            .write(0x3002c, &0xabcdef01u32.to_le_bytes())
            .unwrap();
        assert_eq!(r.run(10), Stop::Returned(0xabcdef01));
        assert_eq!(r.registers()[1], 0x2c);
        assert_eq!(r.registers()[2], 0xabcdef01);
        assert_eq!(r.memory.word(0x30030).unwrap(), 0xabcdef01);
        let mut r = tests::cpu(&[0x64, 0x8b, 0x04, 0x4d, 0x30, 0, 0, 0]); // FS:[ECX*2+30]
        r.memory.map(0x30000, 4096, READ | WRITE, &[]).unwrap();
        r.state[22] = 0x30000;
        r.state[1] = 0x80000000;
        r.memory
            .write(0x30030, &0x11223344u32.to_le_bytes())
            .unwrap();
        assert_eq!(r.run(1), Stop::Budget { pc: 0x10008 });
        assert_eq!(r.registers()[0], 0x11223344);
    }
    #[test]
    fn leave_loads_four_bytes_and_commits_frame_only_after_a_valid_read() {
        let mut r = tests::cpu(&[0xc9]);
        r.state[5] = STACK as u64 + 64;
        r.memory
            .write(STACK as u64 + 64, &0x55667788u32.to_le_bytes())
            .unwrap();
        assert_eq!(r.run(1), Stop::Budget { pc: 0x10001 });
        assert_eq!(r.registers()[4], STACK + 68);
        assert_eq!(r.registers()[5], 0x55667788);
        let mut r = tests::cpu(&[0xc9]);
        r.state[5] = 0;
        let saved = r.state;
        assert!(matches!(r.run(1), Stop::Fault { .. }));
        assert_eq!(r.state, saved);
    }
    #[test]
    fn all_set_condition_register_forms_match_boolean_flag_contract() {
        for bits in 0..32u32 {
            let cf = bits & 1 != 0;
            let pf = bits & 2 != 0;
            let zf = bits & 4 != 0;
            let sf = bits & 8 != 0;
            let of = bits & 16 != 0;
            let flags = 0x202
                | u32::from(cf)
                | (u32::from(pf) << 2)
                | (u32::from(zf) << 6)
                | (u32::from(sf) << 7)
                | (u32::from(of) << 11);
            let expected = [
                of,
                !of,
                cf,
                !cf,
                zf,
                !zf,
                cf || zf,
                !cf && !zf,
                sf,
                !sf,
                pf,
                !pf,
                sf != of,
                sf == of,
                zf || sf != of,
                !zf && sf == of,
            ];
            for (condition, &expected) in expected.iter().enumerate() {
                let mut r = tests::cpu(&[0x0f, 0x90 + condition as u8, 0xc0]);
                r.state[0] = 0xaabbcc00;
                r.state[16] = flags as u64;
                assert_eq!(r.run(1), Stop::Budget { pc: 0x10003 });
                assert_eq!(
                    r.registers()[0],
                    0xaabbcc00 | u32::from(expected),
                    "flags={flags:x} condition={condition}"
                );
                assert_eq!(r.flags(), flags);
            }
        }
    }
}

#[cfg(test)]
mod rotate_tests;

#[cfg(test)]
mod xlat_tests;

#[cfg(test)]
mod double_shift_tests;

#[cfg(test)]
mod prefetch_tests {
    use super::*;
    #[test]
    fn cache_hints_preserve_state_without_architectural_memory_access() {
        for hint in 0..4 {
            for prefix in [None, Some(0x64), Some(0x66)] {
                for address in [0, 0x20000, u32::MAX] {
                    let mut code = vec![];
                    if let Some(p) = prefix {
                        code.push(p);
                    }
                    code.extend([0x0f, 0x18, (hint << 3) | 0x80]);
                    code.extend(0x1234u32.to_le_bytes());
                    let mut r = tests::cpu(&code);
                    r.state[0] = address as u64;
                    r.state[22] = 0x30000;
                    r.state[16] = 0xad7;
                    {
                        let mut f = r.memory.fpu.borrow_mut();
                        f.push(0x3fff8000000000000000);
                        f.mxcsr = 0x5fc1;
                        f.status |= 0x20;
                    }
                    let before = r.state;
                    let fp = r.memory.fpu.borrow().clone();
                    assert!(matches!(r.run(1), Stop::Budget { .. }));
                    assert_eq!(&r.state[..17], &before[..17]);
                    assert_eq!(&r.state[18..], &before[18..]);
                    assert_eq!(r.state[17], 0x10000 + code.len() as u64);
                    let after = r.memory.fpu.borrow();
                    assert_eq!(after.status, fp.status);
                    assert_eq!(after.control, fp.control);
                    assert_eq!(after.tag, fp.tag);
                    assert_eq!(after.registers, fp.registers);
                    assert_eq!(after.mxcsr, fp.mxcsr);
                }
            }
        }
        for code in [
            &[0x0f, 0x18][..],
            &[0x0f, 0x18, 0x80, 1, 2][..],
            &[0xf0, 0x0f, 0x18, 0][..],
            &[0x0f, 0x18, 0xc0][..],
            &[0x0f, 0x18, 0x20][..],
        ] {
            assert!(compiler::compile(0x10000, code).is_err());
        }
    }
}

#[cfg(test)]
mod streaming_store_tests {
    use super::*;
    #[test]
    fn aligned_streaming_stores_copy_all_bits_and_fence_without_fp_side_effects() {
        for kind in 0..3 {
            for reg in 0..8 {
                let mut code = vec![];
                if kind != 0 {
                    code.push(0x66);
                }
                code.extend([0x0f, if kind == 2 { 0xe7 } else { 0x2b }, reg << 3]);
                code.extend([0x0f, 0xae, 0xf8]);
                let mut r = tests::cpu(&code);
                r.memory
                    .map(0x20000, 48, READ | WRITE, &[0xa5; 48])
                    .unwrap();
                r.state[0] = 0x20010;
                r.state[16] = 0xad7;
                for i in 0..16 {
                    r.state[24 + i] = 0x7ff0000000000001u64.rotate_left(i as u32) ^ i as u64;
                }
                {
                    let mut f = r.memory.fpu.borrow_mut();
                    f.push(0x3fff8000000000000000);
                    f.mxcsr = 0x5fc1;
                }
                let before = r.state;
                let fp = r.memory.fpu.borrow().clone();
                assert!(matches!(r.run(2), Stop::Budget { .. }));
                let bytes = r.memory.read(0x20000, 48, READ).unwrap();
                assert_eq!(&bytes[..16], &[0xa5; 16]);
                assert_eq!(&bytes[32..], &[0xa5; 16]);
                assert_eq!(&bytes[16..24], &before[24 + reg as usize * 2].to_le_bytes());
                assert_eq!(&bytes[24..32], &before[25 + reg as usize * 2].to_le_bytes());
                assert_eq!(&r.state[..17], &before[..17]);
                assert_eq!(&r.state[18..], &before[18..]);
                let after = r.memory.fpu.borrow();
                assert_eq!(after.mxcsr, fp.mxcsr);
                assert_eq!(after.status, fp.status);
                assert_eq!(after.registers, fp.registers);
            }
        }
    }
    #[test]
    fn streaming_stores_check_full_destination_alignment_and_write_rights_before_commit() {
        for prefix in [vec![], vec![0x66]] {
            for op in [0x2b, 0xe7] {
                if prefix.is_empty() && op == 0xe7 {
                    continue;
                }
                let mut code = prefix.clone();
                code.extend([0x0f, op, 0]);
                for (address, length, rights, alignment) in [
                    (0x20000, 16, READ, false),
                    (0x20000, 8, READ | WRITE, false),
                    (0x20001, 16, READ | WRITE, true),
                ] {
                    let mut r = tests::cpu(&code);
                    r.state[0] = address;
                    r.state[16] = 0xad7;
                    r.state[24] = u64::MAX;
                    r.state[25] = 0x12345678;
                    r.memory
                        .map(address as u32, length, rights, &vec![0xa5; length])
                        .unwrap();
                    let before = r.state;
                    let stop = r.run(1);
                    if alignment {
                        assert!(matches!(stop, Stop::CpuFault { .. }));
                    } else {
                        assert!(matches!(stop, Stop::Fault { .. }));
                    }
                    assert_eq!(&r.state[..17], &before[..17]);
                    assert_eq!(&r.state[18..], &before[18..]);
                    assert_eq!(
                        r.memory.read(address, length, READ).unwrap(),
                        vec![0xa5; length]
                    );
                }
            }
        }
        for code in [
            &[0x0f, 0x2b, 0xc0][..],
            &[0xf0, 0x0f, 0x2b, 0][..],
            &[0x0f, 0x2b][..],
        ] {
            assert!(compiler::compile(0x10000, code).is_err());
        }
    }
}

#[cfg(test)]
#[path = "../../pe/src/input_trace32.rs"]
mod input_state_trace_tests;
