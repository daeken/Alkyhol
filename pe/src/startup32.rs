//! Checked, staged Win32 startup buffers. Native pointers never enter PE32 slots.
use super::{Cpu32, LoadedImage32, LowMapping, PAGE, unsupported};
use alkyhol::ffi;
use std::{ffi::c_void, ptr};

const LIMIT: usize = 1024 * 1024;
fn startup_info(handles: [usize; 3]) -> [u8; 68] {
    // This host profile explicitly inherits its three streams. It has no
    // Windows desktop/title/position request or private CRT inheritance block.
    let mut bytes = [0u8; 68];
    bytes[..4].copy_from_slice(&68u32.to_le_bytes());
    bytes[44..48].copy_from_slice(&0x100u32.to_le_bytes());
    for (index, handle) in handles.into_iter().enumerate() {
        let value = match handle {
            0 => 0,
            usize::MAX => u32::MAX,
            _ => 0x70000001 + index as u32,
        };
        bytes[56 + index * 4..60 + index * 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes
}
pub(super) struct Block {
    mapping: LowMapping,
    size: usize,
    kind: BlockKind,
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    Command,
    Argv,
    Environment,
}
pub(super) fn permits(cpu: &Cpu32, address: usize, len: usize, write: bool) -> bool {
    let Some(end) = address.checked_add(len) else {
        return false;
    };
    cpu.blocks.borrow().iter().any(|b| {
        let base = b.mapping.base as usize;
        (!write || b.kind == BlockKind::Argv) && address >= base && end <= base + b.size
    })
}
fn fail(error: u32) -> u32 {
    ffi::SetLastError(error);
    0
}
fn checked(image: &LoadedImage32<'_>, address: u32, len: usize, write: bool) {
    if !image.permits(address as usize, len, write) {
        unsupported("startup buffer outside checked guest storage");
    }
}
fn read_units<T: Copy + Default + PartialEq>(
    image: &LoadedImage32<'_>,
    address: u32,
    length: i32,
) -> Option<Vec<T>> {
    if address == 0
        || !(address as usize).is_multiple_of(std::mem::align_of::<T>())
        || length == 0
        || length < -1
    {
        fail(87);
        return None;
    }
    let width = std::mem::size_of::<T>();
    let count = if length == -1 {
        LIMIT / width
    } else {
        length as usize
    };
    if count > LIMIT / width {
        fail(50);
        return None;
    }
    let mut result = Vec::new();
    for i in 0..count {
        let Some(at) = (address as usize)
            .checked_add(i * width)
            .filter(|&v| v <= u32::MAX as usize)
        else {
            unsupported("startup string address overflow");
        };
        checked(image, at as u32, width, false);
        let unit = unsafe { (at as *const T).read() };
        result.push(unit);
        if length == -1 && unit == T::default() {
            return Some(result);
        }
    }
    if length == -1 {
        fail(50);
        None
    } else {
        Some(result)
    }
}
pub(super) fn module_name(image: &LoadedImage32<'_>, address: u32) -> crate::Result<String> {
    let wide = read_units::<u16>(image, address, -1).ok_or("invalid PE32 UTF-16 module name")?;
    String::from_utf16(&wide[..wide.len() - 1])
        .map_err(|_| "invalid PE32 UTF-16 module name".into())
}
fn allocate(cpu: &Cpu32, bytes: &[u8], kind: BlockKind) -> u32 {
    if bytes.is_empty() || bytes.len() > LIMIT || cpu.blocks.borrow().len() >= 64 {
        unsupported("Win32 startup allocation limit");
    }
    let rounded = (bytes.len() + PAGE - 1) & !(PAGE - 1);
    let mapping = LowMapping::new(rounded, PAGE, 0).unwrap_or_else(|e| unsupported(&e));
    unsafe {
        ptr::copy_nonoverlapping(bytes.as_ptr(), mapping.base, bytes.len());
    }
    if kind != BlockKind::Argv {
        mapping
            .protect(0, rounded, 1)
            .unwrap_or_else(|e| unsupported(&e));
    }
    let base = mapping.base as u32;
    cpu.blocks.borrow_mut().push(Block {
        mapping,
        size: bytes.len(),
        kind,
    });
    base
}
// CommandLineToArgvW uses a four-byte pointer table, followed by UTF-16 strings
// in the same LocalFree-owned allocation. No host-sized pointer serialization.
fn argv_bytes(arguments: &[String], base: u32) -> crate::Result<Vec<u8>> {
    let mut bytes = vec![
        0;
        arguments
            .len()
            .checked_mul(4)
            .filter(|&n| n <= LIMIT)
            .ok_or("argv table overflow")?
    ];
    for (i, arg) in arguments.iter().enumerate() {
        if arg.contains('\0') {
            return Err("NUL in argv".into());
        }
        let pointer = base
            .checked_add(u32::try_from(bytes.len()).map_err(|_| "argv offset overflow")?)
            .ok_or("argv pointer overflow")?;
        bytes[i * 4..i * 4 + 4].copy_from_slice(&pointer.to_le_bytes());
        for unit in arg.encode_utf16().chain([0]) {
            if bytes.len() > LIMIT - 2 {
                return Err("argv storage limit".into());
            }
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
    }
    base.checked_add(u32::try_from(bytes.len()).map_err(|_| "argv extent overflow")?)
        .ok_or("argv extent overflow")?;
    Ok(bytes)
}
fn handle(image: &LoadedImage32<'_>, value: u32) -> usize {
    if (0x70000001..=0x70000003).contains(&value) {
        image.handles[(value - 0x70000001) as usize]
    } else {
        crate::abi::kernel_handle(value as usize)
    }
}
fn function(name: &str) -> usize {
    crate::abi::resolve("kernel32.dll", name)
        .unwrap_or_else(|| unsupported("missing host startup API"))
}
pub(super) fn io(image: &LoadedImage32<'_>, args: &[u32; 8], write: bool) -> u32 {
    if args[4] != 0 {
        unsupported("overlapped PE32 I/O is unsupported");
    }
    if args[2] as usize > 16 * LIMIT {
        unsupported("PE32 I/O exceeds 16 MiB");
    }
    if args[3] == 0 || !args[3].is_multiple_of(4) {
        return fail(87);
    }
    checked(image, args[3], 4, true);
    let count = args[2] as usize;
    if count > 0 {
        checked(image, args[1], count, !write);
        let first = args[1] as usize;
        let out = args[3] as usize;
        if first < out + 4 && out < first + count {
            unsupported("PE32 I/O buffer/count overlap is unsupported");
        }
    }
    let mut buffer = vec![0u8; count];
    if write && count > 0 {
        unsafe {
            ptr::copy_nonoverlapping(args[1] as *const u8, buffer.as_mut_ptr(), count);
        }
    }
    let mut transferred = 0u32;
    // Both ABI signatures have identical machine-level argument types. Mutable
    // staging is valid for WriteFile's const source as well as ReadFile output.
    let call: unsafe extern "win64" fn(usize, *mut c_void, u32, *mut u32, *mut c_void) -> i32 =
        unsafe { std::mem::transmute(function(if write { "WriteFile" } else { "ReadFile" })) };
    let result = unsafe {
        call(
            handle(image, args[0]),
            buffer.as_mut_ptr().cast(),
            args[2],
            &mut transferred,
            ptr::null_mut(),
        )
    };
    if transferred > args[2] {
        unsupported("host I/O returned an invalid count");
    }
    unsafe {
        if !write && transferred > 0 {
            ptr::copy_nonoverlapping(buffer.as_ptr(), args[1] as *mut u8, transferred as usize);
        }
        (args[3] as *mut u32).write(transferred);
    }
    result as u32
}
fn convert(image: &LoadedImage32<'_>, args: &[u32; 8], wide_source: bool) -> u32 {
    let page = if args[0] == 0 {
        super::ansi32::ACP
    } else {
        args[0]
    };
    if page == super::ansi32::ACP {
        return convert_ansi(image, args, wide_source);
    }
    if page != 65001 || !(if wide_source { [0, 128] } else { [0, 8] }).contains(&args[1]) {
        return fail(50);
    }
    let capacity = args[5] as i32;
    if capacity < 0
        || (capacity > 0 && (args[4] == 0 || args[2] == args[4]))
        || (wide_source && (args[6] != 0 || args[7] != 0))
    {
        return fail(87);
    }
    let width = if wide_source { 1 } else { 2 };
    if capacity as usize > LIMIT / width {
        return fail(50);
    }
    if capacity > 0 {
        if !(args[4] as usize).is_multiple_of(width) {
            return fail(87);
        }
        checked(image, args[4], capacity as usize * width, true);
    }
    let result;
    if wide_source {
        let Some(source) = read_units::<u16>(image, args[2], args[3] as i32) else {
            return 0;
        };
        let mut target = vec![0u8; capacity as usize];
        let call: unsafe extern "win64" fn(
            u32,
            u32,
            *const u16,
            i32,
            *mut u8,
            i32,
            *const u8,
            *mut i32,
        ) -> i32 = unsafe { std::mem::transmute(function("WideCharToMultiByte")) };
        result = unsafe {
            call(
                page,
                args[1],
                source.as_ptr(),
                source.len() as i32,
                target.as_mut_ptr(),
                capacity,
                ptr::null(),
                ptr::null_mut(),
            )
        };
        if result > 0 && capacity > 0 {
            unsafe {
                ptr::copy_nonoverlapping(target.as_ptr(), args[4] as *mut u8, result as usize);
            }
        }
    } else {
        let Some(source) = read_units::<u8>(image, args[2], args[3] as i32) else {
            return 0;
        };
        let mut target = vec![0u16; capacity as usize];
        let call: unsafe extern "win64" fn(u32, u32, *const u8, i32, *mut u16, i32) -> i32 =
            unsafe { std::mem::transmute(function("MultiByteToWideChar")) };
        result = unsafe {
            call(
                page,
                args[1],
                source.as_ptr(),
                source.len() as i32,
                target.as_mut_ptr(),
                capacity,
            )
        };
        if result > 0 && capacity > 0 {
            unsafe {
                ptr::copy_nonoverlapping(target.as_ptr(), args[4] as *mut u16, result as usize);
            }
        }
    }
    result as u32
}
fn convert_ansi(image: &LoadedImage32<'_>, args: &[u32; 8], wide: bool) -> u32 {
    if !(if wide {
        &[0, 0x400][..]
    } else {
        &[0, 1, 8][..]
    })
    .contains(&args[1])
    {
        return fail(50);
    }
    let capacity = args[5] as i32;
    if capacity < 0 || (capacity > 0 && (args[4] == 0 || args[2] == args[4])) {
        return fail(87);
    }
    let width = if wide { 1 } else { 2 };
    if capacity as usize > LIMIT / width {
        return fail(50);
    }
    if capacity > 0 {
        if !(args[4] as usize).is_multiple_of(width) {
            return fail(87);
        }
        checked(image, args[4], capacity as usize * width, true);
    }
    if wide {
        let default = if args[6] == 0 {
            b'?'
        } else {
            checked(image, args[6], 1, false);
            unsafe { (args[6] as *const u8).read() }
        };
        if args[7] != 0 {
            if !args[7].is_multiple_of(4) {
                return fail(87);
            }
            checked(image, args[7], 4, true);
        }
        let Some(units) = read_units::<u16>(image, args[2], args[3] as i32) else {
            return 0;
        };
        let (bytes, used) = super::ansi32::encode(&units, args[1] == 0x400, default)
            .unwrap_or_else(|e| unsupported(&e));
        if capacity != 0 && (capacity as usize) < bytes.len() {
            return fail(122);
        }
        if args[7] != 0 {
            unsafe {
                (args[7] as *mut u32).write(u32::from(used));
            }
        }
        if capacity != 0 {
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), args[4] as *mut u8, bytes.len());
            }
        }
        bytes.len() as u32
    } else {
        let Some(bytes) = read_units::<u8>(image, args[2], args[3] as i32) else {
            return 0;
        };
        let units = super::ansi32::decode(&bytes);
        if capacity != 0 && (capacity as usize) < units.len() {
            return fail(122);
        }
        if capacity != 0 {
            unsafe {
                ptr::copy_nonoverlapping(units.as_ptr(), args[4] as *mut u16, units.len());
            }
        }
        units.len() as u32
    }
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, args: &[u32; 8]) -> u32 {
    match id {
        7 => {
            if image.cpu.command.get() != 0 {
                return image.cpu.command.get();
            }
            let command = crate::process::command_snapshot()
                .unwrap_or_else(|| unsupported("missing process command line"));
            let bytes: Vec<u8> = command.into_iter().flat_map(u16::to_le_bytes).collect();
            let base = allocate(&image.cpu, &bytes, BlockKind::Command);
            image.cpu.command.set(base);
            base
        }
        8 => {
            if args[1] == 0 || !args[1].is_multiple_of(4) {
                return fail(87);
            }
            checked(image, args[1], 4, true);
            let Some(wide) = read_units::<u16>(image, args[0], -1) else {
                return 0;
            };
            let Ok(text) = String::from_utf16(&wide[..wide.len() - 1]) else {
                return fail(1113);
            };
            let Ok(arguments) = crate::command_line::canonical_arguments(&text) else {
                return fail(50);
            };
            let bytes = argv_bytes(&arguments, 0).unwrap_or_else(|e| unsupported(&e));
            let base = allocate(&image.cpu, &bytes, BlockKind::Argv);
            let bytes = argv_bytes(&arguments, base).unwrap_or_else(|e| unsupported(&e));
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), base as *mut u8, bytes.len());
                (args[1] as *mut u32).write(arguments.len() as u32);
            }
            base
        }
        9 => {
            let call: unsafe extern "win64" fn(usize) -> i32 =
                unsafe { std::mem::transmute(function("CloseHandle")) };
            let h = handle(image, args[0]);
            let result = unsafe { call(h) as u32 };
            if result != 0 {
                image.cpu.files.borrow_mut().remove(&h);
            }
            result
        }
        10 => io(image, args, false),
        11 => {
            if image.cpu.files.borrow().len() >= 4096 {
                unsupported("PE32 open file limit");
            }
            if args[3] != 0 || args[6] != 0 {
                unsupported("PE32 CreateFileW security/template unsupported");
            }
            let Some(name) = read_units::<u16>(image, args[0], -1) else {
                return u32::MAX;
            };
            let call: unsafe extern "win64" fn(
                *const u16,
                u32,
                u32,
                *const c_void,
                u32,
                u32,
                usize,
            ) -> usize = unsafe { std::mem::transmute(function("CreateFileW")) };
            let h = unsafe {
                call(
                    name.as_ptr(),
                    args[1],
                    args[2],
                    ptr::null(),
                    args[4],
                    args[5],
                    0,
                )
            };
            if h == usize::MAX {
                return u32::MAX;
            }
            if h > u32::MAX as usize || (0x70000001..=0x70000003).contains(&h) {
                unsupported("host handle cannot be represented in PE32");
            }
            image.cpu.files.borrow_mut().insert(h);
            h as u32
        }
        12 | 13 => convert(image, args, id == 13),
        14 => {
            match image.cpu.phase.get() {
                super::Phase32::Isolated if !image.dll => {
                    image.cpu.close_files();
                    image.cpu.clear_heap();
                }
                super::Phase32::Running => (),
                _ => unsupported(
                    "ExitProcess during PE32 DLL initialization/teardown is unsupported",
                ),
            }
            image.cpu.exit.set(Some(args[0]));
            args[0]
        }
        15 => {
            if args[0] == 0 {
                return 0;
            }
            let mut blocks = image.cpu.blocks.borrow_mut();
            if let Some(index) = blocks
                .iter()
                .position(|b| b.kind == BlockKind::Argv && b.mapping.base as u32 == args[0])
            {
                blocks.remove(index);
                0
            } else {
                fail(87);
                args[0]
            }
        }
        16 => image
            .cpu
            .heap
            .borrow_mut()
            .handle()
            .unwrap_or_else(|e| unsupported(&e)),
        17 => image
            .cpu
            .heap
            .borrow_mut()
            .allocate(args[0], args[1], args[2])
            .unwrap_or_else(|e| unsupported(&e)),
        18 => image
            .cpu
            .heap
            .borrow_mut()
            .free(args[0], args[1], args[2])
            .unwrap_or_else(|e| unsupported(&e)),
        19 => {
            // Legacy unmanifested Windows compatibility persona, not the Darwin
            // kernel version and not a claim of modern Windows feature support.
            if args[0] == 0 || !args[0].is_multiple_of(4) {
                return fail(87);
            }
            checked(image, args[0], 4, false);
            let size = unsafe { (args[0] as *const u32).read() };
            if size == 156 {
                unsupported("extended GetVersionExA record is unsupported");
            }
            if size != 148 {
                return fail(87);
            }
            checked(image, args[0], 148, true);
            let mut bytes = [0u8; 148];
            for (i, value) in [148u32, 6, 2, 9200, 2].into_iter().enumerate() {
                bytes[i * 4..i * 4 + 4].copy_from_slice(&value.to_le_bytes());
            }
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), args[0] as *mut u8, bytes.len());
            }
            1
        }
        20 => image
            .cpu
            .heap
            .borrow_mut()
            .create(args[0], args[1], args[2])
            .unwrap_or_else(|e| unsupported(&e)),
        21 => {
            let Some(name) = read_units::<u8>(image, args[0], -1) else {
                return 0;
            };
            let wide = super::ansi32::decode(&name);
            let call: unsafe extern "win64" fn(*const u16, *mut u16, u32) -> u32 =
                unsafe { std::mem::transmute(function("GetEnvironmentVariableW")) };
            let required = unsafe { call(wide.as_ptr(), ptr::null_mut(), 0) };
            if required == 0 {
                return 0;
            }
            if required as usize > LIMIT {
                return fail(50);
            }
            let mut value = vec![0u16; required as usize];
            let length = unsafe { call(wide.as_ptr(), value.as_mut_ptr(), required) };
            if length >= required {
                return fail(50);
            }
            let (bytes, _) = super::ansi32::encode(&value[..length as usize + 1], false, b'?')
                .unwrap_or_else(|e| unsupported(&e));
            if args[2] < bytes.len() as u32 {
                return bytes.len() as u32;
            }
            if args[1] == 0 {
                return fail(87);
            }
            checked(image, args[1], bytes.len(), true);
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), args[1] as *mut u8, bytes.len());
            }
            bytes.len() as u32 - 1
        }
        40 => {
            checked(image, args[0], 68, true);
            let bytes = startup_info(image.handles);
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), args[0] as *mut u8, bytes.len());
            }
            0 // VOID return: the caller receives the configured record above.
        }
        41 => {
            let native = handle(image, args[0]);
            if let Some(result) = crate::process::stream_file_type(native) {
                return match result {
                    Ok(kind) => kind,
                    Err(error) => fail(error),
                };
            }
            if image.cpu.files.borrow().contains(&native) {
                // The kernel's CreateFileW path admits regular files only.
                return 1;
            }
            fail(6)
        }
        43 => {
            let units = crate::process::environment_block().unwrap_or_else(|e| unsupported(&e));
            let bytes: Vec<u8> = units.into_iter().flat_map(u16::to_le_bytes).collect();
            allocate(&image.cpu, &bytes, BlockKind::Environment)
        }
        44 => {
            let mut blocks = image.cpu.blocks.borrow_mut();
            let Some(index) = blocks
                .iter()
                .position(|b| b.kind == BlockKind::Environment && b.mapping.base as u32 == args[0])
            else {
                return fail(87);
            };
            blocks.remove(index);
            1
        }
        45 => {
            if image.cpu.command_ansi.get() != 0 {
                return image.cpu.command_ansi.get();
            }
            let units = crate::process::command_snapshot()
                .unwrap_or_else(|| unsupported("missing process command line"));
            let (bytes, _) =
                super::ansi32::encode(&units, false, b'?').unwrap_or_else(|e| unsupported(&e));
            let base = allocate(&image.cpu, &bytes, BlockKind::Command);
            image.cpu.command_ansi.set(base);
            base
        }
        48 => {
            let page = if args[0] == 0 {
                super::ansi32::ACP
            } else {
                args[0]
            };
            let maximum: u32 = match page {
                super::ansi32::ACP => 1,
                65001 => 4,
                _ => return fail(87),
            };
            if args[1] == 0 || !args[1].is_multiple_of(4) {
                return fail(87);
            }
            checked(image, args[1], 20, true);
            let mut bytes = [0u8; 20];
            bytes[..4].copy_from_slice(&maximum.to_le_bytes());
            bytes[4] = b'?';
            unsafe {
                ptr::copy_nonoverlapping(bytes.as_ptr(), args[1] as *mut u8, bytes.len());
            }
            1
        }
        _ => unreachable!(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn argv_uses_four_byte_pointers_and_utf16_including_surrogate_pairs() {
        let args = vec!["x.exe".into(), "λ🌸".into(), "".into()];
        let bytes = argv_bytes(&args, 0x10000).unwrap();
        assert_eq!(&bytes[..12], &[12, 0, 1, 0, 24, 0, 1, 0, 32, 0, 1, 0]);
        assert_eq!(&bytes[24..], &[0xbb, 3, 0x3c, 0xd8, 0x38, 0xdf, 0, 0, 0, 0]);
        assert!(argv_bytes(&args, u32::MAX - 12).is_err());
        assert!(argv_bytes(&["a\0b".into()], 0).is_err());
        assert!(argv_bytes(&["a".repeat(LIMIT)], 0).is_err());
    }
    #[test]
    fn module_identity_is_not_just_a_symbol_match() {
        use crate::ImportSymbol;
        let name = |s: &str| ImportSymbol::Name(s.into());
        assert_eq!(
            super::super::api_id("SHELL32.DLL", &name("CommandLineToArgvW")),
            Some(8)
        );
        assert_eq!(
            super::super::api_id("kernel32.dll", &name("CommandLineToArgvW")),
            None
        );
        assert_eq!(super::super::api_id("shell32.dll", &name("ReadFile")), None);
        assert_eq!(
            super::super::api_id("Kernel32.dll", &name("ReadFile")),
            Some(10)
        );
        assert_eq!(
            super::super::api_id("Kernel32.dll", &name("readfile")),
            None
        );
        assert_eq!(
            super::super::api_id("kernel32.dll", &ImportSymbol::Ordinal(10)),
            None
        );
    }

    #[test]
    fn conversion_errors_do_not_dereference_invalid_guest_addresses_or_publish_output() {
        let image = super::super::tests::layout();
        let saved = ffi::GetLastError();
        for wide in [false, true] {
            for (args, error) in [
                ([932, 0, 0xdead0000, 4, 0, 0, 0, 0], 50),
                ([65001, 1, 0xdead0000, 4, 0, 0, 0, 0], 50),
                ([65001, 0, 0, 4, 0, 0, 0, 0], 87),
                ([65001, 0, 0xdead0000, 0, 0, 0, 0, 0], 87),
                ([65001, 0, 0xdead0000, (-2i32) as u32, 0, 0, 0, 0], 87),
                ([65001, 0, 0xdead0000, 4, 0xdead0000, 4, 0, 0], 87),
                ([65001, 0, 0xdead0000, 4, 0, (-1i32) as u32, 0, 0], 87),
            ] {
                ffi::SetLastError(0xfeedbeef);
                assert_eq!(convert(&image, &args, wide), 0);
                assert_eq!(ffi::GetLastError(), error);
            }
        }
        for at in [6, 7] {
            let mut args = [65001, 0, 0xdead0000, 4, 0, 0, 0, 0];
            args[at] = 1;
            assert_eq!(convert(&image, &args, true), 0);
            assert_eq!(ffi::GetLastError(), 87);
        }
        ffi::SetLastError(saved);
    }

    #[test]
    fn startup_permissions_exclude_padding_and_free_rejects_unowned_and_command_addresses() {
        let image = super::super::tests::layout();
        for (base, kind) in [
            (0x700000, BlockKind::Command),
            (0x710000, BlockKind::Argv),
            (0x720000, BlockKind::Environment),
        ] {
            image.cpu.blocks.borrow_mut().push(Block {
                mapping: LowMapping {
                    base: base as *mut u8,
                    len: PAGE,
                },
                size: 28,
                kind,
            });
            assert!(permits(&image.cpu, base, 28, false));
            assert_eq!(permits(&image.cpu, base, 28, true), kind == BlockKind::Argv);
            assert!(!permits(&image.cpu, base, 29, false));
            assert!(!permits(&image.cpu, base - 1, 1, false));
            assert!(!permits(&image.cpu, base + 28, 1, false));
        }
        assert!(!permits(&image.cpu, usize::MAX - 1, 4, false));
        let saved = ffi::GetLastError();
        ffi::SetLastError(0xfeedbeef);
        assert_eq!(dispatch(&image, 15, &[0; 8]), 0);
        assert_eq!(ffi::GetLastError(), 0xfeedbeef);
        for pointer in [0x700000, 0x710004, 0x720000, 0xdead0000] {
            let mut args = [0; 8];
            args[0] = pointer;
            assert_eq!(dispatch(&image, 15, &args), pointer);
            assert_eq!(ffi::GetLastError(), 87);
            assert_eq!(image.cpu.blocks.borrow().len(), 3);
        }
        for pointer in [0, 0x700000, 0x710000, 0x720002, 0xdead0000] {
            let mut args = [0; 8];
            args[0] = pointer;
            assert_eq!(dispatch(&image, 44, &args), 0);
            assert_eq!(ffi::GetLastError(), 87);
            assert_eq!(image.cpu.blocks.borrow().len(), 3);
        }
        assert_eq!(handle(&image, 0xffffffff), usize::MAX);
        assert_eq!(handle(&image, 0xfffffffe), usize::MAX - 1);
        assert_eq!(handle(&image, 0x42), 0x42);
        assert_eq!(handle(&image, 0x70000001), image.handles[0]);
        ffi::SetLastError(saved);
        // layout is ManuallyDrop: these fabricated address-only maps never unmap.
    }

    #[test]
    fn invalid_file_outputs_fail_before_access_and_exit_blocks_reentry() {
        let image = super::super::tests::layout();
        let saved = ffi::GetLastError();
        for write in [false, true] {
            for output in [0, 3] {
                let args = [0, 0xdead0000, 16, output, 0, 0, 0, 0];
                assert_eq!(io(&image, &args, write), 0);
                assert_eq!(ffi::GetLastError(), 87);
            }
        }
        assert_eq!(dispatch(&image, 9, &[0; 8]), 0);
        assert_eq!(ffi::GetLastError(), 6);
        assert_eq!(dispatch(&image, 11, &[0; 8]), u32::MAX);
        assert_eq!(ffi::GetLastError(), 87);
        let mut args = [0; 8];
        args[0] = 123;
        assert_eq!(dispatch(&image, 14, &args), 123);
        assert_eq!(image.cpu.exit.get(), Some(123));
        assert_eq!(
            unsafe { image.invoke(0x100000, &[], super::super::CallingConvention32::Cdecl) }
                .unwrap_err(),
            "PE32 process already requested ExitProcess"
        );
        ffi::SetLastError(saved);
    }

    #[test]
    fn exit_releases_real_owned_files_without_closing_unrelated_handles() {
        let image = super::super::tests::layout();
        let saved = ffi::GetLastError();
        let root = std::env::temp_dir().join(format!(
            "alkyhol-pe32-exit-{}",
            alkyhol::kernel::reserve_handle_token().unwrap()
        ));
        std::fs::write(&root, b"abc").unwrap();
        let name: Vec<u16> = root.to_str().unwrap().encode_utf16().chain([0]).collect();
        let open =
            || unsafe { ffi::CreateFileW(name.as_ptr(), 0x80000000, 7, ptr::null(), 3, 128, 0) };
        let owned = open();
        let unrelated = open();
        assert_ne!(owned, usize::MAX);
        assert_ne!(unrelated, usize::MAX);
        image.cpu.files.borrow_mut().insert(owned);
        ffi::SetLastError(0xfeedbeef);
        assert_eq!(dispatch(&image, 14, &[0; 8]), 0);
        assert_eq!(ffi::GetLastError(), 0xfeedbeef);
        assert!(image.cpu.files.borrow().is_empty());
        let mut size = -1;
        assert_eq!(unsafe { ffi::GetFileSizeEx(owned, &mut size) }, 0);
        assert_eq!(ffi::GetLastError(), 6);
        assert_eq!(unsafe { ffi::GetFileSizeEx(unrelated, &mut size) }, 1);
        assert_eq!(size, 3);
        assert_eq!(ffi::CloseHandle(unrelated), 1);
        image.cpu.close_files(); // Cleared ownership is safe to clean up again.
        std::fs::remove_file(root).unwrap();
        ffi::SetLastError(saved);
    }
}
