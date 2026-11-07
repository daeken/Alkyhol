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
    if offset < u64::from(prologue) || !nop {
        return Err("only NOP body control points are supported");
    }
    if codes.len() != usize::from(count) * 2 {
        return Err("truncated unwind slots");
    }
    let mut actions = Vec::new();
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
        actions.push(action);
        i += slots * 2;
    }
    if saves.iter().any(|&(_, _, _, end)| end > allocated) {
        return Err("save outside fixed allocation");
    }
    Ok(actions)
}
