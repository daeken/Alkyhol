//! File-backed PE32 module origins. Z: directly mirrors the real host root;
//! bytes-only mappings never acquire an invented source path.
use super::{LoadedImage32, unsupported};
use crate::Result;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    ptr,
};

#[derive(Clone)]
pub(super) struct Origin {
    pub host: PathBuf,
    pub windows: Vec<u16>,
}
fn windows_path(path: &Path) -> Result<Vec<u16>> {
    let text = path
        .to_str()
        .ok_or("non-Unicode module source path is unsupported")?;
    if !path.is_absolute()
        || text.chars().any(|c| c < ' ' || "\\<>:\"|?*".contains(c))
        || text.split('/').any(|p| p.ends_with(['.', ' ']))
    {
        return Err(
            "module source path cannot be represented in the Z: host-root namespace".into(),
        );
    }
    let result: Vec<u16> = format!("Z:{}", text.replace('/', "\\"))
        .encode_utf16()
        .collect();
    if result.len() > 32767 {
        return Err("module source path exceeds Win32 UTF-16 bound".into());
    }
    Ok(result)
}
pub(super) fn read(path: &Path) -> Result<(Vec<u8>, Origin)> {
    let host = std::fs::canonicalize(path).map_err(|e| format!("module source path: {e}"))?;
    let windows = windows_path(&host)?;
    let file = File::open(&host).map_err(|e| format!("module source open: {e}"))?;
    let metadata = file
        .metadata()
        .map_err(|e| format!("module source metadata: {e}"))?;
    const LIMIT: u64 = 1024 * 1024 * 1024;
    if !metadata.is_file() || metadata.len() > LIMIT {
        return Err("module source must be a bounded regular file".into());
    }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| format!("module source read: {e}"))?;
    if bytes.len() as u64 > LIMIT {
        return Err("module source grew beyond the file-size bound".into());
    }
    Ok((bytes, Origin { host, windows }))
}

pub(super) fn directory(path: &Path) -> Result<Vec<u16>> {
    directory_role(path, "system directory")
}
pub(super) fn directory_role(path: &Path, role: &str) -> Result<Vec<u16>> {
    let host = std::fs::canonicalize(path).map_err(|e| format!("{role}: {e}"))?;
    if !host.is_dir() {
        return Err(format!("{role} must identify an existing host directory"));
    }
    windows_path(&host)
}

pub(super) fn system_directory(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    copy_directory(
        image,
        args,
        image.cpu.system_directory.as_deref(),
        "system directory",
    )
}
pub(super) fn system_windows_directory(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    copy_directory(
        image,
        args,
        image.cpu.system_windows_directory.as_deref(),
        "system Windows directory",
    )
}
fn copy_directory(
    image: &LoadedImage32<'_>,
    args: &[u32; 8],
    directory: Option<&[u16]>,
    role: &str,
) -> u32 {
    let [output, capacity, ..] = *args;
    let name = directory.unwrap_or_else(|| {
        unsupported(&format!(
            "PE32 {role} requires an explicitly configured backed guest directory"
        ))
    });
    if capacity > 1024 * 1024 / 2 {
        unsupported(&format!("PE32 {role} output exceeds checked size bound"));
    }
    if capacity as usize <= name.len() {
        return name.len() as u32 + 1;
    }
    if !output.is_multiple_of(2) || !image.permits(output as usize, capacity as usize * 2, true) {
        unsupported(&format!(
            "PE32 {role} output outside checked writable guest storage"
        ));
    }
    unsafe {
        ptr::copy_nonoverlapping(name.as_ptr(), output as usize as *mut u16, name.len());
        (output as usize as *mut u16).add(name.len()).write(0);
    }
    name.len() as u32
}

fn output_count(length: usize, capacity: u32) -> Result<(usize, u32, bool)> {
    if capacity > 1024 * 1024 / 2 {
        return Err("module filename buffer exceeds checked size bound".into());
    }
    let truncated = length >= capacity as usize;
    let count = length.min((capacity as usize).saturating_sub(1));
    Ok((
        count,
        if truncated { capacity } else { length as u32 },
        truncated,
    ))
}

pub(super) fn filename(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let [module, output, capacity, ..] = *args;
    let base = if module == 0 {
        image.cpu.executable.get()
    } else {
        module
    };
    let name = image
        .cpu
        .modules
        .borrow()
        .iter()
        .find(|m| m.base == base)
        .and_then(|m| m.origin.as_ref())
        .map(|o| o.windows.clone())
        .unwrap_or_else(|| {
            unsupported("PE32 module filename requires a known file-backed module origin")
        });
    let (count, result, truncated) =
        output_count(name.len(), capacity).unwrap_or_else(|e| unsupported(&e));
    if capacity != 0 {
        if !output.is_multiple_of(2) || !image.permits(output as usize, capacity as usize * 2, true)
        {
            unsupported("PE32 module filename output outside checked writable guest storage");
        }
        unsafe {
            ptr::copy_nonoverlapping(name.as_ptr(), output as usize as *mut u16, count);
            (output as usize as *mut u16).add(count).write(0);
        }
    }
    // The native Win32 observer clears last error on complete filename output;
    // truncation (including zero capacity) sets ERROR_INSUFFICIENT_BUFFER.
    alkyhol::ffi::SetLastError(if truncated { 122 } else { 0 });
    result
}

pub(super) fn long_name(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let [source, output, capacity, ..] = *args;
    let mut name = super::startup32::read_units::<u16>(image, source, -1)
        .unwrap_or_else(|| unsupported("PE32 long-path source is unsupported"));
    name.pop(); // The bounded reader retains the terminating NUL.
    let origin = image.cpu.modules.borrow().iter().filter_map(|m|m.origin.as_ref())
        .find(|o|o.windows==name).cloned()
        .unwrap_or_else(||unsupported("PE32 long-path lookup requires an exact registered module source; aliases and arbitrary paths are unsupported"));
    // This path is already the canonical long spelling of a real source file.
    // Do not infer alias expansion from absence of '~', or return made-up paths.
    let current = std::fs::canonicalize(&origin.host)
        .unwrap_or_else(|_| unsupported("PE32 long-path source can no longer be verified"));
    if current != origin.host || !current.is_file() {
        unsupported("PE32 long-path source changed or is not a regular file");
    }
    if capacity > 1024 * 1024 / 2 {
        unsupported("PE32 long-path output exceeds checked size bound");
    }
    if capacity as usize <= name.len() {
        return name.len() as u32 + 1;
    }
    if !output.is_multiple_of(2) || !image.permits(output as usize, capacity as usize * 2, true) {
        unsupported("PE32 long-path output outside checked writable guest storage");
    }
    unsafe {
        ptr::copy_nonoverlapping(name.as_ptr(), output as usize as *mut u16, name.len());
        (output as usize as *mut u16).add(name.len()).write(0);
    }
    name.len() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_path_spelling_is_utf16_and_maps_the_real_root_without_invented_directories() {
        assert_eq!(
            String::from_utf16(&windows_path(Path::new("/dir λ🌸/crt.dll")).unwrap()).unwrap(),
            "Z:\\dir λ🌸\\crt.dll"
        );
        for path in [
            "relative.dll",
            "/ambiguous\\name.dll",
            "/name:stream",
            "/trailing. ",
            "/bad\nname",
        ] {
            assert!(windows_path(Path::new(path)).is_err(), "{path}");
        }
    }
    #[test]
    fn filename_sizes_are_units_with_explicit_modern_truncation_and_no_padding_write() {
        assert_eq!(output_count(5, 6).unwrap(), (5, 5, false));
        assert_eq!(output_count(5, 5).unwrap(), (4, 5, true));
        assert_eq!(output_count(5, 1).unwrap(), (0, 1, true));
        assert_eq!(output_count(5, 0).unwrap(), (0, 0, true));
        assert!(output_count(5, u32::MAX).is_err());
    }
    #[test]
    fn file_origin_and_loaded_bytes_come_from_the_same_real_source() {
        let root = std::env::temp_dir().join(format!("alkyhol-origin-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("owned.dll");
        std::fs::write(&path, b"owned source").unwrap();
        let (bytes, origin) = read(&path).unwrap();
        assert_eq!(bytes, b"owned source");
        assert_eq!(origin.host, std::fs::canonicalize(&path).unwrap());
        assert_eq!(origin.windows, windows_path(&origin.host).unwrap());
        assert!(read(&root).is_err());
        assert_eq!(
            directory(&root).unwrap(),
            windows_path(&std::fs::canonicalize(&root).unwrap()).unwrap()
        );
        assert!(directory(&path).is_err());
        assert!(directory(&root.join("missing")).is_err());
        assert!(read(&root.join("missing.dll")).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
