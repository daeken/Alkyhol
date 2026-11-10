//! Physical ARM execution of the sign-mask family reached by original DAO.
use crate::{Stop, tests::cpu};

fn encoding(width: u32, destination: u8, source: u8) -> Vec<u8> {
    let mut code = if width == 32 { vec![] } else { vec![0x66] };
    code.extend([
        0x0f,
        if width == 8 { 0xd7 } else { 0x50 },
        0xc0 | destination << 3 | source,
    ]);
    code
}
fn vector(mask: u32, width: u32) -> u128 {
    let mut value = 0;
    for lane in 0..128 / width {
        // Float forms use signaling-NaN payloads. Extraction must not perform
        // arithmetic or signal invalid, even when exceptions are unmasked.
        let payload = match width {
            8 => u128::from((lane * 31 + mask * 7) & 0x7f),
            32 => 0x7f800001,
            64 => 0x7ff0000000000001,
            _ => unreachable!(),
        };
        value |= (payload | (u128::from((mask >> lane) & 1) << (width - 1))) << (lane * width);
    }
    value
}
#[test]
fn movemask_exhausts_every_output_mask_with_native_emitted_instructions() {
    for width in [8, 32, 64] {
        let code = encoding(width, 0, 1);
        let mut r = cpu(&code);
        for mask in 0..1u32 << (128 / width) {
            r.state[0] = u64::MAX;
            r.state[16] = 0xad7;
            r.state[17] = 0x10000;
            let source = vector(mask, width);
            r.state[26] = source as u64;
            r.state[27] = (source >> 64) as u64;
            r.memory.fpu.borrow_mut().mxcsr = 0x0041;
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            assert_eq!(r.state[0], mask as u64, "width={width} mask={mask:x}");
            assert_eq!(r.state[16], 0xad7);
            assert_eq!(&r.state[26..28], &[source as u64, (source >> 64) as u64]);
            assert_eq!(r.memory.fpu.borrow().mxcsr, 0x0041);
        }
    }
}
#[test]
fn movemask_routes_all_registers_and_preserves_unrelated_integer_vector_and_fpu_state() {
    for width in [8, 32, 64] {
        let maximum = (1u32 << (128 / width)) - 1;
        for dest in 0..8u8 {
            for src in 0..8u8 {
                let code = encoding(width, dest, src);
                let mut r = cpu(&code);
                for mask in [
                    0,
                    1,
                    maximum,
                    maximum / 3,
                    maximum / 3 * 2,
                    1 << (128 / width - 1),
                ] {
                    for (i, value) in r.state[..8].iter_mut().enumerate() {
                        *value = 0xfedcba9801234567u64.rotate_left(i as u32 * 3);
                    }
                    for (i, value) in r.state[24..40].iter_mut().enumerate() {
                        *value = 0x00ff80417fc00001u64.rotate_left(i as u32 * 5);
                    }
                    let input = vector(mask, width);
                    let at = 24 + src as usize * 2;
                    r.state[at] = input as u64;
                    r.state[at + 1] = (input >> 64) as u64;
                    r.state[16] = 0xad7;
                    r.state[17] = 0x10000;
                    let before = r.state;
                    {
                        let mut f = r.memory.fpu.borrow_mut();
                        f.control = 0x137f;
                        f.status = 0x5523;
                        f.tag = 0x1234;
                        f.mxcsr = 0x9fc7;
                        f.registers = std::array::from_fn(|i| {
                            u128::from(i as u64) << 64 | 0xfedcba9876543210
                        });
                    }
                    let fpu_before = r.memory.fpu.borrow().clone();
                    assert!(matches!(r.run(1), Stop::Budget { .. }));
                    let mut expected = before;
                    expected[dest as usize] = mask as u64;
                    expected[17] += code.len() as u64;
                    assert_eq!(
                        r.state, expected,
                        "width={width} dest={dest} src={src} mask={mask:x}"
                    );
                    let f = r.memory.fpu.borrow();
                    assert_eq!(
                        (f.control, f.status, f.tag, f.mxcsr, f.registers),
                        (
                            fpu_before.control,
                            fpu_before.status,
                            fpu_before.tag,
                            fpu_before.mxcsr,
                            fpu_before.registers
                        )
                    );
                }
            }
        }
    }
}
#[test]
fn movemask_declines_memory_mmx_and_invalid_prefix_forms_without_mutation() {
    for code in [
        vec![0x66, 0x0f, 0xd7, 0x00],
        vec![0x0f, 0x50, 0x00],
        vec![0x66, 0x0f, 0x50, 0x00],
        vec![0x0f, 0xd7, 0xc0],
        vec![0xf0, 0x66, 0x0f, 0xd7, 0xc0],
        vec![0x67, 0x66, 0x0f, 0xd7, 0xc0],
        vec![0xc5, 0xf9, 0xd7, 0xc0],
    ] {
        let mut r = cpu(&code);
        let before = r.state;
        assert!(matches!(r.run(1), Stop::Unsupported { .. }), "{code:02x?}");
        assert_eq!(r.state, before);
    }
}

#[test]
#[ignore = "requires independent original x86 assembly observations"]
fn actual_x86_movemask_observations() {
    let data = std::fs::read(std::env::var_os("ALKYHOL_MOVEMASK_REFERENCE").unwrap()).unwrap();
    assert_eq!(data.len(), 65556 * 64);
    let mut at = 0;
    let (
        mut result_differences,
        mut mxcsr_differences,
        mut flags_differences,
        mut loaded_differences,
    ) = (0u64, 0u64, 0u64, 0u64);
    for (kind, width) in [8, 32, 64].into_iter().enumerate() {
        let mut r = cpu(&encoding(width, 0, 1));
        for mask in 0..1u32 << (128 / width) {
            let row = &data[at..at + 64];
            at += 64;
            let words: Vec<u32> = row
                .chunks_exact(4)
                .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
                .collect();
            assert_eq!(&words[..3], &[kind as u32, width, mask]);
            loaded_differences += u64::from(words[3] != words[4]);
            assert_eq!(words[14], mask & 3);
            assert_eq!(words[15], 0x715e7a11);
            r.state[0] = u64::MAX;
            r.state[16] = words[12] as u64;
            r.state[17] = 0x10000;
            r.state[26] = u64::from_le_bytes(row[24..32].try_into().unwrap());
            r.state[27] = u64::from_le_bytes(row[32..40].try_into().unwrap());
            let source = [r.state[26], r.state[27]];
            r.memory.fpu.borrow_mut().mxcsr = words[4];
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            let observed = u64::from_le_bytes(row[40..48].try_into().unwrap());
            assert_eq!(observed, mask as u64);
            result_differences += u64::from(r.state[0] != observed);
            assert_eq!([r.state[26], r.state[27]], source);
            mxcsr_differences += u64::from(r.memory.fpu.borrow().mxcsr != words[5]);
            flags_differences += u64::from(r.flags() & 0xcd5 != words[13] & 0xcd5);
        }
    }
    let report = format!(
        "{{\"rows\":65556,\"result_differences\":{result_differences},\"mxcsr_differences\":{mxcsr_differences},\"flags_differences\":{flags_differences},\"requested_mxcsr_load_differences\":{loaded_differences}}}"
    );
    if let Some(path) = std::env::var_os("ALKYHOL_MOVEMASK_REPORT") {
        std::fs::write(path, &report).unwrap();
    }
    eprintln!("Sign-mask observations: {report}");
    // Keep requested-control mismatches strict too. The separate counters do
    // not turn a changed initial environment into a successful full comparison.
    assert_eq!(
        (
            result_differences,
            mxcsr_differences,
            flags_differences,
            loaded_differences
        ),
        (0, 0, 0, 0)
    );
}
