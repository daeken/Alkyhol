//! Named, initially unowned mutex references in an explicit guest session.
//! OS shared file locks represent live references across processes. This does
//! not implement acquisition, release, abandonment, inheritance or Windows ACLs.
use super::{LoadedImage32, startup32, unsupported};
use crate::Result;
use std::{
    collections::BTreeMap,
    ffi::CString,
    fs::{File, OpenOptions, TryLockError},
    io::{Read, Seek, SeekFrom, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::Path,
};

const O_NOFOLLOW: i32 = 0x100;
const O_DIRECTORY: i32 = 0x100000;
unsafe extern "C" {
    fn openat(directory: i32, path: *const std::ffi::c_char, flags: i32, ...) -> i32;
}

pub(super) struct State {
    directory: File,
    handles: BTreeMap<u32, File>,
}
fn name(units: &[u16]) -> Result<Vec<u16>> {
    // Bare and Local names share this caller-selected session namespace.
    // Global/private namespaces remain unknown, rather than being aliased.
    let local: Vec<_> = "Local\\".encode_utf16().collect();
    if units.is_empty() || units.len() >= 260 || units.contains(&0) {
        return Err("mutex name outside admitted nonempty MAX_PATH profile".into());
    }
    let value = units.strip_prefix(local.as_slice()).unwrap_or(units);
    if value.is_empty() || value.contains(&u16::from(b'\\')) {
        return Err("mutex Global/private/embedded-backslash namespace is unsupported".into());
    }
    Ok(value.to_vec())
}
fn identity(units: &[u16]) -> (String, Vec<u8>) {
    let mut record = b"Alkyhol mutex lease v1\0".to_vec();
    record.extend(units.iter().flat_map(|u| u.to_le_bytes()));
    let hash = record.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    // The complete record is checked too: hash collision is an explicit error.
    (format!("mutex-{hash:016x}.lease"), record)
}
impl State {
    pub(super) fn open(path: &Path) -> Result<Self> {
        let directory = OpenOptions::new()
            .read(true)
            .custom_flags(O_DIRECTORY | O_NOFOLLOW)
            .open(path)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            directory,
            handles: BTreeMap::new(),
        })
    }
    fn file(&self, name: &str) -> Result<File> {
        let name = CString::new(name).map_err(|e| e.to_string())?;
        // Anchor all files to the opened directory, including across renames.
        let fd = unsafe {
            openat(
                self.directory.as_raw_fd(),
                name.as_ptr(),
                2 | 0x200 | O_NOFOLLOW | 0x1000000,
                0o600u32,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let file = unsafe { File::from_raw_fd(fd) };
        let metadata = file.metadata().map_err(|e| e.to_string())?;
        if !metadata.is_file() || metadata.nlink() != 1 {
            return Err("mutex namespace requires regular non-aliased owned files".into());
        }
        Ok(file)
    }
    pub(super) fn clear(&mut self) {
        self.handles.clear();
    }
    pub(super) fn close(&mut self, handle: u32) -> bool {
        self.handles.remove(&handle).is_some()
    }
    fn create(&mut self, units: &[u16]) -> Result<(u32, bool)> {
        if self.handles.len() >= 128 {
            return Err("mutex handle budget exceeded".into());
        }
        let (filename, record) = identity(&name(units)?);
        let gate = self.file("namespace.gate")?;
        gate.lock().map_err(|e| e.to_string())?;
        let mut lease = self.file(&filename)?;
        let existing = match lease.try_lock() {
            Ok(()) => false,
            Err(TryLockError::WouldBlock) => true,
            Err(TryLockError::Error(e)) => return Err(e.to_string()),
        };
        if lease.metadata().map_err(|e| e.to_string())?.len() > 1024 {
            return Err("mutex identity record exceeds bound".into());
        }
        let mut saved = Vec::new();
        lease.read_to_end(&mut saved).map_err(|e| e.to_string())?;
        if saved != record && (!saved.is_empty() || existing) {
            return Err("mutex identity collision or incompatible namespace record".into());
        }
        if !existing {
            lease.set_len(0).map_err(|e| e.to_string())?;
            lease.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
            lease.write_all(&record).map_err(|e| e.to_string())?;
        }
        // Downgrade/acquire while creation remains serialized. Each returned
        // handle has an independent open-file reference and shared OS lease.
        lease.lock_shared().map_err(|e| e.to_string())?;
        let handle = alkyhol::kernel::reserve_handle_token()
            .map_err(|e| format!("mutex handle allocation: {e:?}"))?;
        let handle = u32::try_from(handle).map_err(|_| "mutex handle exceeds PE32")?;
        if (0x70000001..=0x70000003).contains(&handle) {
            return Err("mutex handle overlaps standard-stream namespace".into());
        }
        self.handles.insert(handle, lease);
        Ok((handle, existing))
    }
}
pub(super) fn create(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    if args[0] != 0 && image.cpu.diagnostic.is_some() {
        if args[0].is_multiple_of(4) && image.permits(args[0] as usize, 12, false) {
            let fields = unsafe { (args[0] as *const [u32; 3]).read() };
            eprintln!(
                "PE32 diagnostic mutex SECURITY_ATTRIBUTES: length={}; descriptor=0x{:08x}; inherit={}",
                fields[0], fields[1], fields[2]
            );
        } else {
            eprintln!(
                "PE32 diagnostic mutex SECURITY_ATTRIBUTES: outside aligned readable guest storage"
            );
        }
    }
    // Measured Win32 form: 12 bytes, default descriptor, non-inheritable.
    // This selects the same backed security/handle policy as NULL attributes.
    let defaults = args[0] == 0
        || (args[0].is_multiple_of(4)
            && image.permits(args[0] as usize, 12, false)
            && unsafe { (args[0] as *const [u32; 3]).read() } == [12, 0, 0]);
    if !defaults || args[1] != 0 {
        unsupported("mutex security attributes or initial ownership are unsupported");
    }
    if args[2] == 0 {
        unsupported("unnamed mutexes are unsupported");
    }
    let state = image
        .cpu
        .mutexes
        .as_ref()
        .unwrap_or_else(|| unsupported("mutex creation requires an explicit object directory"));
    let units = startup32::read_units::<u16>(image, args[2], -1)
        .unwrap_or_else(|| unsupported("mutex name outside readable guest storage"));
    let (handle, existing) = state
        .borrow_mut()
        .create(&units[..units.len() - 1])
        .unwrap_or_else(|e| unsupported(&e));
    alkyhol::ffi::SetLastError(if existing { 183 } else { 0 });
    handle
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().collect()
    }
    #[test]
    fn names_preserve_case_and_units_and_only_admit_the_session_namespace() {
        assert_eq!(name(&wide("Local\\Ownedλ")).unwrap(), wide("Ownedλ"));
        assert_ne!(name(&wide("Owned")).unwrap(), name(&wide("owned")).unwrap());
        for s in ["", "Local\\", "Global\\Owned", "a\\b", "a\0b"] {
            assert!(name(&wide(s)).is_err());
        }
        assert!(name(&vec![65; 259]).is_ok());
        assert!(name(&vec![65; 260]).is_err());
    }
    #[test]
    fn independent_handles_keep_an_object_alive_until_the_last_close() {
        let path = std::env::temp_dir().join(format!("alkyhol-mutex-unit-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        {
            let mut a = State::open(&path).unwrap();
            let mut b = State::open(&path).unwrap();
            let (first, exists) = a.create(&wide("Owned")).unwrap();
            assert!(!exists);
            let (second, exists) = b.create(&wide("Local\\Owned")).unwrap();
            assert!(exists);
            assert_ne!(first, second);
            assert!(a.close(first));
            assert!(!a.close(first));
            let (third, exists) = a.create(&wide("Owned")).unwrap();
            assert!(exists);
            assert!(b.close(second));
            a.clear();
            assert!(!a.close(third));
            let (_, exists) = b.create(&wide("Owned")).unwrap();
            assert!(!exists);
            let (_, exists) = a.create(&wide("owned")).unwrap();
            assert!(!exists);
        }
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn mismatching_identity_and_symlinked_files_fail_honestly() {
        let path =
            std::env::temp_dir().join(format!("alkyhol-mutex-corrupt-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        let mut state = State::open(&path).unwrap();
        let (filename, _) = identity(&wide("Owned"));
        std::fs::write(path.join(&filename), b"wrong object").unwrap();
        assert!(state.create(&wide("Owned")).is_err());
        std::fs::remove_file(path.join(&filename)).unwrap();
        std::os::unix::fs::symlink("namespace.gate", path.join(filename)).unwrap();
        assert!(state.create(&wide("Owned")).is_err());
        drop(state);
        std::fs::remove_dir_all(path).unwrap();
    }
}
