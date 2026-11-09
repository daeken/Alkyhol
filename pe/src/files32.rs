//! Checked PE32 filesystem search gateway. Unsupported search semantics stop;
//! observations do not become invented successful handles or missing files.
use super::{LoadedImage32, LowMapping, PAGE, startup32, unsupported};
use crate::{Result, process::FindData};
use std::{fs::Metadata, path::PathBuf, ptr};

pub(super) struct Search {
    _token: LowMapping,
    entries: Vec<(String, Metadata)>,
    next: usize,
}
enum Query {
    Found(String, Metadata, PathBuf),
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
    query_with_casefold(text, false)
}
fn query_with_casefold(text: &str, fold: bool) -> Result<Query> {
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
        let mut folded = None;
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
                folded = Some(entry.path());
            }
            if !name.is_ascii() {
                unknown = true;
            }
        }
        let Some(path) = exact.or(if fold { folded } else { None }) else {
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
            return Ok(Query::Found(
                path.file_name().unwrap().to_str().unwrap().to_owned(),
                meta,
                path,
            ));
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
        Query::Found(_, metadata, path) => Ok(Ok((path, metadata))),
        Query::Failed(error) => Ok(Err(error)),
    }
}
/// Explicit bounded ASCII case-insensitive *read-only open* profile. The strict
/// search/metadata path and its frozen exclusions remain unchanged.
pub(super) fn existing_casefold(
    text: &str,
) -> Result<std::result::Result<(PathBuf, Metadata), u32>> {
    if !text.is_ascii() {
        return Err("case-insensitive file opens currently require ASCII paths".into());
    }
    match query_with_casefold(text, true)? {
        Query::Found(_, metadata, path) => Ok(Ok((path, metadata))),
        Query::Failed(error) => Ok(Err(error)),
    }
}
fn checked(image: &LoadedImage32<'_>, address: u32, ansi: bool) {
    let size = if ansi {
        320
    } else {
        std::mem::size_of::<FindData>()
    };
    if !address.is_multiple_of(4) || !image.permits(address as usize, size, true) {
        unsupported("PE32 find output outside checked writable guest storage");
    }
}
fn fail(error: u32, value: u32) -> u32 {
    alkyhol::ffi::SetLastError(error);
    value
}
fn records(pattern: &str) -> Result<std::result::Result<Vec<(String, Metadata)>, u32>> {
    if !pattern.contains(['*', '?']) {
        return match query(pattern)? {
            Query::Found(name, metadata, _) => Ok(Ok(vec![(name, metadata)])),
            Query::Failed(error) => Ok(Err(error)),
        };
    }
    let (parent, mask) = pattern
        .rsplit_once('\\')
        .ok_or("wildcard search requires a checked parent directory")?;
    if parent.contains(['*', '?'])
        || !(mask == "*"
            || mask == "*.*"
            || mask.strip_prefix("*.").is_some_and(|v| {
                !v.is_empty()
                    && v.is_ascii()
                    && v.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
            }))
    {
        return Err("wildcard search pattern is outside the measured all/extension profile".into());
    }
    let (directory, metadata) = match existing(parent)? {
        Ok(value) => value,
        Err(2 | 3) => return Ok(Err(3)),
        Err(error) => return Ok(Err(error)),
    };
    if !metadata.is_dir() {
        return Ok(Err(3));
    }
    let mut entries = Vec::new();
    if mask == "*" || mask == "*.*" {
        entries.push((".".into(), metadata));
        let ancestor = directory.parent().unwrap_or(&directory);
        entries.push((
            "..".into(),
            std::fs::metadata(ancestor).map_err(|e| e.to_string())?,
        ));
    }
    for (i, entry) in std::fs::read_dir(&directory)
        .map_err(|e| e.to_string())?
        .enumerate()
    {
        if i >= 65536 {
            return Err("wildcard search exceeds directory-entry budget".into());
        }
        let entry = entry.map_err(|e| e.to_string())?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "wildcard search encountered a non-Unicode filename")?;
        let matches = mask == "*"
            || mask == "*.*"
            || name
                .to_ascii_lowercase()
                .ends_with(&mask[1..].to_ascii_lowercase());
        if !matches {
            continue;
        }
        let meta = std::fs::symlink_metadata(entry.path()).map_err(|e| e.to_string())?;
        if meta.file_type().is_symlink() || !(meta.is_file() || meta.is_dir()) {
            return Err("wildcard search encountered an unsupported reparse/special entry".into());
        }
        entries.push((name, meta));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    if entries.is_empty() {
        Ok(Err(2))
    } else {
        Ok(Ok(entries))
    }
}
fn write_record(
    image: &LoadedImage32<'_>,
    address: u32,
    name: &str,
    metadata: &Metadata,
    ansi: bool,
) {
    checked(image, address, ansi);
    if ansi && !name.is_ascii() {
        unsupported("ANSI find results currently require ASCII filenames");
    }
    let mut record = FindData::from_metadata(name, metadata)
        .unwrap_or_else(|_| unsupported("PE32 find cannot represent actual file metadata"));
    record.attributes = (if metadata.is_dir() { 0x10 } else { 0x20 })
        | if metadata.permissions().readonly() {
            1
        } else {
            0
        };
    unsafe {
        let source = (&record as *const FindData).cast::<u8>();
        let output = address as *mut u8;
        ptr::copy_nonoverlapping(source, output, 36);
        if ansi {
            ptr::copy_nonoverlapping(name.as_ptr(), output.add(44), name.len());
            output.add(44 + name.len()).write(0);
            output.add(304).write(0);
        } else {
            ptr::copy_nonoverlapping(
                source.add(44),
                output.add(44),
                (name.encode_utf16().count() + 1) * 2,
            );
            output.add(564).cast::<u16>().write(0);
        }
    }
}
pub(super) fn first(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    first_kind(image, args, false)
}
pub(super) fn first_kind(image: &LoadedImage32<'_>, args: &[u32; 8], ansi: bool) -> u32 {
    let pattern = if ansi {
        super::bulk32::ascii(image, args[0])
    } else {
        let units = startup32::read_units::<u16>(image, args[0], -1)
            .unwrap_or_else(|| unsupported("unsupported PE32 file search pattern"));
        String::from_utf16(&units[..units.len() - 1])
            .unwrap_or_else(|_| unsupported("PE32 find pattern is invalid UTF-16"))
    };
    if image.cpu.diagnostic.is_some() {
        eprintln!("PE32 diagnostic file search: pattern={pattern:?}; ansi={ansi}");
    }
    checked(image, args[1], ansi);
    let pattern =
        super::directory32::ordinary_path(image, &pattern).unwrap_or_else(|e| unsupported(&e));
    let entries = match records(&pattern).unwrap_or_else(|e| unsupported(&e)) {
        Ok(entries) => entries,
        Err(error) => return fail(error, u32::MAX),
    };
    if image.cpu.searches.borrow().len() >= 128 {
        unsupported("PE32 search handle budget exceeded");
    }
    write_record(image, args[1], &entries[0].0, &entries[0].1, ansi);
    let token = image
        .cpu
        .mapping(PAGE, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    token
        .protect(0, PAGE, 0)
        .unwrap_or_else(|e| unsupported(&e));
    let handle = token.base as u32;
    image.cpu.searches.borrow_mut().insert(
        handle,
        Search {
            _token: token,
            entries,
            next: 1,
        },
    );
    handle
}
pub(super) fn next(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    next_kind(image, args, false)
}
pub(super) fn next_kind(image: &LoadedImage32<'_>, args: &[u32; 8], ansi: bool) -> u32 {
    checked(image, args[1], ansi);
    let mut searches = image.cpu.searches.borrow_mut();
    let search = searches
        .get_mut(&args[0])
        .unwrap_or_else(|| unsupported("PE32 find-next requires a live owned search handle"));
    let Some((name, metadata)) = search.entries.get(search.next) else {
        return fail(18, 0);
    };
    write_record(image, args[1], name, metadata, ansi);
    search.next += 1;
    1
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
    fn real_search_snapshots_keep_dots_extension_case_and_absence_distinct() {
        let root = std::env::temp_dir().join(format!("alkyhol-search-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("a.TXT"), b"abc").unwrap();
        std::fs::write(root.join("plain"), b"x").unwrap();
        std::fs::create_dir(root.join("folder")).unwrap();
        let path = format!(
            "Z:{}",
            root.canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('/', "\\")
        );
        let all = records(&format!("{path}\\*")).unwrap().unwrap();
        assert_eq!(
            all.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            [".", "..", "a.TXT", "folder", "plain"]
        );
        let ext = records(&format!("{path}\\*.txt")).unwrap().unwrap();
        assert_eq!(ext.len(), 1);
        assert_eq!(ext[0].0, "a.TXT");
        assert_eq!(ext[0].1.len(), 3);
        assert_eq!(
            records(&format!("{path}\\missing\\*")).unwrap().err(),
            Some(3)
        );
        assert_eq!(
            records(&format!("{path}\\folder\\*.txt")).unwrap().err(),
            Some(2)
        );
        assert!(records(&format!("{path}\\?")).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn opted_in_ascii_casefold_resolves_actual_names_and_preserves_absence_and_alias_errors() {
        let root = std::env::temp_dir().join(format!("alkyhol-fold-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        std::fs::create_dir(root.join("OwnedCase")).unwrap();
        let actual = root.join("OwnedCase/MiXeD.DAT");
        std::fs::write(&actual, b"case").unwrap();
        let guest = format!("Z:{}", root.to_str().unwrap().replace('/', "\\"));
        let spelling = format!("{guest}\\ownedcase\\mixed.dat");
        assert!(existing(&spelling).is_err());
        let (path, metadata) = existing_casefold(&spelling).unwrap().unwrap();
        assert_eq!(path, actual);
        assert_eq!(metadata.len(), 4);
        assert_eq!(std::fs::read(path).unwrap(), b"case");
        assert_eq!(
            existing_casefold(&format!("{guest}\\ownedcase\\absent"))
                .unwrap()
                .err(),
            Some(2)
        );
        assert_eq!(
            existing_casefold(&format!("{guest}\\absent\\file"))
                .unwrap()
                .err(),
            Some(3)
        );
        std::os::unix::fs::symlink(&actual, root.join("OwnedCase/Link")).unwrap();
        assert!(existing_casefold(&format!("{guest}\\ownedcase\\link")).is_err());
        assert!(existing_casefold(&format!("{guest}\\ownedcase\\..\\file")).is_err());
        assert!(existing_casefold(&format!("{guest}\\ownedcase\\桜")).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
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
            Query::Found(name, meta, _) => {
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
