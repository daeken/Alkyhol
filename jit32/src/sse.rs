//! Reached SSE/SSE2 scalar conversions with guest MXCSR and software rounding.
//! Intel SDM Vol. 1 §11.5: these integer conversions signal IE/PE, not DE.
use crate::memory::Memory;
use rustc_apfloat::{
    Float, Round, Status,
    ieee::{Double, Single},
};
pub fn conversion(name: &str) -> Option<(u64, u32, u32)> {
    Some(match name {
        "CVTTSD2SI" => (0, 64, 32),
        "CVTTSS2SI" => (1, 32, 32),
        "CVTSD2SI" => (2, 64, 32),
        "CVTSS2SI" => (3, 32, 32),
        "CVTSI2SD" => (4, 32, 64),
        "CVTSI2SS" => (5, 32, 32),
        _ => return None,
    })
}
fn round(mxcsr: u32) -> Round {
    match mxcsr >> 13 & 3 {
        0 => Round::NearestTiesToEven,
        1 => Round::TowardNegative,
        2 => Round::TowardPositive,
        _ => Round::TowardZero,
    }
}
fn integer<F: Float>(mut input: F, mxcsr: u32, truncate: bool) -> (u64, Status) {
    if mxcsr & 0x40 != 0 && input.is_denormal() {
        input = F::ZERO.copy_sign(input);
    }
    let result = input.to_i128_r(
        32,
        if truncate {
            Round::TowardZero
        } else {
            round(mxcsr)
        },
        &mut false,
    );
    if result.status.contains(Status::INVALID_OP) {
        (0x80000000, Status::INVALID_OP)
    } else {
        (result.value as u32 as u64, result.status)
    }
}
fn convert(op: u64, input: u64, mxcsr: u32) -> Result<(u64, u32), &'static str> {
    let (value, status) = match op {
        0 | 2 => integer(Double::from_bits(input as u128), mxcsr, op == 0),
        1 | 3 => integer(Single::from_bits(input as u32 as u128), mxcsr, op == 1),
        4 => {
            let r = Double::from_i128_r(input as i32 as i128, round(mxcsr));
            (r.value.to_bits() as u64, r.status)
        }
        5 => {
            let r = Single::from_i128_r(input as i32 as i128, round(mxcsr));
            (r.value.to_bits() as u64, r.status)
        }
        _ => return Err("unknown SSE conversion"),
    };
    let flags = u32::from(status.contains(Status::INVALID_OP))
        | (u32::from(status.contains(Status::INEXACT)) << 5);
    if flags & !(mxcsr >> 7) & 0x3f != 0 {
        return Err("unmasked SIMD conversion exception requires #XM delivery");
    }
    Ok((value, mxcsr | flags))
}
pub unsafe extern "C" fn execute(
    memory: *mut Memory,
    output: *mut u64,
    op: u64,
    input: u64,
) -> u64 {
    let memory = unsafe { &mut *memory };
    let mxcsr = memory.fpu.borrow().mxcsr;
    match convert(op, input, mxcsr) {
        Ok((value, mxcsr)) => {
            memory.fpu.borrow_mut().mxcsr = mxcsr;
            unsafe { output.write(value) };
            1
        }
        Err(e) => {
            memory.cpu_fault = Some(e);
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Stop, tests::cpu};
    #[test]
    fn scalar_integer_conversions_honor_mxcsr_and_preserve_other_state() {
        for (prefix, inputs) in [
            (
                0xf2,
                vec![
                    1.5f64.to_bits(),
                    (-1.5f64).to_bits(),
                    0,
                    1,
                    f64::INFINITY.to_bits(),
                    f64::NAN.to_bits(),
                    2147483648f64.to_bits(),
                    (-2147483648f64).to_bits(),
                ],
            ),
            (
                0xf3,
                vec![
                    1.5f32.to_bits() as u64,
                    (-1.5f32).to_bits() as u64,
                    0,
                    1,
                    f32::INFINITY.to_bits() as u64,
                    f32::NAN.to_bits() as u64,
                    2147483648f32.to_bits() as u64,
                    (-2147483648f32).to_bits() as u64,
                ],
            ),
        ] {
            for (index, bits) in inputs.into_iter().enumerate() {
                for rc in 0..4 {
                    for truncate in [true, false] {
                        let mut r =
                            cpu(&[prefix, 0x0f, if truncate { 0x2c } else { 0x2d }, 0xc1, 0xc3]);
                        r.state[26] = bits;
                        r.state[27] = 0x1234567812345678;
                        r.state[2] = 0xabcdef01;
                        r.state[16] = 0xad7;
                        r.memory.fpu.borrow_mut().mxcsr = 0x1f80 | (rc << 13);
                        let expected = match index {
                            0 => {
                                if truncate || [1, 3].contains(&rc) {
                                    1
                                } else {
                                    2
                                }
                            }
                            1 => {
                                if truncate || [2, 3].contains(&rc) {
                                    u32::MAX
                                } else {
                                    0xfffffffe
                                }
                            }
                            2 => 0,
                            3 => u32::from(!truncate && rc == 2),
                            _ => 0x80000000,
                        };
                        assert_eq!(
                            r.run(3),
                            Stop::Returned(expected),
                            "{prefix:x}/{index}/{rc}/{truncate}"
                        );
                        let flag = match index {
                            0 | 1 | 3 => 32,
                            4..=6 => 1,
                            _ => 0,
                        };
                        assert_eq!(r.memory.fpu.borrow().mxcsr, 0x1f80 | (rc << 13) | flag);
                        assert_eq!(
                            (r.state[2], r.state[26], r.state[27], r.state[16]),
                            (0xabcdef01, bits, 0x1234567812345678, 0xad7)
                        );
                    }
                }
            }
        }
        for (prefix, width) in [(0xf2, 64), (0xf3, 32)] {
            for rc in 0..4 {
                let mut r = cpu(&[prefix, 0x0f, 0x2a, 0xc0, 0xc3]);
                r.state[0] = 16777217;
                r.state[24] = 0x123456789abcdef0;
                r.state[25] = 0xfedcba9876543210;
                r.memory.fpu.borrow_mut().mxcsr = 0x1f80 | (rc << 13);
                assert_eq!(r.run(3), Stop::Returned(16777217));
                let expected = if width == 64 {
                    16777217f64.to_bits()
                } else {
                    0x1234567800000000 | if rc == 2 { 0x4b800001 } else { 0x4b800000 }
                };
                assert_eq!(r.state[24], expected);
                assert_eq!(r.state[25], 0xfedcba9876543210);
                assert_eq!(
                    r.memory.fpu.borrow().mxcsr,
                    0x1f80 | (rc << 13) | if width == 32 { 32 } else { 0 }
                );
            }
        }
    }
    #[test]
    fn scalar_conversion_faults_do_not_commit_the_destination() {
        let mut r = cpu(&[0xf2, 0x0f, 0x2c, 0xc1, 0xc3]);
        r.state[0] = 0x12345678;
        r.state[26] = 1.5f64.to_bits();
        r.memory.fpu.borrow_mut().mxcsr = 0xf80;
        assert!(matches!(r.run(3), Stop::CpuFault { pc: 0x10000, .. }));
        assert_eq!(r.state[0], 0x12345678);
        let mut r = cpu(&[0xf2, 0x0f, 0x2c, 0x05, 0, 0, 2, 0, 0xc3]);
        r.state[0] = 0x12345678;
        assert!(matches!(r.run(3), Stop::Fault { pc: 0x10000, .. }));
        assert_eq!(r.state[0], 0x12345678);
        let mut r = cpu(&[0xf2, 0x0f, 0x2c, 0x05, 0, 0, 2, 0, 0xc3]);
        r.memory
            .map(
                0x20000,
                8,
                crate::memory::READ,
                &(-5.75f64).to_bits().to_le_bytes(),
            )
            .unwrap();
        assert_eq!(r.run(3), Stop::Returned((-5i32) as u32));
    }
}

#[cfg(test)]
mod reference {
    use crate::{Stop, tests::cpu};
    #[test]
    #[ignore = "run tools/check_jit32_sse.py to obtain actual Rosetta observations"]
    fn actual_x86_conversion_observations() {
        let data = std::fs::read(
            std::env::var_os("ALKYHOL_SSE_REFERENCE").expect("actual x86 observer output required"),
        )
        .unwrap();
        assert_eq!(data.len(), 512 * 28);
        for row in data.chunks_exact(28) {
            let words: Vec<u32> = row
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            let op = words[0];
            let prefix = if [1, 3, 5].contains(&op) { 0xf3 } else { 0xf2 };
            let opcode = match op {
                0 | 1 => 0x2c,
                2 | 3 => 0x2d,
                4 | 5 => 0x2a,
                _ => panic!("unexpected observer operation"),
            };
            let mut r = cpu(&[prefix, 0x0f, opcode, 0xc1, 0xc3]);
            let input = words[2] as u64 | ((words[3] as u64) << 32);
            if op < 4 {
                r.state[26] = input;
            } else {
                r.state[1] = input;
            }
            r.memory.fpu.borrow_mut().mxcsr = words[1];
            assert!(matches!(r.run(3), Stop::Returned(_)), "{words:x?}");
            let actual = if op < 4 { r.state[0] } else { r.state[24] };
            let expected = words[4] as u64 | ((words[5] as u64) << 32);
            assert_eq!(actual, expected, "{words:x?}");
            assert_eq!(r.memory.fpu.borrow().mxcsr, words[6], "{words:x?}");
        }
    }
}
