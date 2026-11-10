//! Deterministic RSQRT estimate within Intel's architectural relative-error bound.
//! Normal result bits are not claimed to match a particular x86 microarchitecture.
use rustc_apfloat::{
    Float, FloatConvert, Round,
    ieee::{Single, X87DoubleExtended as X},
};
pub(super) fn rsqrt(bits: u32) -> Result<u32, &'static str> {
    let sign = bits & 0x80000000;
    let exponent = bits & 0x7f800000;
    let fraction = bits & 0x7fffff;
    if exponent == 0x7f800000 && fraction != 0 {
        return Ok(bits | 0x400000);
    }
    if exponent == 0 {
        return Ok(sign | 0x7f800000);
    } // Signed zero and all denormals.
    if sign != 0 {
        return Ok(0xffc00000);
    }
    if exponent == 0x7f800000 {
        return Ok(0);
    }
    let source: X = Single::from_bits(bits as u128).convert(&mut false).value;
    let root = crate::fpu_math::sqrt(&mut crate::fpu::State::default(), source.to_bits())?;
    let reciprocal = X::from_bits(0x3fff8000000000000000)
        .div_r(X::from_bits(root), Round::NearestTiesToEven)
        .value;
    let value: Single = reciprocal.convert(&mut false).value;
    Ok(value.to_bits() as u32)
}
#[cfg(test)]
pub(super) fn within_bound(input: u32, output: u32) -> bool {
    let xe = ((input >> 23) & 255) as i32;
    let ye = ((output >> 23) & 255) as i32;
    if input >> 31 != 0 || output >> 31 != 0 || !(1..255).contains(&xe) || !(1..255).contains(&ye) {
        return false;
    }
    let x = ((input & 0x7fffff) | 0x800000) as u128;
    let y = ((output & 0x7fffff) | 0x800000) as u128;
    // (8189/8192)^2 <= x*y*y <= (8195/8192)^2. Evaluate
    // exactly with integer significands; no floating reference approximation.
    let shift = -((xe - 127) + 2 * (ye - 127) - 69 + 26);
    if !(0..=100).contains(&shift) {
        return false;
    }
    let product = x * y * y;
    (8189u128 * 8189) << shift <= product && product <= (8195u128 * 8195) << shift
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normal_estimates_meet_the_exact_integer_architectural_bound() {
        for exponent in 1..255 {
            for fraction in [0, 1, 0x3fffff, 0x400000, 0x400001, 0x7ffffe, 0x7fffff] {
                let bits = (exponent << 23) | fraction;
                assert!(within_bound(bits, rsqrt(bits).unwrap()), "{bits:x}");
            }
        }
        let mut value = 0x19283746u32;
        for _ in 0..65536 {
            value = value.wrapping_mul(1664525).wrapping_add(1013904223);
            let bits = (value & 0x7fffffff).clamp(0x00800000, 0x7f7fffff);
            assert!(within_bound(bits, rsqrt(bits).unwrap()), "{bits:x}");
        }
        assert!(!within_bound(0x3f800000, 0x40000000));
        assert!(!within_bound(0x3f800000, 0x3f7fc000));
    }
    #[test]
    fn signed_zero_denormal_infinity_and_nan_rules_are_explicit() {
        for (x, y) in [
            (0, 0x7f800000),
            (0x80000000, 0xff800000),
            (1, 0x7f800000),
            (0x807fffff, 0xff800000),
            (0x7f800000, 0),
            (0xff800000, 0xffc00000),
            (0xbf800000, 0xffc00000),
            (0x7f801234, 0x7fc01234),
            (0xffc05678, 0xffc05678),
        ] {
            assert_eq!(rsqrt(x).unwrap(), y);
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
    fn scalar_packed_alias_and_memory_boundaries_preserve_control_and_other_state() {
        for packed in [false, true] {
            for alias in [false, true] {
                for memory in [false, true] {
                    let mut code = vec![];
                    if !packed {
                        code.push(0xf3);
                    }
                    code.extend([
                        0x0f,
                        0x52,
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
                    let source = [0x40800000u32, 0xbf800000, 0x7f801234, 1];
                    let bytes: Vec<u8> = source.iter().flat_map(|v| v.to_le_bytes()).collect();
                    let slot = if alias { 24 } else { 26 };
                    r.state[24] = 0x3f800000;
                    r.state[25] = 0x123456789abcdef0;
                    r.state[slot] = u64::from_le_bytes(bytes[..8].try_into().unwrap());
                    r.state[slot + 1] = u64::from_le_bytes(bytes[8..].try_into().unwrap());
                    r.memory.map(0x20000, 16, READ | WRITE, &bytes).unwrap();
                    r.memory.fpu.borrow_mut().mxcsr = 0x8061;
                    let before = r.state;
                    assert!(matches!(r.run(1), Stop::Budget { .. }));
                    let low = r.state[24] as u32;
                    assert!(within_bound(source[0], low));
                    if packed {
                        assert_eq!(r.state[24] >> 32, 0xffc00000);
                        assert_eq!(r.state[25], 0x7f8000007fc01234);
                    } else {
                        assert_eq!(r.state[24] >> 32, before[24] >> 32);
                        assert_eq!(r.state[25], before[25]);
                    }
                    assert_eq!(&r.state[..17], &before[..17]);
                    assert_eq!(&r.state[26..], &before[26..]);
                    assert_eq!(r.memory.fpu.borrow().mxcsr, 0x8061);
                }
            }
        }
        for (packed, address, length, rights) in [
            (false, 0x20001u64, 4, READ),
            (true, 0x20001, 16, READ),
            (true, 0x20000, 8, READ),
            (false, 0x20000, 4, WRITE),
        ] {
            let mut code = vec![];
            if !packed {
                code.push(0xf3);
            }
            code.extend([0x0f, 0x52, 0]);
            let mut r = cpu(&code);
            r.state[0] = address;
            r.memory
                .map(address as u32, length, rights, &vec![0; length])
                .unwrap();
            let before = r.state;
            let stop = r.run(1);
            if !packed && rights == READ {
                assert!(matches!(stop, Stop::Budget { .. }));
                assert_eq!(r.state[24] as u32, 0x7f800000);
            } else {
                assert!(matches!(stop, Stop::Fault { .. } | Stop::CpuFault { .. }));
                assert_eq!(&r.state[..17], &before[..17]);
                assert_eq!(&r.state[18..], &before[18..]);
            }
        }
    }
    #[test]
    #[ignore = "run tools/check_jit32_rsqrt.py for actual x86 bounded estimates"]
    fn actual_x86_estimate_observations() {
        let data = std::fs::read(std::env::var_os("ALKYHOL_RSQRT_REFERENCE").unwrap()).unwrap();
        assert_eq!(data.len(), 16640 * 72);
        let mut different = 0;
        let mut normal = 0;
        for row in data.chunks_exact(72) {
            let w: Vec<u32> = row
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            let packed = w[0] != 0;
            let mut code = vec![];
            if !packed {
                code.push(0xf3);
            }
            code.extend([0x0f, 0x52, 0xc1]);
            let mut r = cpu(&code);
            for (slot, bytes) in [(24, &row[16..32]), (26, &row[32..48])] {
                r.state[slot] = u64::from_le_bytes(bytes[..8].try_into().unwrap());
                r.state[slot + 1] = u64::from_le_bytes(bytes[8..].try_into().unwrap());
            }
            r.state[16] = w[16] as u64;
            r.memory.fpu.borrow_mut().mxcsr = w[1];
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            let out = (r.state[24] as u128) | ((r.state[25] as u128) << 64);
            for lane in 0..4 {
                let actual = (out >> (lane * 32)) as u32;
                let observed = w[12 + lane];
                let input = w[8 + lane];
                let positive_normal = input >> 31 == 0 && (1..255).contains(&((input >> 23) & 255));
                if (packed || lane == 0) && positive_normal {
                    assert!(within_bound(input, actual));
                    assert!(within_bound(input, observed));
                    normal += 1;
                    different += usize::from(actual != observed);
                } else {
                    assert_eq!(actual, observed, "op={} lane={lane} input={input:x}", w[0]);
                }
            }
            assert_eq!(r.memory.fpu.borrow().mxcsr, w[2]);
            assert_eq!(w[1], w[2]);
            assert_eq!(r.flags() & 0x8d5, w[3] & 0x8d5);
            assert_eq!(w[16] & 0x8d5, w[3] & 0x8d5);
        }
        eprintln!(
            "RSQRT bounded profile: {normal} positive-normal lane results; {different} differ in bits from observed CPU; all meet the architectural bound"
        );
    }
}
