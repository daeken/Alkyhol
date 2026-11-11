//! Family adapters for the actual DAO import surface. All state is shared with
//! existing checked gateways; unsupported handle/path profiles remain explicit.
use super::{LoadedImage32, directory32, files32, startup32, unsupported};
pub(super) fn ascii(image: &LoadedImage32<'_>, address: u32) -> String {
    let value = startup32::read_units::<u8>(image, address, -1)
        .unwrap_or_else(|| unsupported("ANSI family input outside checked guest storage"));
    if !value[..value.len() - 1].is_ascii() {
        unsupported("ANSI family paths currently require ASCII spelling");
    }
    String::from_utf8(value[..value.len() - 1].to_vec()).unwrap()
}
pub(super) fn file_size(image: &LoadedImage32<'_>, args: &[u32; 8], wide: bool) -> u32 {
    if !image.cpu.files.borrow().contains(&(args[0] as usize)) {
        unsupported("file-size query requires an owned live file handle");
    }
    let bytes = if wide { 8 } else { 4 };
    if (wide || args[1] != 0)
        && (!args[1].is_multiple_of(4) || !image.permits(args[1] as usize, bytes, true))
    {
        unsupported("file-size output outside checked writable guest storage");
    }
    let mut value = 0i64;
    if unsafe { alkyhol::ffi::GetFileSizeEx(args[0] as usize, &mut value) } == 0 {
        return if wide { 0 } else { u32::MAX };
    }
    if wide {
        unsafe { (args[1] as *mut i64).write_unaligned(value) };
        1
    } else {
        if args[1] != 0 {
            unsafe { (args[1] as *mut u32).write((value as u64 >> 32) as u32) };
        }
        let low = value as u32;
        if low == u32::MAX {
            alkyhol::ffi::SetLastError(0);
        }
        low
    }
}
pub(super) fn attributes(image: &LoadedImage32<'_>, address: u32, ansi: bool) -> u32 {
    let text = if ansi {
        ascii(image, address)
    } else {
        let units = startup32::read_units::<u16>(image, address, -1)
            .unwrap_or_else(|| unsupported("file attributes path outside checked UTF-16 storage"));
        String::from_utf16(&units[..units.len() - 1])
            .unwrap_or_else(|_| unsupported("file attributes filename is invalid UTF-16"))
    };
    let path = directory32::ordinary_path(image, &text).unwrap_or_else(|e| unsupported(&e));
    match files32::existing(&path).unwrap_or_else(|e| unsupported(&e)) {
        Ok((_, meta)) => {
            (if meta.is_dir() { 0x10 } else { 0x20 })
                | if meta.permissions().readonly() { 1 } else { 0 }
        }
        Err(error) => {
            alkyhol::ffi::SetLastError(error);
            u32::MAX
        }
    }
}
