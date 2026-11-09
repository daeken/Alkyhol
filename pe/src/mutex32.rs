//! Named, initially unowned mutex references in an explicit guest session.
//! OS shared file locks represent live references across processes. Separate
//! locks support non-alertable zero-timeout acquisition on the one admitted guest
//! thread. Blocking waits, last-handle close while owned, inheritance and ACLs
//! beyond the explicit default policy remain unsupported.
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
    handles: BTreeMap<u32, Handle>,
    owners: BTreeMap<String, Ownership>,
}
struct Handle {
    _lease: HostLock,
    name: String,
}
struct Ownership {
    file: HostLock,
    depth: u32,
}
// A host fork/dup may temporarily retain an open-file description despite
// FD_CLOEXEC. Logical guest references are non-inheritable: explicitly release
// locks when their real owner ends instead of waiting for every host fd copy.
struct HostLock(File);
impl std::ops::Deref for HostLock {
    type Target = File;
    fn deref(&self) -> &File {
        &self.0
    }
}
impl std::ops::DerefMut for HostLock {
    fn deref_mut(&mut self) -> &mut File {
        &mut self.0
    }
}
impl Drop for HostLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
fn marker(file: &mut File) -> Result<u8> {
    if file.metadata().map_err(|e| e.to_string())?.len() != 1 {
        return Err("missing or malformed mutex ownership record".into());
    }
    file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    let mut byte = [0];
    file.read_exact(&mut byte).map_err(|e| e.to_string())?;
    if byte[0] > 1 {
        return Err("invalid mutex ownership marker".into());
    }
    Ok(byte[0])
}
fn set_marker(file: &mut File, value: u8) -> Result<()> {
    file.seek(SeekFrom::Start(0)).map_err(|e| e.to_string())?;
    file.write_all(&[value]).map_err(|e| e.to_string())
}
pub(super) fn name(units: &[u16]) -> Result<Vec<u16>> {
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
pub(super) fn live_name(directory: &File, units: &[u16]) -> Result<bool> {
    let (filename, _) = identity(&name(units)?);
    super::section32::live_lease(directory, &filename)
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
            owners: BTreeMap::new(),
        })
    }
    fn file(&self, name: &str) -> Result<HostLock> {
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
        Ok(HostLock(file))
    }
    pub(super) fn clear(&mut self) {
        // Ending this admitted guest thread abandons its ownership if another
        // process still holds a reference. Leave marker=1, release OS locks
        // before dropping reference leases. A new object resets the marker.
        self.owners.clear();
        self.handles.clear();
    }
    pub(super) fn close(&mut self, handle: u32) -> Result<bool> {
        if let Some(value) = self.handles.get(&handle)
            && self.owners.contains_key(&value.name)
            && self
                .handles
                .values()
                .filter(|h| h.name == value.name)
                .count()
                == 1
        {
            return Err("closing the last mutex handle while owned is unsupported".into());
        }
        if let Some(value) = self.handles.get(&handle) {
            value._lease.unlock().map_err(|e| e.to_string())?;
        }
        Ok(self.handles.remove(&handle).is_some())
    }
    fn poll(&mut self, handle: u32) -> Result<u32> {
        let name = self
            .handles
            .get(&handle)
            .ok_or("mutex wait requires an owned live handle")?
            .name
            .clone();
        if let Some(owner) = self.owners.get_mut(&name) {
            owner.depth = owner
                .depth
                .checked_add(1)
                .filter(|n| *n <= 1024)
                .ok_or("mutex recursive ownership budget exceeded")?;
            return Ok(0);
        }
        let gate = self.file("namespace.gate")?;
        gate.lock().map_err(|e| e.to_string())?;
        let mut file = self.file(&format!("{name}.owner"))?;
        let abandoned = marker(&mut file)?;
        match file.try_lock() {
            Ok(()) => (),
            Err(TryLockError::WouldBlock) => return Ok(258),
            Err(TryLockError::Error(e)) => return Err(e.to_string()),
        }
        set_marker(&mut file, 1)?;
        self.owners.insert(name, Ownership { file, depth: 1 });
        Ok(if abandoned == 1 { 0x80 } else { 0 })
    }
    fn release(&mut self, handle: u32) -> Result<bool> {
        let name = &self
            .handles
            .get(&handle)
            .ok_or("mutex release requires an owned live handle")?
            .name;
        let Some(owner) = self.owners.get_mut(name) else {
            return Ok(false);
        };
        if owner.depth > 1 {
            owner.depth -= 1;
            return Ok(true);
        }
        let gate = self.file("namespace.gate")?;
        gate.lock().map_err(|e| e.to_string())?;
        let owner = self.owners.get_mut(name).expect("checked owner");
        set_marker(&mut owner.file, 0)?;
        owner.file.unlock().map_err(|e| e.to_string())?;
        self.owners.remove(name);
        Ok(true)
    }
    fn create(&mut self, units: &[u16]) -> Result<(u32, bool)> {
        if self.handles.len() >= 128 {
            return Err("mutex handle budget exceeded".into());
        }
        let (filename, record) = identity(&name(units)?);
        let gate = self.file("namespace.gate")?;
        gate.lock().map_err(|e| e.to_string())?;
        if super::section32::live_name(&self.directory, units)? {
            return Err(
                "mutex/section name collision is outside the admitted object profile".into(),
            );
        }
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
        let mut owner = self.file(&format!("{filename}.owner"))?;
        if !existing {
            owner
                .try_lock()
                .map_err(|e| format!("inconsistent live owner without reference leases: {e}"))?;
            if owner.metadata().map_err(|e| e.to_string())?.len() != 0 {
                marker(&mut owner)?;
            }
            set_marker(&mut owner, 0)?;
        } else {
            marker(&mut owner)?;
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
        self.handles.insert(
            handle,
            Handle {
                _lease: lease,
                name: filename,
            },
        );
        Ok((handle, existing))
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.clear();
    }
}
pub(super) fn ownership(image: &LoadedImage32<'_>, args: &[u32; 8], wait: bool) -> u32 {
    if wait && (args[1] != 0 || args[2] != 0) {
        unsupported("mutex waits require zero timeout and alertable FALSE");
    }
    let state =
        image.cpu.mutexes.as_ref().unwrap_or_else(|| {
            unsupported("mutex ownership requires an explicit object directory")
        });
    let mut state = state.borrow_mut();
    if wait {
        state.poll(args[0]).unwrap_or_else(|e| unsupported(&e))
    } else if state.release(args[0]).unwrap_or_else(|e| unsupported(&e)) {
        1
    } else {
        alkyhol::ffi::SetLastError(288);
        0
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
    if defaults && args[2] == 0 {
        return super::unnamed_mutex32::create(image, args[1] != 0);
    }
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

pub(super) fn create_ansi(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    if args[2] != 0 {
        unsupported("named ANSI mutex conversion is not implemented");
    }
    create(image, args)
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
            assert!(a.close(first).unwrap());
            assert!(!a.close(first).unwrap());
            let (third, exists) = a.create(&wide("Owned")).unwrap();
            assert!(exists);
            assert!(b.close(second).unwrap());
            a.clear();
            assert!(!a.close(third).unwrap());
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
    #[test]
    fn ownership_is_recursive_across_handles_and_contended_between_process_views() {
        let path = std::env::temp_dir().join(format!("alkyhol-owner-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        {
            let mut a = State::open(&path).unwrap();
            let mut b = State::open(&path).unwrap();
            let (first, _) = a.create(&wide("Owned")).unwrap();
            let (second, _) = a.create(&wide("Owned")).unwrap();
            let (other, _) = b.create(&wide("Owned")).unwrap();
            assert!(!a.release(first).unwrap());
            assert_eq!(a.poll(first).unwrap(), 0);
            assert_eq!(a.poll(second).unwrap(), 0);
            assert!(a.close(first).unwrap()); // Ownership belongs to the thread/object.
            assert!(a.close(second).is_err()); // Last-owned-close remains outside the profile.
            assert_eq!(b.poll(other).unwrap(), 258);
            assert!(!b.release(other).unwrap());
            assert!(a.release(second).unwrap());
            assert_eq!(b.poll(other).unwrap(), 258);
            assert!(a.release(second).unwrap());
            assert_eq!(b.poll(other).unwrap(), 0);
            assert_eq!(a.poll(second).unwrap(), 258);
            b.clear();
            assert_eq!(a.poll(second).unwrap(), 0x80); // Real OS ownership released, marker survives.
            assert_eq!(a.poll(second).unwrap(), 0);
            assert!(a.release(second).unwrap());
            assert!(a.release(second).unwrap());
            assert!(a.close(second).unwrap());
            let (fresh, exists) = a.create(&wide("Owned")).unwrap();
            assert!(!exists);
            assert_eq!(a.poll(fresh).unwrap(), 0);
            a.clear();
            let (fresh, exists) = b.create(&wide("Owned")).unwrap();
            assert!(!exists);
            assert_eq!(b.poll(fresh).unwrap(), 0); // A new object cannot inherit abandonment.
            assert!(b.release(fresh).unwrap());
        }
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn malformed_ownership_state_does_not_become_a_successful_wait() {
        let path =
            std::env::temp_dir().join(format!("alkyhol-owner-corrupt-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        let mut state = State::open(&path).unwrap();
        let (handle, _) = state.create(&wide("Owned")).unwrap();
        let (filename, _) = identity(&wide("Owned"));
        std::fs::write(path.join(format!("{filename}.owner")), [2]).unwrap();
        assert!(state.poll(handle).is_err());
        drop(state);
        std::fs::remove_dir_all(path).unwrap();
    }
    #[test]
    fn noninheritable_guest_references_do_not_survive_unrelated_host_fd_copies() {
        let path =
            std::env::temp_dir().join(format!("alkyhol-owner-host-copy-{}", std::process::id()));
        std::fs::create_dir(&path).unwrap();
        let mut a = State::open(&path).unwrap();
        let mut b = State::open(&path).unwrap();
        let (handle, _) = a.create(&wide("Owned")).unwrap();
        let inherited_lease = a.handles[&handle]._lease.try_clone().unwrap();
        assert!(a.close(handle).unwrap());
        let (handle, exists) = a.create(&wide("Owned")).unwrap();
        assert!(!exists);
        let (other, _) = b.create(&wide("Owned")).unwrap();
        assert_eq!(a.poll(handle).unwrap(), 0);
        let (filename, _) = identity(&wide("Owned"));
        let inherited_owner = a.owners[&filename].file.try_clone().unwrap();
        a.clear();
        assert_eq!(b.poll(other).unwrap(), 0x80);
        assert!(b.release(other).unwrap());
        drop(inherited_owner);
        drop(inherited_lease);
        drop(a);
        drop(b);
        std::fs::remove_dir_all(path).unwrap();
    }
}
