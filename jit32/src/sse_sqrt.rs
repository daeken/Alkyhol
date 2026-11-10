//! Correctly rounded legacy SSE square roots from an exact integer radicand.
//! Guest MXCSR supplies rounding/DAZ; no host floating-point operations are used.
fn finish(bits: u128, flags: u32, mxcsr: u32) -> Result<(u128, u32), &'static str> {
    if flags & !(mxcsr >> 7) & 0x3f != 0 {
        return Err("unmasked SIMD square-root exception requires #XM delivery");
    }
    Ok((bits, mxcsr | flags))
}
pub(super) fn sqrt(width: u32, bits: u128, mxcsr: u32) -> Result<(u128, u32), &'static str> {
    let (fraction_bits, exponent_bits, bias) = match width {
        32 => (23u32, 8, 127i32),
        64 => (52u32, 11, 1023i32),
        _ => return Err("unsupported SIMD square-root width"),
    };
    let sign = 1u128 << (width - 1);
    let fraction_mask = (1u128 << fraction_bits) - 1;
    let exponent_mask = (1u128 << exponent_bits) - 1;
    let bits = bits & ((1u128 << width) - 1);
    let fraction = bits & fraction_mask;
    let exponent = (bits >> fraction_bits) & exponent_mask;
    let quiet = 1u128 << (fraction_bits - 1);
    let indefinite = sign | (exponent_mask << fraction_bits) | quiet;
    if exponent == exponent_mask && fraction != 0 {
        return finish(bits | quiet, u32::from(fraction & quiet == 0), mxcsr);
    }
    let denormal = exponent == 0 && fraction != 0;
    if exponent == 0 && (fraction == 0 || mxcsr & 0x40 != 0) {
        return finish(bits & sign, 0, mxcsr);
    }
    if bits & sign != 0 {
        return finish(indefinite, 1, mxcsr);
    }
    if exponent == exponent_mask {
        return finish(bits, 0, mxcsr);
    }
    let mut flags = if denormal { 2 } else { 0 };
    let mut sig = if denormal {
        fraction
    } else {
        fraction | (1u128 << fraction_bits)
    };
    let shift = sig.leading_zeros() - (127 - fraction_bits);
    sig <<= shift;
    let exponent = (if denormal {
        1 - bias
    } else {
        exponent as i32 - bias
    }) - shift as i32;
    let radicand = sig << (fraction_bits + (exponent & 1) as u32);
    let mut root = radicand.isqrt();
    let remainder = radicand - root * root;
    if remainder != 0 {
        flags |= 32;
    }
    let increment = match mxcsr >> 13 & 3 {
        // The squared midpoint is root² + root + 1/4. An integer radicand
        // cannot tie, so remainder > root is the exact nearest criterion.
        0 => remainder > root,
        2 => remainder != 0,
        _ => false,
    };
    if increment {
        root += 1;
    }
    let mut output_exponent = exponent.div_euclid(2) + bias;
    if root == 1u128 << (fraction_bits + 1) {
        root >>= 1;
        output_exponent += 1;
    }
    // Every positive finite binary32/64 input has a finite, normal root.
    // Neither FTZ nor underflow/overflow rounding can affect this result.
    finish(
        ((output_exponent as u128) << fraction_bits) | (root & fraction_mask),
        flags,
        mxcsr,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_roots_denormal_normalization_and_directed_rounding() {
        for (width, input, nearest, lower, upper, flags) in [
            (32, 0x40800000u128, 0x40000000, 0x40000000, 0x40000000, 0),
            (32, 0x40000000, 0x3fb504f3, 0x3fb504f3, 0x3fb504f4, 32),
            (32, 1, 0x1a3504f3, 0x1a3504f3, 0x1a3504f4, 34),
            (32, 0x00800000, 0x20000000, 0x20000000, 0x20000000, 0),
            (
                64,
                0x4000000000000000,
                0x3ff6a09e667f3bcd,
                0x3ff6a09e667f3bcc,
                0x3ff6a09e667f3bcd,
                32,
            ),
            (
                64,
                1,
                0x1e60000000000000,
                0x1e60000000000000,
                0x1e60000000000000,
                2,
            ),
        ] {
            for (rc, expected) in [nearest, lower, upper, lower].into_iter().enumerate() {
                for ftz in [0, 0x8000] {
                    let control = 0x1f80 | ((rc as u32) << 13) | ftz | 4;
                    assert_eq!(
                        sqrt(width, input, control).unwrap(),
                        (expected, control | flags)
                    );
                }
            }
        }
    }
    #[test]
    fn signed_special_values_daz_and_masked_exception_priority() {
        for (width, sign, infinity, qnan, snan) in [
            (32, 0x80000000u128, 0x7f800000, 0x7fc12345, 0x7f812345),
            (
                64,
                0x8000000000000000,
                0x7ff0000000000000,
                0x7ff8000000012345,
                0x7ff0000000012345,
            ),
        ] {
            let quiet = qnan ^ snan;
            for s in [0, sign] {
                assert_eq!(sqrt(width, s, 0x1f80).unwrap(), (s, 0x1f80));
                assert_eq!(sqrt(width, s | qnan, 0x1f80).unwrap(), (s | qnan, 0x1f80));
                assert_eq!(sqrt(width, s | snan, 0x1f80).unwrap(), (s | qnan, 0x1f81));
                assert_eq!(sqrt(width, s | 1, 0x1fc0).unwrap(), (s, 0x1fc0));
            }
            let indefinite = sign | infinity | quiet;
            assert_eq!(sqrt(width, sign | 1, 0x1f80).unwrap(), (indefinite, 0x1f81));
            assert_eq!(
                sqrt(width, sign | infinity, 0x1f80).unwrap(),
                (indefinite, 0x1f81)
            );
            assert_eq!(sqrt(width, infinity, 0x1f80).unwrap(), (infinity, 0x1f80));
            assert!(sqrt(width, sign | 1, 0x1f00).is_err());
            assert!(sqrt(width, snan, 0x1f00).is_err());
            assert!(sqrt(width, 1, 0x1e80).is_err());
        }
        assert!(sqrt(16, 0, 0x1f80).is_err());
    }
}

#[cfg(test)]
mod native_tests {
    use crate::{
        Stop,
        memory::{READ, WRITE},
        tests::cpu,
    };
    fn bytes(form: u8, modrm: u8) -> Vec<u8> {
        let mut b = vec![];
        match form {
            0 => b.push(0xf3),
            1 => b.push(0xf2),
            2 => {}
            3 => b.push(0x66),
            _ => panic!("form"),
        }
        b.extend([0x0f, 0x51, modrm]);
        b
    }
    fn exact_vector(wide: bool, roots: bool) -> u128 {
        if wide {
            if roots {
                0x40080000000000004000000000000000
            } else {
                0x40220000000000004010000000000000
            }
        } else if roots {
            0x40a00000408000004040000040000000
        } else {
            0x41c80000418000004110000040800000
        }
    }
    #[test]
    fn all_register_routes_aliases_scalar_high_lanes_and_unrelated_state() {
        for form in 0..4 {
            for dst in 0..8 {
                for src in 0..8 {
                    let code = bytes(form, 0xc0 | (dst << 3) | src);
                    let mut r = cpu(&code);
                    let d = 24 + dst as usize * 2;
                    let s = 24 + src as usize * 2;
                    let old = 0xdeadbeefcafef00d0123456789abcdefu128;
                    let source = exact_vector(form & 1 != 0, false);
                    let roots = exact_vector(form & 1 != 0, true);
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
                    let scalar_mask = if form & 1 == 0 {
                        u32::MAX as u128
                    } else {
                        u64::MAX as u128
                    };
                    let result = if form >= 2 {
                        roots
                    } else {
                        (if dst == src { source } else { old } & !scalar_mask)
                            | (roots & scalar_mask)
                    };
                    expected[d] = result as u64;
                    expected[d + 1] = (result >> 64) as u64;
                    expected[17] += code.len() as u64;
                    assert!(matches!(r.run(1), Stop::Budget { .. }));
                    assert_eq!(r.state, expected, "form={form} dst={dst} src={src}");
                    let f = r.memory.fpu.borrow();
                    assert_eq!(
                        (f.control, f.status, f.tag, f.mxcsr, f.registers),
                        (fpu.control, fpu.status, fpu.tag, fpu.mxcsr, fpu.registers)
                    );
                }
            }
        }
    }
    #[test]
    fn source_extent_permissions_alignment_and_unmasked_exceptions_are_atomic() {
        for form in 0..4 {
            let width = if form & 1 == 0 { 4 } else { 8 };
            let length = if form >= 2 { 16 } else { width };
            for (address, available, rights, works) in [
                (0x20000u64, length, READ, true),
                (0x20001, length, READ, form < 2),
                (0x20000, length - 1, READ, false),
                (0x20000, length, WRITE, false),
            ] {
                let mut r = cpu(&bytes(form, 8));
                r.state[0] = address;
                r.memory
                    .map(
                        address as u32,
                        available,
                        rights,
                        &exact_vector(form & 1 != 0, false).to_le_bytes()[..available],
                    )
                    .unwrap();
                let before = r.state;
                let control = r.memory.fpu.borrow().mxcsr;
                let result = r.run(1);
                if works {
                    assert!(matches!(result, Stop::Budget { .. }));
                } else {
                    assert!(!matches!(result, Stop::Budget { .. }));
                    assert_eq!(&r.state[..17], &before[..17]);
                    assert_eq!(&r.state[18..], &before[18..]);
                    assert_eq!(r.memory.fpu.borrow().mxcsr, control);
                }
            }
            for precision in [false, true] {
                let mut r = cpu(&bytes(form, 0xc1));
                let wide = form & 1 != 0;
                let mut source = exact_vector(wide, false);
                let lane = if form >= 2 {
                    if wide { 1 } else { 3 }
                } else {
                    0
                };
                let bits = if precision {
                    if wide {
                        0x4000000000000000u128
                    } else {
                        0x40000000
                    }
                } else if wide {
                    0xbff0000000000000
                } else {
                    0xbf800000
                };
                let mask = if wide {
                    u64::MAX as u128
                } else {
                    u32::MAX as u128
                };
                let shift = lane * if wide { 64 } else { 32 };
                source = (source & !(mask << shift)) | (bits << shift);
                r.state[26] = source as u64;
                r.state[27] = (source >> 64) as u64;
                let control = 0x1f80 & !if precision { 0x1000 } else { 0x80 };
                r.memory.fpu.borrow_mut().mxcsr = control;
                let before = r.state;
                assert!(matches!(r.run(1), Stop::CpuFault { .. }));
                assert_eq!(&r.state[..17], &before[..17]);
                assert_eq!(&r.state[18..], &before[18..]);
                assert_eq!(r.memory.fpu.borrow().mxcsr, control);
            }
        }
    }
    #[test]
    #[ignore = "run tools/check_jit32_sqrt.py with frozen actual x86 observations"]
    fn actual_x86_sqrt_observations() {
        let raw = std::fs::read(std::env::var_os("ALKYHOL_SQRT_REFERENCE").unwrap()).unwrap();
        assert_eq!(raw.len(), 32768 * 80);
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
            assert!(kind < 4);
            assert_eq!(kind as usize, index / (16 * 512));
            assert_eq!(w[18] as usize, index % 512);
            assert_eq!(w[19] as usize, index / 512 % 16);
            let mut r = cpu(&bytes(kind as u8, 0xc1));
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
            "{{\"rows\":32768,\"result_differences\":{result_differences},\"mxcsr_differences\":{mxcsr_differences},\"flags_differences\":{flags_differences},\"requested_mxcsr_load_differences\":{loaded_differences},\"examples\":[{}]}}",
            examples.join(",")
        );
        if let Some(path) = std::env::var_os("ALKYHOL_SQRT_REPORT") {
            std::fs::write(path, &report).unwrap();
        }
        eprintln!("SIMD square-root observations: {report}");
        assert_eq!(
            (result_differences, mxcsr_differences, flags_differences),
            (0, 0, 0)
        );
    }
}
