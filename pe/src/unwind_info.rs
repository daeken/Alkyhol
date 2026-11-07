//! Pure bounded AMD64 v1 unwind recipe validation shared by image loading and manual restoration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Action {
    Allocate(u64),
    Pop(u8),
    Save {
        register: u8,
        offset: u64,
        xmm: bool,
    },
}
type Result<T> = std::result::Result<T, &'static str>;
pub(crate) fn decode(header: [u8; 4], codes: &[u8], offset: u64, nop: bool) -> Result<Vec<Action>> {
    let [version, prologue, count, frame] = header;
    if version != 1 || frame != 0 {
        return Err("version, flags or frame register");
    }
    let in_prologue = offset < u64::from(prologue);
    if !in_prologue && !nop {
        return Err("only NOP body control points are supported");
    }
    if codes.len() != usize::from(count) * 2 {
        return Err("truncated unwind slots");
    }
    let mut actions = Vec::new();
    let mut boundary = offset == 0;
    let mut i = 0;
    let mut previous = u16::from(prologue) + 1;
    let mut frame_size = 0u64;
    let mut popped = false;
    let mut stack_changed = false;
    let mut allocated = 0u64;
    let mut saves = Vec::new();
    while i < codes.len() {
        let position = codes[i];
        let opcode = codes[i + 1] & 15;
        let info = codes[i + 1] >> 4;
        if position == 0 || u16::from(position) >= previous {
            return Err("unwind code offsets");
        }
        previous = u16::from(position);
        let (action, slots) = match opcode {
            0 if matches!(info, 3 | 5 | 6 | 7 | 12..=15) => (Action::Pop(info), 1),
            2 => (Action::Allocate(u64::from(info) * 8 + 8), 1),
            1 if info == 0 && i + 4 <= codes.len() => (
                Action::Allocate(u64::from(u16::from_le_bytes([codes[i + 2], codes[i + 3]])) * 8),
                2,
            ),
            1 if info == 1 && i + 6 <= codes.len() => (
                Action::Allocate(u64::from(u32::from_le_bytes(
                    codes[i + 2..i + 6].try_into().unwrap(),
                ))),
                3,
            ),
            4 | 8 if i + 4 <= codes.len() => {
                let xmm = opcode == 8;
                let scale = if xmm { 16 } else { 8 };
                (
                    Action::Save {
                        register: info,
                        offset: u64::from(u16::from_le_bytes([codes[i + 2], codes[i + 3]])) * scale,
                        xmm,
                    },
                    2,
                )
            }
            5 | 9 if i + 6 <= codes.len() => (
                Action::Save {
                    register: info,
                    offset: u64::from(u32::from_le_bytes(codes[i + 2..i + 6].try_into().unwrap())),
                    xmm: opcode == 9,
                },
                3,
            ),
            _ => return Err("unwind opcode, register or operand form"),
        };
        let bytes = match action {
            Action::Allocate(n) if n != 0 && n.is_multiple_of(8) => n,
            Action::Pop(_) => 8,
            Action::Save {
                register,
                offset,
                xmm,
            } => {
                let size = if xmm { 16 } else { 8 };
                if stack_changed
                    || !offset.is_multiple_of(size)
                    || if xmm {
                        !(6..=15).contains(&register)
                    } else {
                        !matches!(register, 3 | 5 | 6 | 7 | 12..=15)
                    }
                {
                    return Err("save register, alignment or ordering");
                }
                let end = offset.checked_add(size).ok_or("save extent overflow")?;
                if saves
                    .iter()
                    .any(|&(other_register, other_xmm, first, last)| {
                        (register == other_register && xmm == other_xmm)
                            || (offset < last && first < end)
                    })
                {
                    return Err("overlapping save slots or duplicate saved register");
                }
                saves.push((register, xmm, offset, end));
                0
            }
            _ => return Err("stack allocation size"),
        };
        match action {
            Action::Pop(_) => {
                popped = true;
                stack_changed = true;
            }
            Action::Allocate(_) if popped => return Err("push/allocate ordering"),
            Action::Allocate(n) => {
                allocated += n;
                stack_changed = true;
            }
            _ => (),
        }
        frame_size = frame_size.checked_add(bytes).ok_or("stack size overflow")?;
        if frame_size > 512 * 1024 * 1024 {
            return Err("stack frame bound");
        }
        // CodeOffset names the first instruction after the operation. Validate
        // even future operations before filtering; malformed skipped slots must
        // never turn an unsupported complete recipe into a successful unwind.
        if u64::from(position) == offset {
            boundary = true;
        }
        if !in_prologue || u64::from(position) <= offset {
            actions.push(action);
        }
        i += slots * 2;
    }
    if saves.iter().any(|&(_, _, _, end)| end > allocated) {
        return Err("save outside fixed allocation");
    }
    if in_prologue && !boundary {
        return Err("prologue control point is not a recorded operation boundary");
    }
    Ok(actions)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prologue_boundaries_apply_only_completed_operations_and_skip_operand_slots() {
        // XMM6 save, R13 save, large allocation, two pushes, in reverse order.
        let codes = [
            21, 0x68, 4, 0, 15, 0xd4, 5, 0, 10, 1, 21, 0, 3, 0xc0, 1, 0x30,
        ];
        let header = [1, 21, 8, 0];
        let all = decode(header, &codes, 21, true).unwrap();
        for (offset, first) in [(0, 5), (1, 4), (3, 3), (10, 2), (15, 1), (21, 0)] {
            assert_eq!(
                decode(header, &codes, offset, offset == 21).unwrap(),
                all[first..]
            );
        }
        for offset in [2, 4, 5, 9, 11, 14, 16, 20] {
            assert!(
                decode(header, &codes, offset, true).is_err(),
                "offset {offset}"
            );
        }
        assert!(decode(header, &codes, 21, false).is_err());
        // Every currently supported multi-slot form can be skipped atomically.
        for (opcode, operand) in [
            (0x01, vec![32, 0]),
            (0x11, vec![0, 1, 0, 0]),
            (0x34, vec![0, 0]),
            (0x35, vec![0, 0, 0, 0]),
            (0x68, vec![0, 0]),
            (0x69, vec![0, 0, 0, 0]),
        ] {
            let mut codes = vec![12, opcode];
            codes.extend(operand);
            if opcode & 15 >= 4 {
                codes.extend([4, 0x32]);
            }
            let header = [1, 16, (codes.len() / 2) as u8, 0];
            assert!(decode(header, &codes, 0, false).unwrap().is_empty());
            assert_eq!(
                decode(header, &codes, 12, false).unwrap(),
                decode(header, &codes, 16, true).unwrap()
            );
        }
    }
    #[test]
    fn skipped_future_recipe_errors_still_reject_before_any_stack_read() {
        for (header, codes) in [
            ([1, 16, 1, 0], vec![12, 3]),
            ([1, 16, 1, 0], vec![12, 1]),
            ([1, 16, 3, 0], vec![12, 0x34, 4, 0, 4, 0x12]),
            ([1, 16, 4, 0], vec![12, 0x34, 0, 0, 8, 0x34, 0, 0]),
            ([1, 16, 2, 0], vec![8, 0x30, 12, 0xc0]),
            ([1, 16, 2, 0], vec![12, 0x34, 0, 0]),
            ([9, 16, 0, 0], vec![]),
            ([1, 16, 0, 5], vec![]),
        ] {
            assert!(
                decode(header, &codes, 0, false).is_err(),
                "{header:?} {codes:?}"
            );
        }
    }
}
