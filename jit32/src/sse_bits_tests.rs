//! Original instruction tests for the vector manipulations reached by DAO's CRT.
use crate::{
    Stop,
    memory::{READ, WRITE},
    tests::cpu,
};
#[test]
fn variable_packed_shifts_use_the_full_low_qword_and_saturate_counts() {
    let vector = 0x7654fedc01239876fedc765489ab0123u128;
    for (bits, opcode, kind) in [
        (16, 0xd1, 0),
        (32, 0xd2, 0),
        (64, 0xd3, 0),
        (16, 0xe1, 1),
        (32, 0xe2, 1),
        (16, 0xf1, 2),
        (32, 0xf2, 2),
        (64, 0xf3, 2),
    ] {
        for count in [
            0,
            1,
            bits - 1,
            bits,
            bits + 1,
            255,
            256,
            0x100000001,
            u64::MAX,
        ] {
            for form in 0..3 {
                let mut r = cpu(&[0x66, 0x0f, opcode, [0xc1, 0xc0, 0x00][form]]);
                r.state[0] = 0x20000;
                let source = count as u128 | (u64::MAX as u128) << 64;
                r.memory
                    .map(0x20000, 16, READ, &source.to_le_bytes())
                    .unwrap();
                r.state[24] = vector as u64;
                r.state[25] = (vector >> 64) as u64;
                r.state[26] = count;
                r.state[27] = u64::MAX;
                r.state[16] = 0xad7;
                r.memory.fpu.borrow_mut().mxcsr = 0x9fc3;
                let before_gpr = r.registers();
                let count = if form == 1 { vector as u64 } else { count };
                let mask = (1u128 << bits) - 1;
                let mut expected = 0;
                for lane in 0..128 / bits {
                    let value = vector >> (lane * bits) & mask;
                    let shifted = if kind == 1 {
                        let signed = ((value << (128 - bits)) as i128) >> (128 - bits);
                        (signed >> count.min(bits - 1)) as u128 & mask
                    } else if count >= bits {
                        0
                    } else if kind == 0 {
                        value >> count
                    } else {
                        value << count & mask
                    };
                    expected |= shifted << (lane * bits);
                }
                let stop = r.run(1);
                assert!(
                    matches!(stop, Stop::Budget { .. }),
                    "{bits}/{kind}/{count}/{form}: {stop:?}"
                );
                assert_eq!(
                    r.state[24] as u128 | ((r.state[25] as u128) << 64),
                    expected,
                    "{bits}/{kind}/{count}/{form}"
                );
                assert_eq!(r.registers(), before_gpr);
                assert_eq!(r.state[16], 0xad7);
                assert_eq!(r.memory.fpu.borrow().mxcsr, 0x9fc3);
                assert_eq!(r.state[26] as u128 | ((r.state[27] as u128) << 64), source);
                assert_eq!(
                    r.memory.read(0x20000, 16, READ).unwrap(),
                    source.to_le_bytes()
                );
            }
        }
        for address in [0, 0x20008, 0x30000] {
            let mut r = cpu(&[0x66, 0x0f, opcode, 0x00]);
            r.memory.map(0x20000, 32, READ, &[0; 32]).unwrap();
            r.memory.map(0x30000, 8, READ, &[0; 8]).unwrap();
            r.state[0] = address;
            let before = r.state;
            assert!(matches!(
                r.run(1),
                Stop::Fault { .. } | Stop::CpuFault { .. }
            ));
            assert_eq!(r.state, before);
        }
        let mut mmx = cpu(&[0x0f, opcode, 0xc1]);
        assert!(matches!(mmx.run(1), Stop::Unsupported { .. }));
    }
}
#[test]
fn float_shuffles_cover_all_selectors_aliases_and_memory() {
    let a = 0x12345678fedcba987fc01234abcdef01u128;
    let b = 0x800000017fffffff9988776655443322u128;
    for width in [32, 64] {
        for selector in 0..=255u8 {
            for form in 0..3 {
                let mut bytes = if width == 64 { vec![0x66] } else { vec![] };
                bytes.extend([0x0f, 0xc6, [0xc1, 0xc0, 0x00][form], selector]);
                let mut r = cpu(&bytes);
                r.state[0] = 0x20000;
                r.memory.map(0x20000, 16, READ, &b.to_le_bytes()).unwrap();
                r.state[24] = a as u64;
                r.state[25] = (a >> 64) as u64;
                r.state[26] = b as u64;
                r.state[27] = (b >> 64) as u64;
                r.state[16] = 0xad7;
                r.memory.fpu.borrow_mut().mxcsr = 0x9fc3;
                let before_gpr = r.registers();
                let other = if form == 1 { a } else { b };
                let mut expected = 0u128;
                let mask = (1u128 << width) - 1;
                let lanes = 128 / width;
                let bits = if width == 32 { 2 } else { 1 };
                for lane in 0..lanes {
                    let source = if lane < lanes / 2 { a } else { other };
                    let index = (selector as u32 >> (lane * bits)) & ((1 << bits) - 1);
                    expected |= ((source >> (index * width)) & mask) << (lane * width);
                }
                assert!(matches!(r.run(1), Stop::Budget { .. }));
                assert_eq!(
                    r.state[24] as u128 | ((r.state[25] as u128) << 64),
                    expected,
                    "width {width} selector {selector:x} form {form}"
                );
                assert_eq!(r.registers(), before_gpr);
                assert_eq!(r.state[16], 0xad7);
                assert_eq!(r.memory.fpu.borrow().mxcsr, 0x9fc3);
                assert_eq!(&r.state[26..28], &[b as u64, (b >> 64) as u64]);
            }
        }
        for address in [0, 0x20008, 0x30000] {
            let mut bytes = if width == 64 { vec![0x66] } else { vec![] };
            bytes.extend([0x0f, 0xc6, 0x00, 0xff]);
            let mut r = cpu(&bytes);
            r.memory.map(0x20000, 32, READ, &[0; 32]).unwrap();
            r.memory.map(0x30000, 8, READ, &[0; 8]).unwrap();
            r.state[0] = address;
            let before = r.state;
            assert!(matches!(
                r.run(1),
                Stop::Fault { .. } | Stop::CpuFault { .. }
            ));
            assert_eq!(r.state, before);
        }
    }
}
#[test]
fn unpack_float_family_interleaves_bits_without_changing_fp_status() {
    let a = 0x7ff00001fedcba987fc0123401234567u128;
    let b = 0x80000000abcdef0198765432ffff0001u128;
    for width in [32, 64] {
        for high in [false, true] {
            for form in 0..3 {
                let mut bytes = if width == 64 { vec![0x66] } else { vec![] };
                bytes.extend([
                    0x0f,
                    if high { 0x15 } else { 0x14 },
                    [0xc1, 0xc0, 0x00][form],
                ]);
                let mut r = cpu(&bytes);
                r.state[0] = 0x20000;
                r.memory.map(0x20000, 16, READ, &b.to_le_bytes()).unwrap();
                r.state[24] = a as u64;
                r.state[25] = (a >> 64) as u64;
                r.state[26] = b as u64;
                r.state[27] = (b >> 64) as u64;
                r.state[16] = 0xad7;
                r.memory.fpu.borrow_mut().mxcsr = 0x9fc3;
                let other = if form == 1 { a } else { b };
                let half = 64 / width;
                let offset = if high { half } else { 0 };
                let mask = (1u128 << width) - 1;
                let mut expected = 0;
                for lane in 0..half {
                    expected |= ((a >> ((lane + offset) * width)) & mask) << (2 * lane * width);
                    expected |=
                        ((other >> ((lane + offset) * width)) & mask) << ((2 * lane + 1) * width);
                }
                assert!(matches!(r.run(1), Stop::Budget { .. }));
                assert_eq!(
                    r.state[24] as u128 | ((r.state[25] as u128) << 64),
                    expected
                );
                assert_eq!(&r.state[26..28], &[b as u64, (b >> 64) as u64]);
                assert_eq!(r.state[16], 0xad7);
                assert_eq!(r.memory.fpu.borrow().mxcsr, 0x9fc3);
                assert_eq!(r.memory.read(0x20000, 16, READ).unwrap(), b.to_le_bytes());
            }
            for address in [0, 0x20008, 0x30000] {
                let mut bytes = if width == 64 { vec![0x66] } else { vec![] };
                bytes.extend([0x0f, if high { 0x15 } else { 0x14 }, 0x00]);
                let mut r = cpu(&bytes);
                r.memory.map(0x20000, 32, READ, &[0x5a; 32]).unwrap();
                r.memory.map(0x30000, 8, READ, &[0x5a; 8]).unwrap();
                r.state[0] = address;
                let before = r.state;
                let stop = r.run(1);
                assert!(matches!(stop, Stop::Fault { .. } | Stop::CpuFault { .. }));
                assert_eq!(r.state, before);
            }
        }
    }
}
#[test]
fn reached_vector_shift_subtract_shuffle_and_zip_are_lane_operations() {
    let a = [0xfedcba9876543210, 0x0123456789abcdef];
    for count in [0, 1, 31, 63, 64, 65, 255] {
        let mut r = cpu(&[0x66, 0x0f, 0x73, 0xd0, count, 0xc3]);
        r.state[24] = a[0];
        r.state[25] = a[1];
        r.state[16] = 0xad7;
        assert_eq!(r.run(3), Stop::Returned(0));
        assert_eq!(
            &r.state[24..26],
            &a.map(|v| if count >= 64 { 0 } else { v >> count })
        );
        assert_eq!(r.state[16], 0xad7);
    }
    let mut r = cpu(&[0x66, 0x0f, 0xfb, 0xc1, 0xc3]);
    r.state[24] = 0;
    r.state[25] = 1;
    r.state[26] = 1;
    r.state[27] = u64::MAX;
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(&r.state[24..26], &[u64::MAX, 2]);
    let src = [0x11111111u128, 0x22222222, 0x33333333, 0x44444444];
    for control in [0, 0x1b, 0xee, 0xff] {
        let mut r = cpu(&[0x66, 0x0f, 0x70, 0xc1, control, 0xc3]);
        r.state[26] = (src[0] | src[1] << 32) as u64;
        r.state[27] = (src[2] | src[3] << 32) as u64;
        assert_eq!(r.run(3), Stop::Returned(0));
        let expected = (0..4).fold(0, |out, i| {
            out | src[(control as usize >> (i * 2)) & 3] << (i * 32)
        });
        assert_eq!(r.state[24] as u128 | (r.state[25] as u128) << 64, expected);
    }
    let mut r = cpu(&[0x66, 0x0f, 0x14, 0xc1, 0xc3]);
    r.state[24] = 1;
    r.state[25] = 2;
    r.state[26] = 3;
    r.state[27] = 4;
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(&r.state[24..26], &[1, 3]);
}
#[test]
fn reached_word_insert_extract_and_exact_i32_to_double_preserve_other_lanes() {
    let vector = 0x123456789abcdef00123456789abcdefu128;
    for selector in (0..8).chain([255]) {
        let lane = selector as u32 & 7;
        let mut r = cpu(&[0x66, 0x0f, 0xc4, 0xc1, selector, 0xc3]);
        r.state[24] = vector as u64;
        r.state[25] = (vector >> 64) as u64;
        r.state[1] = 0xcafebabe;
        assert_eq!(r.run(3), Stop::Returned(0));
        let expected = (vector & !(0xffffu128 << (lane * 16))) | (0xbabeu128 << (lane * 16));
        assert_eq!(
            r.state[24] as u128 | ((r.state[25] as u128) << 64),
            expected
        );
        let mut r = cpu(&[0x66, 0x0f, 0xc5, 0xc1, selector, 0xc3]);
        r.state[26] = vector as u64;
        r.state[27] = (vector >> 64) as u64;
        assert_eq!(
            r.run(3),
            Stop::Returned(((vector >> (lane * 16)) & 0xffff) as u32)
        );
    }
    let mut r = cpu(&[0xf3, 0x0f, 0xe6, 0xc1, 0xc3]);
    r.state[26] = 0x7fffffff80000000;
    r.state[27] = u64::MAX;
    r.memory.fpu.borrow_mut().mxcsr = 0x9fc3;
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(
        &r.state[24..26],
        &[(-2147483648f64).to_bits(), 2147483647f64.to_bits()]
    );
    assert_eq!(r.memory.fpu.borrow().mxcsr, 0x9fc3);
}
#[test]
fn low_double_moves_touch_only_the_selected_eight_bytes() {
    let mut r = cpu(&[
        0x66, 0x0f, 0x12, 0x05, 0, 0, 2, 0, 0x66, 0x0f, 0x13, 0x05, 8, 0, 2, 0, 0xc3,
    ]);
    r.memory
        .map(0x20000, 24, READ | WRITE, &[0x5a; 24])
        .unwrap();
    r.state[24] = 1;
    r.state[25] = 0x12345678;
    assert_eq!(r.run(4), Stop::Returned(0));
    assert_eq!(r.state[24], 0x5a5a5a5a5a5a5a5a);
    assert_eq!(r.state[25], 0x12345678);
    assert_eq!(r.memory.read(0x20000, 24, READ).unwrap(), [0x5a; 24]);
}
#[test]
fn integer_xmm_moves_use_fixed_widths_and_zero_the_specified_upper_bits() {
    let mut r = cpu(&[0x66, 0x0f, 0x6e, 0xc1, 0xc3]);
    r.state[1] = 0xdeadbeef;
    r.state[24] = u64::MAX;
    r.state[25] = u64::MAX;
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(&r.state[24..26], &[0xdeadbeef, 0]);
    let mut r = cpu(&[0x66, 0x0f, 0x7e, 0xc1, 0xc3]);
    r.state[24] = 0x12345678cafebabe;
    r.state[1] = 1;
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(r.state[1], 0xcafebabe);
    let mut r = cpu(&[0xf3, 0x0f, 0x7e, 0xc1, 0xc3]);
    r.state[26] = 0x12345678cafebabe;
    r.state[27] = u64::MAX;
    r.state[25] = u64::MAX;
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(&r.state[24..26], &[0x12345678cafebabe, 0]);
    let mut r = cpu(&[0x66, 0x0f, 0xd6, 0xc1, 0xc3]);
    r.state[24] = 0x12345678cafebabe;
    r.state[25] = u64::MAX;
    r.state[27] = u64::MAX;
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(&r.state[26..28], &[0x12345678cafebabe, 0]);
    let mut r = cpu(&[0x66, 0x0f, 0x6e, 0x05, 0, 0, 2, 0, 0xc3]);
    r.memory
        .map(0x20000, 4, READ, &0xdeadbeefu32.to_le_bytes())
        .unwrap();
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(&r.state[24..26], &[0xdeadbeef, 0]);
    let mut r = cpu(&[0x66, 0x0f, 0xd6, 0x05, 0, 0, 2, 0, 0xc3]);
    r.memory
        .map(0x20000, 16, READ | WRITE, &[0x5a; 16])
        .unwrap();
    r.state[24] = 0x12345678cafebabe;
    assert_eq!(r.run(3), Stop::Returned(0));
    assert_eq!(
        r.memory.read(0x20000, 8, READ).unwrap(),
        0x12345678cafebabeu64.to_le_bytes()
    );
    assert_eq!(r.memory.read(0x20008, 8, READ).unwrap(), [0x5a; 8]);
}
#[test]
fn packed_integer_shift_boundaries_and_add_subtract_wrap_per_lane() {
    let vector = 0x8765432101234567fedcba9876543210u128;
    for (bits, opcode, group) in [
        (16, 0x71, 2),
        (16, 0x71, 4),
        (16, 0x71, 6),
        (32, 0x72, 2),
        (32, 0x72, 4),
        (32, 0x72, 6),
        (64, 0x73, 2),
        (64, 0x73, 6),
    ] {
        for count in [0u8, 1, (bits - 1) as u8, bits as u8, (bits + 1) as u8, 255] {
            let mut r = cpu(&[0x66, 0x0f, opcode, 0xc0 | (group << 3), count, 0xc3]);
            r.state[24] = vector as u64;
            r.state[25] = (vector >> 64) as u64;
            assert_eq!(r.run(3), Stop::Returned(0));
            let mask = (1u128 << bits) - 1;
            let mut expected = 0;
            for lane in 0..128 / bits {
                let v = vector >> (lane * bits) & mask;
                let value = if group == 4 {
                    let signed = ((v << (128 - bits)) as i128) >> (128 - bits);
                    (signed >> u32::from(count).min(bits - 1)) as u128 & mask
                } else if count as u32 >= bits {
                    0
                } else if group == 2 {
                    v >> count
                } else {
                    v << count & mask
                };
                expected |= value << (lane * bits);
            }
            assert_eq!(
                r.state[24] as u128 | ((r.state[25] as u128) << 64),
                expected,
                "{bits}/{group}/{count}"
            );
        }
    }
    for (bits, add, sub) in [
        (8, 0xfc, 0xf8),
        (16, 0xfd, 0xf9),
        (32, 0xfe, 0xfa),
        (64, 0xd4, 0xfb),
    ] {
        for (opcode, subtract) in [(add, false), (sub, true)] {
            let mut r = cpu(&[0x66, 0x0f, opcode, 0xc1, 0xc3]);
            r.state[24] = vector as u64;
            r.state[25] = (vector >> 64) as u64;
            r.state[26] = u64::MAX;
            r.state[27] = u64::MAX;
            assert_eq!(r.run(3), Stop::Returned(0));
            let mask = (1u128 << bits) - 1;
            let mut expected = 0;
            for lane in 0..128 / bits {
                let v = vector >> (lane * bits) & mask;
                expected |= if subtract {
                    v.wrapping_sub(mask) & mask
                } else {
                    v.wrapping_add(mask) & mask
                } << (lane * bits);
            }
            assert_eq!(
                r.state[24] as u128 | ((r.state[25] as u128) << 64),
                expected
            );
        }
    }
}
