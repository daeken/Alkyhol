//! Stateful Winsock bootstrap and real bounded IPv4 BSD sockets.
//! Host structures never escape as guest pointers. Unsupported profiles fail.
use super::{LoadedImage32, LowMapping, PAGE, bulk32, thread32, unsupported};
use crate::Result;
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::{CStr, CString, c_char, c_void},
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    sync::Arc,
};
#[repr(C)]
struct Hostent {
    name: *const c_char,
    aliases: *const *const c_char,
    family: i32,
    length: i32,
    addresses: *const *const u8,
}
unsafe extern "C" {
    fn sendto(
        fd: i32,
        buffer: *const u8,
        length: usize,
        flags: i32,
        address: *const u8,
        address_length: u32,
    ) -> isize;
    fn alkyhol_udp_receive(
        fd: i32,
        buffer: *mut u8,
        capacity: u32,
        address: *mut u8,
        address_length: *mut u32,
        truncated: *mut i32,
        error: *mut i32,
    ) -> i64;
    fn socket(domain: i32, kind: i32, protocol: i32) -> i32;
    fn bind(fd: i32, address: *const u8, length: u32) -> i32;
    fn getsockname(fd: i32, address: *mut u8, length: *mut u32) -> i32;
    fn connect(fd: i32, address: *const u8, length: u32) -> i32;
    fn recv(fd: i32, buffer: *mut u8, length: usize, flags: i32) -> isize;
    fn send(fd: i32, buffer: *const u8, length: usize, flags: i32) -> isize;
    fn getsockopt(fd: i32, level: i32, option: i32, value: *mut c_void, length: *mut u32) -> i32;
    fn setsockopt(fd: i32, level: i32, option: i32, value: *const c_void, length: u32) -> i32;
    fn fcntl(fd: i32, command: i32, ...) -> i32;
    fn ioctl(fd: i32, command: std::ffi::c_ulong, ...) -> i32;
    fn inet_addr(address: *const c_char) -> u32;
    fn getipnodebyname(
        name: *const c_char,
        family: i32,
        flags: i32,
        error: *mut i32,
    ) -> *mut Hostent;
    fn getipnodebyaddr(
        address: *const c_void,
        length: usize,
        family: i32,
        error: *mut i32,
    ) -> *mut Hostent;
    fn freehostent(host: *mut Hostent);
}
struct Socket {
    fd: Arc<OwnedFd>,
    kind: u32,
    bound: bool,
}
#[derive(Default)]
pub(super) struct State {
    references: u32,
    next: u32,
    sockets: BTreeMap<u32, Socket>,
    busy: BTreeSet<u32>,
    hosts: BTreeMap<u32, LowMapping>,
    logged_receive_limit: bool,
}
impl State {
    pub(super) fn permits(&self, address: usize, len: usize, write: bool) -> bool {
        !write
            && address.checked_add(len).is_some_and(|end| {
                self.hosts
                    .values()
                    .any(|m| address >= m.base as usize && end <= m.base as usize + m.len)
            })
    }
}
fn failure(error: u32) -> u32 {
    alkyhol::ffi::SetLastError(error);
    u32::MAX
}
fn native_error() -> u32 {
    errno(std::io::Error::last_os_error().raw_os_error().unwrap_or(22))
}
fn errno(code: i32) -> u32 {
    match code {
        4 => 10004,
        9 => 10038,
        1 | 13 => 10013,
        14 => 10014,
        22 => 10022,
        24 => 10024,
        32 => 10058,
        35..=71 => 10000 + code as u32,
        _ => 10045,
    }
}
fn checked(image: &LoadedImage32<'_>, pointer: u32, len: usize, write: bool) {
    if !image.permits(pointer as usize, len, write) {
        unsupported("Winsock buffer outside checked guest storage");
    }
}
/// WSA event aliases use the same real manual-reset kernel objects as Win32.
/// Event any/all sets are atomic; semaphore sets and APC dispatch remain explicit.
pub(super) fn event_api(image: &LoadedImage32<'_>, id: u32, a: &[u32; 8]) -> u32 {
    if id == 280 {
        return super::event32::dispatch(image, 116, &[0, 1, 0, 0, 0, 0, 0, 0]);
    }
    if id == 279 {
        if !ready(image) {
            return failure(10093);
        }
        if !(1..=64).contains(&a[0]) {
            return failure(87);
        }
        if a[4] != 0 {
            unsupported("alertable Winsock waits require APC delivery");
        }
        checked(image, a[1], a[0] as usize * 4, false);
        let handles: Vec<_> = (0..a[0] as usize)
            .map(|index| unsafe { (a[1] as *const u32).add(index).read_unaligned() })
            .collect();
        for &handle in &handles {
            let state = image.cpu.events.borrow();
            if !state.is_event(handle) && !image.cpu.threads.borrow().contains(handle) {
                if state.contains(handle) {
                    unsupported(
                        "Winsock wait sets currently require manual events or guest-thread completion handles",
                    );
                }
                return failure(6);
            }
        }
        if handles.len() == 1 {
            return thread32::wait_handle(image, handles[0], a[3]);
        }
        let handles: Vec<_> = handles.into_iter().map(|handle| handle as usize).collect();
        match thread32::wait_event_set(image, &handles, a[2] != 0, a[3]) {
            Ok(result) => return result,
            Err(error) if error.0 == 50 => {
                unsupported("Winsock multi-wait contains an unsupported event alias or object kind")
            }
            Err(error) => return failure(error.0),
        }
    }
    if !image.cpu.events.borrow().is_event(a[0]) {
        alkyhol::ffi::SetLastError(6);
        return 0;
    }
    match id {
        281 | 282 => super::event32::dispatch(image, if id == 281 { 117 } else { 118 }, a),
        283 => image
            .cpu
            .events
            .borrow_mut()
            .close(a[0])
            .unwrap_or_else(|e| unsupported(&e)) as u32,
        _ => unreachable!(),
    }
}
fn ready(image: &LoadedImage32<'_>) -> bool {
    image.cpu.winsock.borrow().references != 0
}
fn begin(image: &LoadedImage32<'_>, handle: u32) -> std::result::Result<Arc<OwnedFd>, u32> {
    let mut state = image.cpu.winsock.borrow_mut();
    if state.references == 0 {
        return Err(10093);
    }
    let fd = state
        .sockets
        .get(&handle)
        .map(|socket| socket.fd.clone())
        .ok_or(10038u32)?;
    if !state.busy.insert(handle) {
        return Err(10036);
    }
    Ok(fd)
}
fn finish<T>(
    image: &LoadedImage32<'_>,
    handle: u32,
    result: std::result::Result<T, u32>,
) -> std::result::Result<T, u32> {
    image.cpu.winsock.borrow_mut().busy.remove(&handle);
    result
}
fn startup(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[0] as u16 != 0x202 {
        return 10092;
    } // Explicit 2.2 bootstrap profile.
    checked(image, a[1], 400, true);
    let mut state = image.cpu.winsock.borrow_mut();
    let Some(count) = state.references.checked_add(1) else {
        return 10091;
    };
    let mut bytes = [0u8; 400];
    bytes[..4].copy_from_slice(&[2, 2, 2, 2]);
    let description = b"Alkyhol BSD IPv4 Winsock 2.2 bootstrap";
    bytes[4..4 + description.len()].copy_from_slice(description);
    let status = b"Running";
    bytes[261..261 + status.len()].copy_from_slice(status);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), a[1] as *mut u8, 400);
    }
    state.references = count;
    0
}
fn cleanup(image: &LoadedImage32<'_>) -> u32 {
    let mut state = image.cpu.winsock.borrow_mut();
    if state.references == 0 {
        return failure(10093);
    }
    if state.references == 1 && !state.busy.is_empty() {
        return failure(10036);
    }
    state.references -= 1;
    if state.references == 0 {
        state.sockets.clear();
        state.hosts.clear();
    }
    0
}
/// The original LAN setup requests SOL_SOCKET/SO_BROADCAST. This changes and
/// queries the real host socket option; other options remain explicit.
pub(super) fn broadcast_option(image: &LoadedImage32<'_>, a: &[u32; 8], get: bool) -> u32 {
    if a[1] != 0xffff || a[2] != 0x20 {
        unsupported(&format!(
            "Winsock option outside implemented SO_BROADCAST profile: level={:x} option={:x}",
            a[1], a[2]
        ));
    }
    let length = if get {
        checked(image, a[4], 4, true);
        unsafe { (a[4] as *const u32).read_unaligned() }
    } else {
        a[4]
    };
    if length < 4 {
        return failure(10014);
    }
    if !get && length != 4 {
        unsupported("SO_BROADCAST setters currently require one four-byte BOOL");
    }
    checked(image, a[3], 4, get);
    let mut value = if get {
        0i32
    } else {
        i32::from(unsafe { (a[3] as *const i32).read_unaligned() } != 0)
    };
    let fd = match begin(image, a[0]) {
        Ok(fd) => fd,
        Err(e) => return failure(e),
    };
    let mut actual_length = 4;
    let result = unsafe {
        if get {
            getsockopt(
                fd.as_raw_fd(),
                0xffff,
                0x20,
                std::ptr::from_mut(&mut value).cast(),
                &mut actual_length,
            )
        } else {
            setsockopt(
                fd.as_raw_fd(),
                0xffff,
                0x20,
                std::ptr::from_ref(&value).cast(),
                4,
            )
        }
    };
    let result = if result < 0 {
        Err(native_error())
    } else {
        Ok(())
    };
    if let Err(error) = finish(image, a[0], result) {
        return failure(error);
    }
    if get {
        if actual_length != 4 {
            unsupported("SO_BROADCAST host result has an unexpected extent");
        }
        unsafe {
            // Darwin can expose the socket option bit itself (0x20). Win32's
            // BOOL result is normalized independently of its internal bit flag.
            (a[3] as *mut i32).write_unaligned(i32::from(value != 0));
            (a[4] as *mut u32).write_unaligned(4);
        }
    }
    0
}
fn create(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if !ready(image) {
        return failure(10093);
    }
    if a[0] != 2 {
        return failure(10047);
    }
    if !matches!(a[1], 1 | 2) {
        return failure(10044);
    }
    if !matches!((a[1], a[2]), (1, 0 | 6) | (2, 0 | 17)) {
        return failure(10043);
    }
    let mut state = image.cpu.winsock.borrow_mut();
    if state.sockets.len() >= 256 {
        return failure(10024);
    }
    let fd = unsafe { socket(2, a[1] as i32, a[2] as i32) };
    if fd < 0 {
        return failure(native_error());
    }
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    let enabled = 1i32;
    if unsafe { fcntl(fd.as_raw_fd(), 2, 1) } < 0
        || unsafe {
            setsockopt(
                fd.as_raw_fd(),
                0xffff,
                0x1022,
                std::ptr::from_ref(&enabled).cast(),
                4,
            )
        } < 0
    {
        return failure(native_error());
    }
    let Some(next) = state.next.checked_add(1).filter(|n| *n < 0x10000000) else {
        return failure(10024);
    };
    state.next = next;
    let handle = 0x60000000 | next;
    state.sockets.insert(
        handle,
        Socket {
            fd: Arc::new(fd),
            kind: a[1],
            bound: false,
        },
    );
    handle
}
fn close(image: &LoadedImage32<'_>, h: u32) -> u32 {
    let mut state = image.cpu.winsock.borrow_mut();
    if state.references == 0 {
        return failure(10093);
    }
    if state.busy.contains(&h) {
        return failure(10036);
    }
    if state.sockets.remove(&h).is_none() {
        return failure(10038);
    }
    0
}
/// Bind a real IPv4 socket; query its actual explicitly bound endpoint.
/// No guest port/address is invented to match a native observation.
pub(super) fn endpoint(image: &LoadedImage32<'_>, a: &[u32; 8], get: bool) -> u32 {
    let length = if get {
        checked(image, a[2], 4, true);
        unsafe { (a[2] as *const u32).read_unaligned() }
    } else {
        a[2]
    };
    if length < 16 || !get && length != 16 {
        return failure(10014);
    }
    checked(image, a[1], 16, get);
    let mut address = if get {
        [0; 16]
    } else {
        unsafe { (a[1] as *const [u8; 16]).read_unaligned() }
    };
    if !get {
        if u16::from_le_bytes([address[0], address[1]]) != 2 {
            return failure(10047);
        }
        address[0] = 16;
        address[1] = 2;
    }
    let fd = match begin(image, a[0]) {
        Ok(fd) => fd,
        Err(error) => return failure(error),
    };
    if get && !image.cpu.winsock.borrow().sockets.get(&a[0]).unwrap().bound {
        finish(image, a[0], Ok(())).unwrap();
        unsupported("getsockname currently requires an explicitly bound socket");
    }
    let mut actual_length = 16;
    let result = unsafe {
        if get {
            getsockname(fd.as_raw_fd(), address.as_mut_ptr(), &mut actual_length)
        } else {
            bind(fd.as_raw_fd(), address.as_ptr(), 16)
        }
    };
    let result = if result < 0 {
        Err(native_error())
    } else {
        Ok(())
    };
    if let Err(error) = finish(image, a[0], result) {
        return failure(error);
    }
    if get {
        if actual_length != 16 || address[0] != 16 || address[1] != 2 {
            unsupported("bound host endpoint is outside the IPv4 sockaddr profile");
        }
        address[0] = 2;
        address[1] = 0;
        unsafe {
            (a[1] as *mut [u8; 16]).write_unaligned(address);
            (a[2] as *mut u32).write_unaligned(16);
        }
    } else {
        image
            .cpu
            .winsock
            .borrow_mut()
            .sockets
            .get_mut(&a[0])
            .unwrap()
            .bound = true;
    }
    0
}
fn connect_to(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if a[2] != 16 {
        return failure(10022);
    }
    checked(image, a[1], 16, false);
    let mut address = unsafe { (a[1] as *const [u8; 16]).read_unaligned() };
    if u16::from_le_bytes([address[0], address[1]]) != 2 {
        return failure(10047);
    }
    if address[4] != 127 {
        return failure(10045);
    } // Initial direct backend admits loopback only.
    address[0] = 16;
    address[1] = 2;
    let fd = match begin(image, a[0]) {
        Ok(fd) => fd,
        Err(e) => return failure(e),
    };
    let result = thread32::without_cpu(image, || {
        let n = unsafe { connect(fd.as_raw_fd(), address.as_ptr(), 16) };
        if n < 0 { Err(native_error()) } else { Ok(n) }
    });
    match finish(image, a[0], result) {
        Ok(n) => n as u32,
        Err(e) => failure(e),
    }
}
/// Real bounded IPv4 datagrams. Receive truncation remains an explicit failure
/// boundary until its partial-write behavior is independently measured.
pub(super) fn datagram(image: &LoadedImage32<'_>, a: &[u32; 8], receive: bool) -> u32 {
    if a[3] != 0 {
        unsupported("datagram message flags are outside the zero-flags profile");
    }
    if a[2] > 1024 * 1024 {
        return failure(10040);
    }
    checked(image, a[1], a[2] as usize, receive);
    let mut address = [0u8; 16];
    let mut length = 16;
    if receive {
        if (a[4] == 0) != (a[5] == 0) {
            unsupported("recvfrom address and extent must both be supplied or both omitted");
        }
        if a[4] != 0 {
            checked(image, a[5], 4, true);
            length = unsafe { (a[5] as *const u32).read_unaligned() };
            if length < 16 {
                return failure(10014);
            }
            checked(image, a[4], 16, true);
            length = 16;
        }
    } else {
        if a[5] != 16 {
            unsupported("sendto currently requires an explicit IPv4 destination");
        }
        checked(image, a[4], 16, false);
        address = unsafe { (a[4] as *const [u8; 16]).read_unaligned() };
        if u16::from_le_bytes([address[0], address[1]]) != 2 {
            return failure(10047);
        }
        address[0] = 16;
        address[1] = 2;
    }
    let mut buffer = vec![0u8; a[2] as usize];
    if !receive && !buffer.is_empty() {
        unsafe {
            std::ptr::copy_nonoverlapping(a[1] as *const u8, buffer.as_mut_ptr(), buffer.len());
        }
    }
    let fd = match begin(image, a[0]) {
        Ok(fd) => fd,
        Err(error) => return failure(error),
    };
    if image.cpu.winsock.borrow().sockets.get(&a[0]).unwrap().kind != 2 {
        finish(image, a[0], Ok(())).unwrap();
        unsupported("datagram gateways currently require a UDP socket");
    }
    let mut truncated = 0;
    let mut error = 0;
    let result = thread32::without_cpu(image, || {
        let n = unsafe {
            if receive {
                alkyhol_udp_receive(
                    fd.as_raw_fd(),
                    buffer.as_mut_ptr(),
                    a[2],
                    address.as_mut_ptr(),
                    &mut length,
                    &mut truncated,
                    &mut error,
                )
            } else {
                sendto(
                    fd.as_raw_fd(),
                    buffer.as_ptr(),
                    buffer.len(),
                    0,
                    address.as_ptr(),
                    16,
                ) as i64
            }
        };
        if n < 0 {
            Err(if receive {
                errno(error)
            } else {
                native_error()
            })
        } else {
            Ok(n as usize)
        }
    });
    let n = match finish(image, a[0], result) {
        Ok(n) => n,
        Err(error) => return failure(error),
    };
    if truncated != 0 {
        unsupported("UDP receive truncation requires measured partial-write semantics");
    }
    if n > buffer.len() {
        unsupported("native datagram count exceeded supplied storage");
    }
    if receive {
        checked(image, a[1], n, true);
        if n != 0 {
            unsafe {
                std::ptr::copy_nonoverlapping(buffer.as_ptr(), a[1] as *mut u8, n);
            }
        }
        if a[4] != 0 {
            if length != 16 || address[0] != 16 || address[1] != 2 {
                unsupported("datagram source is outside the IPv4 sockaddr profile");
            }
            address[0] = 2;
            address[1] = 0;
            checked(image, a[4], 16, true);
            checked(image, a[5], 4, true);
            unsafe {
                (a[4] as *mut [u8; 16]).write_unaligned(address);
                (a[5] as *mut u32).write_unaligned(16);
            }
        }
    }
    n as u32
}
/// The reached scatter descriptor contains one WSABUF. Immediate non-overlapped
/// receives reuse the actual UDP path; asynchronous completion is never invented.
pub(super) fn wsa_receive(image: &LoadedImage32<'_>, a: &[u32; 16]) -> u32 {
    if a[7] != 0 || a[8] != 0 {
        let mut state = image.cpu.winsock.borrow_mut();
        if !state.logged_receive_limit {
            eprintln!(
                "PE32 WSARecvFrom asynchronous completion unavailable: WSAEOPNOTSUPP (10045)"
            );
            state.logged_receive_limit = true;
        }
        return failure(10045);
    }
    if a[2] != 1 {
        eprintln!(
            "PE32 WSARecvFrom scatter count outside implemented one-buffer profile: {}",
            a[2]
        );
        return failure(10045);
    }
    if a[3] == 0 || a[4] == 0 {
        return failure(10014);
    }
    checked(image, a[1], 8, false);
    checked(image, a[3], 4, true);
    checked(image, a[4], 4, true);
    let descriptor = unsafe { (a[1] as *const [u32; 2]).read_unaligned() };
    let flags = unsafe { (a[4] as *const u32).read_unaligned() };
    let count = datagram(
        image,
        &[a[0], descriptor[1], descriptor[0], flags, a[5], a[6], 0, 0],
        true,
    );
    if count == u32::MAX {
        return count;
    }
    checked(image, a[3], 4, true);
    checked(image, a[4], 4, true);
    unsafe {
        (a[3] as *mut u32).write_unaligned(count);
        (a[4] as *mut u32).write_unaligned(0);
    }
    0
}
fn transfer(image: &LoadedImage32<'_>, a: &[u32; 8], writing: bool) -> u32 {
    if a[3] != 0 {
        return failure(10045);
    }
    if a[2] > 1024 * 1024 {
        return failure(10040);
    }
    checked(image, a[1], a[2] as usize, !writing);
    let mut buffer = vec![0u8; a[2] as usize];
    if writing {
        unsafe {
            std::ptr::copy_nonoverlapping(a[1] as *const u8, buffer.as_mut_ptr(), buffer.len());
        }
    }
    let fd = match begin(image, a[0]) {
        Ok(fd) => fd,
        Err(e) => return failure(e),
    };
    let result = thread32::without_cpu(image, || {
        let n = unsafe {
            if writing {
                send(fd.as_raw_fd(), buffer.as_ptr(), buffer.len(), 0)
            } else {
                recv(fd.as_raw_fd(), buffer.as_mut_ptr(), buffer.len(), 0)
            }
        };
        if n < 0 {
            Err(native_error())
        } else {
            Ok(n as usize)
        }
    });
    match finish(image, a[0], result) {
        Err(e) => failure(e),
        Ok(n) => {
            if !writing && n != 0 {
                checked(image, a[1], n, true);
                unsafe {
                    std::ptr::copy_nonoverlapping(buffer.as_ptr(), a[1] as *mut u8, n);
                }
            }
            n as u32
        }
    }
}
fn control(image: &LoadedImage32<'_>, a: &[u32; 8]) -> u32 {
    if !matches!(a[1], 0x8004667e | 0x4004667f) {
        return failure(10045);
    }
    checked(image, a[2], 4, true);
    let fd = match begin(image, a[0]) {
        Ok(fd) => fd,
        Err(e) => return failure(e),
    };
    let mut value = unsafe { (a[2] as *const u32).read_unaligned() };
    if a[1] == 0x8004667e {
        value = u32::from(value != 0);
    }
    let n = unsafe {
        ioctl(
            fd.as_raw_fd(),
            a[1] as std::ffi::c_ulong,
            std::ptr::from_mut(&mut value),
        )
    };
    let result = if n < 0 { Err(native_error()) } else { Ok(n) };
    match finish(image, a[0], result) {
        Err(e) => failure(e),
        Ok(n) => {
            if a[1] == 0x4004667f {
                unsafe {
                    (a[2] as *mut u32).write_unaligned(value);
                }
            }
            n as u32
        }
    }
}
struct Host {
    name: Vec<u8>,
    aliases: Vec<Vec<u8>>,
    addresses: Vec<[u8; 4]>,
}
unsafe fn copy_host(pointer: *const Hostent) -> Result<Host> {
    let h = unsafe { pointer.as_ref() }.ok_or("null native host result")?;
    if h.family != 2 || h.length != 4 || h.aliases.is_null() || h.addresses.is_null() {
        return Err("native hostname result is outside IPv4".into());
    }
    unsafe fn text(p: *const c_char) -> Result<Vec<u8>> {
        if p.is_null() {
            return Err("null native hostname text".into());
        }
        let s = unsafe { CStr::from_ptr(p) }.to_bytes();
        if s.len() > 1024 || !s.is_ascii() {
            return Err("native hostname text outside ASCII bounds".into());
        }
        Ok(s.to_vec())
    }
    let name = unsafe { text(h.name) }?;
    let mut aliases = Vec::new();
    let mut addresses = Vec::new();
    for i in 0..=64 {
        let p = unsafe { h.aliases.add(i).read() };
        if p.is_null() {
            break;
        }
        if i == 64 {
            return Err("native hostname alias budget".into());
        }
        aliases.push(unsafe { text(p) }?);
    }
    for i in 0..=64 {
        let p = unsafe { h.addresses.add(i).read() };
        if p.is_null() {
            break;
        }
        if i == 64 {
            return Err("native hostname address budget".into());
        }
        addresses.push(unsafe { p.cast::<[u8; 4]>().read_unaligned() });
    }
    if addresses.is_empty() {
        return Err("native hostname has no IPv4 result".into());
    }
    Ok(Host {
        name,
        aliases,
        addresses,
    })
}
fn pack_host(host: &Host, base: u32) -> Result<Vec<u8>> {
    let mut b = vec![0u8; 16];
    b[8..12].copy_from_slice(&[2, 0, 4, 0]);
    fn append(b: &mut Vec<u8>, base: u32, data: &[u8], zero: bool) -> Result<u32> {
        let p = base
            .checked_add(b.len() as u32)
            .ok_or("hostent pointer overflow")?;
        b.extend_from_slice(data);
        if zero {
            b.push(0);
        }
        if b.len() > PAGE {
            return Err("guest hostent exceeds one page".into());
        }
        Ok(p)
    }
    let name = append(&mut b, base, &host.name, true)?;
    b[..4].copy_from_slice(&name.to_le_bytes());
    let mut aliases = Vec::new();
    for alias in &host.aliases {
        aliases.push(append(&mut b, base, alias, true)?);
    }
    let mut addresses = Vec::new();
    for address in &host.addresses {
        addresses.push(append(&mut b, base, address, false)?);
    }
    while !b.len().is_multiple_of(4) {
        b.push(0);
    }
    let list = base + b.len() as u32;
    for p in aliases.into_iter().chain([0]) {
        b.extend_from_slice(&p.to_le_bytes());
    }
    b[4..8].copy_from_slice(&list.to_le_bytes());
    let list = base + b.len() as u32;
    for p in addresses.into_iter().chain([0]) {
        b.extend_from_slice(&p.to_le_bytes());
    }
    b[12..16].copy_from_slice(&list.to_le_bytes());
    if b.len() > PAGE {
        return Err("guest hostent lists exceed one page".into());
    }
    Ok(b)
}
fn lookup(image: &LoadedImage32<'_>, a: &[u32; 8], reverse: bool) -> u32 {
    if !ready(image) {
        failure(10093);
        return 0;
    }
    let name = if reverse {
        if a[1] != 4 || a[2] != 2 {
            failure(10022);
            return 0;
        }
        checked(image, a[0], 4, false);
        None
    } else {
        Some(CString::new(bulk32::ascii(image, a[0])).unwrap())
    };
    let address = if reverse {
        unsafe { (a[0] as *const [u8; 4]).read_unaligned() }
    } else {
        [0; 4]
    };
    let result = thread32::without_cpu(image, || {
        let mut resolver_error = 0i32;
        let pointer = unsafe {
            if let Some(name) = &name {
                getipnodebyname(name.as_ptr(), 2, 0, std::ptr::from_mut(&mut resolver_error))
            } else {
                getipnodebyaddr(
                    address.as_ptr().cast(),
                    4,
                    2,
                    std::ptr::from_mut(&mut resolver_error),
                )
            }
        };
        if pointer.is_null() {
            let e = resolver_error;
            Err(match e {
                1 => 11001,
                2 => 11002,
                3 => 11003,
                _ => 11004,
            })
        } else {
            let copied = unsafe { copy_host(pointer) };
            unsafe {
                freehostent(pointer);
            }
            copied.map_err(|e| unsupported(&e))
        }
    });
    let host = match result {
        Ok(h) => h,
        Err(e) => {
            failure(e);
            return 0;
        }
    };
    let id = alkyhol::ffi::GetCurrentThreadId();
    let mut state = image.cpu.winsock.borrow_mut();
    state.hosts.entry(id).or_insert_with(|| {
        image
            .cpu
            .mapping(PAGE, PAGE, 0)
            .unwrap_or_else(|e| unsupported(&e))
    });
    let memory = state.hosts.get(&id).unwrap();
    let bytes = pack_host(&host, memory.base as u32).unwrap_or_else(|e| unsupported(&e));
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), memory.base, bytes.len());
    }
    memory.base as u32
}
pub(super) fn dispatch(image: &LoadedImage32<'_>, id: u32, a: &[u32; 8]) -> u32 {
    match id {
        0 => startup(image, a),
        1 => cleanup(image),
        2 => alkyhol::ffi::GetLastError(),
        3 => create(image, a),
        4 => close(image, a[0]),
        5 => connect_to(image, a),
        6 => (a[0] as u16).swap_bytes() as u32,
        7 => {
            let text = CString::new(bulk32::ascii(image, a[0])).unwrap();
            unsafe { inet_addr(text.as_ptr()) }
        }
        8 => control(image, a),
        9 => transfer(image, a, false),
        10 => transfer(image, a, true),
        11 => lookup(image, a, false),
        12 => lookup(image, a, true),
        _ => unreachable!(),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hostent_uses_only_guest_pointers_and_four_byte_address_entries() {
        let host = Host {
            name: b"localhost".to_vec(),
            aliases: vec![b"alias".to_vec()],
            addresses: vec![[127, 0, 0, 1], [127, 0, 0, 2]],
        };
        let b = pack_host(&host, 0x10000).unwrap();
        let word = |p: usize| u32::from_le_bytes(b[p..p + 4].try_into().unwrap());
        assert_eq!(&b[8..12], &[2, 0, 4, 0]);
        assert_eq!(word(0), 0x10010);
        let aliases = (word(4) - 0x10000) as usize;
        assert_eq!(word(aliases + 4), 0);
        let addresses = (word(12) - 0x10000) as usize;
        assert_eq!(word(addresses + 8), 0);
        let first = (word(addresses) - 0x10000) as usize;
        assert_eq!(&b[first..first + 4], &[127, 0, 0, 1]);
        assert!(
            pack_host(
                &Host {
                    name: vec![b'x'; PAGE],
                    aliases: vec![],
                    addresses: vec![[0; 4]]
                },
                0x10000
            )
            .is_err()
        );
    }
    #[test]
    fn native_socket_errors_never_turn_into_success() {
        for (a, b) in [
            (35, 10035),
            (36, 10036),
            (61, 10061),
            (9, 10038),
            (13, 10013),
            (32, 10058),
            (999, 10045),
        ] {
            assert_eq!(errno(a), b);
        }
    }
}
