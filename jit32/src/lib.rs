//! Experimental checked PE32 -> native ARM64 runner; no Rosetta or interpreter fallback.
#![cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod compiler;
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
            &[0xd9, 0xe8],
            &[0xf3, 0xa4],
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
