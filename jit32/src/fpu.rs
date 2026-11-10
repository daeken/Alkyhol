//! Guest-owned x87 environment. Host ARM floating-point state is never reused.
use crate::memory::Memory;
#[derive(Clone, Debug)]
pub(crate) struct State {
    pub control: u16,
    pub status: u16,
    pub tag: u16,
    pub mxcsr: u32,
    pub registers: [u128; 8],
}
impl Default for State {
    fn default() -> Self {
        Self {
            control: 0x037f,
            status: 0,
            tag: 0xffff,
            mxcsr: 0x1f80,
            registers: [0; 8],
        }
    }
}
/// Emitted native calls implement the environment, rather than dropping x87
/// operations as empty templates. Register arithmetic is admitted separately.
pub(crate) unsafe extern "C" fn control(memory: *mut Memory, op: u64, value: u64) -> u64 {
    let memory = unsafe { &mut *memory };
    let mut state = memory.fpu.borrow_mut();
    match op {
        0 => {
            state.status &= !0x80ff;
            0
        } // FNCLEX clears exception flags/SF/ES/B.
        1 => state.control as u64,
        2 => {
            state.control = (value as u16 & 0x1f3f) | 0x40;
            0
        }
        3 => {
            let mxcsr = state.mxcsr;
            *state = State::default();
            state.mxcsr = mxcsr;
            0
        }
        4 => state.status as u64,
        _ => {
            memory
                .cpu_fault
                .get_or_insert("unknown x87 environment operation");
            0
        }
    }
}

/// Declared scalar virtual CPU, matching native-runtime32's machine profile.
/// No host CPUID, feature probing or SIMD capability is inferred from ARM.
pub(crate) unsafe extern "C" fn cpuid(memory: *mut Memory, registers: *mut u64) -> u64 {
    let regs = unsafe { std::slice::from_raw_parts_mut(registers, 91) };
    let experimental = match std::env::var("ALKYHOL_JIT32_EXPERIMENTAL_SSE2") {
        Err(std::env::VarError::NotPresent) => false,
        Ok(v) if v == "1" => true,
        _ => {
            unsafe { &mut *memory }.cpu_fault = Some("invalid experimental SSE2 profile setting");
            return 0;
        }
    };
    let values = cpuid_values(regs[0] as u32, experimental);
    for (reg, value) in [0, 3, 1, 2].into_iter().zip(values) {
        regs[reg] = value as u64;
    }
    1
}
fn cpuid_values(leaf: u32, experimental: bool) -> [u32; 4] {
    // This opt-in development profile unlocks original SSE/SSE2 branches. It
    // does not promise complete ISA coverage: missing operations remain fatal.
    let features = (1 << 4)
        | if experimental {
            (1 << 25) | (1 << 26)
        } else {
            0
        };
    match leaf {
        0 => [1, 0x756e6547, 0x6c65746e, 0x49656e69], // EAX, EBX, ECX, EDX
        1 => [0x600, 1 << 16, 0, features],
        0x80000000 => [0x80000000, 0, 0, 0],
        // Intel SDM CPUID: out-of-range basic/extended inputs return
        // the highest supported basic leaf (our declared leaf 1).
        _ => [0x600, 1 << 16, 0, features],
    }
}

impl State {
    pub fn top(&self) -> usize {
        ((self.status >> 11) & 7) as usize
    }
    pub fn set_top(&mut self, top: usize) {
        self.status = (self.status & !0x3800) | ((top as u16 & 7) << 11);
    }
    pub fn get(&mut self, index: usize) -> u128 {
        let physical = (self.top() + index) & 7;
        if self.tag >> (physical * 2) & 3 == 3 {
            self.status = (self.status | 0x41) & !0x200;
            0xffffc000000000000000
        } else {
            self.registers[physical]
        }
    }
    pub fn set(&mut self, index: usize, value: u128) {
        use rustc_apfloat::{Float, ieee::X87DoubleExtended as X};
        let physical = (self.top() + index) & 7;
        let f = X::from_bits(value);
        let tag = if f.is_zero() {
            1
        } else if !f.is_finite() || f.is_denormal() {
            2
        } else {
            0
        };
        self.registers[physical] = value;
        self.tag = (self.tag & !(3 << (physical * 2))) | (tag << (physical * 2));
    }
    pub fn push(&mut self, value: u128) {
        let next = (self.top() + 7) & 7;
        let full = self.tag >> (next * 2) & 3 != 3;
        self.set_top(next);
        self.status &= !0x200;
        if full {
            self.status |= 0x241;
            self.set(0, 0xffffc000000000000000);
        } else {
            self.set(0, value);
        }
    }
    pub fn pop(&mut self) {
        let top = self.top();
        self.tag |= 3 << (top * 2);
        self.set_top((top + 1) & 7);
    }
}

unsafe extern "C" {
    fn mach_absolute_time() -> u64;
}
/// Read the real host monotonic tick domain as the virtual TSC.
pub(crate) unsafe extern "C" fn timestamp() -> u64 {
    unsafe { mach_absolute_time() }
}

pub(crate) unsafe extern "C" fn wait(memory: *mut Memory) -> u64 {
    let memory = unsafe { &mut *memory };
    let state = memory.fpu.borrow();
    if state.status & !state.control & 0x3f != 0 {
        memory
            .cpu_fault
            .get_or_insert("x87 #MF: pending unmasked exception");
        0
    } else {
        1
    }
}
