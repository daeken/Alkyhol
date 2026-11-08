//! Explicit file-only private-profile reads. Unknown contents never become
//! defaults: only actual checked file/path absence admits the fallback branch.
use super::{LoadedImage32, directory32, files32, ini32, startup32, unsupported};
use crate::Result;
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
};
struct Locked(File);
impl Drop for Locked {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
fn contents(file: &mut File) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    file.take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 65536 {
        return Err("INI backing grew beyond its budget".into());
    }
    Ok(bytes)
}
fn text(units: &[u16]) -> Result<String> {
    String::from_utf16(units).map_err(|_| "INI scalar input is invalid UTF-16".into())
}

fn default_value(mut units: Vec<u16>) -> Result<Vec<u16>> {
    while units.last() == Some(&32) {
        units.pop();
    }
    if units.last().is_some_and(|v| *v < 32) {
        return Err("private-profile trailing control whitespace is unmeasured".into());
    }
    Ok(units)
}
#[cfg(test)]
fn missing(path: &str) -> Result<u32> {
    match files32::existing(path)? {
        Err(error @ (2 | 3)) => Ok(error),
        Err(error) => Err(format!(
            "private-profile I/O error {error} is outside the absent-file profile"
        )),
        Ok(_) => Err("existing private-profile files require measured INI parsing".into()),
    }
}
fn units(image: &LoadedImage32<'_>, address: u32) -> Vec<u16> {
    let mut value = startup32::read_units::<u16>(image, address, -1)
        .unwrap_or_else(|| unsupported("private-profile input outside checked UTF-16 storage"));
    value.pop();
    value
}
pub(super) fn get(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    if args[0] == 0 || args[1] == 0 {
        unsupported("private-profile enumeration is unsupported");
    }
    let section = units(image, args[0]);
    let key = units(image, args[1]);
    let fallback = if args[2] == 0 {
        Vec::new()
    } else {
        units(image, args[2])
    };
    let path = units(image, args[5]);
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 diagnostic private profile: section_units={section:x?}; key_units={key:x?}; default_units={fallback:x?}; file_units={path:x?}"
        );
    }
    if !image.cpu.private_profile_files {
        unsupported(
            "private-profile reads require an explicit file-only INI profile without registry redirection",
        );
    }
    if section.is_empty() || key.is_empty() {
        unsupported("private-profile scalar names must be nonempty");
    }
    if !(1..=32768).contains(&args[4]) {
        unsupported("private-profile buffer sizes outside 1..32768 are unsupported");
    }
    if !args[3].is_multiple_of(2) || !image.permits(args[3] as usize, args[4] as usize * 2, true) {
        unsupported("private-profile output outside writable UTF-16 guest storage");
    }
    let path = String::from_utf16(&path)
        .unwrap_or_else(|_| unsupported("private-profile filename is invalid UTF-16"));
    let path = directory32::private_profile_path(image, &path).unwrap_or_else(|e| unsupported(&e));
    let (fallback, error) = match files32::existing(&path).unwrap_or_else(|e| unsupported(&e)) {
        Err(2) => (
            default_value(fallback).unwrap_or_else(|e| unsupported(&e)),
            2,
        ),
        Err(3) => (
            default_value(fallback).unwrap_or_else(|e| unsupported(&e)),
            3,
        ),
        Err(error) => unsupported(&format!("private-profile I/O error {error} is unsupported")),
        Ok(_) => {
            let documents = image.cpu.documents.as_ref().unwrap_or_else(|| {
                unsupported("existing private-profile files require measured INI parsing")
            });
            let file = documents
                .ini_file(&path, false)
                .unwrap_or_else(|e| unsupported(&e))
                .unwrap_or_else(|_| unsupported("INI file disappeared before read"));
            file.try_lock_shared()
                .unwrap_or_else(|_| unsupported("INI backing is locked by another writer"));
            let mut file = Locked(file);
            let ini = ini32::Ini::parse(&contents(&mut file.0).unwrap_or_else(|e| unsupported(&e)))
                .unwrap_or_else(|e| unsupported(&e));
            let section = text(&section).unwrap_or_else(|e| unsupported(&e));
            let key = text(&key).unwrap_or_else(|e| unsupported(&e));
            let found = ini.get(&section, &key).unwrap_or_else(|e| unsupported(&e));
            match found {
                Some(value) => (value.encode_utf16().collect(), 0),
                None => (
                    default_value(fallback).unwrap_or_else(|e| unsupported(&e)),
                    2,
                ),
            }
        }
    };
    let count = fallback.len().min(args[4] as usize - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(fallback.as_ptr(), args[3] as *mut u16, count);
        (args[3] as *mut u16).add(count).write(0);
    }
    alkyhol::ffi::SetLastError(error);
    count as u32
}
pub(super) fn write(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let values: Vec<_> = args[..4]
        .iter()
        .map(|&at| {
            if at == 0 {
                None
            } else {
                Some(units(image, at))
            }
        })
        .collect();
    if image.cpu.diagnostic.is_some() {
        eprintln!("PE32 diagnostic private profile write: input_units={values:x?}");
    }
    if !image.cpu.private_profile_files {
        unsupported("private-profile writes require an explicit file-only INI profile");
    }
    if values.iter().any(Option::is_none) {
        unsupported("INI deletion, flush and NULL arguments remain unsupported");
    }
    let strings: Vec<_> = values
        .iter()
        .map(|v| text(v.as_ref().unwrap()).unwrap_or_else(|e| unsupported(&e)))
        .collect();
    ini32::name(&strings[0])
        .and_then(|_| ini32::name(&strings[1]))
        .and_then(|_| ini32::value(&strings[2]))
        .unwrap_or_else(|e| unsupported(&e));
    let path =
        directory32::private_profile_path(image, &strings[3]).unwrap_or_else(|e| unsupported(&e));
    let documents = image
        .cpu
        .documents
        .as_ref()
        .unwrap_or_else(|| unsupported("INI writes require explicit Documents backing"));
    let file = match documents
        .ini_file(&path, true)
        .unwrap_or_else(|e| unsupported(&e))
    {
        Ok(file) => file,
        Err(error) => {
            alkyhol::ffi::SetLastError(error);
            return 0;
        }
    };
    file.try_lock()
        .unwrap_or_else(|_| unsupported("INI backing is locked by another reader or writer"));
    let mut file = Locked(file);
    let mut ini = ini32::Ini::parse(&contents(&mut file.0).unwrap_or_else(|e| unsupported(&e)))
        .unwrap_or_else(|e| unsupported(&e));
    ini.set(&strings[0], &strings[1], &strings[2])
        .unwrap_or_else(|e| unsupported(&e));
    let bytes = ini.bytes().unwrap_or_else(|e| unsupported(&e));
    file.0
        .seek(SeekFrom::Start(0))
        .and_then(|_| file.0.set_len(0))
        .and_then(|_| file.0.write_all(&bytes))
        .and_then(|_| file.0.sync_data())
        .unwrap_or_else(|e| unsupported(&format!("INI backing write failed: {e}")));
    1
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fallback_strips_only_the_measured_trailing_spaces_and_preserves_units() {
        assert_eq!(default_value(vec![32, 65, 32, 32]).unwrap(), [32, 65]);
        assert_eq!(default_value(vec![0xd800, 0x67]).unwrap(), [0xd800, 0x67]);
        assert!(default_value(vec![65, 9, 32]).is_err());
        assert!(default_value(vec![32, 32]).unwrap().is_empty());
    }
    #[test]
    fn actual_absence_is_distinct_from_existing_or_aliased_contents() {
        let root = std::env::temp_dir().join(format!("alkyhol-ini-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let guest = format!(
            "Z:{}",
            root.canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('/', "\\")
        );
        assert_eq!(missing(&format!("{guest}\\absent.ini")).unwrap(), 2);
        assert_eq!(
            missing(&format!("{guest}\\absent\\settings.ini")).unwrap(),
            3
        );
        std::fs::write(root.join("existing.ini"), b"[section]\nkey=value\n").unwrap();
        assert!(
            missing(&format!("{guest}\\existing.ini"))
                .unwrap_err()
                .contains("parsing")
        );
        assert!(missing(&format!("{guest}\\EXISTING.ini")).is_err());
        assert!(missing("settings.ini").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
