//! Bounded, thread-scoped process services for trusted PE code. Kernel handles
//! share a unique token allocator; descriptors/searches/mappings have real owners.
#![allow(non_snake_case)]
use crate::{Result, windows_command_line};
use alkyhol::{ffi, kernel::reserve_handle_token};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    ffi::c_void,
    fs::{File, Metadata},
    io::{Read, Write},
    os::fd::{AsRawFd, FromRawFd},
    path::Path,
    ptr,
    time::{SystemTime, UNIX_EPOCH},
};

unsafe extern "C" {
    fn dup(fd: i32) -> i32;
    fn isatty(fd: i32) -> i32;
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
}
const INVALID: usize = usize::MAX;
const INVALID_HANDLE: u32 = 6;
const NOT_SUPPORTED: u32 = 50;
const INVALID_PARAMETER: u32 = 87;
const INSUFFICIENT_BUFFER: u32 = 122;

thread_local! { static PROCESS: RefCell<Option<State>> = const { RefCell::new(None) }; }
struct Stream {
    file: File,
    input: bool,
}
enum Object {
    Stream(Stream),
    Search(VecDeque<FindData>),
}
struct Mapping {
    address: usize,
    len: usize,
}
impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            munmap(self.address as *mut c_void, self.len);
        }
    }
}
struct State {
    command: Vec<u16>,
    executable: Vec<u16>,
    environment: BTreeMap<String, Vec<u16>>,
    standard: [usize; 3],
    objects: BTreeMap<usize, Object>,
    mappings: Vec<Mapping>,
}
/// Installs a single-thread process view until drop. Guest code may not outlive
/// this guard. This does not install a Windows PEB/TEB, static TLS or DLL runtime.
pub struct ProcessContext {
    _thread_bound: std::marker::PhantomData<*mut ()>,
}
impl ProcessContext {
    pub fn install(argv: &[String], environment: &[(String, String)]) -> Result<Self> {
        let command = windows_command_line(argv)?;
        let executable = argv[0].encode_utf16().collect();
        let mut env = BTreeMap::new();
        for (key, value) in environment {
            if key.is_empty() || key.contains(['=', '\0']) || value.contains('\0') {
                return Err("invalid process environment entry".into());
            }
            let key = key.to_ascii_uppercase();
            if env.insert(key, value.encode_utf16().collect()).is_some() {
                return Err("duplicate case-insensitive environment key".into());
            }
        }
        let mut state = State {
            command,
            executable,
            environment: env,
            standard: [0; 3],
            objects: BTreeMap::new(),
            mappings: Vec::new(),
        };
        for i in 0..3 {
            let fd = unsafe { dup(i) };
            if fd < 0 {
                return Err(format!(
                    "duplicate inherited stream {i}: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let file = unsafe { File::from_raw_fd(fd) };
            let token = reserve_handle_token().map_err(|e| format!("handle allocation: {e:?}"))?;
            state.standard[i as usize] = token;
            state.objects.insert(
                token,
                Object::Stream(Stream {
                    file,
                    input: i == 0,
                }),
            );
        }
        PROCESS.with(|p| {
            let mut slot = p.borrow_mut();
            if slot.is_some() {
                return Err("a PE process context is already installed on this thread".into());
            }
            *slot = Some(state);
            Ok(Self {
                _thread_bound: std::marker::PhantomData,
            })
        })
    }
}
impl Drop for ProcessContext {
    fn drop(&mut self) {
        PROCESS.with(|p| {
            p.borrow_mut().take();
        });
    }
}
fn fail(error: u32) -> i32 {
    ffi::SetLastError(error);
    0
}
unsafe fn wide(ptr: *const u16) -> std::result::Result<String, u32> {
    if ptr.is_null() {
        return Err(INVALID_PARAMETER);
    }
    for i in 0..32768 {
        if unsafe { *ptr.add(i) } == 0 {
            return String::from_utf16(unsafe { std::slice::from_raw_parts(ptr, i) })
                .map_err(|_| 1113);
        }
    }
    Err(INVALID_PARAMETER)
}
fn io_error(error: &std::io::Error) -> u32 {
    match error.kind() {
        std::io::ErrorKind::NotFound => 2,
        std::io::ErrorKind::PermissionDenied => 5,
        std::io::ErrorKind::BrokenPipe => 109,
        _ => 31,
    }
}
pub(crate) unsafe fn create_file(
    name: *const u16,
    access: u32,
    share: u32,
    security: *const c_void,
    disposition: u32,
    flags: u32,
    template: usize,
) -> usize {
    let path = match unsafe { wide(name) } {
        Ok(s) => s,
        Err(e) => {
            fail(e);
            return INVALID;
        }
    };
    if path.starts_with("\\\\") || path.starts_with("//") || path.as_bytes().get(1) == Some(&b':') {
        fail(NOT_SUPPORTED);
        return INVALID;
    }
    let name: Vec<u16> = path.replace('\\', "/").encode_utf16().chain([0]).collect();
    unsafe {
        ffi::CreateFileW(
            name.as_ptr(),
            access,
            share,
            security,
            disposition,
            flags,
            crate::abi::kernel_handle(template),
        )
    }
}

pub(crate) fn close_stream(handle: usize) -> Option<i32> {
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let state = state.as_mut()?;
        if matches!(state.objects.get(&handle), Some(Object::Stream(_))) {
            state.objects.remove(&handle);
            Some(1)
        } else {
            None
        }
    })
}
unsafe extern "win64" fn GetStdHandle(kind: u32) -> usize {
    let index = match kind {
        0xfffffff6 => 0,
        0xfffffff5 => 1,
        0xfffffff4 => 2,
        _ => {
            fail(INVALID_PARAMETER);
            return INVALID;
        }
    };
    PROCESS.with(|p| {
        p.borrow().as_ref().map_or_else(
            || {
                fail(NOT_SUPPORTED);
                INVALID
            },
            |p| p.standard[index],
        )
    })
}
unsafe extern "win64" fn GetConsoleMode(handle: usize, mode: *mut u32) -> i32 {
    if mode.is_null() {
        return fail(INVALID_PARAMETER);
    }
    PROCESS.with(|p| {
        let state = p.borrow();
        let Some(Object::Stream(stream)) = state.as_ref().and_then(|s| s.objects.get(&handle))
        else {
            return fail(INVALID_HANDLE);
        };
        if unsafe { isatty(stream.file.as_raw_fd()) } != 1 {
            return fail(INVALID_HANDLE);
        }
        unsafe {
            *mode = if stream.input { 7 } else { 3 };
        }
        1
    })
}
unsafe extern "win64" fn GetCommandLineW() -> *const u16 {
    PROCESS.with(|p| {
        p.borrow().as_ref().map_or_else(
            || {
                fail(NOT_SUPPORTED);
                ptr::null()
            },
            |s| s.command.as_ptr(),
        )
    })
}
unsafe extern "win64" fn GetEnvironmentVariableW(
    name: *const u16,
    buffer: *mut u16,
    capacity: u32,
) -> u32 {
    let name = match unsafe { wide(name) } {
        Ok(n) if !n.is_empty() => n.to_ascii_uppercase(),
        Ok(_) => {
            fail(203);
            return 0;
        }
        Err(e) => {
            fail(e);
            return 0;
        }
    };
    if !name.is_ascii() {
        fail(NOT_SUPPORTED);
        return 0;
    }
    PROCESS.with(|p| {
        let state = p.borrow();
        let Some(state) = state.as_ref() else {
            fail(NOT_SUPPORTED);
            return 0;
        };
        let Some(value) = state.environment.get(&name) else {
            fail(203);
            return 0;
        };
        if capacity as usize <= value.len() {
            return (value.len() + 1) as u32;
        }
        if buffer.is_null() {
            fail(INVALID_PARAMETER);
            return 0;
        }
        unsafe {
            ptr::copy_nonoverlapping(value.as_ptr(), buffer, value.len());
            *buffer.add(value.len()) = 0;
        }
        value.len() as u32
    })
}
unsafe extern "win64" fn GetModuleFileNameW(module: usize, buffer: *mut u16, capacity: u32) -> u32 {
    if module != 0 {
        fail(NOT_SUPPORTED);
        return 0;
    }
    if capacity == 0 {
        fail(INSUFFICIENT_BUFFER);
        return 0;
    }
    if buffer.is_null() {
        fail(INVALID_PARAMETER);
        return 0;
    }
    PROCESS.with(|p| {
        let state = p.borrow();
        let Some(state) = state.as_ref() else {
            fail(NOT_SUPPORTED);
            return 0;
        };
        let len = state.executable.len().min(capacity as usize - 1);
        unsafe {
            ptr::copy_nonoverlapping(state.executable.as_ptr(), buffer, len);
            *buffer.add(len) = 0;
        }
        if len < state.executable.len() {
            fail(INSUFFICIENT_BUFFER);
            capacity
        } else {
            len as u32
        }
    })
}
unsafe extern "win64" fn VirtualAlloc(
    address: *mut c_void,
    size: usize,
    allocation: u32,
    protection: u32,
) -> *mut c_void {
    if !address.is_null() || allocation != 0x3000 || protection != 4 {
        fail(NOT_SUPPORTED);
        return ptr::null_mut();
    }
    if size == 0 {
        fail(INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let Some(length) = size
        .checked_add(4095)
        .map(|s| s & !4095)
        .filter(|s| *s <= 512 * 1024 * 1024)
    else {
        fail(8);
        return ptr::null_mut();
    };
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let Some(state) = state.as_mut() else {
            fail(NOT_SUPPORTED);
            return ptr::null_mut();
        };
        let reserve = length + 65536;
        let raw = unsafe { mmap(ptr::null_mut(), reserve, 3, 2 | 0x1000, -1, 0) };
        if raw as isize == -1 {
            fail(8);
            return ptr::null_mut();
        }
        let start = (raw as usize + 65535) & !65535;
        let prefix = start - raw as usize;
        let suffix = reserve - prefix - length;
        unsafe {
            if prefix != 0 {
                munmap(raw, prefix);
            }
            if suffix != 0 {
                munmap((start + length) as *mut c_void, suffix);
            }
        }
        state.mappings.push(Mapping {
            address: start,
            len: length,
        });
        start as *mut c_void
    })
}
unsafe extern "win64" fn WriteConsoleW(
    handle: usize,
    buffer: *const u16,
    count: u32,
    written: *mut u32,
    reserved: *mut c_void,
) -> i32 {
    if !written.is_null() {
        unsafe {
            *written = 0;
        }
    }
    if !reserved.is_null() {
        return fail(NOT_SUPPORTED);
    }
    if written.is_null() || (count != 0 && buffer.is_null()) {
        return fail(INVALID_PARAMETER);
    }
    let mut mode = 0;
    if unsafe { GetConsoleMode(handle, &mut mode) } == 0 {
        return 0;
    }
    let units = if count == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(buffer, count as usize) }
    };
    let text = match String::from_utf16(units) {
        Ok(s) => s,
        Err(_) => return fail(1113),
    };
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let Some(Object::Stream(stream)) = state.as_mut().and_then(|s| s.objects.get_mut(&handle))
        else {
            return fail(INVALID_HANDLE);
        };
        if stream.input {
            return fail(5);
        }
        match stream.file.write_all(text.as_bytes()) {
            Ok(()) => {
                unsafe {
                    *written = count;
                }
                1
            }
            Err(e) => fail(io_error(&e)),
        }
    })
}
pub(crate) unsafe fn stream_io(
    handle: usize,
    buffer: *mut c_void,
    count: u32,
    transferred: *mut u32,
    overlapped: *mut c_void,
    write: bool,
) -> Option<i32> {
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let Object::Stream(stream) = state.as_mut()?.objects.get_mut(&handle)? else {
            return None;
        };
        if !transferred.is_null() {
            unsafe {
                *transferred = 0;
            }
        }
        if !overlapped.is_null() {
            return Some(fail(NOT_SUPPORTED));
        }
        if transferred.is_null() || (count != 0 && buffer.is_null()) {
            return Some(fail(INVALID_PARAMETER));
        }
        if write == stream.input {
            return Some(fail(5));
        }
        let result = if write {
            let bytes = if count == 0 {
                &[]
            } else {
                unsafe { std::slice::from_raw_parts(buffer.cast::<u8>(), count as usize) }
            };
            stream.file.write(bytes)
        } else {
            let bytes = if count == 0 {
                &mut []
            } else {
                unsafe { std::slice::from_raw_parts_mut(buffer.cast::<u8>(), count as usize) }
            };
            stream.file.read(bytes)
        };
        Some(match result {
            Ok(n) => {
                unsafe {
                    *transferred = n as u32;
                }
                1
            }
            Err(e) => fail(io_error(&e)),
        })
    })
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FindData {
    attributes: u32,
    creation: [u32; 2],
    access: [u32; 2],
    write: [u32; 2],
    size_high: u32,
    size_low: u32,
    reserved: [u32; 2],
    name: [u16; 260],
    alternate: [u16; 14],
}
fn file_time(time: SystemTime) -> std::result::Result<[u32; 2], u32> {
    let ns = match time.duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_nanos() as i128,
        Err(e) => -(e.duration().as_nanos() as i128),
    };
    let ticks = ns.div_euclid(100) + 116444736000000000i128;
    let ticks = u64::try_from(ticks).map_err(|_| NOT_SUPPORTED)?;
    Ok([ticks as u32, (ticks >> 32) as u32])
}
impl FindData {
    fn from_metadata(name: &str, meta: &Metadata) -> std::result::Result<Self, u32> {
        if meta.file_type().is_symlink() || !(meta.is_file() || meta.is_dir()) {
            return Err(NOT_SUPPORTED);
        }
        let units: Vec<_> = name.encode_utf16().collect();
        if units.len() >= 260 {
            return Err(206);
        }
        let mut out = Self {
            attributes: if meta.is_dir() { 0x10 } else { 0x80 },
            creation: file_time(meta.created().map_err(|e| io_error(&e))?)?,
            access: file_time(meta.accessed().map_err(|e| io_error(&e))?)?,
            write: file_time(meta.modified().map_err(|e| io_error(&e))?)?,
            size_high: if meta.is_dir() {
                0
            } else {
                (meta.len() >> 32) as u32
            },
            size_low: if meta.is_dir() { 0 } else { meta.len() as u32 },
            reserved: [0; 2],
            name: [0; 260],
            alternate: [0; 14],
        };
        if meta.permissions().readonly() {
            out.attributes = (out.attributes & !0x80) | 1;
        }
        out.name[..units.len()].copy_from_slice(&units);
        Ok(out)
    }
}
fn matches(pattern: &str, name: &str) -> bool {
    let pattern = if pattern == "*.*" { "*" } else { pattern };
    let p: Vec<_> = pattern.to_ascii_uppercase().encode_utf16().collect();
    let n: Vec<_> = name.to_ascii_uppercase().encode_utf16().collect();
    let mut row = vec![false; n.len() + 1];
    row[0] = true;
    for ch in p {
        if ch == b'*' as u16 {
            for i in 1..row.len() {
                row[i] |= row[i - 1];
            }
        } else {
            for i in (1..row.len()).rev() {
                row[i] = row[i - 1] && (ch == b'?' as u16 || ch == n[i - 1]);
            }
            row[0] = false;
        }
    }
    row[n.len()]
}
fn listing(pattern: &str) -> std::result::Result<VecDeque<FindData>, u32> {
    let normalized = pattern.replace('\\', "/");
    if normalized.as_bytes().get(1) == Some(&b':') || normalized.starts_with("//") {
        return Err(NOT_SUPPORTED);
    }
    let path = Path::new(&normalized);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(INVALID_PARAMETER)?;
    // Windows DOS wildcard edge cases and wildcard directory components are
    // separate; fail those rather than approximate their matching rules.
    if !name.is_ascii()
        || name.contains(['<', '>', '"', '?'])
        || name.ends_with('.')
        || (name.contains('*')
            && name != "*"
            && name != "*.*"
            && !(name.starts_with("*.") && !name[2..].contains('*')))
    {
        return Err(NOT_SUPPORTED);
    }
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    if parent.to_string_lossy().contains(['*', '?']) {
        return Err(NOT_SUPPORTED);
    }
    let mut rows = VecDeque::new();
    for special in [".", ".."] {
        if matches(name, special) {
            rows.push_back(FindData::from_metadata(
                special,
                &std::fs::metadata(parent.join(special)).map_err(|e| io_error(&e))?,
            )?);
        }
    }
    for entry in std::fs::read_dir(parent).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            3
        } else {
            io_error(&e)
        }
    })? {
        let entry = entry.map_err(|e| io_error(&e))?;
        let filename = entry.file_name().into_string().map_err(|_| 1113u32)?;
        if matches(name, &filename) {
            rows.push_back(FindData::from_metadata(
                &filename,
                &std::fs::symlink_metadata(entry.path()).map_err(|e| io_error(&e))?,
            )?);
        }
    }
    if rows.is_empty() { Err(2) } else { Ok(rows) }
}
unsafe extern "win64" fn FindFirstFileW(pattern: *const u16, out: *mut FindData) -> usize {
    if out.is_null() {
        fail(INVALID_PARAMETER);
        return INVALID;
    }
    let pattern = match unsafe { wide(pattern) } {
        Ok(s) => s,
        Err(e) => {
            fail(e);
            return INVALID;
        }
    };
    let mut entries = match listing(&pattern) {
        Ok(v) => v,
        Err(e) => {
            fail(e);
            return INVALID;
        }
    };
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let Some(state) = state.as_mut() else {
            fail(NOT_SUPPORTED);
            return INVALID;
        };
        let token = match reserve_handle_token() {
            Ok(h) => h,
            Err(_) => {
                fail(8);
                return INVALID;
            }
        };
        unsafe {
            ptr::write(out, entries.pop_front().unwrap());
        }
        state.objects.insert(token, Object::Search(entries));
        token
    })
}
unsafe extern "win64" fn FindNextFileW(handle: usize, out: *mut FindData) -> i32 {
    if out.is_null() {
        return fail(INVALID_PARAMETER);
    }
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let Some(Object::Search(entries)) = state.as_mut().and_then(|s| s.objects.get_mut(&handle))
        else {
            return fail(INVALID_HANDLE);
        };
        match entries.pop_front() {
            Some(record) => {
                unsafe {
                    ptr::write(out, record);
                }
                1
            }
            None => fail(18),
        }
    })
}
unsafe extern "win64" fn FindClose(handle: usize) -> i32 {
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let Some(state) = state.as_mut() else {
            return fail(INVALID_HANDLE);
        };
        if !matches!(state.objects.get(&handle), Some(Object::Search(_))) {
            return fail(INVALID_HANDLE);
        }
        state.objects.remove(&handle);
        1
    })
}
pub(crate) fn address(name: &str) -> Option<usize> {
    match name {
        "GetStdHandle" => Some(GetStdHandle as *const () as usize),
        "GetConsoleMode" => Some(GetConsoleMode as *const () as usize),
        "GetCommandLineW" => Some(GetCommandLineW as *const () as usize),
        "GetEnvironmentVariableW" => Some(GetEnvironmentVariableW as *const () as usize),
        "GetModuleFileNameW" => Some(GetModuleFileNameW as *const () as usize),
        "VirtualAlloc" => Some(VirtualAlloc as *const () as usize),
        "WriteConsoleW" => Some(WriteConsoleW as *const () as usize),
        "FindFirstFileW" => Some(FindFirstFileW as *const () as usize),
        "FindNextFileW" => Some(FindNextFileW as *const () as usize),
        "FindClose" => Some(FindClose as *const () as usize),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn context() -> ProcessContext {
        ProcessContext::install(
            &["C:\\test app\\pkg.exe".into(), "λ🌸".into()],
            &[
                ("PKG_CONFIG_PATH".into(), "pc/λ🌸".into()),
                ("EMPTY".into(), "".into()),
            ],
        )
        .unwrap()
    }
    fn w(s: &str) -> Vec<u16> {
        s.encode_utf16().chain([0]).collect()
    }
    #[test]
    fn command_environment_and_module_buffers_obey_windows_size_rules() {
        let _guard = context();
        unsafe {
            assert_eq!(
                wide(GetCommandLineW()).unwrap(),
                "\"C:\\test app\\pkg.exe\" \"λ🌸\""
            );
            let name = w("pkg_config_path");
            let value = w("pc/λ🌸");
            let mut out = [0x5555u16; 64];
            ffi::SetLastError(0x1234);
            assert_eq!(
                GetEnvironmentVariableW(name.as_ptr(), ptr::null_mut(), 0),
                value.len() as u32
            );
            assert_eq!(
                GetEnvironmentVariableW(name.as_ptr(), out.as_mut_ptr(), 64),
                value.len() as u32 - 1
            );
            assert_eq!(&out[..value.len()], &value);
            assert_eq!(out[value.len()], 0x5555);
            assert_eq!(ffi::GetLastError(), 0x1234);
            assert_eq!(
                GetEnvironmentVariableW(w("EMPTY").as_ptr(), ptr::null_mut(), 0),
                1
            );
            assert_eq!(
                GetEnvironmentVariableW(w("EMPTY").as_ptr(), out.as_mut_ptr(), 64),
                0
            );
            assert_eq!(out[0], 0);
            assert_eq!(
                GetEnvironmentVariableW(w("missing").as_ptr(), out.as_mut_ptr(), 64),
                0
            );
            assert_eq!(ffi::GetLastError(), 203);
            let executable = w("C:\\test app\\pkg.exe");
            assert_eq!(
                GetModuleFileNameW(0, out.as_mut_ptr(), 64),
                executable.len() as u32 - 1
            );
            assert_eq!(&out[..executable.len()], &executable);
            out.fill(0x5555);
            assert_eq!(GetModuleFileNameW(0, out.as_mut_ptr(), 4), 4);
            assert_eq!(
                &out[..5],
                &[b'C' as u16, b':' as u16, b'\\' as u16, 0, 0x5555]
            );
            assert_eq!(ffi::GetLastError(), 122);
            assert_eq!(GetModuleFileNameW(0, out.as_mut_ptr(), 0), 0);
            assert_eq!(ffi::GetLastError(), 122);
            assert_eq!(GetModuleFileNameW(1, out.as_mut_ptr(), 64), 0);
            assert_eq!(ffi::GetLastError(), 50);
            assert_eq!(
                GetEnvironmentVariableW(ptr::null(), out.as_mut_ptr(), 64),
                0
            );
            assert_eq!(ffi::GetLastError(), 87);
            assert_eq!(
                GetEnvironmentVariableW(w("PKG_CONFIG_PATH").as_ptr(), ptr::null_mut(), 64),
                0
            );
            assert_eq!(ffi::GetLastError(), 87);
            let invalid = [0xd800, 0];
            assert_eq!(
                GetEnvironmentVariableW(invalid.as_ptr(), out.as_mut_ptr(), 64),
                0
            );
            assert_eq!(ffi::GetLastError(), 1113);
        }
    }
    #[test]
    fn context_lifetime_nested_install_and_environment_identity_are_explicit() {
        let guard = context();
        assert!(ProcessContext::install(&["other.exe".into()], &[]).is_err());
        assert!(
            ProcessContext::install(
                &["other.exe".into()],
                &[("x".into(), "1".into()), ("X".into(), "2".into())]
            )
            .is_err()
        );
        assert!(
            ProcessContext::install(&["other.exe".into()], &[("bad=key".into(), "1".into())])
                .is_err()
        );
        drop(guard);
        assert!(unsafe { GetCommandLineW() }.is_null());
        assert_eq!(ffi::GetLastError(), 50);
        assert!(ProcessContext::install(&["new.exe".into()], &[]).is_ok());
    }
    #[test]
    fn virtual_alloc_owns_aligned_zeroed_disjoint_rw_storage_and_rejects_other_modes() {
        let _guard = context();
        unsafe {
            ffi::SetLastError(0x1234);
            let a = VirtualAlloc(ptr::null_mut(), 65538, 0x3000, 4).cast::<u8>();
            let b = VirtualAlloc(ptr::null_mut(), 1, 0x3000, 4).cast::<u8>();
            assert!(!a.is_null() && !b.is_null());
            assert_eq!(a as usize % 65536, 0);
            assert_eq!(b as usize % 65536, 0);
            assert_ne!(a, b);
            assert_eq!((*a, *a.add(65537), *b), (0, 0, 0));
            *a.add(65537) = 0x5a;
            assert_eq!(*a.add(65537), 0x5a);
            assert_eq!(ffi::GetLastError(), 0x1234);
            for (size, flags, protection, error) in [
                (0, 0x3000, 4, 87),
                (1, 0x1000, 4, 50),
                (1, 0x3000, 0x40, 50),
                (usize::MAX, 0x3000, 4, 8),
            ] {
                assert!(VirtualAlloc(ptr::null_mut(), size, flags, protection).is_null());
                assert_eq!(ffi::GetLastError(), error);
            }
            assert!(VirtualAlloc(a.cast(), 1, 0x3000, 4).is_null());
            assert_eq!(ffi::GetLastError(), 50);
        }
        PROCESS.with(|p| assert_eq!(p.borrow().as_ref().unwrap().mappings.len(), 2));
    }
    #[test]
    fn streams_have_real_io_rights_and_distinct_lifetimes_from_search_and_kernel_handles() {
        let _guard = context();
        let root = std::env::temp_dir().join(format!(
            "alkyhol-pe-stream-{}",
            reserve_handle_token().unwrap()
        ));
        let token = reserve_handle_token().unwrap();
        PROCESS.with(|p| {
            p.borrow_mut().as_mut().unwrap().objects.insert(
                token,
                Object::Stream(Stream {
                    file: File::create(&root).unwrap(),
                    input: false,
                }),
            );
        });
        let bytes = [0, 0xff, 42];
        let mut n = 999;
        unsafe {
            assert_eq!(
                stream_io(
                    token,
                    bytes.as_ptr().cast_mut().cast(),
                    3,
                    &mut n,
                    ptr::null_mut(),
                    true
                ),
                Some(1)
            );
            assert_eq!(n, 3);
            assert_eq!(std::fs::read(&root).unwrap(), bytes);
            let mut mode = 0x12345678;
            assert_eq!(GetConsoleMode(token, &mut mode), 0);
            assert_eq!(mode, 0x12345678);
            assert_eq!(ffi::GetLastError(), 6);
            n = 999;
            assert_eq!(
                WriteConsoleW(token, w("λ🌸").as_ptr(), 3, &mut n, ptr::null_mut()),
                0
            );
            assert_eq!((n, ffi::GetLastError()), (0, 6));
            assert_eq!(
                stream_io(token, ptr::null_mut(), 1, &mut n, ptr::null_mut(), true),
                Some(0)
            );
            assert_eq!((n, ffi::GetLastError()), (0, 87));
            assert_eq!(
                stream_io(token, ptr::null_mut(), 0, &mut n, ptr::null_mut(), false),
                Some(0)
            );
            assert_eq!(ffi::GetLastError(), 5);
            assert_eq!(
                stream_io(
                    token,
                    ptr::null_mut(),
                    0,
                    &mut n,
                    std::ptr::dangling_mut::<c_void>(),
                    true
                ),
                Some(0)
            );
            assert_eq!(ffi::GetLastError(), 50);
            assert_eq!(
                GetStdHandle(0xfffffff5),
                PROCESS.with(|p| p.borrow().as_ref().unwrap().standard[1])
            );
            assert_eq!(GetStdHandle(0), usize::MAX);
            assert_eq!(ffi::GetLastError(), 87);
            assert_eq!(FindClose(token), 0);
            assert_eq!(ffi::GetLastError(), 6);
        }
        assert_eq!(close_stream(token), Some(1));
        assert_eq!(close_stream(token), None);
        std::fs::remove_file(root).unwrap();
    }
    #[test]
    fn directory_search_preserves_layout_names_sizes_types_and_exhaustion() {
        assert_eq!(std::mem::size_of::<FindData>(), 592);
        assert_eq!(std::mem::offset_of!(FindData, name), 44);
        let _guard = context();
        let root = std::env::temp_dir().join(format!(
            "alkyhol-pe-find-{}",
            reserve_handle_token().unwrap()
        ));
        std::fs::create_dir(&root).unwrap();
        for name in ["a.pc", "B.PC", "λ🌸.pc", "ignore.txt"] {
            std::fs::write(root.join(name), "λ🌸").unwrap();
        }
        std::fs::create_dir(root.join("subdir.pc")).unwrap();
        let pattern = w(&format!("{}\\*.pc", root.display()));
        let mut out = std::mem::MaybeUninit::<FindData>::uninit();
        unsafe {
            let h = FindFirstFileW(pattern.as_ptr(), out.as_mut_ptr());
            assert_ne!(h, INVALID);
            let mut found = Vec::new();
            loop {
                let r = out.assume_init();
                let n = r.name.iter().position(|v| *v == 0).unwrap();
                let name = String::from_utf16(&r.name[..n]).unwrap();
                if name == "subdir.pc" {
                    assert_ne!(r.attributes & 16, 0);
                    assert_eq!(r.size_low, 0);
                } else {
                    assert_eq!(r.attributes & 16, 0);
                    assert_eq!((r.size_high, r.size_low), (0, 6));
                }
                found.push(name);
                if FindNextFileW(h, out.as_mut_ptr()) == 0 {
                    break;
                }
            }
            found.sort();
            assert_eq!(found, ["B.PC", "a.pc", "subdir.pc", "λ🌸.pc"]);
            assert_eq!(ffi::GetLastError(), 18);
            assert_eq!(FindClose(h), 1);
            assert_eq!(FindClose(h), 0);
            assert_eq!(ffi::GetLastError(), 6);
            assert_eq!(FindNextFileW(h, out.as_mut_ptr()), 0);
            assert_eq!(ffi::GetLastError(), 6);
            assert_eq!(
                FindFirstFileW(
                    w(&format!("{}/*.absent", root.display())).as_ptr(),
                    out.as_mut_ptr()
                ),
                INVALID
            );
            assert_eq!(ffi::GetLastError(), 2);
            assert_eq!(FindFirstFileW(pattern.as_ptr(), ptr::null_mut()), INVALID);
            assert_eq!(ffi::GetLastError(), 87);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn supported_matching_and_filetime_boundaries_are_explicit() {
        assert!(matches("*.pc", "NAME.PC"));
        assert!(!matches("*.pc", "pc"));
        assert!(matches("*.*", "README"));
        assert_eq!(file_time(UNIX_EPOCH).unwrap(), [0xd53e8000, 0x019db1de]);
        let ticks = file_time(UNIX_EPOCH - std::time::Duration::from_nanos(1)).unwrap();
        assert_eq!(
            (ticks[1] as u64) << 32 | ticks[0] as u64,
            116444735999999999
        );
        for glob in [
            "a?.pc",
            "prefix*",
            "foo.*",
            "C:\\*.pc",
            "//host/share/*.pc",
            "foo/*/a.pc",
        ] {
            assert!(listing(glob).is_err(), "{glob}");
        }
    }
    #[test]
    fn file_bridge_handles_separators_and_undefined_high_template_bits() {
        let root = std::env::temp_dir().join(format!(
            "alkyhol-pe-path-{}",
            reserve_handle_token().unwrap()
        ));
        std::fs::create_dir(&root).unwrap();
        let file = root.join("λ🌸.bin");
        std::fs::write(&file, b"abc").unwrap();
        let name = w(&file.to_str().unwrap().replace('/', "\\"));
        unsafe {
            let handle = create_file(
                name.as_ptr(),
                0x80000000,
                7,
                ptr::null(),
                3,
                128,
                0x1234567800000000,
            );
            assert_ne!(handle, INVALID);
            let tagged = handle | 0x1234567800000000;
            let read: unsafe extern "win64" fn(
                usize,
                *mut c_void,
                u32,
                *mut u32,
                *mut c_void,
            ) -> i32 =
                std::mem::transmute(crate::abi::resolve("kernel32.dll", "ReadFile").unwrap());
            let size: unsafe extern "win64" fn(usize, *mut i64) -> i32 =
                std::mem::transmute(crate::abi::resolve("kernel32.dll", "GetFileSizeEx").unwrap());
            let close: unsafe extern "win64" fn(usize) -> i32 =
                std::mem::transmute(crate::abi::resolve("kernel32.dll", "CloseHandle").unwrap());
            let mut bytes = [0u8; 4];
            let mut n = 0;
            let mut len = 0;
            assert_eq!(
                read(
                    tagged,
                    bytes.as_mut_ptr().cast(),
                    4,
                    &mut n,
                    ptr::null_mut()
                ),
                1
            );
            assert_eq!(n, 3);
            assert_eq!(&bytes, b"abc\0");
            assert_eq!(size(tagged, &mut len), 1);
            assert_eq!(len, 3);
            assert_eq!(close(tagged), 1);
            for path in [r"C:\foreign\file", r"\\server\share"] {
                assert_eq!(
                    create_file(w(path).as_ptr(), 0x80000000, 7, ptr::null(), 3, 128, 0),
                    INVALID
                );
                assert_eq!(ffi::GetLastError(), 50);
            }
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn console_bridge_writes_actual_utf8_to_a_terminal_and_counts_utf16_units() {
        unsafe extern "C" {
            fn openpty(
                master: *mut i32,
                slave: *mut i32,
                name: *mut u8,
                term: *const c_void,
                window: *const c_void,
            ) -> i32;
        }
        let _guard = context();
        let mut master = -1;
        let mut slave = -1;
        assert_eq!(
            unsafe {
                openpty(
                    &mut master,
                    &mut slave,
                    ptr::null_mut(),
                    ptr::null(),
                    ptr::null(),
                )
            },
            0
        );
        let mut master = unsafe { File::from_raw_fd(master) };
        let file = unsafe { File::from_raw_fd(slave) };
        let token = reserve_handle_token().unwrap();
        PROCESS.with(|p| {
            p.borrow_mut()
                .as_mut()
                .unwrap()
                .objects
                .insert(token, Object::Stream(Stream { file, input: false }));
        });
        let mut written = 0;
        let mut mode = 0;
        let value = w("λ🌸");
        unsafe {
            assert_eq!(GetConsoleMode(token, &mut mode), 1);
            assert_eq!(mode, 3);
            assert_eq!(
                WriteConsoleW(token, value.as_ptr(), 3, &mut written, ptr::null_mut()),
                1
            );
            assert_eq!(written, 3);
        }
        let mut bytes = [0u8; 6];
        master.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, "λ🌸".as_bytes());
        assert_eq!(close_stream(token), Some(1));
    }
}
