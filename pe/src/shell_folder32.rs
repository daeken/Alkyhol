//! Explicit current-user Documents backing; no host-user profile inference.
use super::{LoadedImage32, module_path32, unsupported};
use crate::Result;
use std::{
    fs::File,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

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
#[cfg(test)]
mod tests {
    use super::*;
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
