//! Process-owned current directory in the explicit Z: host-root filesystem.
//! Host process cwd is never changed. Relative guest reads consult this state.
use super::{LoadedImage32, files32, startup32, unsupported};
use crate::Result;
use std::{fs::File, os::unix::fs::MetadataExt, path::PathBuf};

pub(super) struct Directory {
    host: PathBuf,
    windows: String,
    file: File,
}
fn absolute(text: &str) -> Result<String> {
    if !text.starts_with("Z:\\") && !text.starts_with("z:\\") {
        return Err("current directory requires an exact absolute Z: backed path".into());
    }
    if text.encode_utf16().count() >= 260 {
        return Err("current directory exceeds admitted MAX_PATH profile".into());
    }
    let text = if text.len() > 3 {
        text.strip_suffix('\\').unwrap_or(text)
    } else {
        text
    };
    Ok(format!("Z:{}", &text[2..]))
}
impl Directory {
    fn open(text: &str) -> Result<std::result::Result<Self, u32>> {
        let windows = absolute(text)?;
        let resolved = if windows == "Z:\\" {
            Ok((
                PathBuf::from("/"),
                std::fs::metadata("/").map_err(|e| e.to_string())?,
            ))
        } else {
            files32::existing(&windows)?
        };
        let (host, metadata) = match resolved {
            Ok(value) => value,
            Err(error) => return Ok(Err(error)),
        };
        if !metadata.is_dir() {
            return Ok(Err(267));
        }
        let file = File::open(&host).map_err(|e| format!("open current directory: {e}"))?;
        let opened = file.metadata().map_err(|e| e.to_string())?;
        if !opened.is_dir() || (metadata.dev(), metadata.ino()) != (opened.dev(), opened.ino()) {
            return Err("current directory changed during open".into());
        }
        Ok(Ok(Self {
            host,
            windows,
            file,
        }))
    }
    fn check(&self) -> Result<()> {
        let live = self.file.metadata().map_err(|e| e.to_string())?;
        let path = std::fs::symlink_metadata(&self.host).map_err(|e| e.to_string())?;
        if !path.is_dir() || (live.dev(), live.ino()) != (path.dev(), path.ino()) {
            return Err("backed current directory was renamed or replaced externally".into());
        }
        Ok(())
    }
}
pub(super) fn set(image: &LoadedImage32<'_>, address: u32) -> u32 {
    if !image
        .cpu
        .modules
        .borrow()
        .iter()
        .any(|m| m.origin.is_some())
    {
        unsupported("current directory requires the file-backed process profile");
    }
    let units = startup32::read_units::<u16>(image, address, -1)
        .unwrap_or_else(|| unsupported("current directory input outside checked guest storage"));
    let text = String::from_utf16(&units[..units.len() - 1])
        .unwrap_or_else(|_| unsupported("current directory input is not valid UTF-16"));
    match Directory::open(&text).unwrap_or_else(|e| unsupported(&e)) {
        Ok(directory) => {
            *image.cpu.current_directory.borrow_mut() = Some(directory);
            1
        }
        Err(error) => {
            alkyhol::ffi::SetLastError(error);
            0
        }
    }
}
pub(super) fn set_ansi(image: &LoadedImage32<'_>, address: u32) -> u32 {
    if !image
        .cpu
        .modules
        .borrow()
        .iter()
        .any(|m| m.origin.is_some())
    {
        unsupported("current directory requires the file-backed process profile");
    }
    let text = super::bulk32::ascii(image, address);
    match Directory::open(&text).unwrap_or_else(|e| unsupported(&e)) {
        Ok(directory) => {
            *image.cpu.current_directory.borrow_mut() = Some(directory);
            1
        }
        Err(error) => {
            alkyhol::ffi::SetLastError(error);
            0
        }
    }
}
pub(super) fn current(image: &LoadedImage32<'_>) -> Result<String> {
    let state = image.cpu.current_directory.borrow();
    let directory = state
        .as_ref()
        .ok_or("path resolution requires an explicitly set current directory")?;
    directory.check()?;
    Ok(directory.windows.clone())
}
pub(super) fn get_ansi(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let state = image.cpu.current_directory.borrow();
    let directory = state
        .as_ref()
        .unwrap_or_else(|| unsupported("current directory has not been explicitly set"));
    directory.check().unwrap_or_else(|e| unsupported(&e));
    if !directory.windows.is_ascii() {
        unsupported("ANSI current-directory query requires ASCII spelling");
    }
    let bytes = directory.windows.as_bytes();
    if args[0] as usize <= bytes.len() {
        return bytes.len() as u32 + 1;
    }
    if !image.permits(args[1] as usize, bytes.len() + 1, true) {
        unsupported("ANSI current-directory output outside writable guest storage");
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), args[1] as *mut u8, bytes.len());
        (args[1] as *mut u8).add(bytes.len()).write(0);
    }
    bytes.len() as u32
}
pub(super) fn get(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let state = image.cpu.current_directory.borrow();
    let directory = state
        .as_ref()
        .unwrap_or_else(|| unsupported("current directory has not been explicitly set"));
    directory.check().unwrap_or_else(|e| unsupported(&e));
    let units: Vec<u16> = directory.windows.encode_utf16().collect();
    if args[0] as usize <= units.len() {
        return units.len() as u32 + 1;
    }
    if !args[1].is_multiple_of(2) || !image.permits(args[1] as usize, (units.len() + 1) * 2, true) {
        unsupported("current directory output outside writable guest storage");
    }
    unsafe {
        std::ptr::copy_nonoverlapping(units.as_ptr(), args[1] as *mut u16, units.len());
        (args[1] as *mut u16).add(units.len()).write(0);
    }
    units.len() as u32
}
/// Ordinary file APIs use the process-owned directory for relative spellings.
/// INI bare-name searching has different semantics and uses its own resolver.
pub(super) fn ordinary_path(image: &LoadedImage32<'_>, text: &str) -> Result<String> {
    if text.starts_with("Z:\\") || text.starts_with("z:\\") {
        return Ok(text.into());
    }
    let relative = text.strip_prefix(".\\").unwrap_or(text);
    if relative.is_empty() || relative.starts_with('\\') || relative.contains([':', '/']) {
        return Err("ordinary file path namespace is unsupported".into());
    }
    let state = image.cpu.current_directory.borrow();
    let directory = state
        .as_ref()
        .ok_or("ordinary file path requires a checked current directory")?;
    directory.check()?;
    Ok(format!(
        "{}\\{relative}",
        directory.windows.trim_end_matches('\\')
    ))
}
/// Explicit dot-relative INI names follow the real process-owned directory.
/// Bare names require Windows-directory searching and remain unsupported.
pub(super) fn private_profile_path(image: &LoadedImage32<'_>, text: &str) -> Result<String> {
    if text.starts_with("Z:\\") || text.starts_with("z:\\") {
        return Ok(text.to_owned());
    }
    let leaf = profile_leaf(text)?;
    let state = image.cpu.current_directory.borrow();
    let directory = state
        .as_ref()
        .ok_or("private-profile relative path requires an explicitly set current directory")?;
    directory.check()?;
    Ok(format!(
        "{}\\{leaf}",
        directory.windows.trim_end_matches('\\')
    ))
}
fn profile_leaf(text: &str) -> Result<&str> {
    let leaf = text.strip_prefix(".\\").ok_or(
        "private-profile paths require absolute Z: or explicit dot-relative leaf spelling",
    )?;
    if leaf.is_empty() || leaf.contains(['\\', '/']) {
        return Err("private-profile dot-relative path must name one ordinary leaf".into());
    }
    Ok(leaf)
}
/// Once cwd is set, existing read-only file opens use actual guest directory
/// state. Unknown aliases and creation/write semantics stop rather than falling
/// back to the host process directory. Older profiles retain their own contract.
pub(super) fn read_path(
    image: &LoadedImage32<'_>,
    units: &[u16],
    args: &[u32; 8],
) -> std::result::Result<Vec<u16>, u32> {
    let state = image.cpu.current_directory.borrow();
    let Some(directory) = state.as_ref() else {
        return Ok(units.to_vec());
    };
    directory.check().unwrap_or_else(|e| unsupported(&e));
    if args[1] != 0x80000000 || args[4] != 3 {
        unsupported("current-directory file creation/writes are unsupported");
    }
    let text = String::from_utf16(&units[..units.len() - 1])
        .unwrap_or_else(|_| unsupported("current-directory filename is not valid UTF-16"));
    let full = ordinary_path(image, &text).unwrap_or_else(|e| unsupported(&e));
    let (host, metadata) = files32::existing(&full).unwrap_or_else(|e| unsupported(&e))?;
    if !metadata.is_file() {
        unsupported("current-directory file open requires an existing regular file");
    }
    Ok(host
        .to_str()
        .unwrap_or_else(|| unsupported("current-directory host path is not Unicode"))
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directory_references_are_real_and_do_not_change_host_cwd() {
        let before = std::env::current_dir().unwrap();
        let root = std::env::temp_dir().join(format!("alkyhol-cwd-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("file"), b"owned").unwrap();
        let root = std::fs::canonicalize(root).unwrap();
        let windows = format!("Z:{}", root.to_str().unwrap().replace('/', "\\"));
        let directory = Directory::open(&format!("{windows}\\"))
            .unwrap()
            .ok()
            .unwrap();
        assert_eq!(directory.windows, windows);
        directory.check().unwrap();
        assert_eq!(
            Directory::open(&format!("{windows}\\absent"))
                .unwrap()
                .err(),
            Some(2)
        );
        assert_eq!(
            Directory::open(&format!("{windows}\\file")).unwrap().err(),
            Some(267)
        );
        assert_eq!(std::env::current_dir().unwrap(), before);
        let moved = root.with_extension("moved");
        std::fs::rename(&root, &moved).unwrap();
        assert!(directory.check().is_err());
        drop(directory);
        std::fs::remove_dir_all(moved).unwrap();
    }
    #[test]
    fn profile_relative_spelling_requires_an_explicit_dot_and_one_leaf() {
        assert_eq!(profile_leaf(".\\settings.ini").unwrap(), "settings.ini");
        for text in [
            "settings.ini",
            ".\\",
            ".\\inner\\settings.ini",
            ".\\inner/settings.ini",
            "C:\\settings.ini",
        ] {
            assert!(profile_leaf(text).is_err());
        }
    }
    #[test]
    fn unknown_paths_are_not_normalized_or_mapped_to_invented_drives() {
        for value in [
            "relative",
            "C:\\Windows",
            "\\\\server\\share",
            "Z:\\a\\..\\b",
            "Z:\\a\\\\b",
            "Z:\\NUL",
            "Z:\\a.",
        ] {
            assert!(Directory::open(value).is_err());
        }
        assert!(absolute(&format!("Z:\\{}", "a".repeat(257))).is_err());
    }
}
