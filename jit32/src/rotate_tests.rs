//! Narrow rotates use widened native temporaries; x86 flag semantics stay lifted.
use crate::{
    Stop,
    memory::{READ, WRITE},
    tests::cpu,
};
fn encoding(form: u32, dir: u32, count: Option<u8>) -> Vec<u8> {
    let mut code = vec![];
    if form == 2 || form == 4 {
        code.push(0x66);
    }
    code.push(if count.is_some() {
        if form == 2 || form == 4 { 0xc1 } else { 0xc0 }
    } else if form == 2 || form == 4 {
        0xd3
    } else {
        0xd2
    });
    code.push(
        (match form {
            0 | 2 => 0xc0,
            1 => 0xc4,
            _ => 3,
        }) | ((dir as u8) << 3),
    );
    if let Some(count) = count {
        code.push(count);
    }
    code
}
#[test]
fn narrow_rotate_counts_partial_registers_and_faults() {
    for form in 0..5 {
        for dir in 0..2 {
            for count in [0u8, 1, 7, 8, 15, 16, 31, 32, 33, 255] {
                let code = encoding(form, dir, Some(count));
                let mut r = cpu(&code);
                r.state[0] = 0xa55a8001;
                r.state[1] = count as u64;
                r.state[3] = 0x20000;
                r.state[16] = 0xad7;
                r.memory
                    .map(0x20000, 4, READ | WRITE, &0xa55a8001u32.to_le_bytes())
                    .unwrap();
                assert!(matches!(r.run(1), Stop::Budget { .. }));
                let width = if form == 2 || form == 4 { 16 } else { 8 };
                let shift = if form == 1 { 8 } else { 0 };
                let mask = (1u32 << width) - 1;
                let input = (0xa55a8001 >> shift) & mask;
                let n = count as u32 & (width - 1);
                let output = if dir == 0 {
                    (input << n) | (input >> ((width - n) & (width - 1)))
                } else {
                    (input >> n) | (input << ((width - n) & (width - 1)))
                } & mask;
                let expected = (0xa55a8001 & !(mask << shift)) | (output << shift);
                if form < 3 {
                    assert_eq!(r.state[0], expected as u64);
                } else {
                    assert_eq!(
                        u32::from_le_bytes(
                            r.memory.read(0x20000, 4, READ).unwrap().try_into().unwrap()
                        ),
                        expected
                    );
                }
                assert_eq!(r.state[1], count as u64);
                assert_eq!(r.flags() & 0xd4, 0xad7 & 0xd4);
                // Count zero preserves all flags; single masked step defines OF.
                if count & 31 == 0 {
                    assert_eq!(r.flags() & 0x8d5, 0xad7 & 0x8d5);
                }
            }
        }
    }
    for form in [3, 4] {
        let mut r = cpu(&encoding(form, 0, None));
        r.state[1] = 1;
        r.state[3] = 0x20000;
        r.memory
            .map(0x20000, 4, READ, &0xa55a8001u32.to_le_bytes())
            .unwrap();
        let before = r.state;
        assert!(matches!(r.run(1), Stop::Fault { .. }));
        assert_eq!(&r.state[..17], &before[..17]);
        assert_eq!(
            r.memory.read(0x20000, 4, READ).unwrap(),
            &0xa55a8001u32.to_le_bytes()
        );
    }
}
#[test]
#[ignore = "run tools/check_jit32_rotates.py for actual x86 observations"]
fn actual_x86_rotate_observations() {
    let data = std::fs::read(std::env::var_os("ALKYHOL_ROTATE_REFERENCE").unwrap()).unwrap();
    assert_eq!(data.len(), 5200 * 28);
    for row in data.chunks_exact(28) {
        let w: Vec<u32> = row
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let [form, dir, count, input, initial, result, flags] = w[..] else {
            unreachable!()
        };
        for immediate in [false, true] {
            let mut r = cpu(&encoding(form, dir, immediate.then_some(count as u8)));
            r.state[0] = input as u64;
            r.state[1] = count as u64;
            r.state[3] = 0x20000;
            r.state[16] = initial as u64;
            r.memory
                .map(0x20000, 4, READ | WRITE, &input.to_le_bytes())
                .unwrap();
            assert!(matches!(r.run(1), Stop::Budget { .. }), "{w:x?}");
            if form < 3 {
                assert_eq!(r.state[0], result as u64, "{w:x?}");
            } else {
                assert_eq!(
                    r.memory.read(0x20000, 4, READ).unwrap(),
                    result.to_le_bytes(),
                    "{w:x?}"
                );
                assert_eq!(r.state[0], input as u64);
            }
            let mask = if count & 31 <= 1 { 0x8d5 } else { 0xd5 };
            assert_eq!(
                r.flags() & mask,
                flags & mask,
                "{w:x?} immediate={immediate}"
            );
            assert_eq!(r.state[1], count as u64);
            assert_eq!(r.state[3], 0x20000);
        }
    }
}
