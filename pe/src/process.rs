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
    fn mprotect(addr: *mut c_void, len: usize, prot: i32) -> i32;
    fn sys_icache_invalidate(start: *mut c_void, len: usize);
    fn getpagesize() -> i32;
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
    protections: Vec<u32>,
}
impl Mapping {
    // Return the affected page interval only for a nonempty byte range entirely
    // inside this one owned reservation. Arithmetic never wraps across regions.
    fn pages(&self, address: usize, size: usize) -> Option<(usize, usize)> {
        let end = address.checked_add(size)?;
        if size == 0 || address < self.address || end > self.address.checked_add(self.len)? {
            return None;
        }
        Some((
            (address - self.address) / 4096,
            (end - self.address).div_ceil(4096),
        ))
    }
}
impl Drop for Mapping {
    fn drop(&mut self) {
        if self.len != 0 {
            unsafe {
                munmap(self.address as *mut c_void, self.len);
            }
        }
    }
}
struct State {
    command: Vec<u16>,
    executable: Vec<u16>,
    environment: BTreeMap<String, Vec<u16>>,
    standard: [usize; 3],
    objects: BTreeMap<usize, Object>,
    functions: crate::function_tables::Registry,
    mappings: Vec<Mapping>,
    locals: BTreeMap<usize, Box<[usize]>>,
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
            functions: Default::default(),
            mappings: Vec::new(),
            locals: BTreeMap::new(),
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
pub(crate) fn with_function_tables<T>(
    f: impl FnOnce(Option<&mut crate::function_tables::Registry>) -> T,
) -> T {
    PROCESS.with(|slot| f(slot.borrow_mut().as_mut().map(|state| &mut state.functions)))
}
pub(crate) fn environment_block() -> Result<Vec<u16>> {
    PROCESS.with(|p| {
        let state = p.borrow();
        let state = state.as_ref().ok_or("no installed process environment")?;
        let mut units = Vec::new();
        for (name, value) in &state.environment {
            units.extend(name.encode_utf16());
            units.push(b'=' as u16);
            units.extend(value);
            units.push(0);
        }
        if units.is_empty() {
            units.push(0);
        }
        units.push(0);
        Ok(units)
    })
}
fn fail(error: u32) -> i32 {
    ffi::SetLastError(error);
    0
}
pub(crate) unsafe fn wide(ptr: *const u16) -> std::result::Result<String, u32> {
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
// Describe the actual duplicated host stream, including redirected pipes and
// character devices. This is used by the PE32 CRT's observed standard-I/O setup.
pub(crate) fn stream_file_type(handle: usize) -> Option<std::result::Result<u32, u32>> {
    use std::os::unix::fs::FileTypeExt;
    PROCESS.with(|p| {
        let state = p.borrow();
        let Object::Stream(stream) = state.as_ref()?.objects.get(&handle)? else {
            return None;
        };
        Some(
            stream
                .file
                .metadata()
                .map_err(|e| io_error(&e))
                .and_then(|m| {
                    let kind = m.file_type();
                    if kind.is_file() || kind.is_dir() {
                        Ok(1)
                    } else if kind.is_char_device() {
                        Ok(2)
                    } else if kind.is_fifo() || kind.is_socket() {
                        Ok(3)
                    } else {
                        Err(NOT_SUPPORTED)
                    }
                }),
        )
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
// Width-specific runtimes copy this owned process snapshot into guest storage;
// a native pointer must never be truncated into a PE32 return value.
pub(crate) fn command_snapshot() -> Option<Vec<u16>> {
    PROCESS.with(|p| p.borrow().as_ref().map(|s| s.command.clone()))
}
// A real contiguous Shell32 argv allocation, released with the bounded LocalFree.
unsafe extern "win64" fn CommandLineToArgvW(text: *const u16, count: *mut i32) -> *mut *mut u16 {
    if count.is_null() {
        fail(INVALID_PARAMETER);
        return ptr::null_mut();
    }
    let text = match unsafe { wide(text) } {
        Ok(s) => s,
        Err(e) => {
            fail(e);
            return ptr::null_mut();
        }
    };
    let args = match crate::command_line::canonical_arguments(&text) {
        Ok(a) => a,
        Err(_) => {
            fail(NOT_SUPPORTED);
            return ptr::null_mut();
        }
    };
    let strings: Vec<Vec<u16>> = args
        .iter()
        .map(|s| s.encode_utf16().chain([0]).collect())
        .collect();
    let bytes = strings.len() * 8 + strings.iter().map(|s| s.len() * 2).sum::<usize>();
    if bytes > 1024 * 1024 {
        fail(NOT_SUPPORTED);
        return ptr::null_mut();
    }
    PROCESS.with(|slot| {
        let mut state = slot.borrow_mut();
        let Some(state) = state.as_mut() else {
            fail(NOT_SUPPORTED);
            return ptr::null_mut();
        };
        if state.locals.len() >= 64 {
            fail(NOT_SUPPORTED);
            return ptr::null_mut();
        }
        let mut block = vec![0usize; bytes.div_ceil(8)].into_boxed_slice();
        let address = block.as_mut_ptr();
        let mut target = unsafe { address.add(strings.len()).cast::<u16>() };
        for (index, string) in strings.iter().enumerate() {
            unsafe {
                address.add(index).write(target as usize);
                ptr::copy_nonoverlapping(string.as_ptr(), target, string.len());
                target = target.add(string.len());
            }
        }
        state.locals.insert(address as usize, block);
        unsafe {
            count.write(strings.len() as i32);
        }
        address.cast()
    })
}
unsafe extern "win64" fn LocalFree(pointer: *mut c_void) -> *mut c_void {
    if pointer.is_null() {
        return ptr::null_mut();
    }
    PROCESS.with(|slot| {
        let mut state = slot.borrow_mut();
        if state
            .as_mut()
            .is_some_and(|s| s.locals.remove(&(pointer as usize)).is_some())
        {
            ptr::null_mut()
        } else {
            fail(INVALID_PARAMETER);
            pointer
        }
    })
}
unsafe extern "win64" fn GetCurrentDirectoryW(capacity: u32, buffer: *mut u16) -> u32 {
    if !PROCESS.with(|slot| slot.borrow().is_some()) {
        fail(NOT_SUPPORTED);
        return 0;
    }
    let directory = match std::env::current_dir() {
        Ok(p) => p,
        Err(e) => {
            fail(io_error(&e));
            return 0;
        }
    };
    let Some(text) = directory.to_str() else {
        fail(1113);
        return 0;
    };
    let wide: Vec<u16> = text.encode_utf16().collect();
    if wide.len() + 1 > 32767 {
        fail(206);
        return 0;
    }
    if capacity as usize <= wide.len() {
        return (wide.len() + 1) as u32;
    }
    if buffer.is_null() {
        fail(INVALID_PARAMETER);
        return 0;
    }
    unsafe {
        ptr::copy_nonoverlapping(wide.as_ptr(), buffer, wide.len());
        buffer.add(wide.len()).write(0);
    }
    wide.len() as u32
}
pub(crate) fn shell_address(name: &str) -> Option<usize> {
    (name == "CommandLineToArgvW").then_some(CommandLineToArgvW as *const () as usize)
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
    if unsafe { getpagesize() } != 4096 {
        fail(NOT_SUPPORTED);
        return ptr::null_mut();
    }
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
        let mut protections = Vec::new();
        if protections.try_reserve_exact(length / 4096).is_err()
            || state.mappings.try_reserve(1).is_err()
        {
            fail(8);
            return ptr::null_mut();
        }
        protections.resize(length / 4096, 4);
        let reserve = length + 65536;
        let raw = unsafe { mmap(ptr::null_mut(), reserve, 3, 2 | 0x1000, -1, 0) };
        if raw as isize == -1 {
            fail(8);
            return ptr::null_mut();
        }
        // Keep ownership accurate after each successful trim, including errors.
        // No MAP_FIXED operation can replace somebody else's mapping.
        let mut mapped = Mapping {
            address: raw as usize,
            len: reserve,
            protections,
        };
        let Some(start) = (raw as usize).checked_add(65535).map(|v| v & !65535) else {
            fail(8);
            return ptr::null_mut();
        };
        let prefix = start - raw as usize;
        let suffix = reserve - prefix - length;
        unsafe {
            if prefix != 0 {
                if munmap(raw, prefix) != 0 {
                    fail(8);
                    return ptr::null_mut();
                }
                mapped.address = start;
                mapped.len -= prefix;
            }
            if suffix != 0 {
                if munmap((start + length) as *mut c_void, suffix) != 0 {
                    fail(8);
                    return ptr::null_mut();
                }
                mapped.len -= suffix;
            }
        }
        state.mappings.push(mapped);
        start as *mut c_void
    })
}
unsafe extern "win64" fn VirtualProtect(
    address: *mut c_void,
    size: usize,
    protection: u32,
    old: *mut u32,
) -> i32 {
    if address.is_null()
        || size == 0
        || old.is_null()
        || (address as usize).checked_add(size).is_none()
    {
        return fail(INVALID_PARAMETER);
    }
    let rights = match protection {
        1 => 0,
        2 => 1,
        4 => 3,
        0x20 => 5,
        _ => return fail(NOT_SUPPORTED),
    };
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let Some(state) = state.as_mut() else {
            return fail(NOT_SUPPORTED);
        };
        for mapping in &mut state.mappings {
            if let Some((first, last)) = mapping.pages(address as usize, size) {
                let prior = mapping.protections[first];
                if unsafe {
                    mprotect(
                        (mapping.address + first * 4096) as *mut c_void,
                        (last - first) * 4096,
                        rights,
                    )
                } != 0
                {
                    return fail(487);
                }
                mapping.protections[first..last].fill(protection);
                unsafe {
                    old.write_unaligned(prior);
                }
                return 1;
            }
        }
        fail(487)
    })
}
unsafe extern "win64" fn VirtualFree(address: *mut c_void, size: usize, kind: u32) -> i32 {
    if kind != 0x8000 {
        return fail(NOT_SUPPORTED);
    }
    if address.is_null() || size != 0 {
        return fail(INVALID_PARAMETER);
    }
    PROCESS.with(|p| {
        let mut state = p.borrow_mut();
        let Some(state) = state.as_mut() else {
            return fail(NOT_SUPPORTED);
        };
        let Some(index) = state
            .mappings
            .iter()
            .position(|m| m.address == address as usize)
        else {
            return fail(487);
        };
        let mapping = &mut state.mappings[index];
        if unsafe { munmap(address, mapping.len) } != 0 {
            return fail(487);
        }
        mapping.len = 0; // Successful release consumes ownership before Drop.
        state.mappings.swap_remove(index);
        1
    })
}
unsafe extern "win64" fn FlushInstructionCache(
    process: usize,
    address: *const c_void,
    size: usize,
) -> i32 {
    if crate::abi::kernel_handle(process) != usize::MAX {
        return fail(NOT_SUPPORTED);
    }
    // A null/global flush is not implemented by pretending a zero-byte platform
    // operation flushes the whole guest process.
    if address.is_null() {
        return fail(NOT_SUPPORTED);
    }
    if size == 0 || (address as usize).checked_add(size).is_none() {
        return fail(INVALID_PARAMETER);
    }
    PROCESS.with(|p| {
        let state = p.borrow();
        let Some(state) = state.as_ref() else {
            return fail(NOT_SUPPORTED);
        };
        for mapping in &state.mappings {
            if let Some((first, last)) = mapping.pages(address as usize, size) {
                if mapping.protections[first..last].contains(&1) {
                    return fail(NOT_SUPPORTED);
                }
                // Darwin's public cache-control operation. On physical x86 its
                // documented implementation may be a coherent-cache no-op.
                unsafe {
                    sys_icache_invalidate(address.cast_mut(), size);
                }
                return 1;
            }
        }
        fail(487)
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
        "GetCurrentDirectoryW" => Some(GetCurrentDirectoryW as *const () as usize),
        "LocalFree" => Some(LocalFree as *const () as usize),
        "GetStdHandle" => Some(GetStdHandle as *const () as usize),
        "GetConsoleMode" => Some(GetConsoleMode as *const () as usize),
        "GetCommandLineW" => Some(GetCommandLineW as *const () as usize),
        "GetEnvironmentVariableW" => Some(GetEnvironmentVariableW as *const () as usize),
        "GetModuleFileNameW" => Some(GetModuleFileNameW as *const () as usize),
        "VirtualAlloc" => Some(VirtualAlloc as *const () as usize),
        "VirtualProtect" => Some(VirtualProtect as *const () as usize),
        "VirtualFree" => Some(VirtualFree as *const () as usize),
        "FlushInstructionCache" => Some(FlushInstructionCache as *const () as usize),
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
    fn empty_environment_is_double_terminated_and_requires_a_live_context() {
        assert!(environment_block().is_err());
        let guard = ProcessContext::install(&["x.exe".into()], &[]).unwrap();
        assert_eq!(environment_block().unwrap(), [0, 0]);
        drop(guard);
        assert!(environment_block().is_err());
    }
    #[test]
    fn shell_argv_is_contiguous_utf16_owned_storage_freed_by_its_full_width_address() {
        let guard = context();
        let argv: unsafe extern "win64" fn(*const u16, *mut i32) -> *mut *mut u16 = unsafe {
            std::mem::transmute(crate::abi::resolve("shell32.dll", "CommandLineToArgvW").unwrap())
        };
        let free: unsafe extern "win64" fn(*mut c_void) -> *mut c_void = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "LocalFree").unwrap())
        };
        let args = ["/a b/λ🌸.exe", "", "x\"y", "tail\\", "λ🌸"];
        let command = windows_command_line(&args.map(String::from)).unwrap();
        let mut count = -1;
        ffi::SetLastError(0xbeef);
        let block = unsafe { argv(command.as_ptr(), &mut count) };
        assert!(!block.is_null());
        assert_eq!(count, 5);
        assert_eq!(ffi::GetLastError(), 0xbeef);
        let mut string = unsafe { block.add(count as usize).cast::<u16>() };
        for (index, expected) in args.iter().enumerate() {
            let actual = unsafe { *block.add(index) };
            assert_eq!(actual, string);
            assert_eq!(unsafe { wide(actual) }.unwrap(), *expected);
            string = unsafe { string.add(expected.encode_utf16().count() + 1) };
        }
        assert_eq!(
            unsafe { free(block.cast::<u8>().wrapping_add(2).cast()) },
            block.cast::<u8>().wrapping_add(2).cast()
        );
        assert_eq!(ffi::GetLastError(), 87);
        assert!(unsafe { free(block.cast()) }.is_null());
        assert_eq!(unsafe { free(block.cast()) }, block.cast());
        assert!(unsafe { free(ptr::null_mut()) }.is_null());
        let abandoned = unsafe { argv(command.as_ptr(), &mut count) };
        assert!(!abandoned.is_null());
        drop(guard);
        let _fresh = context();
        assert_eq!(unsafe { free(abandoned.cast()) }, abandoned.cast());
    }
    #[test]
    fn shell_argument_failures_bounds_and_thread_ownership_are_explicit() {
        let command = w("\"app\" \"x\"");
        let mut count = 0x1234;
        assert!(unsafe { CommandLineToArgvW(command.as_ptr(), &mut count) }.is_null());
        assert_eq!(ffi::GetLastError(), 50);
        let _guard = context();
        assert!(unsafe { CommandLineToArgvW(ptr::null(), &mut count) }.is_null());
        assert_eq!(ffi::GetLastError(), 87);
        for input in ["app arg", "", "\"app\"  \"x\""] {
            let text = w(input);
            assert!(unsafe { CommandLineToArgvW(text.as_ptr(), &mut count) }.is_null());
            assert_eq!(ffi::GetLastError(), 50);
        }
        assert_eq!(count, 0x1234);
        assert!(unsafe { CommandLineToArgvW(command.as_ptr(), ptr::null_mut()) }.is_null());
        assert_eq!(ffi::GetLastError(), 87);
        let unterminated = vec![65u16; 32768];
        assert!(unsafe { CommandLineToArgvW(unterminated.as_ptr(), &mut count) }.is_null());
        let mut blocks = Vec::new();
        for _ in 0..64 {
            let block = unsafe { CommandLineToArgvW(command.as_ptr(), &mut count) };
            assert!(!block.is_null());
            blocks.push(block as usize);
        }
        assert!(unsafe { CommandLineToArgvW(command.as_ptr(), &mut count) }.is_null());
        assert_eq!(ffi::GetLastError(), 50);
        let other = blocks[0];
        std::thread::spawn(move || {
            assert_eq!(unsafe { LocalFree(other as *mut c_void) } as usize, other);
        })
        .join()
        .unwrap();
        for block in blocks {
            assert!(unsafe { LocalFree(block as *mut c_void) }.is_null());
        }
        assert!(!unsafe { CommandLineToArgvW(command.as_ptr(), &mut count) }.is_null());
    }
    #[test]
    fn current_directory_reports_real_host_utf16_size_and_terminator_without_invented_drives() {
        let get: unsafe extern "win64" fn(u32, *mut u16) -> u32 = unsafe {
            std::mem::transmute(
                crate::abi::resolve("kernel32.dll", "GetCurrentDirectoryW").unwrap(),
            )
        };
        assert_eq!(unsafe { get(0, ptr::null_mut()) }, 0);
        assert_eq!(ffi::GetLastError(), 50);
        let _guard = context();
        let expected = std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .encode_utf16()
            .collect::<Vec<_>>();
        ffi::SetLastError(0xbeef);
        let size = unsafe { get(0, ptr::null_mut()) };
        assert_eq!(size as usize, expected.len() + 1);
        let mut buffer = vec![0xabab; size as usize + 1];
        assert_eq!(unsafe { get(size - 1, buffer.as_mut_ptr()) }, size);
        assert!(buffer.iter().all(|&v| v == 0xabab));
        assert_eq!(unsafe { get(size, buffer.as_mut_ptr()) }, size - 1);
        assert_eq!(&buffer[..expected.len()], expected);
        assert_eq!(buffer[expected.len()], 0);
        assert_eq!(buffer[size as usize], 0xabab);
        assert_eq!(ffi::GetLastError(), 0xbeef);
        assert_eq!(unsafe { get(size, ptr::null_mut()) }, 0);
        assert_eq!(ffi::GetLastError(), 87);
    }
    #[test]
    fn command_environment_and_module_buffers_obey_windows_size_rules() {
        let _guard = context();
        assert_eq!(
            environment_block().unwrap(),
            "EMPTY=\0PKG_CONFIG_PATH=pc/λ🌸\0\0"
                .encode_utf16()
                .collect::<Vec<_>>()
        );
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
    fn generated_win64_code_runs_after_real_protection_changes_and_rewrites() {
        type Protect = unsafe extern "win64" fn(*mut c_void, usize, u32, *mut u32) -> i32;
        type Free = unsafe extern "win64" fn(*mut c_void, usize, u32) -> i32;
        type Flush = unsafe extern "win64" fn(usize, *const c_void, usize) -> i32;
        let protect: Protect = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "VirtualProtect").unwrap())
        };
        let free: Free = unsafe {
            std::mem::transmute(crate::abi::resolve("kernel32.dll", "VirtualFree").unwrap())
        };
        let flush: Flush = unsafe {
            std::mem::transmute(
                crate::abi::resolve("kernel32.dll", "FlushInstructionCache").unwrap(),
            )
        };
        let _context = context();
        unsafe {
            let allocation = VirtualAlloc(ptr::null_mut(), 4097, 0x3000, 4).cast::<u8>();
            assert!(!allocation.is_null());
            let code = allocation.add(64);
            let instructions = [0xb8, 0x44, 0x33, 0x22, 0x11, 0x31, 0xc8, 0x01, 0xd0, 0xc3];
            ptr::copy_nonoverlapping(instructions.as_ptr(), code, instructions.len());
            let mut old = 0;
            assert_eq!(protect(code.cast(), 10, 0x20, &mut old), 1);
            assert_eq!(old, 4);
            allocation.add(4096).write_volatile(0x5a); // The second page remains writable.
            assert_eq!(flush(usize::MAX, code.cast(), 10), 1);
            let call: unsafe extern "win64" fn(u32, u32) -> u32 = std::mem::transmute(code);
            assert_eq!(call(7, 11), (0x11223344u32 ^ 7).wrapping_add(11));
            assert_eq!(protect(allocation.cast(), 1, 4, &mut old), 1);
            assert_eq!(old, 0x20);
            code.add(1).cast::<u32>().write_unaligned(0xfedcba98);
            assert_eq!(protect(code.cast(), 10, 0x20, &mut old), 1);
            assert_eq!(old, 4);
            assert_eq!(flush(usize::MAX, code.cast(), 10), 1);
            assert_eq!(call(7, 11), (0xfedcba98u32 ^ 7).wrapping_add(11));
            assert_eq!(protect(allocation.add(4095).cast(), 2, 2, &mut old), 1);
            assert_eq!(old, 0x20); // Old protection is that of the first page.
            assert_eq!(protect(allocation.cast(), 4096, 4, &mut old), 1);
            assert_eq!(old, 2);
            assert_eq!(protect(allocation.add(4096).cast(), 4096, 4, &mut old), 1);
            assert_eq!(old, 2); // The straddling byte range changed both pages.
            assert_eq!(allocation.add(4096).read_volatile(), 0x5a);
            assert_eq!(free(allocation.cast(), 0, 0x8000), 1);
            assert_eq!(free(allocation.cast(), 0, 0x8000), 0);
            assert_eq!(ffi::GetLastError(), 487);
        }
        PROCESS.with(|p| assert!(p.borrow().as_ref().unwrap().mappings.is_empty()));
    }
    #[test]
    fn memory_api_failures_preserve_ownership_rights_and_output_sentinels() {
        let mut old = 0xdead;
        let foreign = &mut old as *mut u32 as *mut c_void;
        unsafe {
            assert_eq!(VirtualProtect(foreign, 1, 2, &mut old), 0);
            assert_eq!(ffi::GetLastError(), 50);
            assert_eq!(VirtualFree(foreign, 0, 0x8000), 0);
            assert_eq!(ffi::GetLastError(), 50);
            assert_eq!(FlushInstructionCache(usize::MAX, foreign, 1), 0);
            assert_eq!(ffi::GetLastError(), 50);
        }
        let _context = context();
        unsafe {
            let memory = VirtualAlloc(ptr::null_mut(), 4096, 0x3000, 4).cast::<u8>();
            for (address, size, protection, error) in [
                (ptr::null_mut(), 1, 2, 87),
                (memory, 0, 2, 87),
                (usize::MAX as *mut u8, 2, 2, 87),
                (memory, 1, 0x40, 50),
                (memory, 1, 0x104, 50),
                (foreign.cast(), 1, 2, 487),
                (memory.add(4095), 2, 2, 487),
            ] {
                assert_eq!(
                    VirtualProtect(address.cast(), size, protection, &mut old),
                    0
                );
                assert_eq!(ffi::GetLastError(), error);
                assert_eq!(old, 0xdead);
            }
            assert_eq!(VirtualProtect(memory.cast(), 1, 2, ptr::null_mut()), 0);
            assert_eq!(ffi::GetLastError(), 87);
            for (address, size, kind, error) in [
                (ptr::null_mut(), 0, 0x8000, 87),
                (memory, 1, 0x8000, 87),
                (memory, 0, 0x4000, 50),
                (memory.add(1), 0, 0x8000, 487),
                (foreign.cast(), 0, 0x8000, 487),
            ] {
                assert_eq!(VirtualFree(address.cast(), size, kind), 0);
                assert_eq!(ffi::GetLastError(), error);
            }
            for (process, address, size, error) in [
                (0, memory, 1, 50),
                (usize::MAX, ptr::null_mut(), 0, 50),
                (usize::MAX, memory, 0, 87),
                (usize::MAX, usize::MAX as *mut u8, 2, 87),
                (usize::MAX, memory.add(4095), 2, 487),
            ] {
                assert_eq!(FlushInstructionCache(process, address.cast(), size), 0);
                assert_eq!(ffi::GetLastError(), error);
            }
            memory.write_volatile(0x5a);
            assert_eq!(
                FlushInstructionCache(0x12345678ffffffff, memory.cast(), 1),
                1
            );
            assert_eq!(VirtualProtect(memory.cast(), 1, 1, &mut old), 1);
            assert_eq!(old, 4);
            assert_eq!(FlushInstructionCache(usize::MAX, memory.cast(), 1), 0);
            assert_eq!(ffi::GetLastError(), 50);
            assert_eq!(VirtualProtect(memory.cast(), 1, 4, &mut old), 1);
            assert_eq!(old, 1);
            assert_eq!(memory.read_volatile(), 0x5a);
            assert_eq!(VirtualFree(memory.cast(), 0, 0x8000), 1);
        }
    }
    #[test]
    fn memory_permissions_are_enforced_and_release_unmaps() {
        use std::os::unix::process::ExitStatusExt;
        const CHILD: &str = "ALKYHOL_PE_MEMORY_CHILD";
        if let Ok(mode) = std::env::var(CHILD) {
            let _context = context();
            unsafe {
                let memory = VirtualAlloc(ptr::null_mut(), 65536, 0x3000, 4).cast::<u8>();
                assert!(!memory.is_null());
                ptr::copy_nonoverlapping([0xb8, 42, 0, 0, 0, 0xc3].as_ptr(), memory, 6);
                let mut old = 0;
                match mode.as_str() {
                    "write-rx" => {
                        assert_eq!(VirtualProtect(memory.cast(), 4096, 0x20, &mut old), 1);
                        memory.write_volatile(0);
                    }
                    "execute-rw" => {
                        assert_eq!(FlushInstructionCache(usize::MAX, memory.cast(), 6), 1);
                        let call: unsafe extern "win64" fn() -> u32 = std::mem::transmute(memory);
                        std::hint::black_box(call());
                    }
                    "read-noaccess" => {
                        assert_eq!(VirtualProtect(memory.cast(), 4096, 1, &mut old), 1);
                        std::hint::black_box(memory.read_volatile());
                    }
                    "write-readonly" => {
                        assert_eq!(VirtualProtect(memory.cast(), 4096, 2, &mut old), 1);
                        memory.write_volatile(0);
                    }
                    "released" => {
                        unsafe extern "C" {
                            static mach_task_self_: u32;
                            fn mach_vm_region(
                                task: u32,
                                address: *mut u64,
                                size: *mut u64,
                                flavor: i32,
                                info: *mut i32,
                                count: *mut u32,
                                object: *mut u32,
                            ) -> i32;
                            fn mach_port_deallocate(task: u32, port: u32) -> i32;
                        }
                        fn mapped(address: usize) -> bool {
                            let mut start = address as u64;
                            let mut size = 0;
                            let mut info = [0i32; 9];
                            let mut count = 9;
                            let mut object = 0;
                            let task = unsafe { mach_task_self_ };
                            let status = unsafe {
                                mach_vm_region(
                                    task,
                                    &mut start,
                                    &mut size,
                                    9,
                                    info.as_mut_ptr(),
                                    &mut count,
                                    &mut object,
                                )
                            };
                            assert_eq!(status, 0);
                            if object != 0 {
                                unsafe {
                                    mach_port_deallocate(task, object);
                                }
                            }
                            start <= address as u64 && (address as u64) < start + size
                        }
                        let middle = memory.add(32768) as usize;
                        assert!(mapped(middle));
                        assert_eq!(VirtualFree(memory.cast(), 0, 0x8000), 1);
                        assert!(!mapped(middle));
                        return;
                    }
                    _ => panic!("unknown child mode"),
                }
                panic!("forbidden memory access unexpectedly succeeded");
            }
        }
        for mode in [
            "write-rx",
            "execute-rw",
            "read-noaccess",
            "write-readonly",
            "released",
        ] {
            let run = std::process::Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "process::tests::memory_permissions_are_enforced_and_release_unmaps",
                    "--nocapture",
                ])
                .env(CHILD, mode)
                .output()
                .unwrap();
            if mode == "released" {
                assert!(
                    run.status.success(),
                    "{}",
                    String::from_utf8_lossy(&run.stderr)
                );
            } else {
                assert!(
                    matches!(run.status.signal(), Some(10 | 11)),
                    "{mode}: {:?} {}",
                    run.status,
                    String::from_utf8_lossy(&run.stderr)
                );
            }
        }
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
        assert_eq!(stream_file_type(token), Some(Ok(1)));
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
        assert_eq!(stream_file_type(token), None);
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
    #[test]
    fn redirected_stdin_reads_real_bytes_then_eof_through_the_win64_bridge() {
        let _guard = context();
        let path = std::env::temp_dir().join(format!(
            "alkyhol-pe-stdin-{}",
            reserve_handle_token().unwrap()
        ));
        std::fs::write(&path, [0, 0xff, 42]).unwrap();
        let token = reserve_handle_token().unwrap();
        PROCESS.with(|p| {
            let mut slot = p.borrow_mut();
            let state = slot.as_mut().unwrap();
            state.objects.insert(
                token,
                Object::Stream(Stream {
                    file: File::open(&path).unwrap(),
                    input: true,
                }),
            );
            state.standard[0] = token;
        });
        unsafe {
            let read: unsafe extern "win64" fn(
                usize,
                *mut c_void,
                u32,
                *mut u32,
                *mut c_void,
            ) -> i32 =
                std::mem::transmute(crate::abi::resolve("kernel32.dll", "ReadFile").unwrap());
            let write: unsafe extern "win64" fn(
                usize,
                *const c_void,
                u32,
                *mut u32,
                *mut c_void,
            ) -> i32 =
                std::mem::transmute(crate::abi::resolve("kernel32.dll", "WriteFile").unwrap());
            let handle = GetStdHandle(0xfffffff6);
            assert_eq!(handle, token);
            let tagged = handle | 0x1234567800000000;
            let mut buffer = [0xa5u8; 8];
            let mut transferred = 999;
            assert_eq!(
                read(
                    tagged,
                    buffer.as_mut_ptr().cast(),
                    8,
                    &mut transferred,
                    ptr::null_mut()
                ),
                1
            );
            assert_eq!(transferred, 3);
            assert_eq!(buffer, [0, 0xff, 42, 0xa5, 0xa5, 0xa5, 0xa5, 0xa5]);
            assert_eq!(
                read(
                    handle,
                    buffer.as_mut_ptr().cast(),
                    8,
                    &mut transferred,
                    ptr::null_mut()
                ),
                1
            );
            assert_eq!(transferred, 0);
            assert_eq!(buffer[3..], [0xa5; 5]);
            transferred = 999;
            assert_eq!(
                write(
                    tagged,
                    buffer.as_ptr().cast(),
                    1,
                    &mut transferred,
                    ptr::null_mut()
                ),
                0
            );
            assert_eq!((transferred, ffi::GetLastError()), (0, 5));
            assert_eq!(close_stream(handle), Some(1));
            transferred = 999;
            assert_eq!(
                read(
                    handle,
                    buffer.as_mut_ptr().cast(),
                    8,
                    &mut transferred,
                    ptr::null_mut()
                ),
                0
            );
            assert_eq!((transferred, ffi::GetLastError()), (0, 6));
        }
        std::fs::remove_file(path).unwrap();
    }
}
