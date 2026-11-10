//! Comparison string instructions preserve the last completed REP iteration.
use crate::{Stop, memory::READ, tests::cpu};
fn code(width: u32, scan: bool, repeat: u32) -> Vec<u8> {
    let mut out = vec![];
    if repeat != 0 {
        out.push(if repeat == 1 { 0xf3 } else { 0xf2 });
    }
    if width == 16 {
        out.push(0x66);
    }
    out.push(if scan {
        if width == 8 { 0xae } else { 0xaf }
    } else if width == 8 {
        0xa6
    } else {
        0xa7
    });
    out
}
#[test]
#[ignore = "requires independent actual x86 observations from tools/check_jit32_strings.py"]
fn actual_x86_string_comparison_observations() {
    let data = std::fs::read(std::env::var_os("ALKYHOL_STRING_REFERENCE").unwrap()).unwrap();
    assert_eq!(data.len(), 432 * 104);
    for row in data.chunks_exact(104) {
        let w: Vec<u32> = row[..40]
            .chunks_exact(4)
            .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
            .collect();
        let [
            width,
            scan,
            repeat,
            backward,
            count,
            acc,
            si,
            di,
            remaining,
            flags,
        ] = w[..]
        else {
            unreachable!()
        };
        let mut bytes = code(width, scan != 0, repeat);
        bytes.push(0xc3);
        let mut r = cpu(&bytes);
        r.memory.map(0x20000, 32, READ, &row[40..72]).unwrap();
        r.memory.map(0x30000, 32, READ, &row[72..104]).unwrap();
        r.state[0] = acc as u64;
        r.state[1] = count as u64;
        r.state[6] = 0x20000 + if backward != 0 { 20 } else { 8 };
        r.state[7] = 0x30000 + if backward != 0 { 20 } else { 8 };
        r.state[16] = 0xad7 | (backward as u64) << 10;
        assert_eq!(r.run(10), Stop::Returned(acc), "{w:x?}");
        assert_eq!(r.state[6], 0x20000 + si as u64, "{w:x?}");
        assert_eq!(r.state[7], 0x30000 + di as u64, "{w:x?}");
        assert_eq!(r.state[1], remaining as u64, "{w:x?}");
        assert_eq!(r.flags() & 0xcd5, flags & 0xcd5, "{w:x?}");
        assert_eq!(r.memory.read(0x20000, 32, READ).unwrap(), &row[40..72]);
        assert_eq!(r.memory.read(0x30000, 32, READ).unwrap(), &row[72..104]);
    }
}
#[test]
fn comparison_strings_leave_completed_iterations_on_a_later_fault() {
    for width in [8, 16, 32] {
        for scan in [false, true] {
            for backward in [false, true] {
                let size = width as usize / 8;
                let mut r = cpu(&code(width, scan, 1));
                r.memory
                    .map(0x20000, size * 2, READ, &[0; 8][..size * 2])
                    .unwrap();
                r.memory.map(0x30000, size, READ, &[0; 4][..size]).unwrap();
                r.state[0] = 0;
                r.state[1] = 2;
                r.state[6] = 0x20000 + if backward { size as u64 } else { 0 };
                r.state[7] = 0x30000;
                r.state[16] = 0xad7 | if backward { 0x400 } else { 0 };
                assert!(matches!(r.run(1), Stop::Budget { .. }));
                let before = r.state;
                assert_eq!(r.state[1], 1);
                assert_ne!(r.flags() & 0x40, 0);
                assert_eq!(r.flags() & 0x8d5, 0x44);
                assert!(matches!(r.run(1), Stop::Fault { .. }));
                assert_eq!(r.state, before);
            }
        }
    }
}
#[test]
fn zero_repeat_count_does_not_touch_invalid_sources_or_flags() {
    for width in [8, 16, 32] {
        for scan in [false, true] {
            for repeat in [1, 2] {
                let mut bytes = code(width, scan, repeat);
                bytes.push(0xc3);
                let mut r = cpu(&bytes);
                r.state[16] = 0xad7;
                assert_eq!(r.run(2), Stop::Returned(0));
                assert_eq!(r.flags(), 0xad7);
                assert_eq!(r.state[6], 0);
                assert_eq!(r.state[7], 0);
            }
        }
    }
}
