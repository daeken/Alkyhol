//! Explicit x87 data intrinsics missing from the pinned generated lifter.
//! All numeric conversions use LLVM APFloat's software 80-bit format in Rust.
use crate::{
    fpu::State,
    memory::{Memory, READ, WRITE},
};
use rustc_apfloat::{
    Float, FloatConvert, Round, Status,
    ieee::{Double, Single, X87DoubleExtended as X},
};
fn kind_name_integer(name: &str) -> bool {
    name.starts_with("FI")
}
pub fn operation(name: &str) -> Option<u64> {
    let arith = match name {
        "FADDP" => Some((0, 4, false)),
        "FMULP" => Some((1, 4, false)),
        "FSUBP" => Some((2, 4, false)),
        "FDIVP" => Some((3, 4, false)),
        "FSUBRP" => Some((2, 4, true)),
        "FDIVRP" => Some((3, 4, true)),
        _ => None,
    };
    let arith = arith.or_else(|| {
        let (kind, suffix) = name.split_once('-')?;
        let (kind, reverse) = match kind {
            "FADD" => (0, false),
            "FMUL" => (1, false),
            "FSUB" => (2, false),
            "FDIV" => (3, false),
            "FSUBR" => (2, true),
            "FDIVR" => (3, true),
            "FIADD" => (0, false),
            "FIMUL" => (1, false),
            "FISUB" => (2, false),
            "FIDIV" => (3, false),
            "FISUBR" => (2, true),
            "FIDIVR" => (3, true),
            _ => return None,
        };
        let form = match (kind_name_integer(name), suffix) {
            (true, "M16") => 5,
            (true, "M32") => 6,
            (false, "M32") => 0,
            (false, "M64") => 1,
            (false, "ST") => 2,
            (false, "STI") => 3,
            _ => return None,
        };
        Some((kind, form, reverse))
    });
    if let Some((kind, form, reverse)) = arith {
        return Some(if reverse { 64 } else { 32 } + form * 4 + kind);
    }

    Some(match name {
        "FLD1" => 0,
        "FLDZ" => 1,
        "FLDPI" => 128,
        "FLDL2T" => 129,
        "FLDL2E" => 130,
        "FLDLG2" => 131,
        "FLDLN2" => 132,
        "FXAM" => 133,
        "FLD-M32" => 2,
        "FLD-M64" => 3,
        "FLD-M80" => 4,
        "FST-M32" => 5,
        "FSTP-M32" => 6,
        "FST-M64" => 7,
        "FSTP-M64" => 8,
        "FSTP-M80" => 9,
        "FILD-M16" => 10,
        "FILD-M32" => 11,
        "FILD-M64" => 12,
        "FLD-ST" => 13,
        "FST-ST" => 14,
        "FSTP-ST" => 15,
        "FXCH" => 16,
        "FCHS" => 17,
        "FABS" => 18,
        "FFREE" => 19,
        "FINCSTP" => 20,
        "FDECSTP" => 21,
        "FCOM-ST" => 100,
        "FCOMP-ST" => 101,
        "FCOMPP" => 102,
        "FUCOM" => 103,
        "FUCOMP" => 104,
        "FUCOMPP" => 105,
        "FCOM-M32" => 106,
        "FCOMP-M32" => 107,
        "FCOM-M64" => 108,
        "FCOMP-M64" => 109,
        "FTST" => 110,
        "FCOMI" => 111,
        "FCOMIP" => 112,
        "FUCOMI" => 113,
        "FUCOMIP" => 114,
        "FICOM-M32" => 116,
        "FICOMP-M32" => 117,
        "FICOM-M16" => 118,
        "FICOMP-M16" => 119,
        "FSINCOS" => 124,
        "FSIN" => 125,
        "FCOS" => 126,
        "FSQRT" => 31,
        "FNOP" => 22,
        "FIST-M16" => 23,
        "FISTP-M16" => 24,
        "FISTTP-M16" => 25,
        "FIST-M32" => 26,
        "FISTP-M32" => 27,
        "FISTTP-M32" => 28,
        "FISTP-M64" => 29,
        "FISTTP-M64" => 30,
        _ => return None,
    })
}
pub(super) fn round(state: &State) -> Round {
    match state.control >> 10 & 3 {
        0 => Round::NearestTiesToEven,
        1 => Round::TowardNegative,
        2 => Round::TowardPositive,
        _ => Round::TowardZero,
    }
}
pub(super) fn flags(state: &mut State, status: Status) {
    for (source, dest) in [
        (Status::INVALID_OP, 1),
        (Status::DIV_BY_ZERO, 4),
        (Status::OVERFLOW, 8),
        (Status::UNDERFLOW, 16),
        (Status::INEXACT, 32),
    ] {
        if status.contains(source) {
            state.status |= dest;
        }
    }
}
fn load<F: Float + FloatConvert<X>>(state: &mut State, bits: u128) -> u128 {
    let value = F::from_bits(bits);
    if value.is_denormal() {
        state.status |= 2;
    }
    let result = value.convert(&mut false);
    flags(state, result.status);
    result.value.to_bits()
}
fn store<F: Float + FloatConvert<X>>(state: &mut State, value: u128) -> u128
where
    X: FloatConvert<F>,
{
    let value = X::from_bits(value);
    let result = value.convert_r(round(state), &mut false);
    flags(state, result.status);
    state.status &= !0x200;
    if result.status.contains(Status::INEXACT) {
        let back: X = result.value.convert(&mut false).value;
        if back.abs() > value.abs() {
            state.status |= 0x200;
        }
    }
    result.value.to_bits()
}
fn read(memory: &mut Memory, address: u64, length: usize) -> Result<u128, ()> {
    if let Err(fault) = memory.check(address, length, READ) {
        memory.fault.get_or_insert(fault);
        return Err(());
    }
    let mut bytes = [0; 16];
    bytes[..length].copy_from_slice(memory.read(address, length, READ).unwrap());
    Ok(u128::from_le_bytes(bytes))
}
fn perform(
    memory: &mut Memory,
    state: &mut State,
    op: u64,
    index: usize,
    address: u64,
    flags_register: &mut u64,
) -> Result<Option<(usize, u128)>, ()> {
    let mut output = None;
    match op {
        133 => {
            // Intel SDM FXAM: examine raw ST(0), including empty-register sign.
            // Do not use get(): examining an empty stack raises no stack fault.
            let physical = state.top();
            let bits = state.registers[physical];
            let exponent = (bits >> 64) & 0x7fff;
            let significand = bits as u64;
            let integer = significand >> 63;
            let class = if state.tag >> (physical * 2) & 3 == 3 {
                0x4100
            } else if exponent == 0 {
                if significand == 0 { 0x4000 } else { 0x4400 }
            } else if integer == 0 {
                0 // Unsupported unnormal / pseudo-infinity / pseudo-NaN.
            } else if exponent != 0x7fff {
                0x400
            } else if significand == 1 << 63 {
                0x500
            } else {
                0x100
            };
            state.status = (state.status & !0x4700) | class | (((bits >> 79) as u16 & 1) << 9);
        }
        0 => state.push(0x3fff8000000000000000),
        1 => state.push(0),
        128..=132 => {
            // Positive ROM constants. RC chooses the lower/upper significand;
            // PC is ignored and no precision flag is raised (Intel FLD constants).
            // Independent actual x86/Rosetta rows: tests/x87_constants_reference.c.
            let (lower, nearest_up) = match op {
                128 => (0x4000c90fdaa22168c234u128, true),
                129 => (0x4000d49a784bcd1b8afe, false),
                130 => (0x3fffb8aa3b295c17f0bb, true),
                131 => (0x3ffd9a209a84fbcff798, true),
                _ => (0x3ffeb17217f7d1cf79ab, true),
            };
            let rc = state.control >> 10 & 3;
            state.push(lower + u128::from(rc == 2 || (rc == 0 && nearest_up)));
        }
        2..=4 => {
            let bits = read(
                memory,
                address,
                if op == 2 {
                    4
                } else if op == 3 {
                    8
                } else {
                    10
                },
            )?;
            if op == 4 {
                let exp = (bits >> 64) & 0x7fff;
                let integer = bits >> 63 & 1;
                if (exp != 0 && integer == 0) || (exp == 0 && integer != 0) {
                    memory
                        .cpu_fault
                        .get_or_insert("noncanonical x87 extended load is not implemented");
                    return Err(());
                }
            }
            let value = if op == 2 {
                load::<Single>(state, bits)
            } else if op == 3 {
                load::<Double>(state, bits)
            } else {
                bits
            };
            state.push(value);
        }
        5..=9 => {
            let value = state.get(0);
            let (length, bits) = if op <= 6 {
                (4, store::<Single>(state, value))
            } else if op <= 8 {
                (8, store::<Double>(state, value))
            } else {
                (10, value)
            };
            if op == 9 {
                state.status &= !0x200;
            }
            output = Some((length, bits));
            if [6, 8, 9].contains(&op) {
                state.pop();
            }
        }
        10..=12 => {
            let n = match op {
                10 => read(memory, address, 2)? as i16 as i128,
                11 => read(memory, address, 4)? as i32 as i128,
                _ => read(memory, address, 8)? as i64 as i128,
            };
            let value = X::from_i128(n);
            flags(state, value.status);
            state.push(value.value.to_bits());
        }
        13 => {
            let value = state.get(index);
            state.push(value);
        }
        14 | 15 => {
            let value = state.get(0);
            state.set(index, value);
            state.status &= !0x200;
            if op == 15 {
                state.pop();
            }
        }
        16 => {
            let a = state.get(0);
            let b = state.get(index);
            state.set(0, b);
            state.set(index, a);
            state.status &= !0x200;
        }
        17 | 18 => {
            let value = state.get(0);
            state.set(
                0,
                if op == 17 {
                    value ^ (1 << 79)
                } else {
                    value & !(1 << 79)
                },
            );
            state.status &= !0x200;
        }
        19 => {
            let physical = (state.top() + index) & 7;
            state.tag |= 3 << (physical * 2);
        }
        20 | 21 => {
            state.set_top((state.top() + if op == 20 { 1 } else { 7 }) & 7);
            state.status &= !0x200;
        }
        22 => (),
        23..=30 => {
            let width = if op <= 25 {
                16
            } else if op <= 28 {
                32
            } else {
                64
            };
            let value = X::from_bits(state.get(0));
            let rounding = if [25, 28, 30].contains(&op) {
                Round::TowardZero
            } else {
                round(state)
            };
            let result = value.to_i128_r(width, rounding, &mut false);
            flags(state, result.status);
            state.status &= !0x200;
            let output_bits = if result.status.contains(Status::INVALID_OP) {
                1u128 << (width - 1)
            } else {
                result.value as u128
            };
            if result.status.contains(Status::INEXACT)
                && X::from_i128(result.value).value.abs() > value.abs()
            {
                state.status |= 0x200;
            }
            output = Some((width / 8, output_bits));
            if ![23, 26].contains(&op) {
                state.pop();
            }
        }
        31 => {
            let value = state.get(0);
            let value = match super::fpu_math::sqrt(state, value) {
                Ok(v) => v,
                Err(reason) => {
                    memory.cpu_fault.get_or_insert(reason);
                    return Err(());
                }
            };
            state.set(0, value);
        }
        124..=126 => {
            let bits = state.get(0);
            let value = X::from_bits(bits);
            if value.is_finite() && value.abs() >= X::from_u128(1u128 << 63).value {
                state.status |= 0x400;
                return Ok(None);
            }
            if state.control >> 10 & 3 != 0 {
                memory
                    .cpu_fault
                    .get_or_insert("non-nearest x87 trig control is not implemented");
                return Err(());
            }
            if op == 124 && state.tag >> (((state.top() + 7) & 7) * 2) & 3 != 3 {
                memory
                    .cpu_fault
                    .get_or_insert("FSINCOS stack overflow response is not implemented");
                return Err(());
            }
            let ((sin, su), (cos, cu)) = match super::fpu_trig::sin_cos(value) {
                Ok(v) => v,
                Err(reason) => {
                    memory.cpu_fault.get_or_insert(reason);
                    return Err(());
                }
            };
            state.status &= !0x600;
            if !value.is_zero() {
                state.status |= 0x20;
            }
            if op == 124 {
                state.set(0, sin.to_bits());
                state.push(cos.to_bits());
                if cu {
                    state.status |= 0x200;
                }
            } else if op == 125 {
                state.set(0, sin.to_bits());
                if su {
                    state.status |= 0x200;
                }
            } else {
                state.set(0, cos.to_bits());
                if cu {
                    state.status |= 0x200;
                }
            }
        }
        100..=119 => {
            let a = X::from_bits(state.get(0));
            let b = match op {
                106 | 107 => {
                    let bits = read(memory, address, 4)?;
                    X::from_bits(load::<Single>(state, bits))
                }
                108 | 109 => {
                    let bits = read(memory, address, 8)?;
                    X::from_bits(load::<Double>(state, bits))
                }
                116 | 117 => {
                    let bits = read(memory, address, 4)?;
                    X::from_i128(bits as i32 as i128).value
                }
                118 | 119 => {
                    let bits = read(memory, address, 2)?;
                    X::from_i128(bits as i16 as i128).value
                }
                110 => X::ZERO,
                102 | 105 => X::from_bits(state.get(1)),
                _ => X::from_bits(state.get(index)),
            };
            let quiet = [103, 104, 105, 113, 114].contains(&op);
            let unordered = a.is_nan() || b.is_nan();
            if a.is_denormal() || b.is_denormal() {
                state.status |= 2;
            }
            if unordered && (!quiet || a.is_signaling() || b.is_signaling()) {
                state.status |= 1;
            }
            let flags = if unordered {
                0x4500
            } else if a == b {
                0x4000
            } else if a < b {
                0x100
            } else {
                0
            };
            state.status &= !0x200;
            if (111..=114).contains(&op) {
                *flags_register = (*flags_register & !0x8d5)
                    | ((flags as u64 >> 8) & 1)
                    | ((flags as u64 >> 8) & 4)
                    | ((flags as u64 >> 8) & 0x40);
            } else {
                state.status = (state.status & !0x4500) | flags;
            }
            if [101, 104, 107, 109, 112, 114, 117, 119].contains(&op) {
                state.pop();
            }
            if [102, 105].contains(&op) {
                state.pop();
                state.pop();
            }
        }
        32..=91 => {
            let reverse = op >= 64;
            let code = op - if reverse { 64 } else { 32 };
            let form = code / 4;
            let kind = code % 4;
            let dest = if form == 3 || form == 4 { index } else { 0 };
            let a = state.get(dest);
            let b = match form {
                0 => {
                    let bits = read(memory, address, 4)?;
                    load::<Single>(state, bits)
                }
                1 => {
                    let bits = read(memory, address, 8)?;
                    load::<Double>(state, bits)
                }
                2 => state.get(index),
                5 => {
                    let bits = read(memory, address, 2)?;
                    X::from_i128(bits as i16 as i128).value.to_bits()
                }
                6 => {
                    let bits = read(memory, address, 4)?;
                    X::from_i128(bits as i32 as i128).value.to_bits()
                }
                _ => state.get(0),
            };
            let (a, b) = if reverse { (b, a) } else { (a, b) };
            let value = match super::fpu_math::arithmetic(state, a, b, kind) {
                Ok(value) => value,
                Err(reason) => {
                    memory.cpu_fault.get_or_insert(reason);
                    return Err(());
                }
            };
            state.set(dest, value);
            if form == 4 {
                state.pop();
            }
        }
        _ => unreachable!(),
    }
    Ok(output)
}
/// Checked native helper: a memory failure never pops/pushes the x87 stack or
/// commits flags. Guest addresses are translated only by Memory's owned view.
pub(crate) unsafe extern "C" fn execute(
    memory: *mut Memory,
    operation: u64,
    address: u64,
    registers: *mut u64,
) -> u64 {
    let memory = unsafe { &mut *memory };
    let mut state = memory.fpu.borrow().clone();
    if operation & 255 != 133 && state.control & 0x3f != 0x3f {
        memory
            .cpu_fault
            .get_or_insert("unmasked x87 data exceptions are not implemented");
        return 0;
    }
    let op = operation & 255;
    let index = ((operation >> 8) & 7) as usize;
    let mut cpu_flags = unsafe { registers.add(16).read() };
    let Ok(output) = perform(memory, &mut state, op, index, address, &mut cpu_flags) else {
        return 0;
    };
    if let Some((length, value)) = output {
        if let Err(fault) = memory.check(address, length, WRITE) {
            memory.fault.get_or_insert(fault);
            return 0;
        }
        memory
            .write(address, &value.to_le_bytes()[..length])
            .unwrap();
    }
    *memory.fpu.borrow_mut() = state;
    unsafe {
        registers.add(16).write(cpu_flags);
    }
    1
}

/// Same bounded operand/control contract as Alkyhol's owned x86 CRT gates.
/// Native ARM supplies a real floating-point result; it never calls Rosetta.
pub(crate) fn ci_math(memory: &mut Memory, id: u32) -> Result<(), String> {
    let mut state = memory.fpu.borrow().clone();
    if !(if id == 89 {
        [0x27f, 0x37f].contains(&state.control)
    } else {
        state.control == 0x37f
    }) || state.mxcsr & !0x3f != 0x1f80
        || state.tag >> (state.top() * 2) & 3 == 3
    {
        return Err("_CIsqrt control/stack outside the measured profile".into());
    }
    let value = state.get(0);
    let exp = (value >> 64) as u16;
    let sig = value as u64;
    if !(exp & 0x7fff == 0 && sig == 0)
        && (!(0x3bcd..=0x43fe).contains(&(if id == 89 { exp } else { exp & 0x7fff }))
            || sig >> 63 == 0
            || ((exp & 0x7fff) == 0x43fe && sig > 0xfffffffffffff800))
    {
        return Err("_CIsqrt operand outside measured finite binary64 range".into());
    }
    if id == 92 {
        let free = (state.top() + 7) & 7;
        if state.tag >> (free * 2) & 3 != 3 {
            return Err("_CIatan requires one free x87 stack slot".into());
        }
        let (result, _up) = super::fpu_atan::atan(X::from_bits(value))?;
        state.status &= !0x200;
        if !X::from_bits(value).is_zero() {
            state.status |= 0x20;
        }
        if result.is_denormal() {
            state.status |= 2;
        }
        let value: X = result.convert(&mut false).value;
        state.set(0, value.to_bits());
        *memory.fpu.borrow_mut() = state;
        return Ok(());
    }
    let value = super::fpu_math::sqrt(&mut state, value)?;
    let value = if state.control == 0x37f {
        let bits = store::<Double>(&mut state, value);
        let result = load::<Double>(&mut state, bits);
        state.status &= !0x200;
        result
    } else {
        value
    };
    state.set(0, value);
    *memory.fpu.borrow_mut() = state;
    Ok(())
}

#[cfg(test)]
mod native_rows {
    use super::*;
    #[test]
    fn crt_math_matches_the_unchanged_frozen_windows_rows() {
        let inputs = [
            (0u64, 0u16),
            (0, 0x8000),
            (0x8000000000000000, 0x3fff),
            (0x8000000000000000, 0x4000),
            (0x8000000000000000, 0x4001),
            (0xc000000000000000, 0x4000),
            (0x8000000000000001, 0x3fff),
            (0x8000000000000800, 0x3fff),
            (0x8000000000000000, 0x3bcd),
            (0xfffffffffffff800, 0x43fe),
            (0x8000000000000000, 0x43ff),
            (1, 0),
            (0x8000000000000000, 0xbfff),
        ];
        for (id, source) in [
            (
                89,
                include_str!("../../pe/reference/pe32-ci-sqrt-windows-v1.json"),
            ),
            (
                92,
                include_str!("../../pe/reference/pe32-ci-atan-windows-v1.json"),
            ),
        ] {
            let reference: serde_json::Value = serde_json::from_str(source).unwrap();
            let mut count = 0;
            for row in reference["records"].as_array().unwrap() {
                let input = row["input"].as_u64().unwrap() as usize;
                let cw = row["requested"].as_u64().unwrap() as u16;
                if !(if id == 89 {
                    [0x27f, 0x37f].contains(&cw) && input <= 9
                } else {
                    cw == 0x37f && (input <= 9 || input == 12)
                }) {
                    continue;
                }
                let (sig, exp) = inputs[input];
                let mut memory = Memory::new().unwrap();
                {
                    let mut state = memory.fpu.borrow_mut();
                    state.control = cw;
                    state.push(0x3fffc000000000000000);
                    state.push(((exp as u128) << 64) | sig as u128);
                }
                ci_math(&mut memory, id)
                    .unwrap_or_else(|e| panic!("id={id} input={input} cw={cw:x}: {e}"));
                let mut state = memory.fpu.borrow_mut();
                let value = state.get(0);
                let result = value.to_le_bytes()[..10]
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                assert_eq!(
                    result,
                    row["result80"].as_str().unwrap(),
                    "result id={id} input={input} cw={cw:x}"
                );
                assert_eq!(
                    state.status as u64,
                    row["sw_after"].as_u64().unwrap(),
                    "status id={id} input={input} cw={cw:x}"
                );
                assert_eq!(state.control, cw);
                assert_eq!(state.get(1), 0x3fffc000000000000000);
                count += 1;
            }
            assert_eq!(count, if id == 89 { 20 } else { 11 });
        }
    }
}

#[cfg(test)]
mod constants_tests {
    use crate::{Stop, tests::cpu};
    #[test]
    fn pi_uses_control_rounding_ignores_precision_and_preserves_sticky_flags() {
        for pc in [0, 2, 3] {
            for rc in 0..4 {
                let mut r = cpu(&[0xd9, 0xeb, 0xc3]);
                r.memory.fpu.borrow_mut().control = 0x7f | (pc << 8) | (rc << 10);
                r.memory.fpu.borrow_mut().status = 0x220;
                assert!(matches!(r.run(1), Stop::Budget { .. }));
                let fp = r.memory.fpu.borrow();
                assert_eq!(
                    fp.registers[7],
                    0x4000c90fdaa22168c234 + u128::from(rc == 0 || rc == 2)
                );
                assert_eq!(fp.status, 0x3820);
                assert_eq!(fp.tag, 0x3fff);
            }
        }
    }
    #[test]
    #[ignore = "run tools/check_jit32_constants.py for actual x86 observations"]
    fn actual_x86_constant_observations() {
        let data = std::fs::read(
            std::env::var_os("ALKYHOL_X87_CONSTANT_REFERENCE")
                .expect("actual x86 observer output required"),
        )
        .unwrap();
        assert_eq!(data.len(), 84 * 22);
        for row in data.chunks_exact(22) {
            let op = u32::from_le_bytes(row[0..4].try_into().unwrap());
            let cw = u32::from_le_bytes(row[4..8].try_into().unwrap());
            let sw = u32::from_le_bytes(row[8..12].try_into().unwrap());
            let mut bits = [0; 16];
            bits[..10].copy_from_slice(&row[12..]);
            let bits = u128::from_le_bytes(bits);
            let opcode = [0xeb, 0xe9, 0xea, 0xec, 0xed, 0xe8, 0xee][op as usize];
            let mut r = cpu(&[0xd9, opcode, 0xc3]);
            r.memory.fpu.borrow_mut().control = cw as u16;
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            let state = r.memory.fpu.borrow();
            assert_eq!(state.registers[7], bits, "op={op} cw={cw:x}");
            assert_eq!(state.status, sw as u16, "op={op} cw={cw:x}");
        }
    }
}

#[cfg(test)]
mod examine_tests {
    use crate::{Stop, tests::cpu};

    #[test]
    fn examines_raw_classes_empty_sign_and_top_without_stack_or_exception_changes() {
        let cases = [
            (0, 0x4000),
            (1, 0x4400),
            (0x8000000000000000, 0x4400), // Pseudo-denormal.
            (0x3fff8000000000000000, 0x400),
            (0x7fff8000000000000000, 0x500),
            (0x7fffc000000000000001, 0x100),
            (0x7fff8000000000000001, 0x100), // Signaling NaN: no invalid exception.
            (0x00010000000000000001, 0),
            (0x7fff0000000000000000, 0),
        ];
        for (bits, class) in cases {
            for sign in 0..2 {
                for top in 0..8 {
                    for tag in 0..4 {
                        let mut r = cpu(&[0xd9, 0xe5, 0xc3]);
                        {
                            let mut f = r.memory.fpu.borrow_mut();
                            f.control = 0x340; // Unmasked exceptions, none pending.
                            f.status = (top << 11) | 0x4700;
                            f.tag = (0xffff & !(3 << (top * 2))) | (tag << (top * 2));
                            f.registers[top as usize] = bits | (sign as u128) << 79;
                        }
                        let before = r.memory.fpu.borrow().clone();
                        let regs = r.state;
                        assert!(matches!(r.run(1), Stop::Budget { .. }));
                        let f = r.memory.fpu.borrow();
                        assert_eq!(
                            f.status,
                            (top << 11) | (sign << 9) | if tag == 3 { 0x4100 } else { class }
                        );
                        assert_eq!(f.control, before.control);
                        assert_eq!(f.tag, before.tag);
                        assert_eq!(f.registers, before.registers);
                        assert_eq!(f.mxcsr, before.mxcsr);
                        assert_eq!(&r.state[..17], &regs[..17]);
                    }
                }
            }
        }
    }

    #[test]
    #[ignore = "run tools/check_jit32_examine.py for actual x86 observations"]
    fn actual_x86_examine_observations() {
        let data = std::fs::read(
            std::env::var_os("ALKYHOL_X87_EXAMINE_REFERENCE")
                .expect("actual observer output required"),
        )
        .unwrap();
        assert_eq!(data.len(), 768 * 32);
        for (i, row) in data.chunks_exact(32).enumerate() {
            let mut raw = [0; 16];
            raw[..10].copy_from_slice(&row[..10]);
            let bits = u128::from_le_bytes(raw);
            let top = row[10] as usize;
            let empty = row[11] != 0;
            let mut r = cpu(&[0xd9, 0xe5, 0xc3]);
            {
                let mut f = r.memory.fpu.borrow_mut();
                f.status = u16::from_le_bytes(row[12..14].try_into().unwrap());
                f.tag = if empty {
                    0xffff
                } else {
                    0xffff & !(3 << (top * 2))
                };
                f.registers[top] = bits;
            }
            let before = r.memory.fpu.borrow().clone();
            let regs = r.state;
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            let f = r.memory.fpu.borrow();
            assert_eq!(
                f.status,
                u16::from_le_bytes(row[14..16].try_into().unwrap()),
                "row={i} bits={bits:x}"
            );
            assert_eq!(&f.registers[top].to_le_bytes()[..10], &row[16..26]);
            assert_eq!(
                f.control,
                u16::from_le_bytes(row[26..28].try_into().unwrap())
            );
            assert_eq!(row[28], if empty { 0 } else { 1 << top });
            assert_eq!(f.tag, before.tag);
            assert_eq!(f.registers, before.registers);
            assert_eq!(&r.state[..17], &regs[..17]);
        }
    }
}
