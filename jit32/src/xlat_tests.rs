use crate::{Stop, memory::READ, tests::cpu};
#[test]
fn xlat_selected_byte_wrapping_segments_and_precise_faults() {
    for (code, base, index, segment, address) in [
        (vec![0xd7], 0x20000u32, 255u8, 0u64, 0x200ffu64),
        (vec![0x66, 0xd7], 0xfffffff0, 0x20, 0, 0x10),
        (vec![0x64, 0xd7], 0xfffffff0, 0x20, 0x30000, 0x30010),
    ] {
        for mapped in [false, true] {
            let mut r = cpu(&code);
            r.state[0] = 0xa55a8000 | index as u64;
            r.state[3] = base as u64;
            r.state[22] = segment;
            r.state[16] = 0xad7;
            if mapped {
                r.memory.map(address as u32, 1, READ, &[0x96]).unwrap();
            }
            let before = r.state;
            let fpu = r.memory.fpu.borrow().clone();
            let stop = r.run(1);
            if mapped {
                assert!(matches!(stop, Stop::Budget { .. }), "{stop:?}");
                assert_eq!(r.state[0], 0xa55a8096);
            } else {
                assert!(matches!(stop, Stop::Fault { .. }), "{stop:?}");
                assert_eq!(r.state[0], before[0]);
            }
            assert_eq!(&r.state[1..17], &before[1..17]);
            assert_eq!(&r.state[18..], &before[18..]);
            assert_eq!(r.memory.fpu.borrow().registers, fpu.registers);
        }
    }
    for code in [vec![0x67, 0xd7], vec![0xf0, 0xd7], vec![0x66; 16]] {
        let mut r = cpu(&code);
        assert!(matches!(r.run(1), Stop::Unsupported { .. }));
    }
    let mut code = vec![0x66; 14];
    code.push(0xd7);
    let mut r = cpu(&code);
    r.state[3] = 0x20000;
    r.memory.map(0x20000, 1, READ, &[7]).unwrap();
    assert!(matches!(r.run(1), Stop::Budget { .. }));
    assert_eq!(r.state[0], 7);
    assert_eq!(r.state[17], 0x1000f);
}
#[test]
#[ignore = "run tools/check_jit32_xlat.py for actual x86 observations"]
fn actual_x86_xlat_observations() {
    let data = std::fs::read(std::env::var_os("ALKYHOL_XLAT_REFERENCE").unwrap()).unwrap();
    assert_eq!(data.len(), 512 * 16);
    let table: Vec<u8> = (0u32..256).map(|i| ((i * 73) ^ 0xa5) as u8).collect();
    for row in data.chunks_exact(16) {
        let words: Vec<u32> = row
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let [input, initial, result, flags] = words[..] else {
            unreachable!()
        };
        let mut r = cpu(&[0xd7]);
        r.state[0] = input as u64;
        r.state[3] = 0x20000;
        r.state[16] = initial as u64;
        r.memory.map(0x20000, 256, READ, &table).unwrap();
        assert!(matches!(r.run(1), Stop::Budget { .. }));
        assert_eq!(r.state[0], result as u64);
        assert_eq!(r.flags() & 0x8d5, flags & 0x8d5);
        assert_eq!(r.state[3], 0x20000);
        assert_eq!(r.memory.read(0x20000, 256, READ).unwrap(), table);
    }
}
