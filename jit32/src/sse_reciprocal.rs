//! Deterministic RCPSS/RCPPS profile within Intel's 3/8192 estimate bound.
//! Finite estimate bits are not a particular CPU's approximation table.
use rustc_apfloat::{Float, Round, ieee::Single};
pub(super) fn rcp(bits: u32) -> u32 {
    let sign = bits & 0x80000000;
    let exponent = bits & 0x7f800000;
    let fraction = bits & 0x007fffff;
    if exponent == 0x7f800000 {
        return if fraction == 0 {
            sign
        } else {
            bits | 0x00400000
        };
    }
    if exponent == 0 {
        return sign | 0x7f800000; // Denormals count as signed zero, regardless of DAZ.
    }
    let result = Single::from_bits(0x3f800000)
        .div_r(Single::from_bits(bits as u128), Round::NearestTiesToEven)
        .value
        .to_bits() as u32;
    // RCP always flushes tiny results, regardless of MXCSR.FTZ/RC/masks, and
    // signals no SIMD floating exception. Software division flags stay private.
    if result & 0x7f800000 == 0 {
        sign
    } else {
        result
    }
}
#[cfg(test)]
fn within_bound(input: u32, output: u32) -> bool {
    if input >> 31 != output >> 31 {
        return false;
    }
    let xe = ((input >> 23) & 255) as i32;
    let ye = ((output >> 23) & 255) as i32;
    if !(1..255).contains(&xe) {
        return false;
    }
    let x = ((input & 0x7fffff) | 0x800000) as u128;
    if output & 0x7fffffff == 0 {
        // A bounded pre-flush estimate may be tiny near the min-normal
        // boundary. Zero is permitted only if x * min_normal >= 8189/8192.
        let shift = 263 - xe;
        return (0..=100).contains(&shift) && x >= (8189u128 << shift);
    }
    if !(1..255).contains(&ye) {
        return false;
    }
    let y = ((output & 0x7fffff) | 0x800000) as u128;
    let shift = 287 - xe - ye;
    (0..=100).contains(&shift) && (8189u128 << shift) <= x * y && x * y <= (8195u128 << shift)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_special_values_and_tiny_result_boundaries_are_explicit() {
        for (x, y) in [
            (0, 0x7f800000),
            (1, 0x7f800000),
            (0x007fffff, 0x7f800000),
            (0x7f800000, 0),
            (0x7f801234, 0x7fc01234),
            (0x7fc05678, 0x7fc05678),
            (0x3f800000, 0x3f800000),
            (0x40000000, 0x3f000000),
            (0x00800000, 0x7e800000),
            (0x7e800000, 0x00800000),
            (0x7e800001, 0),
            (0x7f7fffff, 0),
        ] {
            for sign in [0, 0x80000000] {
                assert_eq!(rcp(x | sign), y | sign, "{x:x}");
            }
        }
        assert!(!within_bound(0x3f800000, 0));
        assert!(!within_bound(0x3f800000, 0x40000000));
        for sign in [0, 0x80000000] {
            for (result, allowed) in [
                (0x3f7fe7ff, false),
                (0x3f7fe800, true),
                (0x3f800c00, true),
                (0x3f800c01, false),
            ] {
                assert_eq!(within_bound(0x3f800000 | sign, result | sign), allowed);
            }
        }
        assert!(!within_bound(0x3f800000, 0xbf800000));
        assert!(!within_bound(0x7e7fe7ff, 0));
        assert!(within_bound(0x7e800000, 0));
        assert!(within_bound(0x7e800000, 0x00800000));
    }
    #[test]
    fn finite_estimates_meet_an_exact_integer_bound_including_flush_transition() {
        let mut random = 0x12345678u32;
        for exponent in 1..255 {
            for fraction in [
                0, 1, 0x3fffff, 0x400000, 0x400001, 0x7fe7ff, 0x7fe800, 0x7ffffe, 0x7fffff,
            ] {
                for sign in [0, 0x80000000] {
                    let x = sign | (exponent << 23) | fraction;
                    assert!(within_bound(x, rcp(x)), "{x:x}");
                }
            }
        }
        for _ in 0..65536 {
            random = random.wrapping_mul(1664525).wrapping_add(1013904223);
            let x = (random & 0x7fffffff).clamp(0x00800000, 0x7f7fffff) | (random & 0x80000000);
            assert!(within_bound(x, rcp(x)), "{x:x}");
        }
    }
}

#[cfg(test)]
mod native_tests {
    use super::*;
    use crate::{
        Stop,
        memory::{READ, WRITE},
        tests::cpu,
    };
    #[test]
    fn scalar_packed_alias_and_memory_forms_preserve_environment_and_unrelated_state() {
        for packed in [false, true] {
            for alias in [false, true] {
                for memory in [false, true] {
                    for rc in 0..4 {
                        for control in [0, 0x1f80, 0x8040, 0x9fc0] {
                            let mut code = vec![];
                            if !packed {
                                code.push(0xf3);
                            }
                            code.extend([
                                0x0f,
                                0x53,
                                if memory {
                                    0
                                } else if alias {
                                    0xc0
                                } else {
                                    0xc1
                                },
                            ]);
                            let mut r = cpu(&code);
                            r.state[0] = 0x20000;
                            r.state[16] = 0xad7;
                            let source = [0xc0800000u32, 0x7f801234, 1, 0x7f7fffff];
                            let bytes: Vec<u8> =
                                source.iter().flat_map(|v| v.to_le_bytes()).collect();
                            r.state[24] = 0x1122334455667788;
                            r.state[25] = 0xabcdef0123456789;
                            let slot = if alias { 24 } else { 26 };
                            r.state[slot] = u64::from_le_bytes(bytes[..8].try_into().unwrap());
                            r.state[slot + 1] = u64::from_le_bytes(bytes[8..].try_into().unwrap());
                            r.memory.map(0x20000, 16, READ | WRITE, &bytes).unwrap();
                            let mxcsr = control | (rc << 13) | 0x21;
                            r.memory.fpu.borrow_mut().mxcsr = mxcsr;
                            let fpu = r.memory.fpu.borrow().clone();
                            let before = r.state;
                            assert!(matches!(r.run(1), Stop::Budget { .. }));
                            let old = (before[24] as u128) | ((before[25] as u128) << 64);
                            let mut expected = old;
                            for lane in 0..if packed { 4 } else { 1 } {
                                expected = (expected & !(0xffffffffu128 << (lane * 32)))
                                    | ((rcp(source[lane]) as u128) << (lane * 32));
                            }
                            assert_eq!(
                                (r.state[24], r.state[25]),
                                (expected as u64, (expected >> 64) as u64)
                            );
                            assert_eq!(&r.state[..17], &before[..17]);
                            assert_eq!(&r.state[18..24], &before[18..24]);
                            assert_eq!(&r.state[26..], &before[26..]);
                            assert_eq!(r.state[17], before[17] + code.len() as u64);
                            let after = r.memory.fpu.borrow();
                            assert_eq!(after.mxcsr, mxcsr);
                            assert_eq!(
                                (after.control, after.status, after.tag, after.registers),
                                (fpu.control, fpu.status, fpu.tag, fpu.registers)
                            );
                        }
                    }
                }
            }
        }
    }
    #[test]
    fn memory_alignment_extent_and_read_rights_are_checked_before_any_commit() {
        for (packed, address, length, rights) in [
            (false, 0x20001, 4, READ),
            (true, 0x20001, 16, READ),
            (true, 0x20000, 8, READ),
            (false, 0x20000, 3, READ),
            (false, 0x20000, 4, WRITE),
        ] {
            let mut code = vec![];
            if !packed {
                code.push(0xf3);
            }
            code.extend([0x0f, 0x53, 0]);
            let mut r = cpu(&code);
            r.state[0] = address;
            r.state[24] = 0xabcdef;
            r.state[25] = 0x123456;
            r.memory
                .map(address as u32, length, rights, &vec![0; length])
                .unwrap();
            let before = r.state;
            let mxcsr = r.memory.fpu.borrow().mxcsr;
            let stop = r.run(1);
            if !packed && length == 4 && rights == READ {
                assert!(matches!(stop, Stop::Budget { .. }));
                assert_eq!(r.state[24] as u32, 0x7f800000);
            } else {
                assert!(matches!(stop, Stop::Fault { .. } | Stop::CpuFault { .. }));
                assert_eq!(&r.state[..17], &before[..17]);
                assert_eq!(&r.state[18..], &before[18..]);
            }
            assert_eq!(r.memory.fpu.borrow().mxcsr, mxcsr);
        }
    }
}

#[cfg(test)]
mod observations {
    use super::*;
    use crate::{Stop, tests::cpu};
    #[test]
    #[ignore = "run tools/check_jit32_rcp.py with an actual frozen native observation"]
    fn actual_x86_reciprocal_observations() {
        let raw = std::fs::read(std::env::var_os("ALKYHOL_RCP_REFERENCE").unwrap()).unwrap();
        assert_eq!(raw.len(), 67584 * 72);
        let mut lanes = 0u64;
        let mut different = 0u64;
        let mut flush_differences = 0u64;
        let mut load_differences = 0u64;
        let mut control_differences = 0u64;
        let mut flag_differences = 0u64;
        for row in raw.chunks_exact(72) {
            let w: Vec<u32> = row
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            assert!(w[0] < 2);
            let packed = w[0] != 0;
            let mut code = vec![];
            if !packed {
                code.push(0xf3);
            }
            code.extend([0x0f, 0x53, 0xc1]);
            let mut r = cpu(&code);
            for (slot, bytes) in [(24, &row[16..32]), (26, &row[32..48])] {
                r.state[slot] = u64::from_le_bytes(bytes[..8].try_into().unwrap());
                r.state[slot + 1] = u64::from_le_bytes(bytes[8..].try_into().unwrap());
            }
            r.state[16] = w[16] as u64;
            // Isolate RCP from the separately observed LDMXCSR operation. Some
            // translators do not load requested mask bits; retain that count.
            r.memory.fpu.borrow_mut().mxcsr = w[17];
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            let out = (r.state[24] as u128) | ((r.state[25] as u128) << 64);
            for lane in 0..4 {
                let model = (out >> (lane * 32)) as u32;
                let actual = w[12 + lane];
                let x = w[8 + lane];
                if packed || lane == 0 {
                    if (1..255).contains(&((x >> 23) & 255)) {
                        assert!(within_bound(x, model), "model input={x:x} result={model:x}");
                        assert!(
                            within_bound(x, actual),
                            "native input={x:x} result={actual:x}"
                        );
                        lanes += 1;
                        different += u64::from(model != actual);
                        flush_differences +=
                            u64::from((model & 0x7fffffff == 0) != (actual & 0x7fffffff == 0));
                    } else {
                        assert_eq!(model, actual, "special op={} lane={lane} input={x:x}", w[0]);
                    }
                } else {
                    assert_eq!(model, actual, "scalar preserved lane={lane}");
                }
            }
            assert_eq!(r.memory.fpu.borrow().mxcsr, w[17]);
            assert_eq!(r.flags() & 0x8d5, w[16] & 0x8d5);
            load_differences += u64::from(w[1] != w[17]);
            control_differences += u64::from(w[2] != w[17]);
            flag_differences += u64::from(w[3] & 0x8d5 != w[16] & 0x8d5);
        }
        let report = format!(
            "{{\"rows\":67584,\"normal_lanes\":{lanes},\"different_normal_bits\":{different},\"different_flush_choices\":{flush_differences},\"requested_mxcsr_load_differences\":{load_differences},\"instruction_mxcsr_differences\":{control_differences},\"flags_differences\":{flag_differences},\"normal_bit_equality_required\":false}}"
        );
        if let Some(path) = std::env::var_os("ALKYHOL_RCP_REPORT") {
            std::fs::write(path, &report).unwrap();
        }
        eprintln!("RCP observations: {report}");
        assert_eq!(lanes, 1045 * 5 * 32); // 1056 inputs minus 11 special values.
        assert_eq!(control_differences, 0);
        assert_eq!(flag_differences, 0);
    }
}
