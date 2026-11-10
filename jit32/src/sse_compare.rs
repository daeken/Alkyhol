//! Legacy SSE comparisons with guest MXCSR and no host floating environment.
use crate::memory::{Memory, READ};
use rustc_apfloat::{
    Float,
    ieee::{Double, Single},
};
use std::cmp::Ordering;
pub fn operation(name: &str) -> Option<u64> {
    Some(match name {
        "CMPSS" => 0,
        "CMPSD-X" => 1,
        "CMPPS" => 2,
        "CMPPD" => 3,
        "COMISS" => 4,
        "UCOMISS" => 5,
        "COMISD" => 6,
        "UCOMISD" => 7,
        _ => return None,
    })
}
fn relation<F: Float>(
    a: u128,
    b: u128,
    signal_nan: bool,
    mxcsr: u32,
) -> Result<(Option<Ordering>, u32), &'static str> {
    let (mut a, mut b) = (F::from_bits(a), F::from_bits(b));
    if mxcsr & 0x40 != 0 {
        if a.is_denormal() {
            a = F::ZERO.copy_sign(a)
        }
        if b.is_denormal() {
            b = F::ZERO.copy_sign(b)
        }
    }
    let unordered = a.is_nan() || b.is_nan();
    let invalid = a.is_signaling() || b.is_signaling() || (unordered && signal_nan);
    // Follow the frozen native Windows AMD observation. Rosetta additionally
    // sets DE in 152 ORD/UNORD matrix rows; its strict comparison stays failed
    // and the distinct raw observations are preserved, rather than blended.
    let flags = u32::from(invalid)
        | if !unordered && (a.is_denormal() || b.is_denormal()) {
            2
        } else {
            0
        };
    if flags & !(mxcsr >> 7) & 0x3f != 0 {
        return Err("unmasked SIMD comparison exception requires #XM delivery");
    }
    Ok((
        if unordered { None } else { a.partial_cmp(&b) },
        mxcsr | flags,
    ))
}
fn predicate(order: Option<Ordering>, pred: u64) -> bool {
    match pred & 7 {
        0 => order == Some(Ordering::Equal),
        1 => order == Some(Ordering::Less),
        2 => matches!(order, Some(Ordering::Less | Ordering::Equal)),
        3 => order.is_none(),
        4 => order != Some(Ordering::Equal),
        5 => order != Some(Ordering::Less),
        6 => !matches!(order, Some(Ordering::Less | Ordering::Equal)),
        7 => order.is_some(),
        _ => unreachable!(),
    }
}
fn perform(
    memory: &mut Memory,
    registers: &mut [u64],
    description: u64,
    address: u64,
) -> Result<(), &'static str> {
    let kind = description & 15;
    if kind > 7 {
        return Err("unknown SIMD comparison operation");
    }
    let dst = (description >> 8 & 7) as usize;
    let src = (description >> 12 & 7) as usize;
    let pred = description >> 20 & 7;
    let packed = kind == 2 || kind == 3;
    let width = if kind == 1 || kind == 3 || kind >= 6 {
        64
    } else {
        32
    };
    let old = registers[24 + dst * 2] as u128 | ((registers[25 + dst * 2] as u128) << 64);
    let source = if description & (1 << 16) != 0 {
        registers[24 + src * 2] as u128 | ((registers[25 + src * 2] as u128) << 64)
    } else {
        if packed && address & 15 != 0 {
            return Err("unaligned legacy packed SIMD comparison requires #GP delivery");
        }
        let length = if packed { 16 } else { width as usize / 8 };
        if let Err(fault) = memory.check(address, length, READ) {
            memory.fault = Some(fault);
            return Err("SIMD comparison source memory fault");
        }
        let bytes = memory.read(address, length, READ).unwrap();
        let mut value = [0; 16];
        value[..length].copy_from_slice(bytes);
        u128::from_le_bytes(value)
    };
    let signal_nan = if kind < 4 {
        matches!(pred, 1 | 2 | 5 | 6)
    } else {
        kind == 4 || kind == 6
    };
    let mut mxcsr = memory.fpu.borrow().mxcsr;
    let mask = (1u128 << width) - 1;
    let mut output = old;
    let mut flags = 0;
    for lane in 0..if packed { 128 / width } else { 1 } {
        let shift = lane * width;
        let a = old >> shift & mask;
        let b = source >> shift & mask;
        let (order, next) = if width == 32 {
            relation::<Single>(a, b, signal_nan, mxcsr)?
        } else {
            relation::<Double>(a, b, signal_nan, mxcsr)?
        };
        mxcsr = next;
        if kind < 4 {
            let value = if predicate(order, pred) { mask } else { 0 };
            output = (output & !(mask << shift)) | (value << shift);
        } else {
            flags = match order {
                None => 0x45,
                Some(Ordering::Less) => 1,
                Some(Ordering::Equal) => 0x40,
                Some(Ordering::Greater) => 0,
            };
        }
    }
    if kind < 4 {
        registers[24 + dst * 2] = output as u64;
        registers[25 + dst * 2] = (output >> 64) as u64;
    } else {
        registers[16] = (registers[16] & !0x8d5) | flags;
    }
    memory.fpu.borrow_mut().mxcsr = mxcsr;
    Ok(())
}
pub unsafe extern "C" fn execute(
    memory: *mut Memory,
    registers: *mut u64,
    description: u64,
    address: u64,
) -> u64 {
    let memory = unsafe { &mut *memory };
    let registers = unsafe { std::slice::from_raw_parts_mut(registers, 91) };
    match perform(memory, registers, description, address) {
        Ok(()) => 1,
        Err(reason) => {
            if memory.fault.is_none() {
                memory.cpu_fault = Some(reason)
            }
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Stop, memory::READ, tests::cpu};
    fn code(kind: u32, pred: u8, memory: bool) -> Vec<u8> {
        let mut code = vec![];
        match kind {
            0 => code.push(0xf3),
            1 => code.push(0xf2),
            3 | 6 | 7 => code.push(0x66),
            _ => (),
        }
        code.extend([
            0x0f,
            if kind < 4 {
                0xc2
            } else if kind == 4 || kind == 6 {
                0x2f
            } else {
                0x2e
            },
            if memory { 0 } else { 0xc1 },
        ]);
        if kind < 4 {
            code.push(pred)
        }
        code
    }
    #[test]
    #[ignore = "requires actual observations; use tools/check_jit32_compare_windows.py for the frozen Windows contract"]
    fn actual_x86_comparison_observations() {
        let data = std::fs::read(std::env::var_os("ALKYHOL_COMPARE_REFERENCE").unwrap()).unwrap();
        assert_eq!(data.len(), 18432 * 72);
        for row in data.chunks_exact(72) {
            let w: Vec<u32> = row
                .chunks_exact(4)
                .map(|b| u32::from_le_bytes(b.try_into().unwrap()))
                .collect();
            let (kind, pred, before, flags) = (w[0], w[1], w[2], w[3]);
            let mut code = code(kind, pred as u8, false);
            code.push(0xc3);
            let mut r = cpu(&code);
            for i in 0..4 {
                r.state[24 + i] =
                    u64::from_le_bytes(row[16 + i * 8..24 + i * 8].try_into().unwrap());
            }
            r.state[16] = flags as u64;
            r.memory.fpu.borrow_mut().mxcsr = before;
            assert_eq!(
                r.run(3),
                Stop::Returned(0),
                "kind {kind} pred {pred} before {before:x}"
            );
            for i in 0..2 {
                assert_eq!(
                    r.state[24 + i],
                    u64::from_le_bytes(row[48 + i * 8..56 + i * 8].try_into().unwrap()),
                    "kind {kind} pred {pred} before {before:x} input {:x?}",
                    &w[4..12]
                );
            }
            assert_eq!(
                r.memory.fpu.borrow().mxcsr,
                w[16],
                "kind {kind} pred {pred} before {before:x} input {:x?}",
                &w[4..12]
            );
            assert_eq!(
                r.flags() & 0xcd5,
                w[17] & 0xcd5,
                "kind {kind} pred {pred} before {before:x}"
            );
            assert_eq!(
                &r.state[26..28],
                &[
                    u64::from_le_bytes(row[32..40].try_into().unwrap()),
                    u64::from_le_bytes(row[40..48].try_into().unwrap())
                ]
            );
        }
    }
    fn width(kind: u32) -> usize {
        if kind == 1 || kind == 3 || kind >= 6 {
            64
        } else {
            32
        }
    }
    fn vector(kind: u32, bits: u64) -> u128 {
        (0..128 / width(kind)).fold(0, |out, lane| {
            out | ((bits as u128) << (lane * width(kind)))
        })
    }
    #[test]
    fn memory_comparisons_preserve_sources_and_admit_unaligned_scalars() {
        for kind in 0..8 {
            let packed = kind == 2 || kind == 3;
            let bits = if width(kind) == 32 {
                0x3f800000
            } else {
                0x3ff0000000000000
            };
            let input = vector(kind, bits);
            let length = if packed { 16 } else { width(kind) / 8 };
            let offset = usize::from(!packed);
            let mut bytes = vec![0x5a; offset];
            bytes.extend_from_slice(&input.to_le_bytes()[..length]);
            let mut r = cpu(&code(kind, 0, true));
            r.memory.map(0x20000, bytes.len(), READ, &bytes).unwrap();
            r.state[0] = 0x20000 + offset as u64;
            r.state[24] = input as u64;
            r.state[25] = (input >> 64) as u64;
            r.state[16] = 0xad7;
            r.memory.fpu.borrow_mut().mxcsr = 0x9fc3;
            let before = r.registers();
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            assert_eq!(r.registers(), before);
            let mask = (1u128 << width(kind)) - 1;
            let expected = if packed {
                u128::MAX
            } else if kind < 4 {
                input | mask
            } else {
                input
            };
            assert_eq!(
                r.state[24] as u128 | ((r.state[25] as u128) << 64),
                expected
            );
            assert_eq!(r.flags(), if kind < 4 { 0xad7 } else { 0x242 });
            assert_eq!(r.memory.fpu.borrow().mxcsr, 0x9fc3);
            assert_eq!(r.memory.read(0x20000, bytes.len(), READ).unwrap(), bytes);
            for address in [0, 0x30000].into_iter().chain(packed.then_some(0x20001)) {
                let mut r = cpu(&code(kind, 0, true));
                r.memory.map(0x20000, 32, READ, &[0; 32]).unwrap();
                r.memory.map(0x30000, 1, READ, &[0]).unwrap();
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
    fn unmasked_comparison_failures_do_not_commit_lanes_flags_or_mxcsr() {
        for kind in 0..8 {
            for denormal in [false, true] {
                let input = vector(
                    kind,
                    if width(kind) == 32 {
                        0x3f800000
                    } else {
                        0x3ff0000000000000
                    },
                );
                let bad: u64 = if denormal {
                    1
                } else if width(kind) == 32 {
                    0x7f801234
                } else {
                    0x7ff0000000001234
                };
                let source = if kind == 2 || kind == 3 {
                    (input & ((1u128 << (128 - width(kind))) - 1))
                        | ((bad as u128) << (128 - width(kind)))
                } else {
                    bad as u128
                };
                let mut r = cpu(&code(kind, 0, false));
                r.state[24] = input as u64;
                r.state[25] = (input >> 64) as u64;
                r.state[26] = source as u64;
                r.state[27] = (source >> 64) as u64;
                r.state[16] = 0xad7;
                let mxcsr = 0x1f80 & !if denormal { 0x100 } else { 0x80 };
                r.memory.fpu.borrow_mut().mxcsr = mxcsr;
                let before = r.state;
                assert!(matches!(r.run(1), Stop::CpuFault { .. }));
                assert_eq!(r.state, before);
                assert_eq!(r.memory.fpu.borrow().mxcsr, mxcsr);
            }
        }
    }
    #[test]
    fn high_immediate_bits_are_ignored_by_legacy_predicates() {
        for kind in 0..4 {
            for low in 0..8 {
                for high in [0, 8, 0xf8] {
                    let mut r = cpu(&code(kind, low | high, false));
                    let a = vector(
                        kind,
                        if width(kind) == 32 {
                            0x3f800000
                        } else {
                            0x3ff0000000000000
                        },
                    );
                    let b = vector(
                        kind,
                        if width(kind) == 32 {
                            0x40000000
                        } else {
                            0x4000000000000000
                        },
                    );
                    r.state[24] = a as u64;
                    r.state[25] = (a >> 64) as u64;
                    r.state[26] = b as u64;
                    r.state[27] = (b >> 64) as u64;
                    assert!(matches!(r.run(1), Stop::Budget { .. }));
                    let mask = (1u128 << width(kind)) - 1;
                    let yes = matches!(low, 1 | 2 | 4 | 7);
                    let expected = if kind >= 2 {
                        if yes { u128::MAX } else { 0 }
                    } else {
                        (a & !mask) | if yes { mask } else { 0 }
                    };
                    assert_eq!(
                        r.state[24] as u128 | ((r.state[25] as u128) << 64),
                        expected
                    );
                }
            }
        }
    }
    #[test]
    fn native_windows_nan_priority_and_aliasing_are_explicit() {
        for kind in 0..4 {
            for pred in [3, 7] {
                let nan = if width(kind) == 32 {
                    0x7fc01234
                } else {
                    0x7ff8000000001234
                };
                let a = vector(kind, nan);
                let b = vector(kind, 1);
                let mut r = cpu(&code(kind, pred, false));
                r.state[24] = a as u64;
                r.state[25] = (a >> 64) as u64;
                r.state[26] = b as u64;
                r.state[27] = (b >> 64) as u64;
                assert!(matches!(r.run(1), Stop::Budget { .. }));
                assert_eq!(r.memory.fpu.borrow().mxcsr, 0x1f80);
            }
        }
        for kind in 0..8 {
            let mut bytes = code(kind, 0, false);
            let modrm = if bytes[0] == 0x0f { 2 } else { 3 };
            bytes[modrm] = 0xc0;
            let input = vector(
                kind,
                if width(kind) == 32 {
                    0x3f800000
                } else {
                    0x3ff0000000000000
                },
            );
            let mut r = cpu(&bytes);
            r.state[24] = input as u64;
            r.state[25] = (input >> 64) as u64;
            r.state[16] = 0xad7;
            assert!(matches!(r.run(1), Stop::Budget { .. }));
            assert_eq!(r.flags(), if kind < 4 { 0xad7 } else { 0x242 });
            let expected = if kind >= 4 {
                input
            } else if kind >= 2 {
                u128::MAX
            } else {
                input | ((1u128 << width(kind)) - 1)
            };
            assert_eq!(
                r.state[24] as u128 | ((r.state[25] as u128) << 64),
                expected
            );
        }
    }
}
