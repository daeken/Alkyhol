//! Bounded manual AMD64 v1 body unwinding; no exception dispatch or resumption.
#![allow(non_snake_case)]
use crate::function_tables::RuntimeFunction;
use std::{ffi::c_void, ptr};

const RSP: usize = 152;
const RIP: usize = 248;
const CONTEXT_SIZE: usize = 1232;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    Allocate(u64),
    Pop(u8),
}
type Result<T> = std::result::Result<T, &'static str>;
fn unsupported(reason: &str) -> ! {
    eprintln!("Alkyhol PE virtual unwind unsupported: {reason}");
    std::process::exit(78)
}
fn decode(header: [u8; 4], codes: &[u8], offset: u64, nop: bool) -> Result<Vec<Action>> {
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
            _ => return Err("unwind opcode, register or operand form"),
        };
        let bytes = match action {
            Action::Allocate(n) if n != 0 && n.is_multiple_of(8) => n,
            Action::Pop(_) => 8,
            _ => return Err("stack allocation size"),
        };
        match action {
            Action::Pop(_) => popped = true,
            Action::Allocate(_) if popped => return Err("push/allocate ordering"),
            _ => (),
        }
        frame_size = frame_size.checked_add(bytes).ok_or("stack size overflow")?;
        if frame_size > 512 * 1024 * 1024 {
            return Err("stack frame bound");
        }
        actions.push(action);
        i += slots * 2;
    }
    Ok(actions)
}
struct Plan {
    restored: Vec<(usize, u64)>,
    initial_rsp: u64,
    rsp: u64,
    rip: u64,
}
unsafe fn plan(actions: &[Action], context: *mut u8) -> Result<Plan> {
    // Read only fields needed for this recipe, never uninitialized unrelated
    // CONTEXT bytes. Plan every write before changing the caller's record.
    let initial_rsp = unsafe { context.add(RSP).cast::<u64>().read_unaligned() };
    if !initial_rsp.is_multiple_of(8) {
        return Err("unaligned stack pointer");
    }
    let mut rsp = initial_rsp;
    let mut restored = Vec::new();
    for &action in actions {
        match action {
            Action::Allocate(bytes) => {
                rsp = rsp.checked_add(bytes).ok_or("stack address overflow")?
            }
            Action::Pop(register) => {
                let next = rsp.checked_add(8).ok_or("stack address overflow")?;
                let value = unsafe { (rsp as *const u64).read_unaligned() };
                restored.push((120 + usize::from(register) * 8, value));
                rsp = next;
            }
        }
    }
    let next = rsp.checked_add(8).ok_or("return address overflow")?;
    let rip = unsafe { (rsp as *const u64).read_unaligned() };
    Ok(Plan {
        restored,
        initial_rsp,
        rsp: next,
        rip,
    })
}
unsafe fn unwind(
    base: u64,
    pc: u64,
    entry: RuntimeFunction,
    context: *mut u8,
    establisher: *mut u64,
) -> Result<()> {
    let info = base
        .checked_add(u64::from(entry.unwind))
        .ok_or("unwind address overflow")?;
    let header = unsafe { (info as *const [u8; 4]).read_unaligned() };
    let length = usize::from(header[2]) * 2;
    info.checked_add(4 + length as u64)
        .ok_or("unwind slot address overflow")?;
    let codes = unsafe { std::slice::from_raw_parts((info + 4) as *const u8, length) };
    if u32::from(header[1]) > entry.end - entry.begin {
        return Err("prologue beyond function");
    }
    let offset = pc - base - u64::from(entry.begin);
    let actions = decode(
        header,
        codes,
        offset,
        unsafe { (pc as *const u8).read() } == 0x90,
    )?;
    let state = unsafe { plan(&actions, context) }?;
    unsafe {
        for (offset, value) in state.restored {
            context.add(offset).cast::<u64>().write_unaligned(value);
        }
        context.add(RSP).cast::<u64>().write_unaligned(state.rsp);
        context.add(RIP).cast::<u64>().write_unaligned(state.rip);
        establisher.write_unaligned(state.initial_rsp);
    }
    Ok(())
}
unsafe extern "win64" fn RtlVirtualUnwind(
    handler: u32,
    base: u64,
    pc: u64,
    function: *const RuntimeFunction,
    context: *mut c_void,
    handler_data: *mut *mut c_void,
    establisher: *mut u64,
    pointers: *mut c_void,
) -> *const c_void {
    if handler != 0 || !pointers.is_null() {
        unsupported("handlers or context pointers");
    }
    if context.is_null()
        || !(context as usize).is_multiple_of(16)
        || (context as usize).checked_add(CONTEXT_SIZE).is_none()
        || handler_data.is_null()
        || establisher.is_null()
    {
        unsupported("context or output pointers");
    }
    let entry = crate::process::with_function_tables(|registry| {
        registry.and_then(|r| r.selected(function, base, pc))
    })
    .unwrap_or_else(|| unsupported("unregistered entry, base or control point"));
    if let Err(reason) = unsafe { unwind(base, pc, entry, context.cast(), establisher) } {
        unsupported(reason);
    }
    // No handler/chain forms are accepted. HandlerData has no native parity
    // claim in this subset and is left untouched; no exception is dispatched.
    ptr::null()
}
pub(crate) fn address(name: &str) -> Option<usize> {
    (name == "RtlVirtualUnwind").then_some(RtlVirtualUnwind as *const () as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[repr(C, align(16))]
    struct Context([u8; CONTEXT_SIZE]);
    fn put(context: &mut Context, offset: usize, value: u64) {
        context.0[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn get(context: &Context, offset: usize) -> u64 {
        u64::from_le_bytes(context.0[offset..offset + 8].try_into().unwrap())
    }
    fn make_image(header: [u8; 4], codes: &[u8]) -> Vec<u8> {
        let mut image = vec![0x90; 1024];
        image[128..132].copy_from_slice(&header);
        image[132..132 + codes.len()].copy_from_slice(codes);
        image
    }
    fn entry() -> RuntimeFunction {
        RuntimeFunction {
            begin: 16,
            end: 96,
            unwind: 128,
        }
    }
    #[test]
    fn decoder_accepts_supported_sizes_and_registers_and_rejects_missing_or_unknown_semantics() {
        for info in 0..16 {
            assert_eq!(
                decode([1, 7, 1, 0], &[7, 2 | (info << 4)], 8, true).unwrap(),
                vec![Action::Allocate(u64::from(info) * 8 + 8)]
            );
        }
        for register in [3, 5, 6, 7, 12, 13, 14, 15] {
            assert_eq!(
                decode([1, 1, 1, 0], &[1, register << 4], 8, true).unwrap(),
                vec![Action::Pop(register)]
            );
        }
        assert_eq!(
            decode([1, 7, 2, 0], &[7, 1, 96, 0], 8, true).unwrap(),
            vec![Action::Allocate(768)]
        );
        assert_eq!(
            decode([1, 7, 3, 0], &[7, 0x11, 0, 0, 8, 0], 8, true).unwrap(),
            vec![Action::Allocate(0x80000)]
        );
        let original = [7, 0x42, 3, 0xc0, 1, 0x30];
        for length in 0..original.len() {
            assert!(decode([1, 7, 3, 0], &original[..length], 8, true).is_err());
        }
        for (header, codes, offset, nop) in [
            ([2, 0, 0, 0], vec![], 0, true),
            ([9, 0, 0, 0], vec![], 0, true),
            ([1, 0, 0, 5], vec![], 0, true),
            ([1, 7, 0, 0], vec![], 6, true),
            ([1, 0, 0, 0], vec![], 8, false),
            ([1, 7, 1, 0], vec![0, 2], 8, true),
            ([1, 7, 2, 0], vec![1, 0x30, 3, 0xc0], 8, true),
            ([1, 7, 2, 0], vec![3, 0x30, 3, 0xc0], 8, true),
            ([1, 7, 1, 0], vec![7, 0], 8, true),
            ([1, 7, 1, 0], vec![7, 3], 8, true),
            ([1, 7, 1, 0], vec![7, 8], 8, true),
            ([1, 7, 1, 0], vec![7, 10], 8, true),
            ([1, 7, 1, 0], vec![7, 1], 8, true),
            ([1, 7, 2, 0], vec![7, 1, 0, 0], 8, true),
            ([1, 7, 3, 0], vec![7, 0x11, 7, 0, 0, 0], 8, true),
            ([1, 7, 3, 0], vec![7, 0x11, 8, 0, 0, 0x20], 8, true),
            ([1, 7, 2, 0], vec![7, 0x30, 3, 2], 8, true),
        ] {
            assert!(
                decode(header, &codes, offset, nop).is_err(),
                "{header:?} {codes:?}"
            );
        }
    }
    #[test]
    fn body_unwind_restores_real_stack_words_and_preserves_unrelated_context_bytes() {
        let image = make_image([1, 7, 3, 0], &[7, 0x42, 3, 0xc0, 1, 0x30]);
        let base = image.as_ptr() as u64;
        let mut stack = [0u64; 16];
        stack[5] = 0xcccc12345678;
        stack[6] = 0xbbbbfedcba98;
        stack[7] = 0x1122334455667788;
        let mut context = Context([0xa5; CONTEXT_SIZE]);
        let original = stack.as_ptr() as u64;
        put(&mut context, RSP, original);
        let mut expected = context.0;
        expected[216..224].copy_from_slice(&stack[5].to_le_bytes());
        expected[144..152].copy_from_slice(&stack[6].to_le_bytes());
        expected[RSP..RSP + 8].copy_from_slice(&(original + 64).to_le_bytes());
        expected[RIP..RIP + 8].copy_from_slice(&stack[7].to_le_bytes());
        let mut frame = 0;
        unsafe { unwind(base, base + 32, entry(), context.0.as_mut_ptr(), &mut frame) }.unwrap();
        assert_eq!(frame, original);
        assert_eq!(context.0, expected);
        assert_eq!(
            stack[5..8],
            [0xcccc12345678, 0xbbbbfedcba98, 0x1122334455667788]
        );
        for register in [3, 5, 6, 7, 12, 13, 14, 15] {
            let stack = [0x1020304050607080u64, 0x8877665544332211];
            let mut context = Context([0; CONTEXT_SIZE]);
            put(&mut context, RSP, stack.as_ptr() as u64);
            let state = unsafe { plan(&[Action::Pop(register)], context.0.as_mut_ptr()) }.unwrap();
            assert_eq!(
                state.restored,
                vec![(120 + usize::from(register) * 8, stack[0])]
            );
            assert_eq!(state.rip, stack[1]);
        }
        let mut context = Context([0; CONTEXT_SIZE]);
        put(&mut context, RSP, u64::MAX - 7);
        assert!(unsafe { plan(&[Action::Allocate(8)], context.0.as_mut_ptr()) }.is_err());
        assert!(unsafe { plan(&[Action::Pop(3)], context.0.as_mut_ptr()) }.is_err());
        assert!(unsafe { plan(&[], context.0.as_mut_ptr()) }.is_err());
        put(&mut context, RSP, 3);
        assert!(unsafe { plan(&[], context.0.as_mut_ptr()) }.is_err());
        let unsupported = make_image([1, 1, 1, 0], &[1, 3]);
        let mut frame = 0xbeef;
        let saved = context.0;
        assert!(
            unsafe {
                unwind(
                    unsupported.as_ptr() as u64,
                    unsupported.as_ptr() as u64 + 32,
                    entry(),
                    context.0.as_mut_ptr(),
                    &mut frame,
                )
            }
            .is_err()
        );
        assert_eq!(context.0, saved);
        assert_eq!(frame, 0xbeef);
    }
    #[test]
    fn win64_eight_argument_bridge_selects_registered_entry_and_unwinds_leaf_context() {
        let _scope = crate::ProcessContext::install(&["unwind-unit".into()], &[]).unwrap();
        let image = make_image([1, 0, 0, 0], &[]);
        let base = image.as_ptr() as u64;
        let function = entry();
        let add: unsafe extern "win64" fn(*const RuntimeFunction, u32, u64) -> u8 = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "RtlAddFunctionTable").unwrap())
        };
        let virtual_unwind: unsafe extern "win64" fn(
            u32,
            u64,
            u64,
            *const RuntimeFunction,
            *mut c_void,
            *mut *mut c_void,
            *mut u64,
            *mut c_void,
        ) -> *const c_void = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "RtlVirtualUnwind").unwrap())
        };
        assert_eq!(unsafe { add(&function, 1, base) }, 1);
        let stack = [0x1234567887654321u64];
        let mut context = Context([0x5a; CONTEXT_SIZE]);
        let rsp = stack.as_ptr() as u64;
        put(&mut context, RSP, rsp);
        let mut frame = 0;
        let mut data = ptr::dangling_mut();
        let old = data;
        assert!(
            unsafe {
                virtual_unwind(
                    0,
                    base,
                    base + 32,
                    &function,
                    context.0.as_mut_ptr().cast(),
                    &mut data,
                    &mut frame,
                    ptr::null_mut(),
                )
            }
            .is_null()
        );
        assert_eq!(get(&context, RSP), rsp + 8);
        assert_eq!(get(&context, RIP), stack[0]);
        assert_eq!(frame, rsp);
        assert_eq!(data, old); // Host no-handler-data contract; native value unmeasured.
    }
    #[test]
    fn unsupported_frontend_cases_terminate_honestly_instead_of_returning_no_handler_success() {
        const CHILD: &str = "ALKYHOL_PE_VIRTUAL_UNWIND_CHILD";
        if let Ok(mode) = std::env::var(CHILD) {
            let _scope = crate::ProcessContext::install(&["unwind-child".into()], &[]).unwrap();
            let mut image = make_image([1, 0, 0, 0], &[]);
            if mode == "epilogue" {
                image[32] = 0xc3;
            }
            let base = image.as_ptr() as u64;
            let function = entry();
            let add: unsafe extern "win64" fn(*const RuntimeFunction, u32, u64) -> u8 = unsafe {
                std::mem::transmute(
                    crate::abi::resolve("kernel32.dll", "RtlAddFunctionTable").unwrap(),
                )
            };
            assert_eq!(unsafe { add(&function, 1, base) }, 1);
            let stack = [0x1234u64];
            let mut context = Context([0; CONTEXT_SIZE]);
            put(&mut context, RSP, stack.as_ptr() as u64);
            let mut frame = 0;
            let mut data = ptr::null_mut();
            unsafe {
                RtlVirtualUnwind(
                    if mode == "handler" { 1 } else { 0 },
                    base,
                    base + 32,
                    if mode == "unregistered" {
                        ptr::null()
                    } else {
                        &function
                    },
                    if mode == "null-context" {
                        ptr::null_mut()
                    } else {
                        context.0.as_mut_ptr().cast()
                    },
                    &mut data,
                    &mut frame,
                    if mode == "pointers" {
                        ptr::dangling_mut()
                    } else {
                        ptr::null_mut()
                    },
                );
            }
            panic!("unsupported virtual unwind returned no-handler success");
        }
        for mode in [
            "handler",
            "pointers",
            "unregistered",
            "null-context",
            "epilogue",
        ] {
            let run=std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact","virtual_unwind::tests::unsupported_frontend_cases_terminate_honestly_instead_of_returning_no_handler_success","--nocapture"])
                .env(CHILD,mode).output().unwrap();
            assert_eq!(run.status.code(), Some(78));
            assert!(
                String::from_utf8_lossy(&run.stderr)
                    .contains("Alkyhol PE virtual unwind unsupported:")
            );
        }
    }
}
