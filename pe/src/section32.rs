//! Real shared section descriptors and low, independently owned mapped views.
//! Named lifetimes use shared OS leases in the explicit guest session directory.
use super::{LoadedImage32, LowMapping, PAGE, bulk32, startup32, unsupported};
use crate::Result;
use std::{
    collections::BTreeMap,
    ffi::{CString, c_void},
    fs::{File, OpenOptions, TryLockError},
    io::{Read, Seek, SeekFrom, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::Path,
    rc::Rc,
};
const HEADER: usize = 65536;
unsafe extern "C" {
    fn openat(dir: i32, name: *const std::ffi::c_char, flags: i32, ...) -> i32;
    fn unlinkat(dir: i32, name: *const std::ffi::c_char, flags: i32) -> i32;
}
struct Lease(File);
impl Drop for Lease {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}
struct Section {
    _lease: Option<Lease>,
    file: File,
    length: usize,
    protection: u32,
}
struct View {
    mapping: LowMapping,
    _section: Rc<Section>,
    write: bool,
}
#[derive(Default)]
pub(super) struct State {
    directory: Option<File>,
    handles: BTreeMap<u32, (Rc<Section>, bool)>,
    views: BTreeMap<u32, View>,
}
fn file(directory: &File, name: &str, create: bool) -> Result<Option<File>> {
    let name = CString::new(name).unwrap();
    let fd = unsafe {
        openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            2 | 0x100 | 0x1000000 | if create { 0x200 } else { 0 },
            0o600u32,
        )
    };
    if fd < 0 {
        let e = std::io::Error::last_os_error();
        if !create && e.kind() == std::io::ErrorKind::NotFound {
            return Ok(None);
        }
        return Err(format!("section namespace I/O: {e}"));
    }
    let file = unsafe { File::from_raw_fd(fd) };
    let metadata = file.metadata().map_err(|e| e.to_string())?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err("section namespace requires ordinary files".into());
    }
    Ok(Some(file))
}
fn identity(units: &[u16]) -> (String, Vec<u8>) {
    let mut record = b"Alkyhol section v1\0".to_vec();
    record.extend(units.iter().flat_map(|n| n.to_le_bytes()));
    let hash = record.iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    (format!("section-{hash:016x}"), record)
}
pub(super) fn live_lease(directory: &File, name: &str) -> Result<bool> {
    let Some(f) = file(directory, name, false)? else {
        return Ok(false);
    };
    match f.try_lock() {
        Ok(()) => {
            f.unlock().map_err(|e| e.to_string())?;
            Ok(false)
        }
        Err(TryLockError::WouldBlock) => Ok(true),
        Err(TryLockError::Error(e)) => Err(e.to_string()),
    }
}
pub(super) fn live_name(directory: &File, units: &[u16]) -> Result<bool> {
    let (name, _) = identity(&super::mutex32::name(units)?);
    live_lease(directory, &format!("{name}.lease"))
}
impl State {
    pub(super) fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            directory: Some(
                OpenOptions::new()
                    .read(true)
                    .custom_flags(0x100 | 0x100000 | 0x1000000)
                    .open(path)
                    .map_err(|e| e.to_string())?,
            ),
            ..Default::default()
        })
    }
    pub(super) fn close(&mut self, h: u32) -> bool {
        self.handles.remove(&h).is_some()
    }
    pub(super) fn clear_handles(&mut self) {
        self.handles.clear();
    }
    pub(super) fn permits(&self, address: usize, len: usize, write: bool) -> bool {
        address.checked_add(len).is_some_and(|end| {
            self.views.values().any(|v| {
                (!write || v.write)
                    && address >= v.mapping.base as usize
                    && end <= v.mapping.base as usize + v.mapping.len
            })
        })
    }
    fn anonymous(&self, size: usize, protection: u32) -> Result<Section> {
        let directory = self
            .directory
            .as_ref()
            .ok_or("sections require an explicit guest session directory")?;
        let mut random = [0u8; 8];
        if unsafe { super::getentropy(random.as_mut_ptr().cast(), 8) } != 0 {
            return Err("anonymous section entropy unavailable".into());
        }
        let name = format!("anonymous-section-{:016x}", u64::from_ne_bytes(random));
        let c = CString::new(name).unwrap();
        let fd = unsafe {
            openat(
                directory.as_raw_fd(),
                c.as_ptr(),
                2 | 0x200 | 0x800 | 0x100 | 0x1000000,
                0o600u32,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error().to_string());
        }
        let file = unsafe { File::from_raw_fd(fd) };
        if unsafe { unlinkat(directory.as_raw_fd(), c.as_ptr(), 0) } != 0 {
            return Err("anonymous section unlink failed".into());
        }
        file.set_len((HEADER + size) as u64)
            .map_err(|e| e.to_string())?;
        Ok(Section {
            _lease: None,
            file,
            length: size,
            protection,
        })
    }
    fn named(
        &self,
        units: &[u16],
        size: usize,
        protection: u32,
        open_only: bool,
    ) -> Result<Option<(Section, bool)>> {
        let directory = self
            .directory
            .as_ref()
            .ok_or("sections require an explicit guest session directory")?;
        let canonical = super::mutex32::name(units)?;
        let (name, record) = identity(&canonical);
        let gate = file(directory, "namespace.gate", true)?.unwrap();
        let _gate = Lease(gate);
        _gate.0.lock().map_err(|e| e.to_string())?;
        if super::mutex32::live_name(directory, &canonical)? {
            return Err(
                "mutex/section name collision is outside the admitted object profile".into(),
            );
        }
        let Some(lease) = file(directory, &format!("{name}.lease"), !open_only)? else {
            return Ok(None);
        };
        let existing = match lease.try_lock() {
            Ok(()) => false,
            Err(TryLockError::WouldBlock) => true,
            Err(TryLockError::Error(e)) => return Err(e.to_string()),
        };
        if open_only && !existing {
            lease.unlock().map_err(|e| e.to_string())?;
            return Ok(None);
        }
        let mut backing = file(directory, &format!("{name}.data"), !existing)?
            .ok_or("live section has no backing file")?;
        let (length, protection) = if existing {
            let mut header = vec![0u8; 24 + record.len()];
            backing.read_exact(&mut header).map_err(|e| e.to_string())?;
            if &header[..8] != b"ALKYSEC1"
                || u32::from_le_bytes(header[20..24].try_into().unwrap()) as usize != record.len()
                || header[24..] != record
            {
                return Err("section identity collision or malformed record".into());
            }
            let length = u64::from_le_bytes(header[8..16].try_into().unwrap());
            let protection = u32::from_le_bytes(header[16..20].try_into().unwrap());
            if length == 0
                || length > 256 * 1024 * 1024
                || !matches!(protection, 2 | 4)
                || backing.metadata().map_err(|e| e.to_string())?.len() != HEADER as u64 + length
            {
                return Err("live section size/protection record invalid".into());
            }
            (length as usize, protection)
        } else {
            // A stale backing is not a live Windows object. Reset all data on creation.
            backing.set_len(0).map_err(|e| e.to_string())?;
            backing
                .set_len((HEADER + size) as u64)
                .map_err(|e| e.to_string())?;
            let mut header = b"ALKYSEC1".to_vec();
            header.extend((size as u64).to_le_bytes());
            header.extend(protection.to_le_bytes());
            header.extend((record.len() as u32).to_le_bytes());
            header.extend(record);
            backing
                .seek(SeekFrom::Start(0))
                .map_err(|e| e.to_string())?;
            backing.write_all(&header).map_err(|e| e.to_string())?;
            (size, protection)
        };
        lease.lock_shared().map_err(|e| e.to_string())?;
        Ok(Some((
            Section {
                _lease: Some(Lease(lease)),
                file: backing,
                length,
                protection,
            },
            existing,
        )))
    }
    fn token(&mut self, section: Section, write: bool) -> Result<u32> {
        if self.handles.len() >= 256 {
            return Err("section handle budget exceeded".into());
        }
        let h = alkyhol::kernel::reserve_handle_token()
            .map_err(|e| format!("section handle allocation: {e:?}"))?;
        let h = u32::try_from(h).map_err(|_| "section handle outside Win32")?;
        self.handles.insert(h, (Rc::new(section), write));
        Ok(h)
    }
}
fn fail(code: u32) -> u32 {
    alkyhol::ffi::SetLastError(code);
    0
}
fn name(image: &LoadedImage32<'_>, address: u32, ansi: bool) -> Vec<u16> {
    if ansi {
        bulk32::ascii(image, address).encode_utf16().collect()
    } else {
        let mut u = startup32::read_units::<u16>(image, address, -1)
            .unwrap_or_else(|| unsupported("section name outside checked storage"));
        u.pop();
        u
    }
}
pub(super) fn create(image: &LoadedImage32<'_>, a: &[u32; 8], ansi: bool) -> u32 {
    let name = if a[5] == 0 {
        Vec::new()
    } else {
        name(image, a[5], ansi)
    };
    eprintln!(
        "PE32 shared section create: name={:?}; file={:x}; size={:x}:{:x}; protection={:x}",
        String::from_utf16_lossy(&name),
        a[0],
        a[3],
        a[4],
        a[2]
    );
    if a[0] != u32::MAX || a[1] != 0 || a[2] & !0x08000006 != 0 || !matches!(a[2] & 0xff, 2 | 4) {
        return fail(50);
    }
    if a[3] != 0 || a[4] == 0 || a[4] > 256 * 1024 * 1024 {
        return fail(87);
    }
    if !name.is_empty() && super::mutex32::name(&name).is_err() {
        return fail(50);
    }
    let mut state = image.cpu.sections.borrow_mut();
    let (section, existed) = if name.is_empty() {
        (
            state
                .anonymous(a[4] as usize, a[2] & 0xff)
                .unwrap_or_else(|e| unsupported(&e)),
            false,
        )
    } else {
        state
            .named(&name, a[4] as usize, a[2] & 0xff, false)
            .unwrap_or_else(|e| unsupported(&e))
            .expect("create returns backing")
    };
    let write = section.protection == 4;
    let h = state
        .token(section, write)
        .unwrap_or_else(|e| unsupported(&e));
    alkyhol::ffi::SetLastError(if existed { 183 } else { 0 });
    h
}
pub(super) fn open(image: &LoadedImage32<'_>, a: &[u32; 8], ansi: bool) -> u32 {
    if a[1] != 0 {
        return fail(50);
    }
    let write = match a[0] {
        4 => false,
        2 | 6 | 0xf001f => true,
        _ => return fail(50),
    };
    let name = name(image, a[2], ansi);
    if super::mutex32::name(&name).is_err() {
        return fail(50);
    }
    let mut state = image.cpu.sections.borrow_mut();
    let Some((section, _)) = state
        .named(&name, 0, 0, true)
        .unwrap_or_else(|e| unsupported(&e))
    else {
        return fail(2);
    };
    if write && section.protection != 4 {
        return fail(5);
    }
    state
        .token(section, write)
        .unwrap_or_else(|e| unsupported(&e))
}
pub(super) fn map(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    let (write, private) = match a[1] {
        4 => (false, false),
        2 | 6 | 0xf001f => (true, false),
        1 => (true, true),
        _ => return fail(50),
    };
    let mut state = image.cpu.sections.borrow_mut();
    let Some((section, granted)) = state.handles.get(&a[0]).cloned() else {
        return fail(6);
    };
    if write && !private && !granted {
        return fail(5);
    }
    if a[2] != 0 || !a[3].is_multiple_of(65536) {
        return fail(1132);
    }
    let offset = a[3] as usize;
    let Some(remaining) = section.length.checked_sub(offset).filter(|n| *n > 0) else {
        return fail(87);
    };
    let length = if a[4] == 0 { remaining } else { a[4] as usize };
    if length > remaining || length == 0 {
        return fail(87);
    }
    if state.views.len() >= 256 {
        return fail(8);
    }
    let size = length.div_ceil(PAGE) * PAGE;
    let mapping = image
        .cpu
        .mapping(size, 65536, 0)
        .unwrap_or_else(|e| unsupported(&e));
    let protection = if write { 3 } else { 1 };
    let pointer = unsafe {
        super::mmap(
            mapping.base.cast(),
            size,
            protection,
            0x10 | if private { 2 } else { 1 },
            section.file.as_raw_fd(),
            (HEADER + offset) as i64,
        )
    };
    if pointer as isize == -1 {
        return fail(8);
    }
    if pointer != mapping.base.cast::<c_void>() {
        unsupported("fixed shared mapping moved outside its owned reservation");
    }
    let base = mapping.base as u32;
    state.views.insert(
        base,
        View {
            mapping,
            _section: section,
            write,
        },
    );
    base
}
pub(super) fn unmap(image: &LoadedImage32<'_>, base: u32) -> u32 {
    if image
        .cpu
        .sections
        .borrow_mut()
        .views
        .remove(&base)
        .is_none()
    {
        return fail(487);
    }
    1
}
