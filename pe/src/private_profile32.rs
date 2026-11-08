//! Explicit file-only private-profile reads. Unknown contents never become
//! defaults: only actual checked file/path absence admits the fallback branch.
use super::{LoadedImage32, directory32, files32, startup32, unsupported};
use crate::Result;

fn default_value(mut units: Vec<u16>) -> Result<Vec<u16>> {
    while units.last() == Some(&32) {
        units.pop();
    }
    if units.last().is_some_and(|v| *v < 32) {
        return Err("private-profile trailing control whitespace is unmeasured".into());
    }
    Ok(units)
}
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
    let error = missing(&path).unwrap_or_else(|e| unsupported(&e));
    let fallback = default_value(fallback).unwrap_or_else(|e| unsupported(&e));
    let count = fallback.len().min(args[4] as usize - 1);
    unsafe {
        std::ptr::copy_nonoverlapping(fallback.as_ptr(), args[3] as *mut u16, count);
        (args[3] as *mut u16).add(count).write(0);
    }
    alkyhol::ffi::SetLastError(error);
    count as u32
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
