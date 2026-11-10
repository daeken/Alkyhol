//! Exact finite x87 FPREM/FPREM1 complete reductions (exponent difference <64).
//! Partial reductions and exceptional/noncanonical operands fail explicitly.
#[derive(Debug, PartialEq)]
pub(super) struct Outcome {
    pub value: u128,
    pub condition: u16,
    pub exceptions: u16,
}
struct Parts {
    significand: u128,
    exponent: i32,
    negative: bool,
    denormal: bool,
}
fn parts(bits: u128) -> Result<Parts, &'static str> {
    let exponent = (bits >> 64) & 0x7fff;
    let sig = bits as u64;
    if bits >> 80 != 0 || exponent == 0x7fff || (exponent != 0) != (sig >> 63 != 0) {
        return Err("nonfinite/noncanonical x87 remainder operand is not implemented");
    }
    let shift = if sig == 0 { 0 } else { sig.leading_zeros() };
    Ok(Parts {
        significand: (sig as u128) << shift,
        exponent: (if exponent == 0 {
            -16382
        } else {
            exponent as i32 - 16383
        }) - shift as i32,
        negative: bits >> 79 != 0,
        denormal: exponent == 0 && sig != 0,
    })
}
fn pack(sig: u128, exponent: i32, negative: bool) -> Result<u128, &'static str> {
    let sign = u128::from(negative) << 79;
    if sig == 0 {
        return Ok(sign);
    }
    if sig > u64::MAX as u128 {
        return Err("x87 remainder significand exceeded exact bound");
    }
    let shift = (sig as u64).leading_zeros();
    let sig = sig << shift;
    let exponent = exponent - shift as i32;
    if exponent >= -16382 {
        return Ok(sign | (((exponent + 16383) as u128) << 64) | sig);
    }
    let drop = (-16382 - exponent) as u32;
    if drop >= 64 || (sig >> drop) << drop != sig {
        return Err("x87 remainder is outside the exact denormal profile");
    }
    Ok(sign | (sig >> drop))
}
pub(super) fn remainder(x: u128, y: u128, nearest: bool) -> Result<Outcome, &'static str> {
    let (a, b) = (parts(x)?, parts(y)?);
    if b.significand == 0 {
        return Err("zero-divisor x87 remainder response is not implemented");
    }
    let exceptions = if a.denormal || b.denormal { 2 } else { 0 };
    let difference = a.exponent - b.exponent;
    if a.significand != 0 && difference >= 64 {
        return Err("partial x87 remainder reduction is not implemented");
    }
    let (quotient, value) = if a.significand == 0 || difference < 0 {
        if nearest && difference == -1 && a.significand > b.significand {
            (
                1,
                pack(
                    (b.significand << 1) - a.significand,
                    a.exponent,
                    !a.negative,
                )?,
            )
        } else {
            (0, x)
        }
    } else {
        let numerator = a.significand << difference as u32;
        let mut q = numerator / b.significand;
        let r = numerator % b.significand;
        let up = nearest && (r * 2 > b.significand || (r * 2 == b.significand && q & 1 != 0));
        let (sig, negative) = if up {
            q += 1;
            (b.significand - r, !a.negative)
        } else {
            (r, a.negative)
        };
        (q, pack(sig, b.exponent, negative)?)
    };
    // Observed x87 condition bits encode the magnitude quotient, independent
    // of either operand's sign. Native observations are kept separately.
    let q = (quotient & 7) as u16;
    Ok(Outcome {
        value,
        condition: ((q & 4) << 6) | ((q & 2) << 13) | ((q & 1) << 9),
        exceptions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustc_apfloat::{Float, ieee::X87DoubleExtended as X};
    #[test]
    fn quotient_bits_signs_ties_and_exact_zero_do_not_depend_on_precision() {
        for nearest in [false, true] {
            for sx in [-1i128, 1] {
                for sy in [-1i128, 1] {
                    for (x, y, q, r) in [
                        (
                            8,
                            3,
                            if nearest { 3 } else { 2 },
                            if nearest { -1 } else { 2 },
                        ),
                        (5, 2, 2, 1),
                        (
                            7,
                            2,
                            if nearest { 4 } else { 3 },
                            if nearest { -1 } else { 1 },
                        ),
                        (
                            3,
                            2,
                            if nearest { 2 } else { 1 },
                            if nearest { -1 } else { 1 },
                        ),
                        (6, 3, 2, 0),
                    ] {
                        let result = remainder(
                            X::from_i128(x * sx).value.to_bits(),
                            X::from_i128(y * sy).value.to_bits(),
                            nearest,
                        )
                        .unwrap();
                        let expected = if r == 0 && sx < 0 {
                            1u128 << 79
                        } else {
                            X::from_i128(r * sx).value.to_bits()
                        };
                        assert_eq!(result.value, expected);
                        assert_eq!(result.exceptions, 0);
                        assert_eq!(
                            result.condition,
                            ((q & 4) << 6) | ((q & 2) << 13) | ((q & 1) << 9)
                        );
                    }
                }
            }
        }
        for n in 0..=26u128 {
            let q = n / 3;
            let r = n % 3;
            let o = remainder(
                X::from_u128(n).value.to_bits(),
                X::from_u128(3).value.to_bits(),
                false,
            )
            .unwrap();
            assert_eq!(o.value, X::from_u128(r).value.to_bits());
            assert_eq!(
                o.condition,
                (((q as u16) & 4) << 6) | (((q as u16) & 2) << 13) | (((q as u16) & 1) << 9)
            );
        }
    }
    #[test]
    fn exact_denormals_extreme_complete_quotients_and_explicit_declines() {
        for nearest in [false, true] {
            assert_eq!(
                remainder(7, 3, nearest).unwrap(),
                Outcome {
                    value: 1,
                    condition: 0x4000,
                    exceptions: 2
                }
            );
            assert_eq!(
                remainder(
                    (0x403eu128 << 64) | u64::MAX as u128,
                    0x3fff8000000000000000,
                    nearest
                )
                .unwrap(),
                Outcome {
                    value: 0,
                    condition: 0x4300,
                    exceptions: 0
                }
            );
            assert_eq!(
                remainder(1u128 << 79, 0x3fff8000000000000000, nearest)
                    .unwrap()
                    .value,
                1u128 << 79
            );
            for (x, y) in [
                (0x403f8000000000000000, 0x3fff8000000000000000),
                (0x3fff8000000000000000, 0),
                (0x7fff8000000000000000, 0x3fff8000000000000000),
                (0x3fff0000000000000001, 0x3fff8000000000000000),
                (0x8000000000000000, 0x3fff8000000000000000),
            ] {
                assert!(remainder(x, y, nearest).is_err());
            }
        }
    }
}

#[cfg(test)]
mod native_tests {
    use crate::{Stop, tests::cpu};
    use rustc_apfloat::{Float, ieee::X87DoubleExtended as X};
    fn bytes(nearest: bool) -> [u8; 2] {
        [0xd9, if nearest { 0xf5 } else { 0xf8 }]
    }
    fn fields(s: &crate::fpu::State) -> (u16, u16, u16, u32, [u128; 8]) {
        (s.control, s.status, s.tag, s.mxcsr, s.registers)
    }
    #[test]
    fn native_routes_all_stack_rotations_controls_and_unrelated_state() {
        for nearest in [false, true] {
            for top in 0..8 {
                for pc in [0, 2, 3] {
                    for rc in 0..4 {
                        for ic in [0, 0x1000] {
                            let mut r = cpu(&bytes(nearest));
                            r.state[16] = 0xad7;
                            for n in [0usize, 1, 2, 3, 5, 6, 7] {
                                r.state[n] = 0x76543210 + (n as u64) * 0x10101;
                            }
                            for n in 24..40 {
                                r.state[n] = 0x123456789abcdef0 ^ ((n as u64) * 0x0101010101010101);
                            }
                            {
                                let mut s = r.memory.fpu.borrow_mut();
                                s.control = 0x7f | (pc << 8) | (rc << 10) | ic;
                                s.mxcsr = 0x9fe5;
                                s.set_top(top);
                                s.status |= 0x4723;
                                for i in 0..8 {
                                    s.set(i, X::from_i128(16 + i as i128).value.to_bits());
                                }
                                s.set(0, X::from_u128(8).value.to_bits());
                                s.set(1, X::from_u128(3).value.to_bits());
                            }
                            let before = r.state;
                            let mut expected = r.memory.fpu.borrow().clone();
                            let v = if nearest {
                                X::from_i128(-1).value.to_bits()
                            } else {
                                X::from_u128(2).value.to_bits()
                            };
                            expected.set(0, v);
                            expected.status =
                                (expected.status & !0x4700) | if nearest { 0x4200 } else { 0x4000 };
                            assert!(matches!(r.run(1), Stop::Budget { .. }));
                            let mut registers = before;
                            registers[17] += 2;
                            assert_eq!(r.state, registers);
                            assert_eq!(fields(&r.memory.fpu.borrow()), fields(&expected));
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn partial_exceptional_empty_reserved_and_unmasked_states_fail_without_commit() {
        for nearest in [false, true] {
            for kind in 0..6 {
                let mut r = cpu(&bytes(nearest));
                r.state[16] = 0xad7;
                {
                    let mut s = r.memory.fpu.borrow_mut();
                    s.set(0, 0x3fff8000000000000000);
                    s.set(1, 0x3fff8000000000000000);
                    s.status |= 0x4720;
                    match kind {
                        0 => s.set(0, 0x403f8000000000000000),
                        1 => s.set(1, 0),
                        2 => s.set(0, 0x7fff8000000000000000),
                        3 => s.tag |= 3,
                        4 => s.control = 0x017f,
                        _ => s.control &= !1,
                    };
                }
                let before = r.state;
                let fpu = r.memory.fpu.borrow().clone();
                assert!(matches!(r.run(1), Stop::CpuFault { .. }));
                assert_eq!(&r.state[..17], &before[..17]);
                assert_eq!(&r.state[18..], &before[18..]);
                assert_eq!(fields(&r.memory.fpu.borrow()), fields(&fpu));
            }
        }
    }
    #[test]
    #[ignore = "run tools/check_jit32_remainder.py with frozen actual x86 observations"]
    fn actual_x86_complete_remainder_observations() {
        let raw = std::fs::read(std::env::var_os("ALKYHOL_REMAINDER_REFERENCE").unwrap()).unwrap();
        assert_eq!(raw.len(), 24576 * 80);
        let mut result_differences = 0u64;
        let mut state_differences = 0u64;
        let mut flags_differences = 0u64;
        let mut examples = vec![];
        let word = |b: &[u8]| u16::from_le_bytes(b.try_into().unwrap());
        let dword = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap());
        let extended = |b: &[u8]| {
            let mut v = [0u8; 16];
            v[..10].copy_from_slice(b);
            u128::from_le_bytes(v)
        };
        for (index, row) in raw.chunks_exact(80).enumerate() {
            let kind = dword(&row[..4]);
            let mode = row[43];
            assert!(kind < 2);
            assert_eq!(kind as usize, index / (24 * 512));
            assert_eq!(mode as usize, index / 512 % 24);
            assert_eq!(dword(&row[68..72]) as usize, index % 512);
            let mut r = cpu(&bytes(kind == 1));
            r.state[16] = dword(&row[60..64]) as u64;
            {
                let mut s = r.memory.fpu.borrow_mut();
                s.control = dword(&row[4..8]) as u16;
                s.status = word(&row[58..60]);
                s.mxcsr = 0x1fa0;
                s.set(0, extended(&row[8..18]));
                s.set(1, extended(&row[18..28]));
                s.set(2, extended(&row[48..58]));
                s.tag = word(&row[74..76]);
                s.control = word(&row[76..78]);
                s.status = word(&row[78..80]);
            }
            for n in [0usize, 1, 2, 3, 5, 6, 7] {
                r.state[n] = 0x76543210 + (n as u64) * 0x10101;
            }
            for n in 0..16 {
                let mut bytes = [0u8; 8];
                for (j, b) in bytes.iter_mut().enumerate() {
                    *b = ((160 + n * 8 + j) as u8).wrapping_mul(37).wrapping_add(11);
                }
                r.state[24 + n] = u64::from_le_bytes(bytes);
            }
            let mut expected_cpu = r.state;
            expected_cpu[17] += 2;
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            assert_eq!(r.state, expected_cpu, "unrelated CPU state row={index}");
            let s = r.memory.fpu.borrow();
            let result_bad = s.registers[s.top()] != extended(&row[28..38]);
            if result_bad && result_differences < 12 {
                eprintln!(
                    "REMAINDER RESULT row={index} kind={kind} x={:020x} y={:020x} model={:020x} observed={:020x}",
                    extended(&row[8..18]),
                    extended(&row[18..28]),
                    s.registers[s.top()],
                    extended(&row[28..38])
                );
            }
            let state_bad = s.status != word(&row[38..40])
                || s.control != word(&row[40..42])
                || s.tag != word(&row[72..74])
                || s.mxcsr != dword(&row[44..48])
                || s.registers[(s.top() + 1) & 7] != extended(&row[18..28])
                || s.registers[(s.top() + 2) & 7] != extended(&row[48..58]);
            let flags_bad = r.flags() & 0x8d5 != dword(&row[64..68]) & 0x8d5;
            result_differences += u64::from(result_bad);
            state_differences += u64::from(state_bad);
            flags_differences += u64::from(flags_bad);
            if (result_bad || state_bad || flags_bad) && examples.len() < 12 {
                examples.push(format!("{{\"row\":{index},\"result_differs\":{result_bad},\"model_status\":{},\"observed_status\":{},\"model_tag\":{},\"observed_tag\":{}}}",s.status,word(&row[38..40]),s.tag,word(&row[72..74])));
            }
        }
        let report = format!(
            "{{\"rows\":24576,\"result_differences\":{result_differences},\"state_differences\":{state_differences},\"flags_differences\":{flags_differences},\"examples\":[{}]}}",
            examples.join(",")
        );
        if let Some(p) = std::env::var_os("ALKYHOL_REMAINDER_REPORT") {
            std::fs::write(p, &report).unwrap();
        }
        eprintln!("Complete x87 remainder observations: {report}");
        assert_eq!(
            (result_differences, state_differences, flags_differences),
            (0, 0, 0)
        );
    }
}
