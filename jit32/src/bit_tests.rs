//! Owned bit-test instructions: only CF is asserted among arithmetic flags.
use crate::{
    Stop,
    memory::{READ, WRITE},
    tests::cpu,
};
const OPS: [u8; 4] = [0xa3, 0xab, 0xb3, 0xbb];
#[test]
#[ignore = "requires actual x86 assembly observations from tools/check_jit32_bits.py"]
fn actual_x86_memory_bit_observations() {
    let data = std::fs::read(std::env::var_os("ALKYHOL_BIT_REFERENCE").unwrap()).unwrap();
    assert_eq!(data.len(), 112 * 84);
    for row in data.chunks_exact(84) {
        let w: Vec<u32> = row[..20]
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let [width, op, locked, index, flags] = w[..] else {
            unreachable!()
        };
        let mut r = cpu(&encoding(width, op as usize, true, None, locked != 0));
        r.memory
            .map(0x20000, 32, READ | WRITE, &row[20..52])
            .unwrap();
        r.state[0] = 0x20010;
        r.state[1] = index as u64;
        assert!(matches!(r.run(1), Stop::Budget { .. }), "{w:x?}");
        assert_eq!(
            r.memory.read(0x20000, 32, READ).unwrap(),
            &row[52..84],
            "{w:x?}"
        );
        assert_eq!(r.flags() & 1, flags & 1, "{w:x?}");
    }
}
fn changed(value: u32, mask: u32, op: usize) -> u32 {
    match op {
        0 => value,
        1 => value | mask,
        2 => value & !mask,
        3 => value ^ mask,
        _ => unreachable!(),
    }
}
fn encoding(width: u32, op: usize, memory: bool, index: Option<u8>, locked: bool) -> Vec<u8> {
    let mut code = vec![];
    if locked {
        code.push(0xf0)
    }
    if width == 16 {
        code.push(0x66)
    }
    code.push(0x0f);
    if let Some(index) = index {
        code.extend([
            0xba,
            (if memory { 0 } else { 0xc0 }) | ((4 + op as u8) << 3),
            index,
        ]);
    } else {
        code.extend([OPS[op], if memory { 0x08 } else { 0xc8 }]);
    }
    code
}
#[test]
fn register_bit_tests_mask_indices_and_preserve_partial_registers() {
    for width in [16u32, 32] {
        for op in 0..4 {
            for index in [0, 1, width - 1, width, width + 1, 255, u32::MAX] {
                for immediate in [false, true] {
                    if immediate && index > 255 {
                        continue;
                    }
                    let mut r = cpu(&encoding(
                        width,
                        op,
                        false,
                        immediate.then_some(index as u8),
                        false,
                    ));
                    let input = 0xa5a55a5au32;
                    r.state[0] = input as u64;
                    r.state[1] = index as u64;
                    r.state[16] = 0xad7;
                    assert!(matches!(r.run(1), Stop::Budget { .. }));
                    let mask = 1 << (index & (width - 1));
                    assert_eq!(r.state[0], changed(input, mask, op) as u64);
                    assert_eq!(r.state[1], index as u64);
                    assert_eq!(r.flags() & 1, u32::from(input & mask != 0));
                    assert_eq!(r.flags() & 0x600, 0x200);
                }
            }
        }
    }
    for width in [16, 32] {
        for (op, opcode) in OPS.into_iter().enumerate() {
            let mut code = if width == 16 { vec![0x66] } else { vec![] };
            code.extend([0x0f, opcode, 0xc0]); // destination and index both AX/EAX
            let mut r = cpu(&code);
            let input = 0xa5a55a5au32;
            r.state[0] = input as u64;
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            let mask = 1 << (input & (width - 1));
            assert_eq!(r.state[0], changed(input, mask, op) as u64);
            assert_eq!(r.flags() & 1, u32::from(input & mask != 0));
        }
    }
}
#[test]
fn memory_bit_strings_use_signed_word_offsets_and_mask_immediates() {
    let initial: [u8; 32] = std::array::from_fn(|i| (i as u8).wrapping_mul(37));
    for width in [16u32, 32] {
        for op in 0..4 {
            for locked in [false, true] {
                if locked && op == 0 {
                    continue;
                }
                for index in [
                    -(width as i32) - 1,
                    -(width as i32),
                    -1,
                    0,
                    1,
                    width as i32 - 1,
                    width as i32,
                    width as i32 + 3,
                ] {
                    let mut r = cpu(&encoding(width, op, true, None, locked));
                    r.memory.map(0x20000, 32, READ | WRITE, &initial).unwrap();
                    r.state[0] = 0x20010;
                    r.state[1] = index as u32 as u64;
                    r.state[16] = 0xad7;
                    let byte = (16 + index.div_euclid(width as i32) * (width as i32 / 8)) as usize;
                    let bit = index.rem_euclid(width as i32) as u32;
                    let value = u32::from_le_bytes(initial[byte..byte + 4].try_into().unwrap());
                    let mask = 1 << bit;
                    let mut expected = initial;
                    expected[byte..byte + (width / 8) as usize].copy_from_slice(
                        &changed(value, mask, op).to_le_bytes()[..(width / 8) as usize],
                    );
                    let before_gpr = r.registers();
                    let stop = r.run(1);
                    assert!(
                        matches!(stop, Stop::Budget { .. }),
                        "{width}/{op}/{locked}/{index}: {stop:?}"
                    );
                    assert_eq!(r.memory.read(0x20000, 32, READ).unwrap(), expected);
                    assert_eq!(r.flags() & 1, u32::from(value & mask != 0));
                    assert_eq!(r.registers(), before_gpr);
                }
                for index in [0, 15, 31, 32, 255] {
                    let mut r = cpu(&encoding(width, op, true, Some(index), locked));
                    r.memory.map(0x20000, 32, READ | WRITE, &initial).unwrap();
                    r.state[0] = 0x20010;
                    let value = u32::from_le_bytes(initial[16..20].try_into().unwrap());
                    let mask = 1 << (index as u32 & (width - 1));
                    let mut expected = initial;
                    expected[16..16 + (width / 8) as usize].copy_from_slice(
                        &changed(value, mask, op).to_le_bytes()[..(width / 8) as usize],
                    );
                    let stop = r.run(1);
                    assert!(
                        matches!(stop, Stop::Budget { .. }),
                        "{width}/{op}/{index}: {stop:?}"
                    );
                    assert_eq!(r.memory.read(0x20000, 32, READ).unwrap(), expected);
                    assert_eq!(r.flags() & 1, u32::from(value & mask != 0));
                }
            }
        }
    }
}
#[test]
fn bit_memory_faults_commit_neither_carry_nor_bytes() {
    for width in [16, 32] {
        for op in 0..4 {
            for locked in [false, true] {
                if locked && op == 0 {
                    continue;
                }
                for address in [0, 0x20000, 0x30000] {
                    let mut r = cpu(&encoding(width, op, true, None, locked));
                    r.memory.map(0x20000, 4, READ, &[0xa5; 4]).unwrap();
                    r.memory.map(0x30000, 1, READ | WRITE, &[0xa5]).unwrap();
                    r.state[0] = address;
                    r.state[1] = 0;
                    r.state[16] = 0xad7;
                    let before = r.state;
                    let stop = r.run(1);
                    if address == 0x20000 && op == 0 {
                        assert!(matches!(stop, Stop::Budget { .. }));
                        continue;
                    }
                    assert!(
                        matches!(stop, Stop::Fault { .. } | Stop::CpuFault { .. }),
                        "{stop:?}"
                    );
                    assert_eq!(r.state, before);
                    assert_eq!(r.memory.read(0x20000, 4, READ).unwrap(), [0xa5; 4]);
                    assert_eq!(r.memory.read(0x30000, 1, READ).unwrap(), [0xa5]);
                }
            }
        }
    }
}

#[test]
fn bit_string_offsets_wrap_before_segment_translation_and_exclusions_stop() {
    let mut code = vec![0x64];
    code.extend(encoding(32, 1, true, None, false));
    let mut r = cpu(&code);
    r.memory.map(0x20000, 32, READ | WRITE, &[0; 32]).unwrap();
    r.state[22] = 0x20000;
    r.state[0] = 16;
    r.state[1] = u32::MAX as u64;
    assert!(matches!(r.run(1), Stop::Budget { .. }));
    assert_eq!(r.memory.word(0x2000c).unwrap(), 0x80000000);
    let mut r = cpu(&encoding(32, 1, true, None, false));
    r.memory.map(0x10, 4, READ | WRITE, &[0; 4]).unwrap();
    r.state[0] = 0xfffffff0;
    r.state[1] = 256;
    assert!(matches!(r.run(1), Stop::Budget { .. }));
    assert_eq!(r.memory.word(0x10).unwrap(), 1);
    for op in 0..4 {
        let mut r = cpu(&encoding(32, op, false, None, true));
        let before = r.state;
        assert!(matches!(r.run(1), Stop::Unsupported { .. }));
        assert_eq!(r.state, before);
        let mut code = vec![0x67];
        code.extend(encoding(32, op, true, None, false));
        let mut r = cpu(&code);
        assert!(matches!(r.run(1), Stop::Unsupported { .. }));
    }
}
