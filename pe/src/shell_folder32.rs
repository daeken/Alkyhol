//! Explicit current-user Documents backing; no host-user profile inference.
use super::{LoadedImage32, files32, module_path32, startup32, unsupported};
use crate::Result;
use std::{
    ffi::CString,
    fs::File,
    os::fd::{AsRawFd, FromRawFd},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

const O_DIRECTORY: i32 = 0x100000;
const O_NOFOLLOW: i32 = 0x100;
const O_CLOEXEC: i32 = 0x1000000;
unsafe extern "C" {
    fn openat(directory: i32, path: *const std::ffi::c_char, flags: i32, ...) -> i32;
    fn mkdirat(directory: i32, path: *const std::ffi::c_char, mode: u32) -> i32;
}
pub(super) struct Documents {
    host: PathBuf,
    file: File,
    units: Vec<u16>,
}
impl Documents {
    pub(super) fn open(path: &Path) -> Result<Self> {
        let host = path
            .canonicalize()
            .map_err(|e| format!("Documents directory: {e}"))?;
        let units = module_path32::directory_role(&host, "Documents directory")?;
        if units.len() >= 260 {
            return Err("Documents directory exceeds the MAX_PATH profile".into());
        }
        let file = File::open(&host).map_err(|e| format!("open Documents directory: {e}"))?;
        let value = Self { host, file, units };
        value.check()?;
        Ok(value)
    }
    // Only this explicitly selected profile directory is a write capability.
    // Existing lexical/case checks reject DOS aliases, symlinks and guessed
    // absence; mkdirat stays anchored to a checked, no-follow directory handle.
    fn create(&self, text: &str) -> Result<std::result::Result<(), u32>> {
        self.check()?;
        if text.encode_utf16().count() >= 248 {
            return Err("directory creation exceeds the bounded path-length profile".into());
        }
        // Preserve the caller's single final separator; it names the same
        // directory. Other normalization remains unsupported.
        let trailing = text.len() > 3 && text.ends_with('\\');
        let text = if trailing {
            &text[..text.len() - 1]
        } else {
            text
        };
        let root = String::from_utf16(&self.units).map_err(|_| "invalid Documents role")?;
        if text == root {
            return Ok(Err(183));
        }
        // The original caller walks path prefixes before creating descendants.
        // Existing ordinary ancestors can report ALREADY_EXISTS using read-only
        // evidence; this never grants a write capability outside Documents.
        if text.len() > 3 && root.starts_with(&format!("{text}\\")) {
            return match files32::existing(text)? {
                Ok((_, metadata)) if metadata.is_dir() => Ok(Err(183)),
                _ => Err("Documents ancestor identity changed".into()),
            };
        }
        let relative = text
            .strip_prefix(&format!("{root}\\"))
            .ok_or("directory creation is restricted to the explicit Documents backing")?;
        if !relative.is_ascii() {
            return Err("directory creation requires measured ASCII descendant names".into());
        }
        match files32::existing(text)? {
            Ok((_, metadata)) => {
                if trailing && !metadata.is_dir() {
                    return Err(
                        "directory creation with a trailing separator on a file is unmeasured"
                            .into(),
                    );
                }
                return Ok(Err(183));
            }
            Err(3) => return Ok(Err(3)),
            Err(2) => (),
            Err(error) => {
                return Err(format!(
                    "directory creation I/O status {error} is unsupported"
                ));
            }
        }
        // The checked walk above establishes exact ordinary spelling and parent
        // existence. Open each parent relative to the held capability without
        // following links, then verify the final parent before the write.
        let mut parts: Vec<_> = relative.split('\\').collect();
        let leaf = CString::new(parts.pop().ok_or("missing directory leaf")?)
            .map_err(|e| e.to_string())?;
        let mut parent = self.file.try_clone().map_err(|e| e.to_string())?;
        for component in parts {
            let name = CString::new(component).map_err(|e| e.to_string())?;
            let fd = unsafe {
                openat(
                    parent.as_raw_fd(),
                    name.as_ptr(),
                    O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC,
                )
            };
            if fd < 0 {
                return Err(format!(
                    "directory parent changed during open: {}",
                    std::io::Error::last_os_error()
                ));
            }
            parent = unsafe { File::from_raw_fd(fd) };
        }
        self.check()?;
        let parent_path = text.rsplit_once('\\').ok_or("missing directory parent")?.0;
        let (_, metadata) = files32::existing(parent_path)?
            .map_err(|_| "directory parent changed before creation")?;
        let opened = parent.metadata().map_err(|e| e.to_string())?;
        if !opened.is_dir() || (opened.dev(), opened.ino()) != (metadata.dev(), metadata.ino()) {
            return Err("directory parent identity changed before creation".into());
        }
        if unsafe { mkdirat(parent.as_raw_fd(), leaf.as_ptr(), 0o777) } == 0 {
            return Ok(Ok(()));
        }
        let error = std::io::Error::last_os_error();
        match error.kind() {
            std::io::ErrorKind::AlreadyExists => Ok(Err(183)),
            std::io::ErrorKind::PermissionDenied => Ok(Err(5)),
            std::io::ErrorKind::NotFound | std::io::ErrorKind::NotADirectory => Ok(Err(3)),
            _ => Err(format!("unsupported directory creation failure: {error}")),
        }
    }
    fn check(&self) -> Result<()> {
        let opened = self.file.metadata().map_err(|e| e.to_string())?;
        let live = std::fs::symlink_metadata(&self.host).map_err(|e| e.to_string())?;
        if !opened.is_dir()
            || !live.is_dir()
            || (opened.dev(), opened.ino()) != (live.dev(), live.ino())
        {
            return Err("backed Documents directory was renamed or replaced externally".into());
        }
        Ok(())
    }
}
pub(super) fn get(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    if args[..4] != [0, 5, 0, 0] {
        unsupported(
            "SHGetFolderPathW requires NULL owner/token, CSIDL_PERSONAL and current-path flags",
        );
    }
    let documents = image.cpu.documents.as_ref().unwrap_or_else(|| {
        unsupported("SHGetFolderPathW requires an explicit backed Documents directory")
    });
    documents.check().unwrap_or_else(|e| unsupported(&e));
    if !args[4].is_multiple_of(2) || !image.permits(args[4] as usize, 260 * 2, true) {
        unsupported("SHGetFolderPathW output requires writable MAX_PATH WCHAR storage");
    }
    unsafe {
        std::ptr::copy_nonoverlapping(
            documents.units.as_ptr(),
            args[4] as *mut u16,
            documents.units.len(),
        );
        (args[4] as *mut u16).add(documents.units.len()).write(0);
    }
    // S_OK is backed by the configured real directory. HRESULT is the API's
    // error channel; no undocumented LastError normalization is claimed.
    0
}
pub(super) fn create(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let units = startup32::read_units::<u16>(image, args[0], -1)
        .unwrap_or_else(|| unsupported("directory creation input outside checked UTF-16 storage"));
    if image.cpu.diagnostic.is_some() {
        eprintln!("PE32 diagnostic directory creation: path_units={units:x?}");
    }
    if args[1] != 0 {
        unsupported("directory creation security attributes are unsupported");
    }
    let text = String::from_utf16(&units[..units.len() - 1])
        .unwrap_or_else(|_| unsupported("directory creation path is invalid UTF-16"));
    let documents = image.cpu.documents.as_ref().unwrap_or_else(|| {
        unsupported("directory creation requires an explicit Documents backing")
    });
    match documents.create(&text).unwrap_or_else(|e| unsupported(&e)) {
        Ok(()) => 1,
        Err(error) => {
            alkyhol::ffi::SetLastError(error);
            0
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn creations_are_real_scoped_and_do_not_follow_links_or_replace_files() {
        let root = std::env::temp_dir().join(format!("alkyhol-mkdir-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let inside = root.join("Documents");
        let outside = root.join("outside");
        std::fs::create_dir(&inside).unwrap();
        std::fs::create_dir(&outside).unwrap();
        let documents = Documents::open(&inside).unwrap();
        let base = String::from_utf16(&documents.units).unwrap();
        assert_eq!(documents.create(&base).unwrap(), Err(183));
        let ancestor = base.rsplit_once('\\').unwrap().0;
        assert_eq!(documents.create(ancestor).unwrap(), Err(183));
        assert_eq!(documents.create(&format!("{base}\\new")).unwrap(), Ok(()));
        assert!(inside.join("new").is_dir());
        assert_eq!(documents.create(&format!("{base}\\new")).unwrap(), Err(183));
        assert_eq!(
            documents.create(&format!("{base}\\new\\child")).unwrap(),
            Ok(())
        );
        assert!(inside.join("new/child").is_dir());
        assert_eq!(
            documents.create(&format!("{base}\\new\\child\\")).unwrap(),
            Err(183)
        );
        assert_eq!(
            documents.create(&format!("{base}\\trailing\\")).unwrap(),
            Ok(())
        );
        assert!(inside.join("trailing").is_dir());
        assert!(documents.create(&format!("{base}\\new\\\\")).is_err());
        assert_eq!(
            documents
                .create(&format!("{base}\\missing\\child"))
                .unwrap(),
            Err(3)
        );
        assert!(!inside.join("missing").exists());
        std::fs::write(inside.join("file"), b"unchanged").unwrap();
        assert_eq!(
            documents.create(&format!("{base}\\file")).unwrap(),
            Err(183)
        );
        assert_eq!(
            documents.create(&format!("{base}\\file\\child")).unwrap(),
            Err(3)
        );
        assert_eq!(std::fs::read(inside.join("file")).unwrap(), b"unchanged");
        std::os::unix::fs::symlink(&outside, inside.join("linked")).unwrap();
        for path in [
            format!("{base}\\linked\\escape"),
            format!("{base}\\..\\escape"),
            format!("{base}-outside\\escape"),
            format!("{base}\\NEW"),
            format!("{base}\\NUL"),
            format!("{base}\\new-桜"),
        ] {
            assert!(documents.create(&path).is_err(), "{path}");
        }
        assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 0);
        std::fs::rename(&inside, root.join("moved")).unwrap();
        assert!(documents.create(&format!("{base}\\after-move")).is_err());
        drop(documents);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn directory_identity_unicode_and_replacement_are_checked() {
        let root = std::env::temp_dir().join(format!("alkyhol-documents-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("Documents-桜");
        std::fs::create_dir(&path).unwrap();
        let documents = Documents::open(&path).unwrap();
        let expected = format!(
            "Z:{}",
            path.canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .replace('/', "\\")
        );
        assert_eq!(String::from_utf16(&documents.units).unwrap(), expected);
        documents.check().unwrap();
        std::fs::rename(&path, root.join("old")).unwrap();
        std::fs::create_dir(&path).unwrap();
        assert!(documents.check().unwrap_err().contains("replaced"));
        let file = root.join("file");
        std::fs::write(&file, b"not a directory").unwrap();
        assert!(Documents::open(&file).is_err());
        assert!(Documents::open(&root.join("missing")).is_err());
        let mut long = root.clone();
        for _ in 0..4 {
            long = long.join("x".repeat(64));
        }
        std::fs::create_dir_all(&long).unwrap();
        assert!(Documents::open(&long).err().unwrap().contains("MAX_PATH"));
        drop(documents);
        std::fs::remove_dir_all(root).unwrap();
    }
}
