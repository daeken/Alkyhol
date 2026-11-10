//! Finite FPATAN contract tests. Native CPU approximation differences are
//! retained as diagnostics, separate from the mathematical rounding profile.
use crate::{Stop, fpu::State, fpu_atan2, tests::cpu};
use rustc_apfloat::{
    Float, FloatConvert,
    ieee::{Double, Quad, X87DoubleExtended as X},
};
const ONE: u128 = 0x3fff8000000000000000;
const SIGN: u128 = 1 << 79;
const PI: u128 = 0x4000c90fdaa22168c235;
fn snapshot(s: &State) -> (u16, u16, u16, u32, [u128; 8]) {
    (s.control, s.status, s.tag, s.mxcsr, s.registers)
}
#[test]
fn axes_signed_zeros_and_quadrants_round_at_extended_precision() {
    for negative in [false, true] {
        let sign = if negative { SIGN } else { 0 };
        for x in [0, ONE, SIGN, SIGN | ONE] {
            let a = fpu_atan2::atan2(sign, x).unwrap();
            assert_eq!(
                a.value.to_bits(),
                if x & SIGN == 0 { sign } else { PI | sign }
            );
            assert_eq!(a.inexact, x & SIGN != 0);
        }
        for x in [0, SIGN] {
            assert_eq!(
                fpu_atan2::atan2(ONE | sign, x).unwrap().value.to_bits(),
                (PI - (1 << 64)) | sign
            );
        }
        assert_eq!(
            fpu_atan2::atan2(ONE | sign, ONE).unwrap().value.to_bits(),
            (PI - (2 << 64)) | sign
        );
        assert_eq!(
            fpu_atan2::atan2(ONE | sign, ONE | SIGN)
                .unwrap()
                .value
                .to_bits(),
            0x400096cbe3f9990e91a8 | sign
        );
    }
}
#[test]
fn finite_angles_are_monotonic_symmetric_and_certify_tiny_corrections() {
    let mut previous = X::ZERO;
    for n in 1..=64 {
        let y = X::from_u128(n).value.scalbn(-6).to_bits();
        let a = fpu_atan2::atan2(y, ONE).unwrap();
        let b = fpu_atan2::atan2(y | SIGN, ONE).unwrap();
        assert!(a.value > previous);
        assert_eq!(b.value.to_bits(), a.value.to_bits() | SIGN);
        previous = a.value;
    }
    // Exact quotient residual lies beyond Quad precision, but well above
    // the cubic correction; the mathematical result rounds downward.
    let a = fpu_atan2::atan2(0x3f9b8000000000000000, ONE + 1).unwrap();
    assert_eq!(a.value.to_bits(), 0x3f9afffffffffffffffe);
    assert!(!a.rounded_up);
    for power in [64, 100, 512, 1024] {
        let tiny = X::from_u128(1).value.scalbn(-power).to_bits();
        for sign in [0, SIGN] {
            let a = fpu_atan2::atan2(tiny | sign, ONE).unwrap();
            assert_eq!(a.value.to_bits(), tiny | sign);
            assert!(a.rounded_up && a.inexact);
        }
    }
}
#[test]
fn native_fpatan_pops_once_ignores_pc_and_preserves_unrelated_state() {
    for top in 0..8 {
        for cw in [0x7f, 0x27f, 0x37f] {
            for (x, y, expected, flags) in [
                (ONE, ONE, PI - (2 << 64), 0x220),
                (ONE, SIGN, SIGN, 0),
                (SIGN, 0, PI, 0x220),
            ] {
                let mut r = cpu(&[0xd9, 0xf3]);
                for i in 0..8 {
                    r.state[i] = 0x12340000 + i as u64;
                }
                for i in 24..40 {
                    r.state[i] = 0xa5a5000000000000 + i as u64;
                }
                r.state[16] = 0xad7;
                {
                    let mut s = r.memory.fpu.borrow_mut();
                    s.control = cw;
                    s.status = 0x4704;
                    s.set_top(top);
                    s.mxcsr = 0xffc5;
                    s.set(0, x);
                    s.set(1, y);
                    s.set(2, 0x4005a900000000000000);
                }
                let gpr = r.registers();
                let xmm = r.state[24..40].to_vec();
                let before = r.memory.fpu.borrow().clone();
                assert!(matches!(r.run(1), Stop::Budget { .. }));
                assert_eq!(r.registers(), gpr);
                assert_eq!(&r.state[24..40], xmm);
                assert_eq!(r.flags(), 0xad7);
                assert_eq!(r.state[17], 0x10002);
                let s = r.memory.fpu.borrow();
                assert_eq!(s.top(), (top + 1) & 7);
                assert_eq!(s.control, cw);
                assert_eq!(s.mxcsr, before.mxcsr);
                assert_eq!(
                    s.status,
                    (before.status & !0x3a00) | ((((top + 1) & 7) as u16) << 11) | flags
                );
                assert_eq!(s.registers[(top + 1) & 7], expected);
                assert_eq!((s.tag >> (top * 2)) & 3, 3);
                assert_eq!(
                    (s.tag >> (((top + 1) & 7) * 2)) & 3,
                    u16::from(expected & !SIGN == 0)
                );
                for p in 0..8 {
                    if p != top && p != ((top + 1) & 7) {
                        assert_eq!(s.registers[p], before.registers[p]);
                        assert_eq!(s.tag >> (p * 2) & 3, before.tag >> (p * 2) & 3);
                    }
                }
            }
        }
    }
}
#[test]
fn unsupported_controls_operands_and_empty_slots_leave_state_uncommitted() {
    let invalid = [
        0x7fff8000000000000000,
        0x7fffc000000000000001,
        1,
        0x40000000000000000001,
        (1u128 << 100) | ONE,
    ];
    for variant in 0..12 {
        let mut r = cpu(&[0xd9, 0xf3]);
        {
            let mut s = r.memory.fpu.borrow_mut();
            s.set(0, ONE);
            s.set(1, ONE);
            match variant {
                0 => s.control = 0x77f,
                1 => s.control = 0x35f,
                2 => s.control = 0x17f,
                3 => s.tag |= 3,
                4 => s.tag |= 3 << 2,
                5..=9 => s.registers[0] = invalid[variant - 5],
                10 => {
                    s.set(0, 0x7ffe8000000000000000);
                    s.set(1, 0x00018000000000000000);
                }
                _ => {
                    s.set(0, ONE);
                    s.set(1, 1);
                }
            }
        }
        let before = snapshot(&r.memory.fpu.borrow());
        let regs = r.state;
        assert!(
            matches!(r.run(1), Stop::CpuFault { .. }),
            "variant {variant}"
        );
        assert_eq!(snapshot(&r.memory.fpu.borrow()), before);
        assert_eq!(r.state, regs);
    }
    let mut r = cpu(&[0xf0, 0xd9, 0xf3]);
    assert!(matches!(r.run(1), Stop::Unsupported { .. }));
}
fn observe(strict: bool) {
    let data = std::fs::read(std::env::var_os("ALKYHOL_ATAN2_REFERENCE").unwrap()).unwrap();
    assert_eq!(data.len(), 768 * 64);
    let (mut bits_different, mut status_different, mut binary64_different) = (0, 0, 0);
    for (i, row) in data.chunks_exact(64).enumerate() {
        let bits = |a: usize| {
            let mut b = [0; 16];
            b[..10].copy_from_slice(&row[a..a + 10]);
            u128::from_le_bytes(b)
        };
        let word = |a: usize| u16::from_le_bytes(row[a..a + 2].try_into().unwrap());
        let mut r = cpu(&[0xd9, 0xf3]);
        {
            let mut s = r.memory.fpu.borrow_mut();
            s.control = word(4);
            s.mxcsr = 0x1fa0;
            s.set(0, bits(8));
            s.set(1, bits(18));
            s.set(2, bits(48));
        }
        let stop = r.run(1);
        assert!(matches!(stop, Stop::Budget { .. }), "row {i}: {stop:?}");
        let s = r.memory.fpu.borrow();
        let actual = X::from_bits(s.registers[1]);
        let native = X::from_bits(bits(28));
        assert_eq!(s.control, word(40));
        assert_eq!(s.top(), 1);
        assert_eq!(s.mxcsr, u32::from_le_bytes(row[44..48].try_into().unwrap()));
        assert_eq!(s.registers[2], bits(48));
        let tag = (0..8).fold(0u8, |v, p| v | u8::from(s.tag >> (p * 2) & 3 != 3) << p);
        assert_eq!(tag, row[42]);
        if actual.is_zero() || native.is_zero() {
            assert_eq!(actual.to_bits(), native.to_bits(), "row {i}");
        } else {
            assert_eq!(actual.is_negative(), native.is_negative(), "row {i}");
            let a: Quad = actual.convert(&mut false).value;
            let b: Quad = native.convert(&mut false).value;
            let exp = (((actual.to_bits() >> 64) & 0x7fff).max((native.to_bits() >> 64) & 0x7fff))
                as i32
                - 16383;
            assert!(
                (a - b).value.abs() <= Quad::from_u128(1).value.scalbn(exp - 63),
                "row {i}: {actual:?} {native:?}"
            );
        }
        bits_different += usize::from(actual.to_bits() != native.to_bits());
        status_different += usize::from((s.status ^ word(38)) & !0x4500 != 0);
        let a: Double = actual.convert(&mut false).value;
        let b: Double = native.convert(&mut false).value;
        binary64_different += usize::from(a.to_bits() != b.to_bits());
    }
    let report = format!(
        "{{\"rows\":768,\"math_profile_bound_checks_passed\":true,\"cpu_result_bit_differences\":{bits_different},\"cpu_defined_status_differences\":{status_different},\"cpu_binary64_result_differences\":{binary64_different},\"strict_cpu_parity\":{}}}",
        bits_different == 0 && status_different == 0
    );
    println!("{report}");
    if let Some(path) = std::env::var_os("ALKYHOL_ATAN2_REPORT") {
        std::fs::write(path, report).unwrap();
    }
    if strict {
        assert_eq!(
            (bits_different, status_different),
            (0, 0),
            "strict CPU approximation/status comparison"
        );
    }
}
#[test]
#[ignore = "requires independent owned native FPATAN observations"]
fn actual_x86_fpatan_math_profile_observations() {
    observe(false);
}
#[test]
#[ignore = "retains exact CPU approximation/status differences as a strict diagnostic"]
fn actual_x86_fpatan_strict_observations() {
    observe(true);
}
