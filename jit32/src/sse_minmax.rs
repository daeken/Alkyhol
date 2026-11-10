//! Legacy SSE minimum/maximum selection; guest MXCSR only, never host FP state.
use rustc_apfloat::{
    Float,
    ieee::{Double, Single},
};
use std::cmp::Ordering;
fn lane<F: Float>(
    a: u128,
    b: u128,
    maximum: bool,
    mxcsr: u32,
) -> Result<(u128, u32), &'static str> {
    let (mut a, mut b) = (F::from_bits(a), F::from_bits(b));
    if mxcsr & 0x40 != 0 {
        if a.is_denormal() {
            a = F::ZERO.copy_sign(a);
        }
        if b.is_denormal() {
            b = F::ZERO.copy_sign(b);
        }
    }
    let unordered = a.is_nan() || b.is_nan();
    let flags = if unordered {
        1
    } else if a.is_denormal() || b.is_denormal() {
        2
    } else {
        0
    };
    if flags & !(mxcsr >> 7) & 0x3f != 0 {
        return Err("unmasked SIMD minimum/maximum exception requires #XM delivery");
    }
    let select_first = a.partial_cmp(&b)
        == Some(if maximum {
            Ordering::Greater
        } else {
            Ordering::Less
        });
    // Equality (including opposite signed zeros) and unordered choose operand2.
    // A selected signaling NaN remains signaling; FTZ/RC do not round a copy.
    Ok((if select_first { a } else { b }.to_bits(), mxcsr | flags))
}
pub(super) fn select(
    width: u32,
    a: u128,
    b: u128,
    maximum: bool,
    mxcsr: u32,
) -> Result<(u128, u32), &'static str> {
    match width {
        32 => lane::<Single>(a, b, maximum, mxcsr),
        64 => lane::<Double>(a, b, maximum, mxcsr),
        _ => Err("unsupported SIMD minimum/maximum width"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn zero_ties_nan_operand_order_and_no_quieting_are_explicit() {
        for (width, sign, one, negative, qnan, snan) in [
            (
                32,
                0x80000000u128,
                0x3f800000,
                0xbf800000,
                0x7fc01234,
                0x7f801234,
            ),
            (
                64,
                0x8000000000000000,
                0x3ff0000000000000,
                0xbff0000000000000,
                0x7ff8000000001234,
                0x7ff0000000001234,
            ),
        ] {
            for max in [false, true] {
                for rc in 0..4 {
                    for ftz in [0, 0x8000] {
                        let m = 0x1f80 | (rc << 13) | ftz;
                        for (a, b, y, flags) in [
                            (0, sign, sign, 0),
                            (sign, 0, 0, 0),
                            (qnan, one, one, 1),
                            (one, snan, snan, 1),
                            (snan, qnan, qnan, 1),
                            (snan, negative, negative, 1),
                        ] {
                            assert_eq!(select(width, a, b, max, m).unwrap(), (y, m | flags));
                        }
                        assert_eq!(
                            select(width, one, negative, max, m).unwrap().0,
                            if max { one } else { negative }
                        );
                    }
                }
            }
        }
        assert!(select(16, 0, 0, false, 0x1f80).is_err());
    }
    #[test]
    fn denormal_inputs_use_daz_not_ftz_and_nan_suppresses_pair_denormal_flags() {
        for width in [32, 64] {
            for maximum in [false, true] {
                for ftz in [0, 0x8000] {
                    let m = 0x1f80 | ftz;
                    assert_eq!(select(width, 1, 1, maximum, m).unwrap(), (1, m | 2));
                    assert_eq!(
                        select(width, 1, 1, maximum, m | 0x40).unwrap(),
                        (0, m | 0x40)
                    );
                    let nan = if width == 32 {
                        0x7fc01234
                    } else {
                        0x7ff8000000001234
                    };
                    assert_eq!(select(width, nan, 1, maximum, m).unwrap(), (1, m | 1));
                    assert_eq!(
                        select(width, nan, 1, maximum, m | 0x40).unwrap(),
                        (0, m | 0x40 | 1)
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod native_tests {
    use crate::{
        Stop,
        memory::{READ, WRITE},
        tests::cpu,
    };
    fn bytes(form: u8, maximum: bool, modrm: u8) -> Vec<u8> {
        let mut code = match form {
            0 => vec![0xf3],
            1 => vec![0xf2],
            2 => vec![],
            _ => vec![0x66],
        };
        code.extend([0x0f, if maximum { 0x5f } else { 0x5d }, modrm]);
        code
    }
    fn vector(values: &[u64], wide: bool) -> u128 {
        values.iter().enumerate().fold(0, |a, (lane, v)| {
            a | ((*v as u128) << (lane * if wide { 64 } else { 32 }))
        })
    }
    #[test]
    fn all_legacy_forms_register_routes_aliases_and_scalar_upper_lanes_are_exact() {
        for form in 0..4 {
            for maximum in [false, true] {
                for dst in 0..8 {
                    for src in 0..8 {
                        let code = bytes(form, maximum, 0xc0 | (dst << 3) | src);
                        let mut r = cpu(&code);
                        let wide = form & 1 != 0;
                        let old = if wide {
                            vector(&[0x4000000000000000, 0x8000000000000000], true)
                        } else {
                            vector(&[0x40000000, 0xbf800000, 0, 0x80000000], false)
                        };
                        let source = if wide {
                            vector(&[0x3ff0000000000000, 0], true)
                        } else {
                            vector(&[0x3f800000, 0xc0000000, 0x80000000, 0], false)
                        };
                        let selected = if wide {
                            vector(
                                &[
                                    if maximum {
                                        0x4000000000000000
                                    } else {
                                        0x3ff0000000000000
                                    },
                                    0,
                                ],
                                true,
                            )
                        } else {
                            vector(
                                &[
                                    if maximum { 0x40000000 } else { 0x3f800000 },
                                    if maximum { 0xbf800000 } else { 0xc0000000 },
                                    0x80000000,
                                    0,
                                ],
                                false,
                            )
                        };
                        let d = 24 + dst as usize * 2;
                        let s = 24 + src as usize * 2;
                        r.state[d] = old as u64;
                        r.state[d + 1] = (old >> 64) as u64;
                        r.state[s] = source as u64;
                        r.state[s + 1] = (source >> 64) as u64;
                        r.state[16] = 0xad7;
                        {
                            let mut f = r.memory.fpu.borrow_mut();
                            f.control = 0x027f;
                            f.status = 0x3801;
                            f.tag = 0xf0f0;
                            f.registers[0] = 0x3fff8000000000000000;
                        }
                        let fpu = r.memory.fpu.borrow().clone();
                        let before = r.state;
                        let mut expected = before;
                        let result = if dst == src {
                            source
                        } else if form < 2 {
                            let mask = if wide {
                                u64::MAX as u128
                            } else {
                                u32::MAX as u128
                            };
                            (old & !mask) | (selected & mask)
                        } else {
                            selected
                        };
                        expected[d] = result as u64;
                        expected[d + 1] = (result >> 64) as u64;
                        expected[17] += code.len() as u64;
                        assert!(matches!(r.run(1), Stop::Budget { .. }));
                        assert_eq!(
                            r.state, expected,
                            "form={form} max={maximum} dst={dst} src={src}"
                        );
                        let after = r.memory.fpu.borrow();
                        assert_eq!(
                            (
                                after.control,
                                after.status,
                                after.tag,
                                after.mxcsr,
                                after.registers
                            ),
                            (fpu.control, fpu.status, fpu.tag, fpu.mxcsr, fpu.registers)
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn actual_minps_routing_memory_faults_and_unmasked_failures_do_not_partially_commit() {
        for form in 0..4 {
            for maximum in [false, true] {
                let width = if form & 1 == 0 { 4 } else { 8 };
                let length = if form >= 2 { 16 } else { width };
                for (address, available, rights, works) in [
                    (0x20000u64, length, READ, true),
                    (0x20001, length, READ, form < 2),
                    (0x20000, length - 1, READ, false),
                    (0x20000, length, WRITE, false),
                ] {
                    let mut r = cpu(&bytes(form, maximum, 8));
                    r.state[0] = address;
                    r.state[26] = 0x4000000040000000;
                    r.state[27] = 0x4000000040000000;
                    r.memory
                        .map(address as u32, available, rights, &vec![0; available])
                        .unwrap();
                    let before = r.state;
                    let stop = r.run(1);
                    if works {
                        assert!(matches!(stop, Stop::Budget { .. }));
                    } else {
                        assert!(matches!(stop, Stop::Fault { .. } | Stop::CpuFault { .. }));
                        assert_eq!(&r.state[..17], &before[..17]);
                        assert_eq!(&r.state[18..], &before[18..]);
                    }
                }
                for denormal in [false, true] {
                    let mut r = cpu(&bytes(form, maximum, 0xc8));
                    let wide = form & 1 != 0;
                    let lane = if form >= 2 { 1 } else { 0 };
                    let value = if denormal {
                        1
                    } else if wide {
                        0x7ff8000000001234u128
                    } else {
                        0x7fc01234
                    };
                    let source = value << (lane * if wide { 64 } else { 32 });
                    r.state[24] = source as u64;
                    r.state[25] = (source >> 64) as u64;
                    let mxcsr = 0x1f80 & !if denormal { 0x100 } else { 0x80 };
                    r.memory.fpu.borrow_mut().mxcsr = mxcsr;
                    let before = r.state;
                    assert!(matches!(r.run(1), Stop::CpuFault { .. }));
                    assert_eq!(&r.state[..17], &before[..17]);
                    assert_eq!(&r.state[18..], &before[18..]);
                    assert_eq!(r.memory.fpu.borrow().mxcsr, mxcsr);
                }
            }
        }
    }
    #[test]
    #[ignore = "run tools/check_jit32_minmax.py with frozen actual x86 observations"]
    fn actual_x86_minmax_observations() {
        let raw = std::fs::read(std::env::var_os("ALKYHOL_MINMAX_REFERENCE").unwrap()).unwrap();
        assert_eq!(raw.len(), 65536 * 80);
        let mut result_differences = 0u64;
        let mut mxcsr_differences = 0u64;
        let mut flags_differences = 0u64;
        let mut loaded_differences = 0u64;
        let mut examples = vec![];
        for (index, row) in raw.chunks_exact(80).enumerate() {
            let w: Vec<u32> = row
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            let kind = w[0];
            assert!(kind < 8);
            assert_eq!(kind as usize, index / (16 * 512));
            assert_eq!(w[18] as usize, index % 512);
            assert_eq!(w[19] as usize, index / 512 % 16);
            let mut r = cpu(&bytes((kind / 2) as u8, kind & 1 != 0, 0xc1));
            for (slot, part) in [(24, &row[16..32]), (26, &row[32..48])] {
                r.state[slot] = u64::from_le_bytes(part[..8].try_into().unwrap());
                r.state[slot + 1] = u64::from_le_bytes(part[8..].try_into().unwrap());
            }
            r.state[16] = w[16] as u64;
            r.memory.fpu.borrow_mut().mxcsr = w[2];
            let source = [r.state[26], r.state[27]];
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            assert_eq!([r.state[26], r.state[27]], source);
            let expected = [
                u64::from_le_bytes(row[48..56].try_into().unwrap()),
                u64::from_le_bytes(row[56..64].try_into().unwrap()),
            ];
            let result_bad = [r.state[24], r.state[25]] != expected;
            let actual_mxcsr = r.memory.fpu.borrow().mxcsr;
            let control_bad = actual_mxcsr != w[3];
            let flags_bad = r.flags() & 0x8d5 != w[17] & 0x8d5;
            result_differences += u64::from(result_bad);
            mxcsr_differences += u64::from(control_bad);
            flags_differences += u64::from(flags_bad);
            loaded_differences += u64::from(w[1] != w[2]);
            if (result_bad || control_bad || flags_bad) && examples.len() < 12 {
                examples.push(format!("{{\"row\":{index},\"kind\":{kind},\"model_mxcsr\":{actual_mxcsr},\"observed_mxcsr\":{},\"result_differs\":{result_bad}}}",w[3]));
            }
        }
        let report = format!(
            "{{\"rows\":65536,\"result_differences\":{result_differences},\"mxcsr_differences\":{mxcsr_differences},\"flags_differences\":{flags_differences},\"requested_mxcsr_load_differences\":{loaded_differences},\"examples\":[{}]}}",
            examples.join(",")
        );
        if let Some(path) = std::env::var_os("ALKYHOL_MINMAX_REPORT") {
            std::fs::write(path, &report).unwrap();
        }
        eprintln!("SIMD minimum/maximum observations: {report}");
        assert_eq!(
            (result_differences, mxcsr_differences, flags_differences),
            (0, 0, 0)
        );
    }
}
