//! Bounded legacy SSE arithmetic. Floating results never use host FP control.
use crate::memory::{Memory, READ};
use rustc_apfloat::{
    Float, Round, Status,
    ieee::{Double, Single},
};
pub fn operation(name: &str) -> Option<u64> {
    let (prefix, form) = if let Some(p) = name.strip_suffix("SS") {
        (p, 0)
    } else if let Some(p) = name.strip_suffix("SD") {
        (p, 1)
    } else if let Some(p) = name.strip_suffix("PS") {
        (p, 2)
    } else {
        (name.strip_suffix("PD")?, 3)
    };
    let op = match prefix {
        "ADD" => 0,
        "SUB" => 1,
        "MUL" => 2,
        "DIV" => 3,
        _ => return None,
    };
    Some(op | (form << 4))
}
fn calculate<F: Float>(a: u128, b: u128, op: u64, mxcsr: u32) -> Result<(u128, u32), &'static str> {
    let (mut a, mut b) = (F::from_bits(a), F::from_bits(b));
    if !a.is_finite() || !b.is_finite() {
        return Err("nonfinite SIMD arithmetic operands are not implemented");
    }
    if mxcsr & 0x40 != 0 {
        if a.is_denormal() {
            a = F::ZERO.copy_sign(a);
        }
        if b.is_denormal() {
            b = F::ZERO.copy_sign(b);
        }
    }
    if a.is_denormal() || b.is_denormal() {
        return Err("unflushed denormal SIMD arithmetic operands are not implemented");
    }
    let rc = match mxcsr >> 13 & 3 {
        0 => Round::NearestTiesToEven,
        1 => Round::TowardNegative,
        2 => Round::TowardPositive,
        _ => Round::TowardZero,
    };
    let result = match op {
        0 => a.add_r(b, rc),
        1 => a.sub_r(b, rc),
        2 => a.mul_r(b, rc),
        3 => a.div_r(b, rc),
        _ => return Err("unknown SIMD arithmetic operation"),
    };
    if result
        .status
        .intersects(Status::INVALID_OP | Status::DIV_BY_ZERO | Status::OVERFLOW | Status::UNDERFLOW)
        || result.value.is_denormal()
    {
        return Err("exceptional/underflow SIMD arithmetic is not implemented");
    }
    let flags = u32::from(result.status.contains(Status::INEXACT)) << 5;
    if flags & !(mxcsr >> 7) != 0 {
        return Err("unmasked SIMD arithmetic exception requires #XM delivery");
    }
    Ok((result.value.to_bits(), mxcsr | flags))
}
fn perform(
    memory: &mut Memory,
    registers: &mut [u64],
    description: u64,
    address: u64,
) -> Result<(), &'static str> {
    let op = description & 15;
    let form = description >> 4 & 3;
    let dst = (description >> 8 & 7) as usize;
    let src = (description >> 12 & 7) as usize;
    let packed = form >= 2;
    let width = if form & 1 == 0 { 32 } else { 64 };
    let lanes = if packed { 128 / width } else { 1 };
    let old = registers[24 + dst * 2] as u128 | ((registers[25 + dst * 2] as u128) << 64);
    let source = if description & (1 << 16) != 0 {
        registers[24 + src * 2] as u128 | ((registers[25 + src * 2] as u128) << 64)
    } else {
        if packed && address & 15 != 0 {
            return Err("unaligned legacy packed SIMD arithmetic requires #GP delivery");
        }
        let length = if packed { 16 } else { width as usize / 8 };
        if let Err(fault) = memory.check(address, length, READ) {
            memory.fault = Some(fault);
            return Err("SIMD source memory fault");
        }
        let bytes = memory.read(address, length, READ).unwrap();
        let mut value = [0; 16];
        value[..length].copy_from_slice(bytes);
        u128::from_le_bytes(value)
    };
    let mut mxcsr = memory.fpu.borrow().mxcsr;
    let mut output = old;
    let mask = (1u128 << width) - 1;
    for lane in 0..lanes {
        let shift = lane * width;
        let a = old >> shift & mask;
        let b = source >> shift & mask;
        let (value, next) = if width == 32 {
            calculate::<Single>(a, b, op, mxcsr)?
        } else {
            calculate::<Double>(a, b, op, mxcsr)?
        };
        output = (output & !(mask << shift)) | ((value & mask) << shift);
        mxcsr = next;
    }
    registers[24 + dst * 2] = output as u64;
    registers[25 + dst * 2] = (output >> 64) as u64;
    memory.fpu.borrow_mut().mxcsr = mxcsr;
    Ok(())
}
pub unsafe extern "C" fn execute(
    memory: *mut Memory,
    registers: *mut u64,
    description: u64,
    address: u64,
) -> u64 {
    let memory = unsafe { &mut *memory };
    let registers = unsafe { std::slice::from_raw_parts_mut(registers, 91) };
    match perform(memory, registers, description, address) {
        Ok(()) => 1,
        Err(reason) => {
            if memory.fault.is_none() {
                memory.cpu_fault = Some(reason);
            }
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Stop,
        memory::{READ, WRITE},
        tests::cpu,
    };
    #[test]
    fn all_basic_scalar_and_packed_forms_preserve_lanes_flags_and_exact_results() {
        let lhs = [1.5, -2.5, 4.0, -8.0];
        let rhs = [2.0, 0.5, -2.0, -2.0];
        let expected = [
            [3.5, -2.0, 2.0, -10.0],
            [-0.5, -3.0, 6.0, -6.0],
            [3.0, -1.25, -8.0, 16.0],
            [0.75, -5.0, -2.0, 4.0],
        ];
        for (prefix, width, packed) in [
            (Some(0xf3), 32, false),
            (Some(0xf2), 64, false),
            (None, 32, true),
            (Some(0x66), 64, true),
        ] {
            for (operation, opcode) in [0x58, 0x5c, 0x59, 0x5e].into_iter().enumerate() {
                let mut code = Vec::new();
                if let Some(p) = prefix {
                    code.push(p);
                }
                code.extend([0x0f, opcode, 0xc1, 0xc3]);
                let mut r = cpu(&code);
                let mut a = 0u128;
                let mut b = 0u128;
                let mut out = 0u128;
                for lane in 0..128 / width {
                    let encode = |v: f64| {
                        if width == 32 {
                            (v as f32).to_bits() as u128
                        } else {
                            v.to_bits() as u128
                        }
                    };
                    a |= encode(lhs[lane]) << (lane * width);
                    b |= encode(rhs[lane]) << (lane * width);
                    out |= encode(if packed || lane == 0 {
                        expected[operation][lane]
                    } else {
                        lhs[lane]
                    }) << (lane * width);
                }
                r.state[24] = a as u64;
                r.state[25] = (a >> 64) as u64;
                r.state[26] = b as u64;
                r.state[27] = (b >> 64) as u64;
                r.state[16] = 0xad7;
                r.memory.fpu.borrow_mut().mxcsr = 0x1f83;
                assert_eq!(r.run(3), Stop::Returned(0));
                assert_eq!(r.state[24] as u128 | ((r.state[25] as u128) << 64), out);
                assert_eq!(r.state[26] as u128 | ((r.state[27] as u128) << 64), b);
                assert_eq!(r.state[16], 0xad7);
                assert_eq!(r.memory.fpu.borrow().mxcsr, 0x1f83);
            }
        }
    }
    #[test]
    fn rounding_uses_guest_mxcsr_and_rejected_lanes_commit_nothing() {
        for (prefix, one, tie, next) in [
            (
                0xf3,
                1f32.to_bits() as u64,
                (2f32).powi(-24).to_bits() as u64,
                0x3f800001,
            ),
            (
                0xf2,
                1f64.to_bits(),
                (2f64).powi(-53).to_bits(),
                0x3ff0000000000001,
            ),
        ] {
            for rc in 0..4 {
                let mut r = cpu(&[prefix, 0x0f, 0x58, 0xc1, 0xc3]);
                r.state[24] = one;
                r.state[26] = tie;
                r.memory.fpu.borrow_mut().mxcsr = 0x1f80 | (rc << 13);
                assert_eq!(r.run(3), Stop::Returned(0));
                assert_eq!(r.state[24], if rc == 2 { next } else { one });
                assert_eq!(r.memory.fpu.borrow().mxcsr, 0x1fa0 | (rc << 13));
            }
        }
        let mut r = cpu(&[0x0f, 0x58, 0xc1, 0xc3]);
        r.state[24] = 1f32.to_bits() as u64 | ((1f32.to_bits() as u64) << 32);
        r.state[25] = 0x1234567812345678;
        r.state[26] = 1f32.to_bits() as u64 | (((2f32).powi(-24).to_bits() as u64) << 32);
        let before = r.state;
        r.memory.fpu.borrow_mut().mxcsr = 0xf80;
        assert!(matches!(r.run(3), Stop::CpuFault { pc: 0x10000, .. }));
        assert_eq!(r.state, before);
        assert_eq!(r.memory.fpu.borrow().mxcsr, 0xf80);
    }
    #[test]
    fn memory_sources_are_checked_and_packed_legacy_alignment_is_enforced() {
        let mut r = cpu(&[0xf2, 0x0f, 0x58, 0x05, 1, 0, 2, 0, 0xc3]);
        let mut data = [0x5a; 16];
        data[1..9].copy_from_slice(&2.0f64.to_bits().to_le_bytes());
        r.memory.map(0x20000, 16, READ, &data).unwrap();
        r.state[24] = 1.5f64.to_bits();
        assert_eq!(r.run(3), Stop::Returned(0));
        assert_eq!(r.state[24], 3.5f64.to_bits());
        let mut r = cpu(&[0x66, 0x0f, 0x58, 0x05, 8, 0, 2, 0, 0xc3]);
        r.memory.map(0x20000, 32, READ | WRITE, &[0; 32]).unwrap();
        let before = r.state;
        assert!(matches!(r.run(3), Stop::CpuFault { pc: 0x10000, .. }));
        assert_eq!(r.state, before);
        let mut r = cpu(&[0xf2, 0x0f, 0x58, 0x05, 0, 0, 2, 0, 0xc3]);
        let before = r.state;
        assert!(matches!(r.run(3), Stop::Fault { pc: 0x10000, .. }));
        assert_eq!(r.state, before);
    }
}

#[cfg(test)]
mod reference {
    use crate::{Stop, tests::cpu};
    #[test]
    #[ignore = "run tools/check_jit32_sse.py for independent actual x86 observations"]
    fn actual_x86_arithmetic_observations() {
        let data = std::fs::read(
            std::env::var_os("ALKYHOL_SSE_ARITHMETIC_REFERENCE")
                .expect("actual x86 observer output required"),
        )
        .unwrap();
        assert_eq!(data.len(), 1024 * 64);
        for row in data.chunks_exact(64) {
            let op = u32::from_le_bytes(row[..4].try_into().unwrap());
            let form = u32::from_le_bytes(row[4..8].try_into().unwrap());
            let mxcsr = u32::from_le_bytes(row[8..12].try_into().unwrap());
            let mut code = Vec::new();
            match form {
                0 => code.push(0xf3),
                1 => code.push(0xf2),
                2 => (),
                3 => code.push(0x66),
                _ => panic!("invalid observer form"),
            }
            code.extend([0x0f, [0x58, 0x5c, 0x59, 0x5e][op as usize], 0xc1, 0xc3]);
            let mut r = cpu(&code);
            for i in 0..4 {
                r.state[24 + i] =
                    u64::from_le_bytes(row[12 + i * 8..20 + i * 8].try_into().unwrap());
            }
            r.memory.fpu.borrow_mut().mxcsr = mxcsr;
            assert_eq!(
                r.run(3),
                Stop::Returned(0),
                "op={op} form={form} mxcsr={mxcsr:x}"
            );
            for i in 0..2 {
                let expected = u64::from_le_bytes(row[44 + i * 8..52 + i * 8].try_into().unwrap());
                assert_eq!(
                    r.state[24 + i],
                    expected,
                    "op={op} form={form} mxcsr={mxcsr:x} lane64={i}"
                );
            }
            assert_eq!(
                r.memory.fpu.borrow().mxcsr,
                u32::from_le_bytes(row[60..64].try_into().unwrap()),
                "op={op} form={form} mxcsr={mxcsr:x}"
            );
        }
    }
}
