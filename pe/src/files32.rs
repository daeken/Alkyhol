//! Checked PE32 filesystem search gateway. Unsupported search semantics stop;
//! observations do not become invented successful handles or missing files.
use super::{LoadedImage32, LowMapping, PAGE, startup32, unsupported};
use crate::{Result, process::FindData};
use std::{fs::Metadata, path::PathBuf, ptr};

pub(super) struct Search {
    _token: LowMapping,
}
enum Query {
    Found(String, Metadata),
    Failed(u32),
}
fn io(error: std::io::Error) -> Result<Query> {
    match error.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory => Ok(Query::Failed(3)),
        std::io::ErrorKind::PermissionDenied => Ok(Query::Failed(5)),
        _ => Err(format!(
            "unsupported PE32 filesystem I/O condition: {error}"
        )),
    }
}
// Exact canonical spellings only. Inspect each actual directory so an
// unsupported case alias cannot turn into an invented absent-file result.
fn query(text: &str) -> Result<Query> {
    let rest = text
        .strip_prefix("Z:\\")
        .or_else(|| text.strip_prefix("z:\\"))
        .ok_or("PE32 find requires the explicit Z: host-root namespace")?;
    let parts: Vec<_> = rest.split('\\').collect();
    if text.encode_utf16().count() > 32767
        || parts.len() > 256
        || parts.iter().any(|p| {
            p.is_empty()
                || matches!(*p, "." | "..")
                || p.ends_with(['.', ' '])
                || p.chars().any(|c| c < ' ' || "/<>:\"|?*".contains(c))
        })
    {
        return Err("PE32 find supports exact ordinary paths; wildcards/aliases/path normalization are unsupported".into());
    }
    for part in &parts {
        let stem = part.split('.').next().unwrap().to_ascii_uppercase();
        if matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$"
        ) || ["COM", "LPT"].iter().any(|prefix| {
            stem.strip_prefix(prefix).is_some_and(|suffix| {
                matches!(
                    suffix,
                    "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "¹" | "²" | "³"
                )
            })
        }) {
            return Err("PE32 find DOS device aliases are unsupported".into());
        }
    }
    let mut current = PathBuf::from("/");
    for (index, part) in parts.iter().enumerate() {
        let entries = match std::fs::read_dir(&current) {
            Ok(v) => v,
            Err(e) => return io(e),
        };
        let mut exact = None;
        let mut aliases = 0;
        let mut unknown = false;
        for (count, entry) in entries.enumerate() {
            if count >= 65536 {
                return Err("PE32 find directory inspection budget exceeded".into());
            }
            let entry = entry.map_err(|e| format!("PE32 directory inspection: {e}"))?;
            let Ok(name) = entry.file_name().into_string() else {
                unknown = true;
                continue;
            };
            if name == *part {
                exact = Some(entry.path());
            }
            if name.is_ascii() && part.is_ascii() && name.eq_ignore_ascii_case(part) {
                aliases += 1;
            }
            if !name.is_ascii() {
                unknown = true;
            }
        }
        let Some(path) = exact else {
            if aliases != 0 || unknown {
                return Err("PE32 find cannot resolve an unmeasured case/Unicode alias".into());
            }
            return Ok(Query::Failed(if index + 1 == parts.len() { 2 } else { 3 }));
        };
        if aliases > 1 {
            return Err("PE32 find has ambiguous case aliases".into());
        }
        let meta = match std::fs::symlink_metadata(&path) {
            Ok(v) => v,
            Err(e) => return io(e),
        };
        if meta.file_type().is_symlink() {
            return Err("PE32 find symbolic links/reparse points are unsupported".into());
        }
        if index + 1 == parts.len() {
            return Ok(Query::Found((*part).to_owned(), meta));
        }
        if !meta.is_dir() {
            return Ok(Query::Failed(3));
        }
        current = path;
    }
    Err("unsupported PE32 root-only find query".into())
}
/// The same exact, case-checked filesystem walk used by the search gateway.
/// Returns a backed path or measured absence, never a guessed case alias.
pub(super) fn existing(text: &str) -> Result<std::result::Result<(PathBuf, Metadata), u32>> {
    match query(text)? {
        Query::Found(_, metadata) => {
            Ok(Ok((PathBuf::from(text[2..].replace('\\', "/")), metadata)))
        }
        Query::Failed(error) => Ok(Err(error)),
    }
}
fn checked(image: &LoadedImage32<'_>, address: u32) {
    if !address.is_multiple_of(4)
        || !image.permits(address as usize, std::mem::size_of::<FindData>(), true)
    {
        unsupported("PE32 find output outside checked writable guest storage");
    }
}
fn fail(error: u32, value: u32) -> u32 {
    alkyhol::ffi::SetLastError(error);
    value
}

pub(super) fn first(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let units = startup32::read_units::<u16>(image, args[0], -1)
        .unwrap_or_else(|| unsupported("unsupported PE32 file search pattern"));
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 diagnostic file search: units_len={}; units_prefix={:x?}",
            units.len(),
            &units[..units.len().min(512)]
        );
    }
    checked(image, args[1]);
    let pattern = String::from_utf16(&units[..units.len() - 1])
        .unwrap_or_else(|_| unsupported("PE32 find pattern is invalid UTF-16"));
    let (name, metadata) = match query(&pattern).unwrap_or_else(|e| unsupported(&e)) {
        Query::Failed(error) => return fail(error, u32::MAX),
        Query::Found(name, metadata) => (name, metadata),
    };
    let mut record = FindData::from_metadata(&name, &metadata)
        .unwrap_or_else(|_| unsupported("PE32 find cannot represent the actual file metadata"));
    // This Z: filesystem profile has no generated DOS short aliases. Its initial
    // DOS attributes are archive for regular files, directory for directories,
    // plus the backed POSIX read-only permission state; it is not an NTFS dump.
    record.attributes = (if metadata.is_dir() { 0x10 } else { 0x20 })
        | if metadata.permissions().readonly() {
            1
        } else {
            0
        };
    let mut searches = image.cpu.searches.borrow_mut();
    if searches.len() >= 128 {
        unsupported("PE32 search handle budget exceeded");
    }
    let token = image
        .cpu
        .mapping(PAGE, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    token
        .protect(0, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    let handle = token.base as u32;
    // Native exact-file observations leave reserved DWORDs and unused string
    // tails untouched. Stage all actual metadata, then fill only native fields.
    unsafe {
        let source = (&record as *const FindData).cast::<u8>();
        let output = args[1] as usize as *mut u8;
        ptr::copy_nonoverlapping(source, output, 36);
        ptr::copy_nonoverlapping(
            source.add(44),
            output.add(44),
            (name.encode_utf16().count() + 1) * 2,
        );
        output.add(564).cast::<u16>().write(0);
    }
    searches.insert(handle, Search { _token: token });
    handle
}
pub(super) fn next(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    checked(image, args[1]);
    if !image.cpu.searches.borrow().contains_key(&args[0]) {
        unsupported("PE32 find-next requires a live owned search handle");
    }
    fail(18, 0)
}
pub(super) fn close(image: &LoadedImage32<'_>, handle: u32) -> u32 {
    if image.cpu.searches.borrow_mut().remove(&handle).is_some() {
        1
    } else {
        unsupported("PE32 find-close requires a live owned search handle")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_paths_have_real_metadata_and_missing_parents_are_distinct() {
        let root = std::fs::canonicalize(std::env::temp_dir())
            .unwrap()
            .join(format!(
                "alkyhol-find32-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("probe.dat"), b"Alkyhol").unwrap();
        let windows = format!("Z:{}", root.to_str().unwrap().replace('/', "\\"));
        match query(&format!("{windows}\\probe.dat")).unwrap() {
            Query::Found(name, meta) => {
                assert_eq!(name, "probe.dat");
                assert_eq!(meta.len(), 7);
            }
            _ => panic!("actual file missing"),
        }
        assert!(matches!(
            query(&format!("{windows}\\absent.dat")).unwrap(),
            Query::Failed(2)
        ));
        assert!(matches!(
            query(&format!("{windows}\\absent-dir\\probe.dat")).unwrap(),
            Query::Failed(3)
        ));
        assert!(query(&format!("{windows}\\PROBE.DAT")).is_err());
        for path in [
            "C:\\other.dat",
            "Z:\\*",
            "Z:\\a\\..\\b",
            "Z:\\a\\file.",
            "Z:\\NUL.txt",
            "Z:\\COM1",
        ] {
            assert!(query(path).is_err());
        }
        std::os::unix::fs::symlink(root.join("probe.dat"), root.join("link.dat")).unwrap();
        assert!(query(&format!("{windows}\\link.dat")).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
