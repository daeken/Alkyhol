use crate::{
    Stop,
    memory::{READ, WRITE},
    tests::cpu,
};
fn encoding(right: bool, memory: bool, count: Option<u8>, source: u8, dest: u8) -> Vec<u8> {
    let mut code = vec![
        0x0f,
        (if right { 0xac } else { 0xa4 }) | u8::from(count.is_none()),
        (source << 3) | if memory { 3 } else { 0xc0 | dest },
    ];
    if let Some(n) = count {
        code.push(n);
    }
    code
}
#[test]
fn double_shifts_preserve_alias_inputs_and_reject_unsupported_widths() {
    for right in [false, true] {
        for source in [0, 1, 2] {
            for dest in [0, 1, 2] {
                for count in [0, 1, 7, 16, 31, 32, 255] {
                    let mut r = cpu(&encoding(right, false, None, source, dest));
                    r.state[0] = 0x80000001;
                    r.state[1] = count;
                    r.state[2] = 0x7ffffffe;
                    r.state[16] = 0xad7;
                    let before = r.state;
                    let n = (count & 31) as u32;
                    let a = before[dest as usize] as u32;
                    let b = before[source as usize] as u32;
                    let mut expected = 0u32;
                    for bit in 0..32 {
                        let selected = if right {
                            let p = bit + n;
                            if p < 32 {
                                (a >> p) & 1
                            } else {
                                (b >> (p - 32)) & 1
                            }
                        } else if bit >= n {
                            (a >> (bit - n)) & 1
                        } else {
                            (b >> (32 + bit - n)) & 1
                        };
                        expected |= selected << bit;
                    }
                    assert!(matches!(r.run(1), Stop::Budget { .. }));
                    assert_eq!(r.state[dest as usize], expected as u64);
                    for reg in 0..8 {
                        if reg != dest as usize {
                            assert_eq!(r.state[reg], before[reg]);
                        }
                    }
                    if n == 0 {
                        assert_eq!(r.flags() & 0x8d5, 0xad7 & 0x8d5);
                    }
                }
            }
        }
    }
    for right in [false, true] {
        for count in [0, 1] {
            let mut r = cpu(&encoding(right, true, None, 2, 0));
            r.state[1] = count;
            r.state[2] = 0x12345678;
            r.state[3] = 0x20000;
            r.state[16] = 0xad7;
            r.memory.map(0x20000, 4, READ, &[1, 2, 3, 4]).unwrap();
            let before = r.state;
            assert!(matches!(r.run(1), Stop::Fault { .. }));
            assert_eq!(&r.state[..17], &before[..17]);
            assert_eq!(r.memory.read(0x20000, 4, READ).unwrap(), [1, 2, 3, 4]);
        }
    }
    let mut r = cpu(&[0x66, 0x0f, 0xad, 0xd0]);
    assert!(matches!(r.run(1), Stop::Unsupported { .. }));
}
#[test]
#[ignore = "run tools/check_jit32_double_shifts.py for actual x86 observations"]
fn actual_x86_double_shift_observations() {
    let data = std::fs::read(std::env::var_os("ALKYHOL_DOUBLE_SHIFT_REFERENCE").unwrap()).unwrap();
    assert_eq!(data.len(), 8320 * 32);
    for row in data.chunks_exact(32) {
        let w: Vec<u32> = row
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let [op, memory, count, input, source, initial, result, flags] = w[..] else {
            unreachable!()
        };
        for immediate in [false, true] {
            let mut r = cpu(&encoding(
                op != 0,
                memory != 0,
                immediate.then_some(count as u8),
                2,
                0,
            ));
            r.state[0] = input as u64;
            r.state[1] = count as u64;
            r.state[2] = source as u64;
            r.state[3] = 0x20000;
            r.state[16] = initial as u64;
            r.memory
                .map(0x20000, 4, READ | WRITE, &input.to_le_bytes())
                .unwrap();
            assert!(matches!(r.run(1), Stop::Budget { .. }), "{w:x?}");
            if memory != 0 {
                assert_eq!(
                    r.memory.read(0x20000, 4, READ).unwrap(),
                    result.to_le_bytes()
                );
                assert_eq!(r.state[0], input as u64);
            } else {
                assert_eq!(r.state[0], result as u64, "{w:x?}");
            }
            let n = count & 31;
            let mask = if n == 0 {
                0x8d5
            } else if n == 1 {
                0x8c5
            } else {
                0xc5
            };
            assert_eq!(r.flags() & mask, flags & mask, "{w:x?}");
            assert_eq!(r.state[1], count as u64);
            assert_eq!(r.state[2], source as u64);
            assert_eq!(r.state[3], 0x20000);
        }
    }
}
