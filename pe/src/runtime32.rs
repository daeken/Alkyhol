//! Experimental PE32 integer/stdcall path in a 64-bit Rosetta host.
//! Minimal FS/TEB, explicit shared-thread DLL dependencies and EXE static TLS.
//! Bounded FLS callbacks and explicit process graphs; no PEB, general guest
//! threading/fibers, filesystem DLL search or general exception dispatch.
use crate::{ExportTable, ExportTarget, Image, ImportSymbol, ProcessContext, Result};
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    marker::PhantomData,
    ptr,
    rc::Rc,
    sync::{Condvar, Mutex, MutexGuard, OnceLock},
};
#[path = "raise32.rs"]
mod raise32;
#[path = "section32.rs"]
mod section32;
#[path = "seh32.rs"]
mod seh32;
pub use seh32::ExceptionImage32;
#[path = "activation32.rs"]
mod activation32;
#[path = "ansi32.rs"]
mod ansi32;
#[path = "bulk32.rs"]
mod bulk32;
#[path = "com32.rs"]
mod com32;
#[path = "copy_file32.rs"]
mod copy_file32;
#[path = "crt32.rs"]
mod crt32;
#[path = "cursor32.rs"]
mod cursor32;
#[path = "diagnostic32.rs"]
mod diagnostic32;
#[path = "directory32.rs"]
mod directory32;
#[path = "disk32.rs"]
mod disk32;
#[path = "dsound32.rs"]
mod dsound32;
#[path = "event32.rs"]
mod event32;
#[path = "exception_filter32.rs"]
mod exception_filter32;
#[path = "fiber32.rs"]
mod fiber32;
#[path = "fileio32.rs"]
mod fileio32;
#[path = "files32.rs"]
mod files32;
#[path = "gdi32.rs"]
mod gdi32;
#[path = "heap32.rs"]
mod heap32;
#[path = "icon32.rs"]
mod icon32;
#[path = "netbios32.rs"]
mod netbios32;
#[path = "perf32.rs"]
mod perf32;
#[path = "sound32.rs"]
mod sound32;
pub use icon32::IconImage32;
#[path = "d3d9_32.rs"]
mod d3d9_32;
#[path = "image32.rs"]
mod image32;
#[path = "ini32.rs"]
mod ini32;
#[path = "input8_32.rs"]
mod input8_32;
#[path = "loader32.rs"]
mod loader32;
#[path = "machine32.rs"]
mod machine32;
#[path = "math32.rs"]
mod math32;
#[path = "module_path32.rs"]
mod module_path32;
#[path = "mutex32.rs"]
mod mutex32;
#[path = "nls32.rs"]
mod nls32;
#[path = "number_format32.rs"]
mod number_format32;
#[path = "path32.rs"]
mod path32;
#[path = "private_profile32.rs"]
mod private_profile32;
#[path = "process32.rs"]
mod process32;
#[path = "registry32.rs"]
mod registry32;
#[path = "shell_folder32.rs"]
mod shell_folder32;
#[path = "startup32.rs"]
mod startup32;
#[path = "sync32.rs"]
mod sync32;
#[path = "thread32.rs"]
mod thread32;
#[path = "time32.rs"]
mod time32;
#[path = "ui32.rs"]
mod ui32;
#[path = "unnamed_mutex32.rs"]
mod unnamed_mutex32;
#[path = "watch32.rs"]
mod watch32;
#[path = "wave32.rs"]
mod wave32;
#[path = "window_host32.rs"]
mod window_host32;
#[path = "winsock32.rs"]
mod winsock32;
pub use process32::{ProcessImage32, ProcessOptions32, TimeZone32};
#[cfg(target_arch = "x86_64")]
std::arch::global_asm!(include_str!("transition32.S"), options(att_syntax));
#[cfg(target_arch = "aarch64")]
#[path = "executor32.rs"]
pub(crate) mod executor32;
#[cfg(target_arch = "aarch64")]
mod guest_templates {
    include!(concat!(env!("OUT_DIR"), "/guest_templates.rs"));
}
#[cfg(target_arch = "aarch64")]
use executor32::enter as alkyhol_enter_pe32;
unsafe extern "C" {
    fn mmap(
        address: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: i64,
    ) -> *mut c_void;
    fn alkyhol_pe32_allocate_hole(
        minimum: u64,
        end: u64,
        len: u64,
        alignment: u64,
        output: *mut u64,
    ) -> i32;
    fn mprotect(address: *mut c_void, len: usize, prot: i32) -> i32;
    fn munmap(address: *mut c_void, len: usize) -> i32;
    fn getpagesize() -> i32;
    fn getentropy(buffer: *mut c_void, len: usize) -> i32;
    fn i386_get_ldt(start: i32, entries: *mut u64, count: i32) -> i32;
    fn i386_set_ldt(start: i32, entries: *const u64, count: i32) -> i32;
    #[cfg(target_arch = "x86_64")]
    fn alkyhol_enter_pe32(entry: u32, stack: u32, code: u32, data: u32) -> u64;
    #[cfg(target_arch = "x86_64")]
    static alkyhol_pe32_gate_start: u8;
    #[cfg(target_arch = "x86_64")]
    static alkyhol_pe32_gate_end: u8;
    #[cfg(target_arch = "x86_64")]
    static alkyhol_pe32_dispatch_slot: u8;
}
#[cfg(target_arch = "aarch64")]
const GUEST_BIAS: usize = 0x8000000000;
#[cfg(target_arch = "aarch64")]
fn host_address(address: usize) -> usize {
    if address != 0 && address <= u32::MAX as usize {
        GUEST_BIAS + address
    } else {
        address
    }
}
#[cfg(target_arch = "aarch64")]
fn guest_address(address: usize) -> usize {
    if (GUEST_BIAS..GUEST_BIAS + 0x100000000).contains(&address) {
        address - GUEST_BIAS
    } else {
        address
    }
}
fn host_code_address(address: u32) -> usize {
    #[cfg(target_arch = "aarch64")]
    {
        host_address(address as usize)
    }
    #[cfg(target_arch = "x86_64")]
    {
        address as usize
    }
}
const PAGE: usize = 4096;
const STACK: usize = 1024 * 1024;
const APIS: [(&str, u16); 320] = [
    ("GetLastError", 0),
    ("SetLastError", 4),
    ("GetStdHandle", 4),
    ("WriteFile", 20),
    ("GetModuleHandleA", 4),
    ("GetModuleHandleW", 4),
    ("GetProcAddress", 8),
    ("GetCommandLineW", 0),
    ("CommandLineToArgvW", 8),
    ("CloseHandle", 4),
    ("ReadFile", 20),
    ("CreateFileW", 28),
    ("MultiByteToWideChar", 24),
    ("WideCharToMultiByte", 32),
    ("ExitProcess", 4),
    ("LocalFree", 4),
    ("GetProcessHeap", 0),
    ("HeapAlloc", 12),
    ("HeapFree", 12),
    ("GetVersionExA", 4),
    ("HeapCreate", 12),
    ("GetEnvironmentVariableA", 12),
    ("FlsAlloc", 4),
    ("FlsFree", 4),
    ("FlsGetValue", 4),
    ("FlsSetValue", 8),
    ("TlsAlloc", 0),
    ("TlsFree", 4),
    ("TlsGetValue", 4),
    ("TlsSetValue", 8),
    ("EncodePointer", 4),
    ("DecodePointer", 4),
    ("InitializeCriticalSectionAndSpinCount", 8),
    ("InitializeCriticalSection", 4),
    ("EnterCriticalSection", 4),
    ("LeaveCriticalSection", 4),
    ("DeleteCriticalSection", 4),
    ("TryEnterCriticalSection", 4),
    ("InterlockedIncrement", 4),
    ("GetCurrentThreadId", 0),
    ("GetStartupInfoA", 4),
    ("GetFileType", 4),
    ("SetHandleCount", 4),
    ("GetEnvironmentStringsW", 0),
    ("FreeEnvironmentStringsW", 4),
    ("GetCommandLineA", 0),
    ("GetACP", 0),
    ("IsValidCodePage", 4),
    ("GetCPInfo", 8),
    ("GetStringTypeW", 16),
    ("LCMapStringW", 24),
    ("InterlockedDecrement", 4),
    ("IsProcessorFeaturePresent", 4),
    ("GetSystemTimeAsFileTime", 4),
    ("HeapSize", 12),
    ("FindActCtxSectionStringW", 20),
    ("GetModuleFileNameW", 12),
    ("GetLongPathNameW", 12),
    ("GetSystemDirectoryW", 8),
    ("GetCurrentActCtx", 4),
    ("ReleaseActCtx", 4),
    ("CreateActCtxW", 4),
    ("ActivateActCtx", 8),
    ("DeactivateActCtx", 8),
    ("GetSystemWindowsDirectoryW", 8),
    ("FindFirstFileW", 8),
    ("FindNextFileW", 8),
    ("FindClose", 4),
    ("FileTimeToLocalFileTime", 8),
    ("GetTimeZoneInformation", 4),
    ("FileTimeToSystemTime", 8),
    ("GetCurrentProcessId", 0),
    ("GetTickCount", 0),
    ("QueryPerformanceCounter", 4),
    ("QueryPerformanceFrequency", 4),
    ("InterlockedCompareExchange", 12),
    ("InterlockedExchange", 8),
    ("DisableThreadLibraryCalls", 4),
    ("malloc", 4),
    ("_set_error_mode", 4),
    ("?set_terminate@@YAP6AXXZP6AXXZ@Z", 4),
    ("_get_terminate", 0),
    ("?set_unexpected@@YAP6AXXZP6AXXZ@Z", 4),
    ("_get_unexpected", 0),
    ("?terminate@@YAXXZ", 0),
    ("?unexpected@@YAXXZ", 0),
    ("__set_app_type", 4),
    ("GetVersion", 0),
    ("_initterm", 8),
    ("_CIsqrt", 0),
    ("<private x87 rejection>", 8),
    ("_errno", 0),
    ("_CIatan", 0),
    ("<private atan rejection>", 8),
    ("_lock", 4),
    ("_unlock", 4),
    ("__dllonexit", 12),
    ("GetStartupInfoW", 4),
    ("GetModuleFileNameA", 12),
    ("HeapReAlloc", 16),
    ("GetCursorPos", 4),
    ("GetWindowsDirectoryA", 8),
    ("GetSystemDirectoryA", 8),
    ("SetUnhandledExceptionFilter", 4),
    ("HeapLock", 4),
    ("HeapUnlock", 4),
    ("GetSystemInfo", 4),
    ("RegOpenKeyExW", 20),
    ("RegCloseKey", 4),
    ("CreateMutexW", 12),
    ("timeGetTime", 0),
    ("SetCurrentDirectoryW", 4),
    ("GetCurrentDirectoryW", 8),
    ("WaitForSingleObjectEx", 12),
    ("ReleaseMutex", 4),
    ("LoadIconW", 8),
    ("CreateEventW", 16),
    ("SetEvent", 4),
    ("ResetEvent", 4),
    ("CreateSemaphoreW", 16),
    ("SHGetFolderPathW", 20),
    ("GetPrivateProfileStringW", 24),
    ("CreateDirectoryW", 8),
    ("WritePrivateProfileStringW", 16),
    ("ReleaseSemaphore", 12),
    ("CreateEventA", 16),
    ("CreateSemaphoreA", 16),
    ("InterlockedExchangeAdd", 8),
    ("GetFileSize", 8),
    ("GetFileSizeEx", 8),
    ("GetCurrentThread", 0),
    ("CreateDirectoryA", 8),
    ("SetCurrentDirectoryA", 4),
    ("GetCurrentDirectoryA", 8),
    ("GetFileAttributesW", 4),
    ("GetFileAttributesA", 4),
    ("WaitForSingleObject", 8),
    ("CreateFileA", 28),
    ("GetFullPathNameW", 16),
    ("GetFullPathNameA", 16),
    ("FindFirstFileA", 8),
    ("FindNextFileA", 8),
    ("Sleep", 4),
    ("SetFilePointer", 16),
    ("SetFilePointerEx", 20),
    ("FlushFileBuffers", 4),
    ("GetFileInformationByHandle", 8),
    ("SetEndOfFile", 4),
    ("RegisterClassExW", 4),
    ("RegisterClassA", 4),
    ("UnregisterClassA", 8),
    ("CreateWindowExW", 48),
    ("CreateWindowExA", 48),
    ("DefWindowProcW", 16),
    ("DefWindowProcA", 16),
    ("AdjustWindowRect", 12),
    ("AdjustWindowRectEx", 16),
    ("ShowWindow", 8),
    ("UpdateWindow", 4),
    ("GetClientRect", 8),
    ("GetWindowRect", 8),
    ("SetWindowPos", 28),
    ("SetWindowTextW", 8),
    ("IsWindowVisible", 4),
    ("GetDesktopWindow", 0),
    ("GetSystemMetrics", 4),
    ("GetActiveWindow", 0),
    ("GetWindowLongW", 8),
    ("GetWindowLongA", 8),
    ("SetWindowLongW", 12),
    ("DestroyWindow", 4),
    ("SetForegroundWindow", 4),
    ("SetRect", 20),
    ("ClientToScreen", 8),
    ("ScreenToClient", 8),
    ("GetWindowThreadProcessId", 8),
    ("GetClassLongA", 8),
    ("RaiseException", 16),
    ("GetCurrentProcess", 0),
    ("GetProcessAffinityMask", 12),
    ("GetProcessId", 4),
    ("CreateThread", 24),
    ("GetExitCodeThread", 8),
    ("GetThreadId", 4),
    ("ResumeThread", 4),
    ("ExitThread", 4),
    ("SetThreadPriority", 8),
    ("LoadBitmapW", 8),
    ("GetObjectW", 12),
    ("DeleteObject", 4),
    ("CreateCompatibleDC", 4),
    ("DeleteDC", 4),
    ("SelectObject", 8),
    ("GetDIBits", 28),
    ("CreateCompatibleBitmap", 12),
    ("BitBlt", 36),
    ("GetPixel", 12),
    ("GetCurrentObject", 8),
    ("GetDC", 4),
    ("ReleaseDC", 8),
    ("BeginPaint", 8),
    ("EndPaint", 8),
    ("CreateDIBSection", 24),
    ("PeekMessageW", 20),
    ("PeekMessageA", 20),
    ("DispatchMessageW", 4),
    ("TranslateMessage", 4),
    ("PostQuitMessage", 4),
    ("PostMessageW", 16),
    ("PostMessageA", 16),
    ("InvalidateRect", 12),
    ("ValidateRect", 8),
    ("ReadDirectoryChangesW", 32),
    ("GetOverlappedResult", 16),
    ("CancelIo", 4),
    ("GetAdaptersInfo", 8),
    ("RegOpenKeyExA", 20),
    ("SwitchToThread", 0),
    ("MessageBoxW", 16),
    ("MessageBoxA", 16),
    ("RegQueryValueExA", 24),
    ("RegQueryValueExW", 24),
    ("LoadLibraryA", 4),
    ("LoadLibraryW", 4),
    ("FreeLibrary", 4),
    ("WSAStartup", 8),
    ("WSACleanup", 0),
    ("WSAGetLastError", 0),
    ("socket", 12),
    ("closesocket", 4),
    ("connect", 12),
    ("htons", 4),
    ("inet_addr", 4),
    ("ioctlsocket", 12),
    ("recv", 16),
    ("send", 16),
    ("gethostbyname", 4),
    ("gethostbyaddr", 12),
    ("GetVersionExW", 4),
    ("CreateFileMappingA", 24),
    ("CreateFileMappingW", 24),
    ("OpenFileMappingA", 12),
    ("OpenFileMappingW", 12),
    ("MapViewOfFile", 20),
    ("UnmapViewOfFile", 4),
    ("SetThreadAffinityMask", 8),
    ("SetProcessAffinityMask", 8),
    ("GetStockObject", 4),
    ("CreateSolidBrush", 4),
    ("FillRect", 12),
    ("LoadImageW", 24),
    ("LoadImageA", 24),
    ("SetCursor", 4),
    ("DestroyCursor", 4),
    ("GetCursor", 0),
    ("Direct3DCreate9", 4),
    ("UnregisterClassW", 8),
    ("GetWindowInfo", 8),
    ("ClipCursor", 4),
    ("DirectInput8Create", 20),
    ("??2@YAPAXI@Z", 4),
    ("??3@YAXPAX@Z", 4),
    ("??_U@YAPAXI@Z", 4),
    ("??_V@YAXPAX@Z", 4),
    ("free", 4),
    ("memcpy", 12),
    ("memmove", 12),
    ("memset", 12),
    ("strchr", 8),
    ("strncmp", 12),
    ("CreateMutexA", 12),
    ("GetHandleInformation", 8),
    ("CoInitialize", 4),
    ("CoInitializeEx", 8),
    ("CoUninitialize", 0),
    ("GdiFlush", 0),
    ("CoCreateInstance", 20),
    ("waveOutGetNumDevs", 0),
    ("waveInGetNumDevs", 0),
    ("WSAWaitForMultipleEvents", 20),
    ("WSACreateEvent", 0),
    ("WSASetEvent", 4),
    ("WSAResetEvent", 4),
    ("WSACloseEvent", 4),
    ("GetDiskFreeSpaceExA", 16),
    ("OpenSCManagerW", 12),
    ("setsockopt", 20),
    ("getsockopt", 20),
    ("bind", 12),
    ("getsockname", 12),
    ("sendto", 24),
    ("recvfrom", 24),
    ("WSARecvFrom", 36),
    ("Netbios", 4),
    ("select", 20),
    ("timeBeginPeriod", 4),
    ("timeEndPeriod", 4),
    ("WaitForMultipleObjects", 16),
    ("RegOpenKeyA", 12),
    ("SetCapture", 4),
    ("ReleaseCapture", 0),
    ("GetCapture", 0),
    ("setlocale", 8),
    ("_strdup", 4),
    ("GetLocalTime", 4),
    ("CopyFileW", 12),
    ("GetNumberFormatW", 24),
    ("GetClipCursor", 4),
    ("ShowCursor", 4),
    ("DeleteFileW", 4),
    ("DeleteFileA", 4),
    ("DirectSoundCreate", 12),
    ("DirectSoundCreate8", 12),
    ("DirectSoundEnumerateA", 8),
    ("DirectSoundEnumerateW", 8),
    ("GetDeviceID", 8),
    ("DirectSoundCaptureCreate", 12),
    ("DirectSoundCaptureCreate8", 12),
    ("DirectSoundCaptureEnumerateA", 8),
    ("DirectSoundCaptureEnumerateW", 8),
];
// The first 120 API slots retain their original offsets. The second bank
// begins after both x87 bodies, leaving their machine code and failure gates
// fixed for every existing import and frozen PE observation.
const fn api_offset(id: usize) -> usize {
    const FIRST_BANK: usize = (math32::OFFSET - 256) / 32;
    if id < FIRST_BANK {
        256 + id * 32
    } else if id < 232 {
        math32::ATAN_OFFSET + 256 + (id - FIRST_BANK) * 32
    } else {
        // Keep the captured-context SEH body and its private service slot fixed.
        raise32::OFFSET + 512 + (id - 232) * 32
    }
}
const EXIT_API: u32 = 14;
// Win32 promises a reversible process-specific representation, not a published
// bit pattern. Keep every operation DWORD-wide; encoded values are opaque.
fn pointer_transform(value: u32, cookie: u32, decode: bool) -> u32 {
    if decode {
        value.rotate_left(cookie & 31) ^ cookie
    } else {
        (value ^ cookie).rotate_right(cookie & 31)
    }
}
fn api_id(module: &str, symbol: &ImportSymbol) -> Option<usize> {
    let module = if module.eq_ignore_ascii_case("wsock32.dll") {
        "ws2_32.dll"
    } else {
        module
    };
    if module.eq_ignore_ascii_case("ws2_32.dll")
        && let ImportSymbol::Ordinal(n) = symbol
    {
        return match n {
            115 => Some(225),
            116 => Some(226),
            111 => Some(227),
            2 => Some(288),
            6 => Some(289),
            20 => Some(290),
            17 => Some(291),
            18 => Some(294),
            21 => Some(286),
            7 => Some(287),
            23 => Some(228),
            3 => Some(229),
            4 => Some(230),
            9 => Some(231),
            11 => Some(232),
            10 => Some(233),
            16 => Some(234),
            19 => Some(235),
            52 => Some(236),
            51 => Some(237),
            _ => None,
        };
    }
    let ImportSymbol::Name(symbol) = symbol else {
        return None;
    };
    APIS.iter()
        .enumerate()
        .find(|(id, (name, _))| {
            !matches!(*id, 90 | 93)
                && *name == symbol
                && module.eq_ignore_ascii_case(if cdecl_api(*id) {
                    "msvcrt.dll"
                } else if *id == 259 {
                    "dinput8.dll"
                } else if (272..=274).contains(id) || *id == 276 {
                    "ole32.dll"
                } else if *id == 293 {
                    "netapi32.dll"
                } else if (311..320).contains(id) {
                    "dsound.dll"
                } else if *id == 255 {
                    "d3d9.dll"
                } else if matches!(*id, 8 | 120) {
                    "shell32.dll"
                } else if (188..=197).contains(id) || matches!(*id, 202 | 247 | 248 | 275) {
                    "gdi32.dll"
                } else if matches!(*id, 100 | 115 | 187)
                    || (148..177).contains(id)
                    || (198..=201).contains(id)
                    || (203..=211).contains(id)
                    || (218..=219).contains(id)
                    || (249..=254).contains(id)
                    || matches!(*id, 256..=258)
                    || matches!(*id, 299..=301)
                    || (307..=308).contains(id)
                {
                    "user32.dll"
                } else if matches!(*id, 107 | 108 | 216 | 220 | 221 | 285 | 298) {
                    "advapi32.dll"
                } else if (225..=237).contains(id)
                    || (279..=283).contains(id)
                    || (286..=292).contains(id)
                    || *id == 294
                {
                    "ws2_32.dll"
                } else if *id == 215 {
                    "iphlpapi.dll"
                } else if matches!(*id, 110 | 277 | 278 | 295 | 296) {
                    "winmm.dll"
                } else {
                    "kernel32.dll"
                })
        })
        .map(|(id, _)| id)
}
fn cdecl_api(id: usize) -> bool {
    matches!(id, 78..=86 | 88..=96 | 260..=269 | 302..=303)
}
fn cleanup_bytes(id: usize, arguments: u16) -> u16 {
    if cdecl_api(id) { 0 } else { arguments }
}
static SELECTORS: OnceLock<Result<(u16, u16)>> = OnceLock::new();
#[cfg(target_arch = "x86_64")]
fn selectors() -> Result<(u16, u16)> {
    SELECTORS
        .get_or_init(|| {
            // Rosetta aborts for LDT_AUTO_ALLOC. Inspect the task's existing table,
            // reserve two flat entries for this process lifetime and check the
            // third slot used only during an exclusively leased FS entry.
            let mut entries = [0u64; 3];
            let count = unsafe { i386_get_ldt(512, entries.as_mut_ptr(), 3) };
            if !(0..=8192).contains(&count) {
                return Err("cannot inspect task LDT".into());
            }
            if entries
                .iter()
                .enumerate()
                .any(|(index, &value)| count > 512 + index as i32 && value != 0)
            {
                return Err("PE32 host LDT slots 512/513/514 are already occupied".into());
            }
            let descriptors = [0x00cffa000000ffffu64, 0x00cff2000000ffff];
            for (index, descriptor) in descriptors.iter().enumerate() {
                // Rosetta also rejects installing multiple selectors at once.
                let slot = 512 + index as i32;
                if unsafe { i386_set_ldt(slot, descriptor, 1) } != slot {
                    return Err("cannot install flat 32-bit code/data descriptors".into());
                }
            }
            Ok((0x1007, 0x100f))
        })
        .clone()
}
// LDT entries belong to the process, while FS state belongs to a thread.
// Serialize CPU entries so a second thread cannot retarget the active TEB slot.
static FS_SLOT: Mutex<()> = Mutex::new(());
// FIFO admission prevents a polling main thread from starving behind workers.
static CPU_TICKETS: Mutex<(u64, u64)> = Mutex::new((0, 0));
static CPU_READY: Condvar = Condvar::new();
fn finish_cpu_ticket(ticket: u64) {
    let mut queue = CPU_TICKETS.lock().unwrap();
    if queue.1 != ticket {
        unsupported("CPU lease ticket completed out of order");
    }
    queue.1 = queue
        .1
        .checked_add(1)
        .unwrap_or_else(|| unsupported("CPU ticket namespace exhausted"));
    CPU_READY.notify_all();
}
#[cfg(target_arch = "aarch64")]
fn selectors() -> Result<(u16, u16)> {
    // Virtual ring-3 flat selectors; no physical host LDT is installed.
    Ok((0x1b, 0x23))
}
fn teb_descriptor(base: u32) -> u64 {
    0x0040f20000000fff
        | (u64::from(base & 0xffff) << 16)
        | (u64::from((base >> 16) & 0xff) << 32)
        | (u64::from(base >> 24) << 56)
}
struct TebLease {
    guard: Option<MutexGuard<'static, ()>>,
    ticket: u64,
}
impl TebLease {
    fn install(base: u32) -> Result<Self> {
        Self::acquire(base, false)
    }
    fn acquire(base: u32, wait: bool) -> Result<Self> {
        let (guard, ticket) = if !wait {
            let mut queue = CPU_TICKETS
                .lock()
                .map_err(|_| "CPU admission queue poisoned")?;
            if queue.0 != queue.1 {
                return Err("another PE32 entry owns the FS slot".into());
            }
            let guard = FS_SLOT
                .try_lock()
                .map_err(|_| "another PE32 entry owns the FS slot")?;
            let ticket = queue.0;
            queue.0 = queue
                .0
                .checked_add(1)
                .ok_or("CPU ticket namespace exhausted")?;
            (guard, ticket)
        } else {
            let mut queue = CPU_TICKETS
                .lock()
                .map_err(|_| "CPU admission queue poisoned")?;
            let ticket = queue.0;
            queue.0 = queue
                .0
                .checked_add(1)
                .ok_or("CPU ticket namespace exhausted")?;
            while queue.1 != ticket {
                if window_host32::is_main_thread() {
                    drop(queue);
                    window_host32::pump_tasks();
                    std::thread::sleep(std::time::Duration::from_micros(100));
                    queue = CPU_TICKETS
                        .lock()
                        .map_err(|_| "CPU admission queue poisoned")?;
                } else {
                    queue = CPU_READY
                        .wait(queue)
                        .map_err(|_| "CPU admission queue poisoned")?;
                }
            }
            drop(queue);
            let guard = FS_SLOT
                .lock()
                .unwrap_or_else(|_| unsupported("exclusive CPU lease poisoned"));
            (guard, ticket)
        };
        #[cfg(target_arch = "x86_64")]
        {
            let mut descriptor = 0u64;
            let count = unsafe { i386_get_ldt(514, &mut descriptor, 1) };
            if !(0..=8192).contains(&count) || (count > 514 && descriptor != 0) {
                drop(guard);
                finish_cpu_ticket(ticket);
                return Err("PE32 FS slot 514 is already occupied or cannot be inspected".into());
            }
            descriptor = teb_descriptor(base);
            if unsafe { i386_set_ldt(514, &descriptor, 1) } != 514 {
                drop(guard);
                finish_cpu_ticket(ticket);
                return Err("cannot install bounded PE32 TEB descriptor".into());
            }
        }
        Ok(Self {
            guard: Some(guard),
            ticket,
        })
    }
}
impl Drop for TebLease {
    fn drop(&mut self) {
        #[cfg(target_arch = "x86_64")]
        {
            let empty = 0u64;
            if unsafe { i386_set_ldt(514, &empty, 1) } != 514 {
                // Host FS is restored before this guard is dropped. Do not leave an
                // apparently reusable descriptor pointing at soon-to-be-freed memory.
                eprintln!("Alkyhol PE32 unsupported: cannot clear owned FS descriptor");
                std::process::abort();
            }
        }
        drop(self.guard.take());
        finish_cpu_ticket(self.ticket);
    }
}
#[cfg(target_arch = "x86_64")]
fn segment_state() -> [u16; 5] {
    let (mut ds, mut es, mut ss, mut fs, mut gs) = (0u16, 0u16, 0u16, 0u16, 0u16);
    unsafe {
        std::arch::asm!("mov {0:x}, ds", "mov {1:x}, es", "mov {2:x}, ss",
        "mov {3:x}, fs", "mov {4:x}, gs", out(reg) ds, out(reg) es, out(reg) ss,
        out(reg) fs, out(reg) gs, options(nomem, nostack, preserves_flags));
    }
    [ds, es, ss, fs, gs]
}
#[cfg(target_arch = "aarch64")]
fn segment_state() -> [u16; 5] {
    [0; 5]
}
fn teb_bytes(base: u32, stack_limit: u32, stack_size: u32) -> Result<[u8; PAGE]> {
    let stack_base = stack_limit
        .checked_add(stack_size)
        .ok_or("TEB stack extent overflow")?;
    if stack_size == 0 || u64::from(base) + PAGE as u64 > 0x100000000 {
        return Err("invalid TEB/stack extent".into());
    }
    let mut bytes = [0u8; PAGE];
    for (at, value) in [
        (0, u32::MAX),
        (4, stack_base),
        (8, stack_limit),
        (0x18, base),
    ] {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    Ok(bytes)
}
struct TebError(*mut u32);
impl Drop for TebError {
    fn drop(&mut self) {
        unsafe {
            self.0.write_unaligned(alkyhol::ffi::GetLastError());
        }
    }
}
struct LowMapping {
    base: *mut u8,
    len: usize,
}
impl LowMapping {
    fn new_in(
        len: usize,
        alignment: usize,
        hint: usize,
        bounds: Option<machine32::Bounds>,
    ) -> Result<Self> {
        let alignment = if bounds.is_some() {
            alignment.max(65536)
        } else {
            alignment
        };
        let first = Self::new(len, alignment, hint);
        let Some(bounds) = bounds else {
            return first;
        };
        match first {
            Ok(mapping) if bounds.contains(mapping.base as usize, mapping.len) => {
                return Ok(mapping);
            }
            Ok(mapping) => {
                eprintln!(
                    "PE32 bounded mapping retry: len={len:#x} alignment={alignment:#x} rejected={:#x} interval={:#x}..{:#x}",
                    mapping.base as usize, bounds.minimum, bounds.end
                );
                drop(mapping);
            }
            Err(error) => eprintln!("PE32 bounded mapping retry: len={len:#x} first_error={error}"),
        }
        let mut address = 0u64;
        let status = unsafe {
            alkyhol_pe32_allocate_hole(
                host_code_address(bounds.minimum as u32) as u64,
                {
                    #[cfg(target_arch = "x86_64")]
                    {
                        bounds.end
                    }
                    #[cfg(target_arch = "aarch64")]
                    {
                        GUEST_BIAS as u64 + bounds.end
                    }
                },
                len as u64,
                alignment as u64,
                &mut address,
            )
        };
        if status != 0 {
            return Err(format!(
                "no admitted PE32 address hole: len={len:#x} alignment={alignment:#x} interval={:#x}..{:#x} Mach status={status}",
                bounds.minimum, bounds.end
            ));
        }
        let mapping = Self {
            base: address as *mut u8,
            len,
        };
        if !bounds.contains(address as usize, len) || !(address as usize).is_multiple_of(alignment)
        {
            return Err("native hole allocator violated the PE32 bounds/alignment contract".into());
        }
        eprintln!("PE32 bounded mapping placed: address={address:#x} len={len:#x}");
        Ok(mapping)
    }
    #[cfg(target_arch = "aarch64")]
    fn new(len: usize, alignment: usize, hint: usize) -> Result<Self> {
        let page = unsafe { getpagesize() } as usize;
        let physical = len
            .checked_add(page - 1)
            .map(|v| v & !(page - 1))
            .ok_or("native mapping size overflow")?;
        let mut address = 0u64;
        let min = GUEST_BIAS as u64
            + if hint >= 65536 {
                hint as u32 as u64
            } else {
                65536
            };
        let mut status = unsafe {
            alkyhol_pe32_allocate_hole(
                min,
                GUEST_BIAS as u64 + 0x100000000,
                physical as u64,
                alignment.max(page) as u64,
                &mut address,
            )
        };
        if status != 0 && min != GUEST_BIAS as u64 + 65536 {
            status = unsafe {
                alkyhol_pe32_allocate_hole(
                    GUEST_BIAS as u64 + 65536,
                    GUEST_BIAS as u64 + 0x100000000,
                    physical as u64,
                    alignment.max(page) as u64,
                    &mut address,
                )
            };
        }
        if status != 0 {
            return Err(format!(
                "native low guest mapping failed: Mach status {status}"
            ));
        }
        Ok(Self {
            base: address as *mut u8,
            len,
        })
    }
    #[cfg(target_arch = "x86_64")]
    fn new(len: usize, alignment: usize, hint: usize) -> Result<Self> {
        let reserve = len
            .checked_add(alignment)
            .ok_or("low allocation overflow")?;
        let raw = unsafe { mmap(hint as *mut c_void, reserve, 3, 0x1002 | 0x8000, -1, 0) };
        if raw as isize == -1 {
            return Err("low memory unavailable; host must link with a small __PAGEZERO".into());
        }
        let first = raw as usize;
        if first + reserve > 0x100000000 {
            unsafe {
                munmap(raw, reserve);
            };
            return Err("mapping is not below 4 GiB".into());
        }
        let base = (first + alignment - 1) & !(alignment - 1);
        unsafe {
            if base > first {
                munmap(raw, base - first);
            }
            if first + reserve > base + len {
                munmap((base + len) as *mut c_void, first + reserve - base - len);
            }
        }
        Ok(Self {
            base: base as *mut u8,
            len,
        })
    }
    fn protect(&self, start: usize, len: usize, rights: i32) -> Result<()> {
        if start.checked_add(len).is_none_or(|end| end > self.len) {
            return Err("low protection extent".into());
        }
        #[cfg(target_arch = "aarch64")]
        {
            let _ = rights;
            return Ok(());
        } // Guest permissions are checked at every CPU/service access.
        #[cfg(target_arch = "x86_64")]
        if unsafe { mprotect(self.base.add(start).cast(), len, rights) } != 0 {
            return Err("low protection failed".into());
        }
        Ok(())
    }
}
impl Drop for LowMapping {
    fn drop(&mut self) {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            let len = self.len.div_ceil(getpagesize() as usize) * getpagesize() as usize;
            #[cfg(target_arch = "x86_64")]
            let len = self.len;
            munmap(self.base.cast(), len);
        }
    }
}
// The pointer is installed only while enter holds &self; ActiveGuard clears it
// before that borrow ends. No cross-thread/nested entry or guest callback path.
thread_local! {static ACTIVE:Cell<*const c_void>=const {Cell::new(ptr::null())};}
struct ActiveGuard;
impl Drop for ActiveGuard {
    fn drop(&mut self) {
        ACTIVE.with(|slot| slot.set(ptr::null()));
    }
}
fn unsupported(reason: &str) -> ! {
    eprintln!("Alkyhol PE32 unsupported: {reason}");
    std::process::exit(78)
}
struct TlsBlock {
    mapping: LowMapping,
    size: usize,
    initial: Vec<u8>,
    alignment: usize,
}
// Linked images on one guest thread share one stack and FS/last-error state.
// An Rc keeps them thread-bound; the exclusive FS lease still guards CPU entry.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase32 {
    Isolated,
    Preparing,
    Ready,
    Running,
    Detaching,
    Exited,
}
#[derive(Clone)]
struct Module32 {
    name: String,
    base: u32,
    size: usize,
    exports: Option<ExportTable>,
    origin: Option<module_path32::Origin>,
    // Process images reject DLL static TLS before mapping. Future guest thread
    // dispatch must consult this per-module policy; process notifications remain.
    thread_notifications: bool,
    entry: u32,
    loaded: bool,
    references: u32,
    pinned: bool,
    retired: bool,
    dependencies: Vec<u32>,
}
#[cfg(target_arch = "aarch64")]
fn native_generation() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .unwrap_or_else(|_| unsupported("native process generation exhausted"))
}
struct Cpu32 {
    stack: LowMapping,
    teb: LowMapping,
    #[cfg(target_arch = "aarch64")]
    native_gates: RefCell<std::collections::BTreeMap<u32, (u32, u16)>>,
    #[cfg(target_arch = "aarch64")]
    native_code: RefCell<Vec<(u32, u32)>>,
    #[cfg(target_arch = "aarch64")]
    native_generation: u64,
    blocks: RefCell<Vec<startup32::Block>>,
    command: Cell<u32>,
    command_ansi: Cell<u32>,
    exit: Cell<Option<u32>>,
    files: RefCell<std::collections::BTreeSet<usize>>,
    file_leases: RefCell<std::collections::BTreeMap<usize, fileio32::Lease>>,
    searches: RefCell<std::collections::BTreeMap<u32, files32::Search>>,
    phase: Cell<Phase32>,
    modules: RefCell<Vec<Module32>>,
    dynamic_order: RefCell<Vec<u32>>,
    // Real owned tokens for imported host-service namespaces. Their pages are
    // inaccessible: treating a service token as a Windows PE header must fault.
    builtins: RefCell<std::collections::BTreeMap<String, LowMapping>>,
    executable: Cell<u32>,
    diagnostic: Option<diagnostic32::Traps>,
    trace_first_calls: bool,
    trace_seen: RefCell<std::collections::BTreeSet<u32>>,
    heap: RefCell<heap32::Heap>,
    crt: RefCell<crt32::State>,
    fls: RefCell<fiber32::Slots>,
    dynamic_tls: RefCell<[bool; 64]>,
    callback_stack: Cell<u32>,
    callback_depth: Cell<u32>,
    pointer_cookie: u32,
    critical_sections: RefCell<std::collections::BTreeMap<u32, sync32::Section>>,
    service_gates: Cell<u32>,
    activation: RefCell<Result<()>>,
    bound_activation: RefCell<Option<activation32::State>>,
    system_directory: Option<Vec<u16>>,
    system_windows_directory: Option<Vec<u16>>,
    windows_directory: Option<Vec<u16>>,
    documents: Option<shell_folder32::Documents>,
    private_profile_files: bool,
    casefold_file_opens: bool,
    unhandled_filter: Cell<u32>,
    machine: Option<machine32::Machine>,
    registry: Option<RefCell<registry32::State>>,
    mutexes: Option<RefCell<mutex32::State>>,
    current_directory: RefCell<Option<directory32::Directory>>,
    icons: RefCell<icon32::State>,
    events: RefCell<event32::State>,
    ui: RefCell<ui32::State>,
    gdi: RefCell<gdi32::State>,
    utc_timezone: bool,
    version_workstation: bool,
    threads: RefCell<thread32::State>,
    watches: RefCell<watch32::State>,
    winsock: RefCell<winsock32::State>,
    sections: RefCell<section32::State>,
    images: RefCell<image32::State>,
    d3d9: RefCell<d3d9_32::State>,
    input8: RefCell<input8_32::State>,
    dsound: RefCell<dsound32::State>,
    unnamed_mutexes: RefCell<unnamed_mutex32::State>,
    com: RefCell<com32::State>,
    wave: RefCell<wave32::State>,
    com_storage: RefCell<Vec<(LowMapping, bool)>>,
    com_arenas: RefCell<Vec<(u32, u32)>>,
}
impl Cpu32 {
    fn stack(&self) -> &LowMapping {
        thread32::stack(self).unwrap_or(&self.stack)
    }
    fn teb(&self) -> &LowMapping {
        thread32::teb(self).unwrap_or(&self.teb)
    }
    fn callback_stack(&self) -> &Cell<u32> {
        thread32::callback_stack(self).unwrap_or(&self.callback_stack)
    }
    fn callback_depth(&self) -> &Cell<u32> {
        thread32::callback_depth(self).unwrap_or(&self.callback_depth)
    }
    fn clear_heap(&self) {
        if self.fls.borrow().pending_callbacks() {
            unsupported("PE32 thread-exit FLS callbacks are unsupported; pending values remain");
        }
        self.fls.borrow_mut().clear();
        self.unnamed_mutexes.borrow_mut().clear();
        self.com.borrow_mut().clear();
        self.heap.borrow_mut().clear();
        *self.crt.borrow_mut() = Default::default();
    }
    fn close_files(&self) {
        let error = alkyhol::ffi::GetLastError();
        self.watches.borrow_mut().clear();
        self.sections.borrow_mut().clear_handles();
        self.events.borrow_mut().clear();
        self.current_directory.borrow_mut().take();
        if let Some(mutexes) = &self.mutexes {
            mutexes.borrow_mut().clear();
        }
        self.searches.borrow_mut().clear();
        self.file_leases.borrow_mut().clear();
        for handle in std::mem::take(&mut *self.files.borrow_mut()) {
            alkyhol::ffi::CloseHandle(handle);
        }
        alkyhol::ffi::SetLastError(error);
    }
    fn new() -> Result<Self> {
        Self::new_with_machine(None)
    }
    fn mapping(&self, len: usize, alignment: usize, hint: usize) -> Result<LowMapping> {
        LowMapping::new_in(
            len,
            alignment,
            hint,
            self.machine.as_ref().map(|m| m.bounds),
        )
    }
    fn new_with_machine(machine: Option<machine32::Machine>) -> Result<Self> {
        let bounds = machine.as_ref().map(|m| m.bounds);
        let stack = LowMapping::new_in(STACK + 2 * PAGE, PAGE, 0, bounds)?;
        let teb = LowMapping::new_in(PAGE, PAGE, 0, bounds)?;
        let bytes = teb_bytes(
            teb.base as u32,
            stack.base as u32 + PAGE as u32,
            STACK as u32,
        )?;
        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), teb.base, PAGE);
            teb.base
                .add(0x24)
                .cast::<u32>()
                .write(alkyhol::ffi::GetCurrentThreadId());
        }
        stack.protect(0, PAGE, 0)?;
        stack.protect(PAGE + STACK, PAGE, 0)?;
        Ok(Self {
            stack,
            teb,
            #[cfg(target_arch = "aarch64")]
            native_gates: RefCell::new(Default::default()),
            #[cfg(target_arch = "aarch64")]
            native_code: RefCell::new(Vec::new()),
            #[cfg(target_arch = "aarch64")]
            native_generation: native_generation(),
            blocks: RefCell::new(Vec::new()),
            command: Cell::new(0),
            command_ansi: Cell::new(0),
            exit: Cell::new(None),
            files: RefCell::new(Default::default()),
            file_leases: RefCell::new(Default::default()),
            searches: RefCell::new(Default::default()),
            phase: Cell::new(Phase32::Isolated),
            modules: RefCell::new(Vec::new()),
            dynamic_order: RefCell::new(Vec::new()),
            builtins: RefCell::new(Default::default()),
            executable: Cell::new(0),
            diagnostic: None,
            trace_first_calls: matches!(
                std::env::var("ALKYHOL_PE32_TRACE_CALLS").as_deref(),
                Ok("first")
            ),
            trace_seen: RefCell::new(Default::default()),
            heap: RefCell::new(heap32::Heap::with_bounds(bounds)),
            crt: RefCell::new(Default::default()),
            fls: RefCell::new(Default::default()),
            dynamic_tls: RefCell::new([false; 64]),
            callback_stack: Cell::new(0),
            callback_depth: Cell::new(0),
            pointer_cookie: cookie_entropy()?,
            critical_sections: RefCell::new(Default::default()),
            service_gates: Cell::new(0),
            activation: RefCell::new(Err(
                "PE32 activation context is unknown outside the checked process profile".into(),
            )),
            bound_activation: RefCell::new(None),
            system_directory: None,
            system_windows_directory: None,
            windows_directory: None,
            documents: None,
            private_profile_files: false,
            casefold_file_opens: false,
            unhandled_filter: Cell::new(0),
            machine,
            registry: None,
            mutexes: None,
            current_directory: RefCell::new(None),
            icons: RefCell::new(Default::default()),
            events: RefCell::new(Default::default()),
            ui: RefCell::new(Default::default()),
            gdi: RefCell::new(Default::default()),
            utc_timezone: false,
            version_workstation: false,
            threads: RefCell::new(Default::default()),
            watches: RefCell::new(Default::default()),
            winsock: RefCell::new(Default::default()),
            sections: RefCell::new(Default::default()),
            images: RefCell::new(Default::default()),
            d3d9: RefCell::new(Default::default()),
            input8: RefCell::new(Default::default()),
            dsound: RefCell::new(Default::default()),
            unnamed_mutexes: RefCell::new(Default::default()),
            com: RefCell::new(Default::default()),
            wave: RefCell::new(Default::default()),
            com_storage: RefCell::new(Default::default()),
            com_arenas: RefCell::new(Default::default()),
        })
    }
}
impl Drop for Cpu32 {
    fn drop(&mut self) {
        self.threads.borrow_mut().shutdown();
        self.close_files();
        self.clear_heap();
    }
}
struct MapContext32<'a> {
    cpu: Option<Rc<Cpu32>>,
    resolve: &'a dyn Fn(&str, &ImportSymbol) -> Result<Option<u32>>,
    regions: &'a [(usize, usize, u32)],
}
enum Binding32 {
    Api(usize),
    Direct(u32),
}
fn dependency_cpu(dependencies: &[(&str, &LoadedLibrary32<'_>)]) -> Result<Option<Rc<Cpu32>>> {
    if dependencies.len() > 64 {
        return Err("PE32 dependency list exceeds 64 modules".into());
    }
    let mut names = std::collections::BTreeSet::new();
    let shared = dependencies.first().map(|(_, lib)| &lib.mapping.cpu);
    for &(name, library) in dependencies {
        if !names.insert(crate::runtime::dll_basename(name)?) {
            return Err("duplicate case-insensitive PE32 DLL dependency".into());
        }
        if shared.is_some_and(|cpu| !Rc::ptr_eq(cpu, &library.mapping.cpu)) {
            return Err("PE32 dependencies must share one guest thread context".into());
        }
    }
    Ok(shared.cloned())
}
const DEFAULT_GS_COOKIE: u32 = 0xbb40e64e;
fn select_cookie(
    declared_size: usize,
    initial: u32,
    mut entropy: impl FnMut() -> Result<u32>,
) -> Result<u32> {
    // Native observations: the declared structure length controls this action,
    // even when the data-directory length disagrees. Other tested seeds persist.
    if declared_size != 72 || initial != DEFAULT_GS_COOKIE {
        return Ok(initial);
    }
    for _ in 0..128 {
        let value = entropy()?;
        if value != 0 && value != DEFAULT_GS_COOKIE {
            return Ok(value);
        }
    }
    Err("system entropy did not yield a nondefault security cookie".into())
}
fn cookie_entropy() -> Result<u32> {
    let mut bytes = [0u8; 4];
    if unsafe { getentropy(bytes.as_mut_ptr().cast(), bytes.len()) } != 0 {
        return Err("system entropy unavailable for PE32 security cookie".into());
    }
    Ok(u32::from_ne_bytes(bytes))
}
/// Trusted i386 EXE mapping with separate low stack and checked API gates.
/// The process guard and all mappings remain alive until drop. Minimal FS/TEB
/// self/stack/last-error fields and single-thread static TLS data are supplied.
/// No guest threads, PEB, callbacks,
/// SEH, arbitrary syscalls or escaping pointers.
/// The host executable must use a small __PAGEZERO (see translate_pe32.py).
///
/// ```compile_fail,E0505
/// # use alkyhol_pe::{ProcessContext,LoadedImage32};
/// # fn example(bytes:&[u8])->alkyhol_pe::Result<()> {
/// let process=ProcessContext::install(&["guest32".into()],&[])?;
/// let image=LoadedImage32::load(bytes,false,&process)?;
/// drop(process);
/// drop(image);
/// # Ok(()) }
/// ```
pub struct LoadedImage32<'a> {
    image: LowMapping,
    exports: Option<ExportTable>,
    gates: LowMapping,
    cpu: Rc<Cpu32>,
    tls: Option<TlsBlock>,
    tls_entered: Cell<bool>,
    entry: u32,
    relocated: bool,
    code: u16,
    data: u16,
    dll: bool,
    regions: Vec<(usize, usize, u32)>,
    code_regions: Vec<(usize, usize)>,
    seh_handlers: Vec<u32>,
    handles: [usize; 3],
    _process: PhantomData<&'a ProcessContext>,
}
impl<'a> LoadedImage32<'a> {
    /// Preflight all features/imports, map checked sections and HIGHLOW fixups,
    /// build low gates with explicit call conventions and a guarded stack. No guest code runs here.
    pub fn load(bytes: &[u8], rebase: bool, _process: &'a ProcessContext) -> Result<Self> {
        Self::load_kind(bytes, rebase, _process, false, &[], false)
    }
    fn load_kind(
        bytes: &[u8],
        rebase: bool,
        _process: &'a ProcessContext,
        dll: bool,
        dependencies: &[(&str, &'a LoadedLibrary32<'a>)],
        exceptions: bool,
    ) -> Result<Self> {
        let shared = dependency_cpu(dependencies)?;
        let regions: Vec<_> = dependencies
            .iter()
            .flat_map(|(_, d)| d.mapping.regions.iter().copied())
            .collect();
        let resolve = |name: &str, symbol: &ImportSymbol| {
            dependencies
                .iter()
                .find(|(n, _)| n.eq_ignore_ascii_case(name))
                .map(|(_, d)| d.import_address(symbol).map(Some))
                .unwrap_or(Ok(None))
        };
        Self::map_resolved(
            bytes,
            rebase,
            _process,
            dll,
            exceptions,
            MapContext32 {
                cpu: shared,
                resolve: &resolve,
                regions: &regions,
            },
        )
    }
    fn map_resolved(
        bytes: &[u8],
        rebase: bool,
        _process: &'a ProcessContext,
        dll: bool,
        exceptions: bool,
        context: MapContext32<'_>,
    ) -> Result<Self> {
        if ACTIVE.with(|s| !s.get().is_null()) {
            return Err("PE32 loading during guest entry is unsupported".into());
        }
        #[cfg(target_arch = "aarch64")]
        if !executor32::installed() {
            return Err("native PE32 requires an installed checked executor".into());
        }
        #[cfg(target_arch = "x86_64")]
        if unsafe { getpagesize() } != PAGE as i32 {
            return Err("PE32 requires 4-KiB host pages".into());
        }
        let parsed = if context.cpu.as_ref().is_some_and(|c| c.diagnostic.is_some()) {
            Image::parse_i386_diagnostic(bytes, dll)?
        } else if dll {
            Image::parse_i386_dll(bytes)?
        } else {
            Image::parse_i386(bytes)?
        };
        if context.cpu.as_ref().is_some_and(|c| c.diagnostic.is_some()) {
            parsed.check_diagnostic_startup32()?;
        } else if exceptions {
            parsed.check_exception_startup32()?;
        } else if dll {
            parsed.check_library_startup()?;
        } else {
            parsed.check_startup()?;
        }
        let tls_info = parsed.static_tls32()?;
        let load_config = parsed.legacy_load_config32()?;
        let imports = parsed.imports()?;
        let fixups = parsed.relocations()?;
        let mut bindings = Vec::new();
        for import in imports {
            let binding = if let Some(id) = api_id(&import.module, &import.symbol) {
                Some(Binding32::Api(id))
            } else {
                (context.resolve)(&import.module, &import.symbol)?.map(Binding32::Direct)
            }
            .ok_or_else(|| {
                format!(
                    "unsupported Win32 import {}!{:?}",
                    import.module, import.symbol
                )
            })?;
            if fixups
                .iter()
                .any(|&at| at < import.iat + 4 && import.iat < at + 4)
            {
                return Err("IAT overlaps HIGHLOW fixup".into());
            }
            bindings.push((import.iat, binding));
        }
        let (code, data) = selectors()?;
        let bounds = context
            .cpu
            .as_ref()
            .and_then(|c| c.machine.as_ref())
            .map(|m| m.bounds);
        let image = LowMapping::new_in(
            parsed.size,
            65536,
            if rebase {
                if parsed.base == 0x10000000 {
                    0x20000000
                } else {
                    0x10000000
                }
            } else {
                parsed.base as usize
            },
            bounds,
        )?;
        let relocated = u64::from(image.base as u32) != parsed.base;
        if rebase && !relocated {
            return Err("forced PE32 rebase did not move".into());
        }
        if relocated && fixups.is_empty() {
            return Err("PE32 preferred base unavailable and no HIGHLOW fixups".into());
        }
        let gates = LowMapping::new_in(math32::GATE_SIZE, PAGE, 0, bounds)?;
        let cpu = match context.cpu {
            Some(cpu) => cpu,
            None => Rc::new(Cpu32::new()?),
        };
        for import in parsed.imports()? {
            if api_id(&import.module, &import.symbol).is_some() {
                let name = import.module.to_ascii_lowercase();
                let mut builtins = cpu.builtins.borrow_mut();
                if let std::collections::btree_map::Entry::Vacant(entry) = builtins.entry(name) {
                    if entry.key() == "dsound.dll" {
                        cpu.dsound.borrow_mut().static_loaded = true;
                    }
                    let token = cpu.mapping(PAGE, PAGE, 0)?;
                    token.protect(0, PAGE, 0)?;
                    entry.insert(token);
                }
            }
        }
        let stack = &cpu.stack;
        if cpu.service_gates.get() == 0 {
            cpu.service_gates.set(gates.base as u32);
        }
        let teb = &cpu.teb;
        let mut tls = tls_info
            .as_ref()
            .map(|info| -> Result<TlsBlock> {
                let size = (info.total_size + PAGE - 1) & !(PAGE - 1);
                Ok(TlsBlock {
                    mapping: cpu.mapping(size, info.alignment.max(PAGE), 0)?,
                    size: info.total_size,
                    initial: Vec::new(),
                    alignment: info.alignment.max(PAGE),
                })
            })
            .transpose()?;
        let mut template = vec![0u8; math32::GATE_SIZE];
        #[allow(unused_mut)]
        let mut cs64 = 0u16;
        #[cfg(target_arch = "x86_64")]
        {
            unsafe {
                std::arch::asm!("mov {0:x}, cs",out(reg) cs64,options(nomem,nostack,preserves_flags));
            }
            // 32-bit near return -> far jump into low 64-bit gate -> host continuation.
            template[..2].copy_from_slice(&[0xff, 0x2d]);
            template[2..6].copy_from_slice(&((gates.base as u32) + 8).to_le_bytes());
            template[8..12].copy_from_slice(&((gates.base as u32) + 16).to_le_bytes());
            template[12..14].copy_from_slice(&cs64.to_le_bytes());
            // Capture actual post-RET ESP before recovering the host stack. This
            // checks cdecl/stdcall cleanup rather than hiding a guest stack imbalance.
            template[16..24].copy_from_slice(&[0x89, 0xe2, 0x4c, 0x89, 0xe4, 0x41, 0xff, 0xe5]);
            let start = ptr::addr_of!(alkyhol_pe32_gate_start) as usize;
            let end = ptr::addr_of!(alkyhol_pe32_gate_end) as usize;
            let slot = ptr::addr_of!(alkyhol_pe32_dispatch_slot) as usize - start;
            if end - start > 192 || slot + 8 > end - start {
                return Err("invalid CPU gate template".into());
            }
            template[64..64 + end - start].copy_from_slice(unsafe {
                std::slice::from_raw_parts(start as *const u8, end - start)
            });
            template[64 + slot..72 + slot]
                .copy_from_slice(&(dispatch as *const () as u64).to_le_bytes());
        }
        if api_offset(APIS.len() - 1) + 32 > math32::GATE_SIZE {
            return Err("API table exceeds its checked gateway banks".into());
        }
        #[cfg(target_arch = "aarch64")]
        cpu.native_code.borrow_mut().push((
            gates.base as u32,
            gates.base as u32 + math32::GATE_SIZE as u32,
        ));
        for (id, &(_, argument_bytes)) in APIS.iter().enumerate() {
            #[cfg(target_arch = "aarch64")]
            {
                cpu.native_gates.borrow_mut().insert(
                    gates.base as u32 + api_offset(id) as u32,
                    (id as u32, cleanup_bytes(id, argument_bytes)),
                );
            }
            let mut at = api_offset(id);
            let mut base = gates.base as u32 + at as u32;
            if matches!(id, 89 | 92) {
                let target = gates.base as u32
                    + if id == 89 {
                        math32::OFFSET
                    } else {
                        math32::ATAN_OFFSET
                    } as u32;
                template[at] = 0xe9;
                template[at + 1..at + 5]
                    .copy_from_slice(&target.wrapping_sub(base + 5).to_le_bytes());
                let emit = if id == 89 {
                    math32::emit
                } else {
                    math32::emit_atan
                };
                emit(
                    &mut template,
                    gates.base as u32,
                    gates.base as u32 + api_offset(if id == 89 { 90 } else { 93 }) as u32,
                )?;
                at += 5;
                base += 5;
            }
            template[at] = 0x68;
            template[at + 1..at + 5].copy_from_slice(&(base + 16).to_le_bytes());
            template[at + 5] = 0x68;
            template[at + 6..at + 10].copy_from_slice(&(id as u32).to_le_bytes());
            template[at + 10..at + 12].copy_from_slice(&[0xff, 0x2d]);
            template[at + 12..at + 16].copy_from_slice(&(base + 20).to_le_bytes());
            template[at + 16] = 0xc2;
            template[at + 17..at + 19]
                .copy_from_slice(&cleanup_bytes(id, argument_bytes).to_le_bytes());
            template[at + 20..at + 24].copy_from_slice(&(gates.base as u32 + 64).to_le_bytes());
            template[at + 24..at + 26].copy_from_slice(&cs64.to_le_bytes());
        }
        // The public software exception stub captures the real guest CPU state.
        // Its private service copy keeps the ordinary host bridge unchanged.
        let public = api_offset(177);
        let private = raise32::OFFSET + 256;
        let private_base = gates.base as u32 + private as u32;
        #[cfg(target_arch = "aarch64")]
        cpu.native_gates
            .borrow_mut()
            .insert(private_base, (177 as u32, 0));
        let normal: [u8; 32] = template[public..public + 32].try_into().unwrap();
        template[private..private + 32].copy_from_slice(&normal);
        template[private + 1..private + 5].copy_from_slice(&(private_base + 16).to_le_bytes());
        template[private + 12..private + 16].copy_from_slice(&(private_base + 20).to_le_bytes());
        raise32::emit(&mut template, gates.base as u32, private_base)?;
        template[public] = 0xe9;
        template[public + 1..public + 5].copy_from_slice(
            &((raise32::OFFSET as u32).wrapping_sub(public as u32 + 5)).to_le_bytes(),
        );
        unsafe {
            ptr::copy_nonoverlapping(template.as_ptr(), gates.base, math32::GATE_SIZE);
            ptr::copy_nonoverlapping(bytes.as_ptr(), image.base, parsed.headers);
            for section in &parsed.sections {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(section.raw),
                    image.base.add(section.rva),
                    section.raw_size,
                );
            }
            if relocated {
                let delta = (image.base as u32).wrapping_sub(parsed.base as u32);
                for at in fixups {
                    let p = image.base.add(at).cast::<u32>();
                    p.write_unaligned(p.read_unaligned().wrapping_add(delta));
                }
            }
            for (at, binding) in bindings {
                let address = match binding {
                    Binding32::Api(id) => cpu.service_gates.get() + api_offset(id) as u32,
                    Binding32::Direct(address) => address,
                };
                image.base.add(at).cast::<u32>().write_unaligned(address);
            }
            if let Some(config) = &load_config
                && let Some(rva) = config.cookie_rva
            {
                let target = image.base.add(rva).cast::<u32>();
                let value = select_cookie(
                    config.declared_size,
                    target.read_unaligned(),
                    cookie_entropy,
                )?;
                target.write_unaligned(value);
            }
            if let (Some(info), Some(block)) = (&tls_info, &mut tls) {
                // Copy the mapped template after relocations and import binding,
                // including any pointer initializer's adjusted image address.
                image
                    .base
                    .add(info.index_rva)
                    .cast::<u32>()
                    .write_unaligned(0);
                ptr::copy_nonoverlapping(
                    image.base.add(info.template_rva),
                    block.mapping.base,
                    info.template_size,
                );
                block.initial = std::slice::from_raw_parts(block.mapping.base, block.size).to_vec();
                teb.base
                    .add(0x2c)
                    .cast::<u32>()
                    .write_unaligned(teb.base as u32 + 0x100);
                teb.base
                    .add(0x100)
                    .cast::<u32>()
                    .write_unaligned(block.mapping.base as u32);
            }
            stack
                .base
                .add(PAGE + STACK - 4)
                .cast::<u32>()
                .write_unaligned(gates.base as u32);
        }
        gates.protect(0, math32::GATE_SIZE, 5)?;
        image.protect(0, parsed.size, 0)?;
        image.protect(0, parsed.headers, 1)?;
        let mut regions = Vec::new();
        #[cfg(target_arch = "aarch64")]
        regions.push((
            image.base as usize,
            image.base as usize + parsed.headers,
            0x40000000,
        ));
        let mut code_regions = Vec::new();
        for section in &parsed.sections {
            let rights = ((section.rights >> 30) & 1)
                | (((section.rights >> 31) & 1) << 1)
                | (((section.rights >> 29) & 1) << 2);
            image.protect(section.rva, section.size, rights as i32)?;
            regions.push((
                image.base as usize + section.rva,
                image.base as usize + section.rva + section.size,
                section.rights,
            ));
            if section.rights & 0x20000000 != 0 {
                code_regions.push((
                    image.base as usize + section.rva,
                    image.base as usize + section.rva + section.raw_size,
                ));
            }
        }
        // A direct dependency may call its imported API gate while this image
        // owns ACTIVE. All borrowed dependency data must retain its real rights.
        regions.extend_from_slice(context.regions);
        regions.sort_unstable();
        regions.dedup();
        let standard = alkyhol_host_abi::host_call!((u32) -> usize; crate::abi::resolve("kernel32.dll", "GetStdHandle").unwrap());
        let handles = unsafe {
            [
                standard(0xfffffff6),
                standard(0xfffffff5),
                standard(0xfffffff4),
            ]
        };
        let entry = if parsed.entry == 0 {
            0
        } else {
            image.base as u32 + parsed.entry as u32
        };
        let seh_handlers = load_config
            .as_ref()
            .map(|c| {
                c.seh_handlers
                    .iter()
                    .map(|&rva| image.base as u32 + rva as u32)
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self {
            image,
            exports: parsed.exports()?,
            gates,
            cpu,
            tls,
            tls_entered: Cell::new(false),
            entry,
            relocated,
            code,
            data,
            dll,
            regions,
            code_regions,
            seh_handlers,
            handles,
            _process: PhantomData,
        })
    }
    /// Whether HIGHLOW relocation was required for this mapping.
    pub fn relocated(&self) -> bool {
        self.relocated
    }
    fn permits(&self, address: usize, len: usize, write: bool) -> bool {
        #[cfg(target_arch = "aarch64")]
        let address = host_address(address);
        let Some(end) = address.checked_add(len) else {
            return false;
        };
        if self.cpu.com_storage.borrow().iter().any(|(m, writable)| {
            (!write || *writable) && address >= m.base as usize && end <= m.base as usize + m.len
        }) {
            return true;
        }
        if self.cpu.sections.borrow().permits(address, len, write) {
            return true;
        }
        if self.cpu.winsock.borrow().permits(address, len, write) {
            return true;
        }
        if self.cpu.gdi.borrow().permits(address, len) {
            return true;
        }
        if self.cpu.threads.borrow().permits(address, len) {
            return true;
        }
        let primary = self.cpu.stack.base as usize + PAGE;
        if address >= primary && end <= primary + STACK {
            return true;
        }
        if startup32::permits(&self.cpu, address, len, write) {
            return true;
        }
        if self
            .cpu
            .bound_activation
            .borrow()
            .as_ref()
            .is_some_and(|state| state.permits(address, len, write))
        {
            return true;
        }
        if self.cpu.heap.borrow().permits(address, len) {
            return true;
        }
        if let Some(block) = &self.tls {
            let first = block.mapping.base as usize;
            if address >= first && end <= first + block.size {
                return true;
            }
        }
        let teb = self.cpu.teb().base as usize;
        if address >= teb && end <= teb + self.cpu.teb().len {
            return true;
        }
        let first = self.cpu.stack().base as usize + PAGE;
        if address >= first && end <= first + STACK {
            return true;
        }
        let mask = if write { 0x80000000 } else { 0x40000000 };
        self.regions
            .iter()
            .any(|&(first, last, rights)| rights & mask != 0 && address >= first && end <= last)
    }
    fn export_name(&self, address: u32) -> Result<String> {
        query_name(address, |at| {
            if !self.permits(at as usize, 1, false) {
                return Err("export query name outside readable guest storage".into());
            }
            Ok(unsafe { (at as usize as *const u8).read() })
        })
    }
    fn disable_thread_notifications(&self, module: u32) -> Result<()> {
        if module == 0 || module == self.cpu.executable.get() {
            return Err("DisableThreadLibraryCalls requires an owned DLL module handle".into());
        }
        let mut modules = self.cpu.modules.borrow_mut();
        let target = modules
            .iter_mut()
            .find(|m| m.base == module)
            .ok_or("DisableThreadLibraryCalls target is not a registered process DLL")?;
        // Every process DLL's static-TLS directory is rejected before entry.
        // This changes the owned loader policy, including repeated calls, while
        // retaining PROCESS_ATTACH/DETACH. Guest thread creation remains rejected.
        target.thread_notifications = false;
        Ok(())
    }
    fn module_handle(&self, address: u32, wide: bool) -> Result<Option<u32>> {
        if self.cpu.modules.borrow().is_empty() {
            if self.dll {
                return Err("isolated PE32 DLL has no process executable module lookup".into());
            }
            if address != 0 {
                return Err("named PE32 module lookup is unsupported".into());
            }
            return Ok(Some(self.image.base as u32));
        }
        if address == 0 {
            return Ok(Some(self.cpu.executable.get()));
        }
        let mut name = if wide {
            startup32::module_name(self, address)?
        } else {
            self.export_name(address)?
        };
        if name.len() > 255
            || !name
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err("PE32 module lookup requires an explicit ASCII basename".into());
        }
        if !name.contains('.') {
            name.push_str(".dll");
        }
        if let Some(token) = self.cpu.builtins.borrow().get(&name.to_ascii_lowercase()) {
            if name.eq_ignore_ascii_case("dsound.dll") && !dsound32::loaded(&self.cpu) {
                return Ok(None);
            }
            return Ok(Some(token.base as u32));
        }
        if crate::abi::is_builtin(&name) {
            return Err(format!(
                "PE32 builtin DLL module handle is unsupported: {name}"
            ));
        }
        Ok(self
            .cpu
            .modules
            .borrow()
            .iter()
            .find(|m| m.loaded && m.name.eq_ignore_ascii_case(&name))
            .map(|m| m.base))
    }
    fn export_address(&self, module: u32, symbol: u32) -> Result<Option<u32>> {
        let module = if module == 0 {
            self.module_handle(0, false)?
                .ok_or("no process executable for NULL GetProcAddress")?
        } else {
            module
        };
        if let Some((name, _)) = self
            .cpu
            .builtins
            .borrow()
            .iter()
            .find(|(_, token)| token.base as u32 == module)
        {
            if name.eq_ignore_ascii_case("dsound.dll") && !dsound32::loaded(&self.cpu) {
                return Err("DirectSound export query on an unloaded module".into());
            }
            if symbol <= u16::MAX as u32 {
                let id = api_id(name, &ImportSymbol::Ordinal(symbol as u16))
                    .ok_or("PE32 host-service ordinal lookup is unsupported")?;
                return Ok(Some(self.cpu.service_gates.get() + api_offset(id) as u32));
            }
            let symbol = self.export_name(symbol)?;
            // Frozen actual 32-bit system msvcrt export observations: this
            // optional getter is absent, while __set_app_type exists. Do not
            // invent a callable getter because the bundled CRT probes its name.
            if name.eq_ignore_ascii_case("msvcrt.dll") && symbol == "__get_app_type" {
                return Ok(None);
            }
            let id = api_id(name, &ImportSymbol::Name(symbol.clone())).ok_or_else(|| {
                format!("PE32 dynamic host-service export is unsupported: {name}!{symbol}")
            })?;
            return Ok(Some(self.cpu.service_gates.get() + api_offset(id) as u32));
        }
        let modules = self.cpu.modules.borrow();
        let exports = if modules.is_empty() {
            if module != self.image.base as u32 {
                return Err("GetProcAddress requires the current PE32 executable module".into());
            }
            self.exports.as_ref()
        } else {
            modules
                .iter()
                .find(|m| m.base == module)
                .ok_or("GetProcAddress requires a known PE32 process module")?
                .exports
                .as_ref()
        };
        let name = if symbol > u16::MAX as u32 {
            Some(self.export_name(symbol)?)
        } else {
            None
        };
        let entry = exports.and_then(|table| {
            table.entries.iter().find(|e| match &name {
                Some(n) => e.names.contains(n),
                None => e.ordinal == symbol,
            })
        });
        match entry.map(|e| &e.target) {
            None => Ok(None),
            Some(ExportTarget::Address { rva }) => module
                .checked_add(*rva as u32)
                .map(Some)
                .ok_or_else(|| "export address overflow".into()),
            Some(ExportTarget::Forwarder(_)) => {
                Err("PE32 export forwarders are unsupported".into())
            }
        }
    }
    /// Execute trusted i386 code and return its DWORD result to the host.
    ///
    /// # Safety
    /// Input must obey this type's stated restricted native-code contract.
    /// This is execution, not an instruction validator or security sandbox.
    pub unsafe fn enter(&self) -> Result<u32> {
        unsafe { self.invoke(self.entry, &[], CallingConvention32::Cdecl) }
    }
    // Synchronous stdcall callback while an API gateway has suspended the guest.
    // Reuse the exclusively leased TEB, but place the frame below the suspended
    // guest ESP. Never overwrite the outer frame or reinitialize static TLS.
    unsafe fn callback(&self, entry: u32, value: u32) -> Result<()> {
        if unsafe { self.callback_frame(entry, &[value], CallingConvention32::Stdcall) }?.is_none()
        {
            return Err("non-returning exit from an FLS callback is unsupported".into());
        }
        Ok(())
    }
    // Some is an ordinary return; None is an actual non-returning ExitProcess
    // escape. The caller must propagate or explicitly reject that outcome.
    unsafe fn callback_frame(
        &self,
        entry: u32,
        args: &[u32],
        convention: CallingConvention32,
    ) -> Result<Option<u32>> {
        if args.len() > 16 {
            return Err("PE32 synchronous callbacks support at most sixteen DWORDs".into());
        }
        let frame_bytes = (args.len() + 1) * 4;
        if self.cpu.callback_depth().get() >= 16 {
            return Err("PE32 callback nesting limit".into());
        }
        if !self.regions.iter().any(|&(first, last, rights)| {
            rights & 0x20000000 != 0
                && host_code_address(entry) >= first
                && host_code_address(entry) < last
        }) {
            return Err("PE32 callback outside executable guest storage".into());
        }
        let top = self.cpu.callback_stack().get();
        let limit = self.cpu.stack().base as u32 as usize + PAGE + STACK;
        let frame = top
            .checked_sub(32 + frame_bytes as u32)
            .and_then(|v| (v & !15).checked_sub(4))
            .filter(|&v| {
                v >= self.cpu.stack().base as u32 + PAGE as u32 + 64
                    && (v as usize) + frame_bytes <= limit
                    && top as usize <= limit
            })
            .ok_or("PE32 callback stack exhausted or no suspended API call")?;
        if !self.permits(frame as usize, frame_bytes, true) {
            return Err("PE32 callback frame outside guest stack".into());
        }
        let before = segment_state();
        if !thread32::host_fs_valid(before[3]) {
            return Err(format!(
                "PE32 callback entered with unexpected host FS: {before:x?}"
            ));
        }
        unsafe {
            (frame as *mut u32).write(self.gates.base as u32);
            for (i, &value) in args.iter().enumerate() {
                ((frame + 4 + i as u32 * 4) as *mut u32).write(value);
            }
            self.cpu
                .teb()
                .base
                .add(0x34)
                .cast::<u32>()
                .write_unaligned(alkyhol::ffi::GetLastError());
        }
        self.cpu
            .callback_depth()
            .set(self.cpu.callback_depth().get() + 1);
        let result =
            unsafe { alkyhol_enter_pe32(entry, frame, u32::from(self.code), u32::from(self.data)) };
        self.cpu
            .callback_depth()
            .set(self.cpu.callback_depth().get() - 1);
        if segment_state() != before {
            return Err("PE32 callback did not restore selectors".into());
        }
        alkyhol::ffi::SetLastError(unsafe {
            self.cpu.teb().base.add(0x34).cast::<u32>().read_unaligned()
        });
        let observed = (result >> 32) as u32;
        if observed == 0 && self.cpu.exit.get().is_some() {
            return Ok(None);
        }
        let expected = frame
            + 4
            + if matches!(convention, CallingConvention32::Stdcall) {
                args.len() as u32 * 4
            } else {
                0
            };
        if observed != expected {
            return Err(
                "PE32 callback did not restore its declared calling-convention stack".into(),
            );
        }
        Ok(Some(result as u32))
    }
    unsafe fn invoke(
        &self,
        entry: u32,
        args: &[u32],
        convention: CallingConvention32,
    ) -> Result<u32> {
        let (offset, expected_offset) = call_stack(args.len(), convention)?;
        if !self.code_regions.iter().any(|&(first, last)| {
            host_code_address(entry) >= first && host_code_address(entry) < last
        }) {
            return Err("PE32 call target is outside executable image storage".into());
        }
        ACTIVE.with(|slot| {
            if !slot.get().is_null() {
                return Err("nested PE32 entry unsupported".into());
            }
            if self.cpu.exit.get().is_some() && self.cpu.phase.get() != Phase32::Detaching {
                return Err("PE32 process already requested ExitProcess".into());
            }
            if self.tls.is_some() && self.tls_entered.get() {
                return Err("PE32 static TLS images support one entry".into());
            }
            let before = segment_state();
            if before[3] != 0 {
                return Err("PE32 requires an unused host FS selector".into());
            }
            #[cfg(target_arch = "x86_64")]
            let _diagnostic_signals = if self.cpu.diagnostic.is_some() {
                Some(seh32::SignalGuard::install_diagnostic(self)?)
            } else {
                None
            };
            let _teb_lease = thread32::MainLease::install(self.cpu.teb().base as u32)?;
            slot.set(ptr::from_ref(self).cast());
            let _guard = ActiveGuard;
            let sp = self.cpu.stack().base as u32 + offset as u32;
            unsafe {
                let frame = self.cpu.stack().base.add(offset).cast::<u32>();
                frame.write_unaligned(self.gates.base as u32);
                ptr::copy_nonoverlapping(args.as_ptr(), frame.add(1), args.len());
            }
            if self.tls.is_some() {
                self.tls_entered.set(true);
            }
            if self.cpu.diagnostic.is_some() {
                let modules = self.cpu.modules.borrow();
                if let Some(module) = modules
                    .iter()
                    .find(|m| entry >= m.base && (entry as usize) < m.base as usize + m.size)
                {
                    eprintln!(
                        "PE32 diagnostic entry: {}+0x{:x}",
                        module.name,
                        entry - module.base
                    );
                }
            }
            let result = unsafe {
                alkyhol_enter_pe32(entry, sp, u32::from(self.code), u32::from(self.data))
            };
            self.cpu.threads.borrow().require_completed();
            if segment_state() != before {
                unsupported("PE32 host segment selectors were not restored");
            }
            alkyhol::ffi::SetLastError(unsafe {
                self.cpu.teb().base.add(0x34).cast::<u32>().read_unaligned()
            });
            let returned_sp = (result >> 32) as u32;
            let expected_sp = self.cpu.stack().base as u32 + expected_offset as u32;
            if returned_sp == 0 {
                return match self.cpu.exit.get() {
                    Some(status) if status == result as u32 && !self.dll => Ok(status),
                    _ => Err(
                        "PE32 exit gate returned without a matching process-exit request".into(),
                    ),
                };
            }
            if returned_sp != expected_sp {
                return Err(format!(
                    "PE32 {convention:?} call returned with an unbalanced guest stack"
                ));
            }
            Ok(result as u32)
        })
    }
}
/// Stack cleanup expected from an integer-only PE32 exported function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallingConvention32 {
    Cdecl,
    Stdcall,
}
fn call_stack(count: usize, convention: CallingConvention32) -> Result<(usize, usize)> {
    if count > 16 {
        return Err("PE32 calls support at most sixteen DWORD arguments".into());
    }
    // Align the argument vector to 16 bytes; the return address precedes it.
    let top = PAGE + STACK;
    let arguments = (top - count * 4) & !15;
    let frame = arguments - 4;
    let expected = arguments
        + if convention == CallingConvention32::Stdcall {
            count * 4
        } else {
            0
        };
    Ok((frame, expected))
}

/// A trusted i386 DLL with real dynamic PROCESS_ATTACH/DETACH and integer calls.
/// Explicit borrowed dependencies share one guest stack/TEB. This is not a DLL bundle,
/// filesystem search, process module cache or support for DLL TLS/SEH/CRT.
/// The ProcessContext borrow remains live through guest DETACH and unmapping.
///
/// ```compile_fail,E0505
/// # use alkyhol_pe::{ProcessContext, LoadedLibrary32};
/// # fn example(bytes: &[u8]) -> alkyhol_pe::Result<()> {
/// let process = ProcessContext::install(&["library32".into()], &[])?;
/// let library = unsafe { LoadedLibrary32::load(bytes, false, &process) }?;
/// drop(process);
/// drop(library);
/// # Ok(()) }
/// ```
pub struct LoadedLibrary32<'a> {
    mapping: LoadedImage32<'a>,
    attached: bool,
}
impl<'a> LoadedLibrary32<'a> {
    /// Map/bind the complete supported DLL before calling DllMain. FALSE attach
    /// receives one matching dynamic detach; the rejected library never escapes.
    ///
    /// # Safety
    /// DLL code must obey LoadedImage32's trusted integer-only, single-thread
    /// native-code contract and retain no pointers after this object is dropped.
    pub unsafe fn load(bytes: &[u8], rebase: bool, process: &'a ProcessContext) -> Result<Self> {
        unsafe { Self::load_linked(bytes, rebase, process, &[]) }
    }
    /// Bind an explicit set of already initialized DLL dependencies before
    /// attaching this DLL on their shared guest thread. References keep those
    /// mappings alive through this DLL's detach. No search/cache/forwarders.
    ///
    /// # Safety
    /// The same trusted-code contract as `load` applies to the entire dependency
    /// closure. Supplied basenames must identify the actual intended modules.
    ///
    /// ```compile_fail,E0505
    /// # use alkyhol_pe::{ProcessContext, LoadedLibrary32};
    /// # fn example(a: &[u8], b: &[u8]) -> alkyhol_pe::Result<()> {
    /// let process = ProcessContext::install(&["linked32".into()], &[])?;
    /// let leaf = unsafe { LoadedLibrary32::load(a, false, &process) }?;
    /// let child = unsafe { LoadedLibrary32::load_linked(b, false, &process, &[("leaf.dll", &leaf)]) }?;
    /// drop(leaf);
    /// drop(child);
    /// # Ok(()) }
    /// ```
    pub unsafe fn load_linked(
        bytes: &[u8],
        rebase: bool,
        process: &'a ProcessContext,
        dependencies: &[(&str, &'a LoadedLibrary32<'a>)],
    ) -> Result<Self> {
        let mut library = Self {
            mapping: LoadedImage32::load_kind(bytes, rebase, process, true, dependencies, false)?,
            attached: false,
        };
        if library.mapping.entry != 0 {
            if unsafe { library.entry(1)? } == 0 {
                unsafe {
                    library.entry(0)?;
                }
                return Err("PE32 DLL PROCESS_ATTACH returned FALSE".into());
            }
            library.attached = true;
        }
        Ok(library)
    }
    fn import_address(&self, symbol: &ImportSymbol) -> Result<u32> {
        if matches!(symbol, ImportSymbol::Ordinal(0)) {
            return Err("zero PE32 dependency import ordinals are unsupported".into());
        }
        let export = self
            .mapping
            .exports
            .as_ref()
            .and_then(|t| {
                t.entries.iter().find(|e| match symbol {
                    ImportSymbol::Name(name) => e.names.contains(name),
                    ImportSymbol::Ordinal(ordinal) => e.ordinal == u32::from(*ordinal),
                })
            })
            .ok_or_else(|| format!("missing PE32 dependency export {symbol:?}"))?;
        match export.target {
            ExportTarget::Address { rva } => (self.mapping.image.base as u32)
                .checked_add(rva as u32)
                .ok_or_else(|| "PE32 dependency export address overflow".into()),
            ExportTarget::Forwarder(_) => Err("PE32 dependency forwarders are unsupported".into()),
        }
    }
    unsafe fn entry(&self, reason: u32) -> Result<u32> {
        unsafe {
            self.mapping.invoke(
                self.mapping.entry,
                &[self.mapping.image.base as u32, reason, 0],
                CallingConvention32::Stdcall,
            )
        }
    }
    /// Whether this DLL actually required HIGHLOW relocation.
    pub fn relocated(&self) -> bool {
        self.mapping.relocated()
    }
    /// Invoke a named direct code export with up to sixteen DWORD arguments.
    /// Missing names, data exports, forwarders and stack-cleanup mismatches fail.
    ///
    /// # Safety
    /// The caller must supply the export's actual DWORD signature/convention;
    /// guest pointers must remain valid for the entire trusted call.
    pub unsafe fn call_name(
        &self,
        name: &str,
        args: &[u32],
        convention: CallingConvention32,
    ) -> Result<u32> {
        let export = self
            .mapping
            .exports
            .as_ref()
            .and_then(|t| t.entries.iter().find(|e| e.names.iter().any(|n| n == name)))
            .ok_or_else(|| format!("missing PE32 DLL export {name}"))?;
        let address = match export.target {
            ExportTarget::Address { rva } => self.mapping.image.base as u32 + rva as u32,
            ExportTarget::Forwarder(_) => {
                return Err("PE32 DLL export forwarders are unsupported".into());
            }
        };
        unsafe { self.mapping.invoke(address, args, convention) }
    }
}
impl Drop for LoadedLibrary32<'_> {
    fn drop(&mut self) {
        if self.attached {
            self.attached = false;
            if let Err(error) = unsafe { self.entry(0) } {
                // A failed teardown is not a successfully unloaded DLL.
                unsupported(&error);
            }
        }
    }
}
// Keep the process borrow live through field destruction, including implicit drop.
impl Drop for LoadedImage32<'_> {
    fn drop(&mut self) {}
}
fn query_name(address: u32, mut read: impl FnMut(u32) -> Result<u8>) -> Result<String> {
    let mut text = Vec::new();
    for offset in 0..4096u32 {
        let at = address
            .checked_add(offset)
            .ok_or("export query pointer overflow")?;
        let byte = read(at)?;
        if byte == 0 {
            if text.is_empty() {
                return Err("empty export query name is unsupported".into());
            }
            return String::from_utf8(text).map_err(|_| "invalid export query name".into());
        }
        if !byte.is_ascii() {
            return Err("non-ASCII export query is unsupported".into());
        }
        text.push(byte);
    }
    Err("export query exceeds 4096-byte bound".into())
}
extern "C" fn dispatch(id: u32, arguments: *const u32) -> u64 {
    if seh32::in_handler() {
        seh32::async_fail(b"Alkyhol PE32 unsupported: imports from an exception handler\n");
    }
    let exit_state = || {
        ACTIVE.with(|slot| {
            unsafe { slot.get().cast::<LoadedImage32<'_>>().as_ref() }
                .and_then(|image| image.cpu.exit.get())
        })
    };
    let before = exit_state();
    let thread_before = thread32::exit_state();
    let timer = perf32::start();
    let result = dispatch_inner(id, arguments);
    if let Some(timer) = timer {
        let category = if d3d9_32::signature(id).is_some() {
            if (id - d3d9_32::BASE) / 256 == 1
                && [81, 82, 83, 84].contains(&((id - d3d9_32::BASE) % 256))
            {
                1
            } else {
                2
            }
        } else {
            0
        };
        perf32::record(category, Some(timer));
    }
    let timer = perf32::start();
    thread32::yield_cpu();
    perf32::record(3, timer);
    // ExitProcess and CRT termination escapes are non-returning guest operations.
    // The assembly gate uses
    // a separate control bit, never a forged stdcall return-stack observation.
    u64::from(result)
        | if id == EXIT_API
            || (before.is_none() && exit_state().is_some())
            || (thread_before.is_none() && thread32::exit_state().is_some())
        {
            1u64 << 32
        } else {
            0
        }
}
fn dispatch_inner(id: u32, arguments: *const u32) -> u32 {
    if seh32::in_handler() {
        seh32::async_fail(b"Alkyhol PE32 unsupported: imports from an exception handler\n");
    }
    ACTIVE.with(|slot| {
        let Some(image) = (unsafe { slot.get().cast::<LoadedImage32<'_>>().as_ref() }) else {
            unsupported("no active Win32 image");
        };
        // Import calls observe direct guest FS writes. Reflect all host API
        // outcomes back into the same guest field, including early returns.
        let error = unsafe { image.cpu.teb().base.add(0x34).cast::<u32>() };
        alkyhol::ffi::SetLastError(unsafe { error.read_unaligned() });
        let _teb_error = TebError(error);
        struct Suspended<'a>(&'a Cell<u32>, u32);
        impl Drop for Suspended<'_> {
            fn drop(&mut self) {
                self.0.set(self.1);
            }
        }
        let Some((name, bytes)) = APIS
            .get(id as usize)
            .copied()
            .or_else(|| d3d9_32::signature(id))
            .or_else(|| input8_32::signature(id))
            .or_else(|| dsound32::signature(id))
        else {
            unsupported("unknown gateway");
        };
        if !(arguments as usize).is_multiple_of(4)
            || !image.permits(arguments as usize, bytes as usize, false)
        {
            unsupported("argument stack outside checked storage");
        }
        let saved_stack = image.cpu.callback_stack().replace(
            (arguments as u32)
                .checked_sub(4)
                .unwrap_or_else(|| unsupported("missing guest return address")),
        );
        let _suspended = Suspended(image.cpu.callback_stack(), saved_stack);
        // Snapshot values rather than borrowing guest argument memory across a
        // host call: a valid output pointer may alias those stack words.
        let mut words = [0u32; 16];
        if bytes as usize > words.len() * 4 {
            unsupported("gateway argument count exceeds checked snapshot");
        }
        for (index, value) in words.iter_mut().enumerate().take(bytes as usize / 4) {
            *value = unsafe { arguments.add(index).read_unaligned() };
        }
        let args: &[u32; 8] = words[..8].try_into().expect("fixed gateway prefix");
        let trace_call = image.cpu.diagnostic.is_some()
            && (!image.cpu.trace_first_calls || image.cpu.trace_seen.borrow_mut().insert(id));
        if trace_call {
            let return_address = (arguments as usize)
                .checked_sub(4)
                .filter(|&p| image.permits(p, 4, false))
                .unwrap_or_else(|| unsupported("diagnostic return address outside guest storage"));
            let pc = unsafe { (return_address as *const u32).read_unaligned() };
            eprintln!(
                "PE32 diagnostic call: {}; return_pc=0x{pc:08x}; stack_words={:x?}",
                name,
                &words[..bytes as usize / 4]
            );
        }
        match id {
            0 => alkyhol::ffi::GetLastError(),
            1 => {
                alkyhol::ffi::SetLastError(args[0]);
                0
            }
            2 => {
                let index = match args[0] {
                    0xfffffff6 => 0,
                    0xfffffff5 => 1,
                    0xfffffff4 => 2,
                    _ => {
                        alkyhol::ffi::SetLastError(87);
                        return u32::MAX;
                    }
                };
                match image.handles[index] {
                    0 => 0,
                    usize::MAX => u32::MAX,
                    _ => 0x70000001 + index as u32,
                }
            }
            3 => startup32::io(image, args, true),
            4 | 5 => match image.module_handle(args[0], id == 5) {
                Ok(Some(base)) => base,
                Ok(None) => {
                    alkyhol::ffi::SetLastError(126);
                    0
                }
                Err(error) => unsupported(&error),
            },
            6 => match image.export_address(args[0], args[1]) {
                Ok(Some(address)) => address,
                Ok(None) => {
                    // Native PE32 distinguishes absent names from out-of-range
                    // ordinals (ERROR_PROC_NOT_FOUND / ERROR_INVALID_ORDINAL).
                    alkyhol::ffi::SetLastError(if args[1] <= u16::MAX as u32 { 182 } else { 127 });
                    0
                }
                Err(error) => unsupported(&error),
            },
            7..=21 => startup32::dispatch(image, id, args),
            22..=29 => fiber32::dispatch(image, id, args),
            30 | 31 => pointer_transform(args[0], image.cpu.pointer_cookie, id == 31),
            32..=38 => sync32::dispatch(image, id, args),
            39 => alkyhol::ffi::GetCurrentThreadId(),
            40 | 41 => startup32::dispatch(image, id, args),
            // Documented obsolete DOS API: Win32 deliberately has no effect
            // and returns the exact requested count. This is its API contract.
            42 => args[0],
            43..=45 => startup32::dispatch(image, id, args),
            46 => ansi32::ACP,
            47 => u32::from(matches!(args[0], ansi32::ACP | 65001)),
            48 => startup32::dispatch(image, id, args),
            49 => startup32::dispatch(image, id, args),
            50 => startup32::dispatch(image, id, args),
            51 => sync32::dispatch(image, id, args),
            52 => machine32::processor_feature(args[0]).unwrap_or_else(|e| unsupported(&e)),
            53 => startup32::dispatch(image, id, args),
            54 => image
                .cpu
                .heap
                .borrow()
                .size(args[0], args[1], args[2])
                .unwrap_or_else(|e| unsupported(&e)),
            55 => activation32::find(image, args),
            56 => module_path32::filename(image, args),
            57 => module_path32::long_name(image, args),
            58 => module_path32::system_directory(image, args),
            59..=63 => activation32::dispatch(image, id, args),
            64 => module_path32::system_windows_directory(image, args),
            65 => files32::first(image, args),
            66 => files32::next(image, args),
            67 => files32::close(image, args[0]),
            68..=70 | 304 => time32::dispatch(image, id, args),
            305 => copy_file32::dispatch(image, args),
            306 => number_format32::dispatch(image, args),
            71 => std::process::id(),
            72 | 110 => time32::tick_count(),
            73 | 74 => {
                if !args[0].is_multiple_of(4) || !image.permits(args[0] as usize, 8, true) {
                    unsupported("PE32 performance-counter output outside checked guest storage");
                }
                let value = if id == 73 {
                    alkyhol::kernel::performance_counter()
                } else {
                    alkyhol::kernel::PERFORMANCE_FREQUENCY
                };
                let bytes = value.to_le_bytes();
                unsafe {
                    ptr::copy_nonoverlapping(bytes.as_ptr(), args[0] as usize as *mut u8, 8);
                }
                1
            }
            75 | 76 => sync32::dispatch(image, id, args),
            77 => {
                image
                    .disable_thread_notifications(args[0])
                    .unwrap_or_else(|e| unsupported(&e));
                1
            }
            78..=86 => crt32::dispatch(image, id, args),
            87 => startup32::packed_version(),
            88 => crt32::initializers(image, args[0], args[1]),
            89 => unsupported("_CIsqrt requires its x87 gateway"),
            90 => math32::rejected(image, args[0], args[1]),
            91 => crt32::dispatch(image, id, args),
            92 => unsupported("_CIatan requires its x87 gateway"),
            93 => math32::rejected_atan(image, args[0], args[1]),
            94 | 95 => crt32::dispatch(image, id, args),
            96 => crt32::dll_onexit(image, args),
            97 => startup32::dispatch(image, 40, args),
            98 => module_path32::filename_ansi(image, args),
            99 => image
                .cpu
                .heap
                .borrow_mut()
                .reallocate(args[0], args[1], args[2], args[3])
                .unwrap_or_else(|e| unsupported(&e)),
            100 => cursor32::position(image, args[0]),
            299 => ui32::set_capture(image, args[0]),
            300 => ui32::release_capture(image),
            301 => ui32::get_capture(image),
            302 => crt32::set_locale(image, args[0], args[1]),
            303 => crt32::duplicate_string(image, args[0]),
            101 => module_path32::windows_directory_ansi(image, args),
            102 => module_path32::system_directory_ansi(image, args),
            103 => exception_filter32::replace(image, args[0]),
            104 | 105 => image
                .cpu
                .heap
                .borrow_mut()
                .lock(args[0], alkyhol::ffi::GetCurrentThreadId(), id == 104)
                .unwrap_or_else(|e| unsupported(&e)),
            106 => machine32::information(image, args[0]),
            107 | 108 | 216 => registry32::dispatch(image, id, args),
            109 => mutex32::create(image, args),
            111 => directory32::set(image, args[0]),
            112 => directory32::get(image, args),
            113 | 114 if image.cpu.unnamed_mutexes.borrow().contains(args[0]) => {
                unnamed_mutex32::operation(image, args, id == 113)
            }
            113 if image.cpu.watches.borrow().contains(args[0]) => thread32::wait(image, args),
            113 if image.cpu.threads.borrow().contains(args[0]) => thread32::wait(image, args),
            113 if image.cpu.events.borrow().contains(args[0]) => {
                event32::dispatch(image, id, args)
            }
            113 | 114 => mutex32::ownership(image, args, id == 113),
            115 => icon32::load(image, args),
            116..=119 => event32::dispatch(image, id, args),
            120 => shell_folder32::get(image, args),
            121 => private_profile32::get(image, args),
            122 => shell_folder32::create(image, args),
            123 => private_profile32::write(image, args),
            124 => event32::release(image, args),
            125 => event32::dispatch(image, 116, args),
            126 => event32::dispatch(image, 119, args),
            127 => bulk32::exchange_add(image, args),
            128 | 129 => bulk32::file_size(image, args, id == 129),
            130 => 0xfffffffe,
            131 => shell_folder32::create_ansi(image, args),
            132 => directory32::set_ansi(image, args[0]),
            133 => directory32::get_ansi(image, args),
            134 | 135 => bulk32::attributes(image, args[0], id == 135),
            136 => {
                if image.cpu.unnamed_mutexes.borrow().contains(args[0]) {
                    return unnamed_mutex32::operation(image, args, true);
                }
                if image.cpu.threads.borrow().contains(args[0])
                    || image.cpu.watches.borrow().contains(args[0])
                {
                    thread32::wait(image, args)
                } else if image.cpu.events.borrow().contains(args[0]) {
                    event32::dispatch(image, 113, args)
                } else {
                    mutex32::ownership(image, args, true)
                }
            }
            137 => startup32::create_file(image, args, true),
            138 | 139 => path32::full(image, args, id == 139),
            140 => files32::first_kind(image, args, true),
            141 => files32::next_kind(image, args, true),
            142 => {
                if args[0] > 60000 {
                    unsupported("Sleep exceeds the bounded non-alertable host wait profile");
                }
                thread32::without_cpu(image, || alkyhol::ffi::Sleep(args[0]));
                0
            }
            143 | 144 => fileio32::seek(image, args, id == 144),
            145 => fileio32::flush(image, args[0]),
            146 => fileio32::information(image, args),
            147 => fileio32::end(image, args[0]),
            148..=176 => ui32::dispatch(image, id - 148, &words),
            177 => {
                #[cfg(target_arch = "x86_64")]
                {
                    raise32::deliver(image, args)
                }
                #[cfg(target_arch = "aarch64")]
                {
                    raise32::deliver_registers(image, executor32::exception_registers())
                }
            }
            178 => u32::MAX, // The documented current-process pseudo handle.
            179 => machine32::affinity(image, args),
            180 => {
                if args[0] == u32::MAX {
                    std::process::id()
                } else {
                    alkyhol::ffi::SetLastError(6);
                    0
                }
            }
            181 => thread32::create(image, args),
            182 => thread32::query(image, args, 0),
            183 => thread32::query(image, args, 1),
            184 => thread32::query(image, args, 2),
            185 => thread32::exit_thread(args[0]),
            186 => {
                alkyhol::ffi::SetLastError(50);
                0
            } // Native priority mapping is not implemented.
            187 => gdi32::load(image, args),
            188..=197 => gdi32::dispatch(image, id - 188, &words),
            198..=201 => ui32::dispatch(image, id - 198 + 29, &words),
            202 => gdi32::dib_section(image, args),
            203..=211 => ui32::dispatch(image, id - 203 + 33, &words),
            212 => watch32::read(image, args),
            213 => watch32::result(image, args),
            214 => watch32::cancel(image, args[0]),
            217 => thread32::switch_to_thread(image),
            218 | 219 => ui32::message_box(image, args, id == 219),
            220 | 221 => registry32::query(image, args, id == 220),
            222 | 223 => loader32::load(image, args[0], id == 222),
            224 => loader32::free(image, args[0]),
            225..=237 => winsock32::dispatch(image, id - 225, args),
            238 => startup32::version(image, args[0], true),
            239 | 240 => section32::create(image, args, id == 239),
            241 | 242 => section32::open(image, args, id == 241),
            243 => section32::map(image, args),
            244 => section32::unmap(image, args[0]),
            245 | 246 => machine32::set_affinity(image, args, id == 245),
            247 => gdi32::stock(image, args[0]),
            248 => gdi32::solid(image, args[0]),
            249 => gdi32::fill_rect(image, args),
            250 | 251 => image32::load(image, args, id == 251),
            252 => image32::set(image, args[0]),
            253 => image32::destroy(image, args[0]),
            254 => image32::current(image),
            255 => d3d9_32::create(image, args[0]),
            256 => ui32::dispatch(image, 42, &words),
            257 => ui32::window_info(image, args[0], args[1]),
            258 => cursor32::clip(image, args[0]),
            307 => cursor32::clipping_rectangle(image, args[0]),
            308 => ui32::show_cursor(image, args[0]),
            309 | 310 => shell_folder32::delete(image, args[0], id == 310),
            259 => input8_32::create(image, &words),
            260 | 262 => crt32::dispatch(image, 78, args),
            261 | 263 | 264 => crt32::deallocate(image, args[0]),
            265..=269 => crt32::memory(image, id, args),
            270 => mutex32::create_ansi(image, args),
            271 => fileio32::handle_information(image, args[0], args[1]),
            272..=274 => com32::dispatch(image, id, args),
            275 => gdi32::flush(),
            276 => com32::create(image, args),
            277 | 278 => wave32::count(image, id == 278),
            279..=283 => winsock32::event_api(image, id, args),
            284 => disk32::query(image, args),
            285 => {
                // No guest Windows service controller/database is implemented.
                // SC_HANDLE NULL is failure, never an invented successful object.
                eprintln!("PE32 OpenSCManagerW unavailable: ERROR_CALL_NOT_IMPLEMENTED (120)");
                alkyhol::ffi::SetLastError(120);
                0
            }
            286 | 287 => winsock32::broadcast_option(image, args, id == 287),
            288 | 289 => winsock32::endpoint(image, args, id == 289),
            290 | 291 => winsock32::datagram(image, args, id == 291),
            292 => winsock32::wsa_receive(image, &words),
            293 => netbios32::reset_unavailable(image, args[0]),
            294 => winsock32::select_sockets(image, args),
            297 => event32::wait_set(image, args),
            298 => registry32::legacy_open(image, args),
            295 | 296 => {
                // No minimum periodic-timer resolution policy is configured.
                // TIMERR_NOCANDO is failure, not an accepted timer request.
                eprintln!(
                    "PE32 {} unavailable: TIMERR_NOCANDO (97); period={}ms; timer policy unchanged",
                    APIS[id as usize].0, args[0]
                );
                97
            }
            311 | 312 => dsound32::create(image, args),
            313 | 314 => dsound32::enumerate(image, args, id == 314),
            315 => dsound32::get_device_id(image, args),
            316 | 317 => {
                if args[1] != 0 {
                    dsound32::clear_output(image, args[1]);
                }
                0x80004001
            }
            318 | 319 => 0x80004001,
            dsound32::BASE.. => dsound32::dispatch(image, id, &words),
            input8_32::BASE.. => input8_32::dispatch(image, id, &words),
            d3d9_32::BASE.. => d3d9_32::dispatch(image, id, &words),
            215 => {
                // No adapter/DHCP projection is implemented. Return the explicit
                // API failure and let the original caller handle it; never invent
                // an empty adapter list or successful hardware identity.
                eprintln!("PE32 GetAdaptersInfo unavailable: ERROR_NOT_SUPPORTED (50)");
                50
            }
            _ => unreachable!(),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn window_info_keeps_existing_gate_indices_and_user32_ownership() {
        let symbol = ImportSymbol::Name("GetWindowInfo".into());
        assert_eq!(api_id("USER32.dll", &symbol), Some(257));
        assert_eq!(api_id("kernel32.dll", &symbol), None);
        assert_eq!(APIS[257], ("GetWindowInfo", 8));
    }
    #[test]
    fn com_and_gdi_flush_imports_have_correct_module_and_stdcall_ownership() {
        for (id, module) in [
            (272, "ole32.dll"),
            (273, "ole32.dll"),
            (274, "ole32.dll"),
            (275, "gdi32.dll"),
            (276, "ole32.dll"),
        ] {
            let symbol = ImportSymbol::Name(APIS[id].0.into());
            assert_eq!(api_id(module, &symbol), Some(id));
            assert_eq!(api_id("kernel32.dll", &symbol), None);
            assert_eq!(cleanup_bytes(id, APIS[id].1), APIS[id].1);
        }
    }
    #[test]
    fn wave_device_count_gates_belong_to_winmm_with_no_stack_arguments() {
        for id in [277, 278] {
            let symbol = ImportSymbol::Name(APIS[id].0.into());
            assert_eq!(api_id("WINMM.dll", &symbol), Some(id));
            assert_eq!(api_id("kernel32.dll", &symbol), None);
            assert_eq!(APIS[id].1, 0);
        }
    }
    #[test]
    fn lan_socket_ordinals_alias_names_and_extended_receive_has_full_stack_extent() {
        for (ordinal, id) in [
            (21, 286),
            (7, 287),
            (2, 288),
            (6, 289),
            (20, 290),
            (17, 291),
            (18, 294),
        ] {
            assert_eq!(
                api_id("ws2_32.dll", &ImportSymbol::Ordinal(ordinal)),
                Some(id)
            );
            assert_eq!(
                api_id("wsock32.dll", &ImportSymbol::Ordinal(ordinal)),
                Some(id)
            );
            assert_eq!(
                api_id("ws2_32.dll", &ImportSymbol::Name(APIS[id].0.into())),
                Some(id)
            );
        }
        assert_eq!(
            api_id("ws2_32.dll", &ImportSymbol::Name("WSARecvFrom".into())),
            Some(292)
        );
        assert_eq!(APIS[292].1, 36);
        assert_eq!(cleanup_bytes(292, 36), 36);
        assert_eq!(
            api_id("advapi32.dll", &ImportSymbol::Name("OpenSCManagerW".into())),
            Some(285)
        );
        assert_eq!(APIS[285].1, 12);
    }
    #[test]
    fn legacy_registry_uses_three_stdcall_arguments_and_advapi32() {
        let symbol = ImportSymbol::Name("RegOpenKeyA".into());
        assert_eq!(api_id("advapi32.dll", &symbol), Some(298));
        assert_eq!(api_id("kernel32.dll", &symbol), None);
        assert_eq!(APIS[298].1, 12);
        assert_eq!(cleanup_bytes(298, 12), 12);
    }
    #[test]
    fn mouse_capture_imports_use_user32_and_their_measured_stdcall_widths() {
        for (id, bytes) in [(299, 4), (300, 0), (301, 0)] {
            let symbol = ImportSymbol::Name(APIS[id].0.into());
            assert_eq!(api_id("user32.dll", &symbol), Some(id));
            assert_eq!(api_id("kernel32.dll", &symbol), None);
            assert_eq!(APIS[id].1, bytes);
            assert_eq!(cleanup_bytes(id, bytes), bytes);
        }
    }
    #[test]
    fn win32_event_sets_use_kernel32_without_winsock_startup() {
        let symbol = ImportSymbol::Name("WaitForMultipleObjects".into());
        assert_eq!(api_id("kernel32.dll", &symbol), Some(297));
        assert_eq!(api_id("ws2_32.dll", &symbol), None);
        assert_eq!(APIS[297].1, 16);
        assert_eq!(cleanup_bytes(297, APIS[297].1), 16);
    }
    #[test]
    fn multimedia_timer_requests_are_winmm_uint_stdcall_operations() {
        for id in [295, 296] {
            let symbol = ImportSymbol::Name(APIS[id].0.into());
            assert_eq!(api_id("WINMM.dll", &symbol), Some(id));
            assert_eq!(api_id("kernel32.dll", &symbol), None);
            assert_eq!(APIS[id].1, 4);
            assert_eq!(cleanup_bytes(id, APIS[id].1), 4);
        }
    }
    #[test]
    fn netbios_uses_netapi32_and_one_stdcall_pointer() {
        let symbol = ImportSymbol::Name("Netbios".into());
        assert_eq!(api_id("NETAPI32.dll", &symbol), Some(293));
        assert_eq!(api_id("kernel32.dll", &symbol), None);
        assert_eq!(APIS[293].1, 4);
        assert_eq!(cleanup_bytes(293, APIS[293].1), 4);
    }
    #[test]
    fn current_local_time_is_one_kernel32_stdcall_output_pointer() {
        let symbol = ImportSymbol::Name("GetLocalTime".into());
        assert_eq!(api_id("kernel32.dll", &symbol), Some(304));
        assert_eq!(api_id("user32.dll", &symbol), None);
        assert_eq!(APIS[304].1, 4);
        assert_eq!(cleanup_bytes(304, 4), 4);
    }
    #[test]
    fn custom_number_format_keeps_prior_gate_offsets_and_uses_six_stdcall_arguments() {
        let symbol = ImportSymbol::Name("GetNumberFormatW".into());
        assert_eq!(api_id("kernel32.dll", &symbol), Some(306));
        assert_eq!(api_id("user32.dll", &symbol), None);
        assert_eq!(APIS[306].1, 24);
        assert_eq!(cleanup_bytes(306, 24), 24);
        assert_eq!(APIS[304].0, "GetLocalTime");
        assert_eq!(APIS[305].0, "CopyFileW");
    }
    #[test]
    fn delete_gateways_are_kernel32_one_pointer_stdcall_imports() {
        for (name, id) in [("DeleteFileW", 309), ("DeleteFileA", 310)] {
            let symbol = ImportSymbol::Name(name.into());
            assert_eq!(api_id("kernel32.dll", &symbol), Some(id));
            assert_eq!(api_id("user32.dll", &symbol), None);
            assert_eq!(APIS[id as usize].1, 4);
            assert_eq!(cleanup_bytes(id, 4), 4);
        }
    }
    #[test]
    fn clip_query_is_one_user32_stdcall_rect_pointer() {
        let symbol = ImportSymbol::Name("GetClipCursor".into());
        assert_eq!(api_id("USER32.dll", &symbol), Some(307));
        assert_eq!(api_id("kernel32.dll", &symbol), None);
        assert_eq!(APIS[307].1, 4);
        assert_eq!(cleanup_bytes(307, 4), 4);
        let symbol = ImportSymbol::Name("ShowCursor".into());
        assert_eq!(api_id("USER32.dll", &symbol), Some(308));
        assert_eq!(api_id("kernel32.dll", &symbol), None);
        assert_eq!(APIS[308].1, 4);
        assert_eq!(cleanup_bytes(308, 4), 4);
    }
    #[test]
    fn cdecl_crt_gateways_keep_arguments_for_the_guest_caller() {
        let locale = ImportSymbol::Name("setlocale".into());
        assert_eq!(api_id("msvcrt.dll", &locale), Some(302));
        assert_eq!(api_id("kernel32.dll", &locale), None);
        assert_eq!(APIS[302].1, 8);
        assert_eq!(cleanup_bytes(302, 8), 0);
        assert_eq!(
            api_id("msvcrt.dll", &ImportSymbol::Name("_strdup".into())),
            Some(303)
        );
        assert_eq!(APIS[303].1, 4);
        assert_eq!(cleanup_bytes(303, 4), 0);
        for id in 0..120 {
            assert_eq!(api_offset(id), 256 + id * 32);
        }
        assert_eq!(api_offset(120), math32::ATAN_OFFSET + 256);
        assert_eq!(api_offset(231) + 32, raise32::OFFSET);
        assert_eq!(api_offset(232), raise32::OFFSET + 512);
        assert_eq!(api_offset(343) + 32, math32::GATE_SIZE);
        assert!(api_offset(344) + 32 > math32::GATE_SIZE);
        for id in 0..APIS.len() {
            let at = api_offset(id);
            assert!(at >= 256 && at + 32 <= math32::GATE_SIZE);
            assert!(at + 32 <= math32::OFFSET || at >= math32::ATAN_OFFSET + 256);
            assert!(at + 32 <= raise32::OFFSET || at >= raise32::OFFSET + 512);
            if id > 0 {
                assert!(api_offset(id - 1) + 32 <= at);
            }
        }
        assert_eq!(
            api_id(
                "shell32.dll",
                &ImportSymbol::Name("SHGetFolderPathW".into())
            ),
            Some(120)
        );
        assert_eq!(
            api_id(
                "kernel32.dll",
                &ImportSymbol::Name("SHGetFolderPathW".into())
            ),
            None
        );
        assert!(
            api_id(
                "msvcrt.dll",
                &ImportSymbol::Name("<private x87 rejection>".into())
            )
            .is_none()
        );
        let math_id = api_id("msvcrt.dll", &ImportSymbol::Name("_CIsqrt".into())).unwrap();
        assert_eq!(math_id, 89);
        assert_eq!(APIS[math_id].1, 0); // Real operands/results live on the x87 stack.
        assert!(
            api_id(
                "msvcrt.dll",
                &ImportSymbol::Name("<private atan rejection>".into())
            )
            .is_none()
        );
        let atan = api_id("msvcrt.dll", &ImportSymbol::Name("_CIatan".into())).unwrap();
        assert_eq!(atan, 92);
        assert_eq!(APIS[atan].1, 0);
        for name in [
            "malloc",
            "??2@YAPAXI@Z",
            "??3@YAXPAX@Z",
            "??_U@YAPAXI@Z",
            "??_V@YAXPAX@Z",
            "free",
            "_set_error_mode",
            "?set_terminate@@YAP6AXXZP6AXXZ@Z",
            "__set_app_type",
        ] {
            let symbol = ImportSymbol::Name(name.into());
            let id = api_id("MSVCRT.DLL", &symbol).unwrap();
            assert_eq!(APIS[id].1, 4);
            assert_eq!(cleanup_bytes(id, APIS[id].1), 0);
            assert!(api_id("kernel32.dll", &symbol).is_none());
            assert!(api_id("msvcr80.dll", &symbol).is_none());
        }
        let id = api_id("kernel32.dll", &ImportSymbol::Name("WriteFile".into())).unwrap();
        assert_eq!(cleanup_bytes(id, APIS[id].1), 20);
        assert!(!cdecl_api(APIS.len()));
        let symbol = ImportSymbol::Name("__get_app_type".into());
        assert!(api_id("msvcrt.dll", &symbol).is_none());
        let id = api_id("msvcrt.dll", &ImportSymbol::Name("_initterm".into())).unwrap();
        assert_eq!(APIS[id].1, 8);
        assert_eq!(cleanup_bytes(id, 8), 0);
        assert!(api_id("kernel32.dll", &ImportSymbol::Name("GetVersion".into())).is_some());
        let timer = ImportSymbol::Name("timeGetTime".into());
        let id = api_id("WINMM.dll", &timer).unwrap();
        assert_eq!(cleanup_bytes(id, APIS[id].1), 0);
        assert!(api_id("kernel32.dll", &timer).is_none());
    }
    #[test]
    fn pointer_encoding_roundtrips_the_entire_dword_without_dereferencing() {
        for cookie in [0, 1, 31, 0x12345678, u32::MAX] {
            for value in [0, 1, 0x80000000, 0xdeadbeef, u32::MAX] {
                assert_eq!(
                    pointer_transform(pointer_transform(value, cookie, false), cookie, true),
                    value
                );
            }
        }
    }
    // No mapping is dereferenced or dropped: pure address/metadata assertions,
    // separate from the actual unchanged-PE execution probe.
    pub(super) fn layout() -> std::mem::ManuallyDrop<LoadedImage32<'static>> {
        std::mem::ManuallyDrop::new(LoadedImage32 {
            image: LowMapping {
                base: ptr::null_mut(),
                len: 0,
            },
            exports: None,
            gates: LowMapping {
                base: ptr::null_mut(),
                len: 0,
            },
            cpu: Rc::new(Cpu32 {
                #[cfg(target_arch = "aarch64")]
                native_generation: native_generation(),
                #[cfg(target_arch = "aarch64")]
                native_gates: Default::default(),
                #[cfg(target_arch = "aarch64")]
                native_code: Default::default(),
                stack: LowMapping {
                    base: 0x400000usize as *mut u8,
                    len: STACK + 2 * PAGE,
                },
                teb: LowMapping {
                    base: 0x600000usize as *mut u8,
                    len: PAGE,
                },
                blocks: RefCell::new(Vec::new()),
                command: Cell::new(0),
                command_ansi: Cell::new(0),
                exit: Cell::new(None),
                files: RefCell::new(Default::default()),
                file_leases: RefCell::new(Default::default()),
                searches: RefCell::new(Default::default()),
                phase: Cell::new(Phase32::Isolated),
                modules: RefCell::new(Vec::new()),
                dynamic_order: RefCell::new(Vec::new()),
                builtins: RefCell::new(Default::default()),
                executable: Cell::new(0),
                diagnostic: None,
                trace_first_calls: matches!(
                    std::env::var("ALKYHOL_PE32_TRACE_CALLS").as_deref(),
                    Ok("first")
                ),
                trace_seen: RefCell::new(Default::default()),
                heap: RefCell::new(Default::default()),
                crt: RefCell::new(Default::default()),
                fls: RefCell::new(Default::default()),
                dynamic_tls: RefCell::new([false; 64]),
                callback_stack: Cell::new(0),
                callback_depth: Cell::new(0),
                pointer_cookie: 0x89345621,
                critical_sections: RefCell::new(Default::default()),
                service_gates: Cell::new(0),
                activation: RefCell::new(Err("test layout has no activation context".into())),
                bound_activation: RefCell::new(None),
                system_directory: None,
                system_windows_directory: None,
                windows_directory: None,
                documents: None,
                private_profile_files: false,
                casefold_file_opens: false,
                unhandled_filter: Cell::new(0),
                machine: None,
                registry: None,
                mutexes: None,
                current_directory: RefCell::new(None),
                icons: RefCell::new(Default::default()),
                events: RefCell::new(Default::default()),
                ui: RefCell::new(Default::default()),
                gdi: RefCell::new(Default::default()),
                utc_timezone: false,
                version_workstation: false,
                threads: RefCell::new(Default::default()),
                watches: RefCell::new(Default::default()),
                winsock: RefCell::new(Default::default()),
                sections: RefCell::new(Default::default()),
                images: RefCell::new(Default::default()),
                d3d9: RefCell::new(Default::default()),
                input8: RefCell::new(Default::default()),
                dsound: RefCell::new(Default::default()),
                unnamed_mutexes: RefCell::new(Default::default()),
                com: RefCell::new(Default::default()),
                wave: RefCell::new(Default::default()),
                com_storage: RefCell::new(Default::default()),
                com_arenas: RefCell::new(Default::default()),
            }),
            tls: None,
            tls_entered: Cell::new(false),
            entry: 0,
            relocated: false,
            code: 0,
            data: 0,
            dll: false,
            handles: [0; 3],
            regions: vec![
                (0x100000, 0x101000, 0x60000000),
                (0x200000, 0x201000, 0xc0000000),
            ],
            code_regions: vec![(0x100000, 0x100200)],
            seh_handlers: Vec::new(),
            _process: PhantomData,
        })
    }
    #[test]
    fn buffer_validation_obeys_actual_section_permissions_stack_bounds_and_overflow() {
        let image = layout();
        assert!(image.permits(0x100000, 4096, false));
        assert!(!image.permits(0x100000, 1, true));
        assert!(image.permits(0x200000, 4096, true));
        assert!(!image.permits(0x200fff, 2, true));
        assert!(!image.permits(0x101000, 1, false));
        assert!(!image.permits(0, 4, false));
        assert!(!image.permits(0x400fff, 1, false));
        assert!(image.permits(0x401000, STACK, true));
        assert!(!image.permits(0x401000, STACK + 1, true));
        assert!(!image.permits(usize::MAX - 1, 4, false));
    }
    #[test]
    fn export_query_names_are_owned_bounded_ascii_and_never_read_after_failure() {
        assert_eq!(
            query_name(100, |a| Ok(b"Data\0"[(a - 100) as usize])).unwrap(),
            "Data"
        );
        for data in [&b"\0"[..], &b"\xff\0"[..]] {
            assert!(query_name(0, |a| Ok(data[a as usize])).is_err());
        }
        let mut calls = 0;
        assert!(
            query_name(0, |_| {
                calls += 1;
                Ok(b'X')
            })
            .is_err()
        );
        assert_eq!(calls, 4096);
        calls = 0;
        assert!(
            query_name(u32::MAX, |_| {
                calls += 1;
                Ok(b'X')
            })
            .is_err()
        );
        assert_eq!(calls, 1);
        calls = 0;
        assert!(
            query_name(100, |_| {
                calls += 1;
                Err("unreadable".into())
            })
            .is_err()
        );
        assert_eq!(calls, 1);
        assert!(layout().export_name(0).is_err());
    }
    #[test]
    fn self_export_addresses_require_own_module_and_preserve_biased_ordinals() {
        let mut image = layout();
        image.image.base = 0x100000usize as *mut u8;
        assert_eq!(image.export_address(0x100000, 17).unwrap(), None);
        assert_eq!(image.export_address(0, 17).unwrap(), None);
        image.exports = Some(ExportTable {
            dll_name: "owned.exe".into(),
            ordinal_base: 17,
            entries: vec![
                crate::Export {
                    ordinal: 17,
                    names: vec!["Direct".into(), "Alias".into()],
                    target: ExportTarget::Address { rva: 0x1000 },
                },
                crate::Export {
                    ordinal: 18,
                    names: vec![],
                    target: ExportTarget::Address { rva: 0x2000 },
                },
            ],
        });
        assert_eq!(image.export_address(0x100000, 17).unwrap(), Some(0x101000));
        assert_eq!(image.export_address(0x100000, 18).unwrap(), Some(0x102000));
        for ordinal in [0, 16, 19, 65535] {
            assert_eq!(image.export_address(0x100000, ordinal).unwrap(), None);
        }
        assert_eq!(image.export_address(0, 17).unwrap(), Some(0x101000));
        assert!(image.export_address(0x200000, 17).is_err());
        assert!(image.export_address(0x100000, 0xdead0000).is_err());
        image.exports.as_mut().unwrap().entries[0].target =
            ExportTarget::Forwarder("kernel32.Sleep".into());
        assert!(image.export_address(0x100000, 17).is_err());
        assert!(image.export_address(0, 17).is_err());
        image.exports.as_mut().unwrap().entries[0].target =
            ExportTarget::Address { rva: 0xffff0000 };
        assert!(image.export_address(0x100000, 17).is_err());
        assert!(image.export_address(0, 17).is_err());
    }
    #[test]
    fn teb_descriptor_has_a_bounded_writable_user_data_segment_and_exact_base() {
        for base in [0, 0x12345678u32, 0xfffff000] {
            let d = teb_descriptor(base);
            let decoded_base =
                ((d >> 16) & 0xffff) | (((d >> 32) & 0xff) << 16) | (((d >> 56) & 0xff) << 24);
            let limit = (d & 0xffff) | (((d >> 48) & 0xf) << 16);
            assert_eq!(decoded_base, u64::from(base));
            assert_eq!(limit, PAGE as u64 - 1);
            assert_eq!((d >> 40) & 0xff, 0xf2); // Present, user, writable data.
            assert_eq!((d >> 52) & 0xf, 4); // Byte limit, 32-bit, no long-mode bit.
        }
    }
    #[test]
    fn minimal_teb_initializes_self_stack_and_error_without_inventing_tls_or_peb() {
        let b = teb_bytes(0x600000, 0x401000, STACK as u32).unwrap();
        let read = |at| u32::from_le_bytes(b[at..at + 4].try_into().unwrap());
        assert_eq!(read(0), u32::MAX);
        assert_eq!(read(4), 0x501000);
        assert_eq!(read(8), 0x401000);
        assert_eq!(read(0x18), 0x600000);
        for at in [0x2c, 0x30, 0x34] {
            assert_eq!(read(at), 0);
        }
        assert!(teb_bytes(0x600000, u32::MAX - 4, 8).is_err());
        assert!(teb_bytes(0x600000, 0x401000, 0).is_err());
        assert!(teb_bytes(u32::MAX - 4, 0x401000, 4096).is_err());
        let image = layout();
        assert!(image.permits(0x600000, PAGE, false));
        assert!(image.permits(0x600034, 4, true));
        assert!(!image.permits(0x600fff, 2, true));
        assert!(!image.permits(0x5fffff, 1, false));
    }
    #[test]
    fn last_error_guard_reflects_early_host_outcomes_and_exclusive_entry_fails_before_ldt() {
        let previous = alkyhol::ffi::GetLastError();
        let mut guest_error = 0x31415926u32;
        {
            let _sync = TebError(&mut guest_error);
            alkyhol::ffi::SetLastError(127);
        }
        assert_eq!(guest_error, 127);
        alkyhol::ffi::SetLastError(previous);
        let _occupied = FS_SLOT.lock().unwrap();
        assert!(TebLease::install(0x600000).is_err());
    }
    #[test]
    fn tls_buffer_permissions_use_the_logical_template_extent_without_page_padding() {
        let mut image = layout();
        image.tls = Some(TlsBlock {
            mapping: LowMapping {
                base: 0x700000usize as *mut u8,
                len: PAGE,
            },
            size: 28,
            initial: vec![0; 28],
            alignment: PAGE,
        });
        assert!(image.permits(0x700000, 28, false));
        assert!(image.permits(0x700018, 4, true));
        assert!(!image.permits(0x700000, 29, true));
        assert!(!image.permits(0x70001c, 1, false));
        assert!(!image.permits(0x6fffff, 1, false));
    }
    #[test]
    fn cookie_policy_preserves_legacy64_and_nondefault_seeds_without_requesting_entropy() {
        for (size, seed) in [
            (64, DEFAULT_GS_COOKIE),
            (64, 0x1234abcd),
            (72, 0),
            (72, 0x1234),
            (72, 0x1234abcd),
        ] {
            assert_eq!(
                select_cookie(size, seed, || panic!("entropy must not be requested")).unwrap(),
                seed
            );
        }
        let mut values = [0, DEFAULT_GS_COOKIE, 0x1234abcd].into_iter();
        assert_eq!(
            select_cookie(72, DEFAULT_GS_COOKIE, || Ok(values.next().unwrap())).unwrap(),
            0x1234abcd
        );
        assert_eq!(values.next(), None);
    }
    #[test]
    fn unavailable_or_exhausted_entropy_never_becomes_a_successful_fixed_cookie() {
        assert!(
            select_cookie(72, DEFAULT_GS_COOKIE, || Err("entropy unavailable".into())).is_err()
        );
        let mut requests = 0;
        assert!(
            select_cookie(72, DEFAULT_GS_COOKIE, || {
                requests += 1;
                Ok(0)
            })
            .is_err()
        );
        assert_eq!(requests, 128);
    }
    #[test]
    fn call_frames_align_arguments_bound_arity_and_distinguish_actual_stack_cleanup() {
        for count in 0..=16 {
            let (cdecl, c_return) = call_stack(count, CallingConvention32::Cdecl).unwrap();
            let (stdcall, s_return) = call_stack(count, CallingConvention32::Stdcall).unwrap();
            assert_eq!(cdecl, stdcall);
            assert_eq!((cdecl + 4) % 16, 0);
            assert!(cdecl >= PAGE && cdecl + 4 + count * 4 <= PAGE + STACK);
            assert_eq!(c_return, cdecl + 4);
            assert_eq!(s_return, cdecl + 4 + count * 4);
        }
        for count in [17, usize::MAX] {
            assert!(call_stack(count, CallingConvention32::Cdecl).is_err());
            assert!(call_stack(count, CallingConvention32::Stdcall).is_err());
        }
    }
    #[test]
    fn explicit_dependency_roster_requires_unique_basenames_and_one_actual_cpu_context() {
        let a = std::mem::ManuallyDrop::new(LoadedLibrary32 {
            mapping: std::mem::ManuallyDrop::into_inner(layout()),
            attached: false,
        });
        let mut b = std::mem::ManuallyDrop::new(LoadedLibrary32 {
            mapping: std::mem::ManuallyDrop::into_inner(layout()),
            attached: false,
        });
        assert!(dependency_cpu(&[]).unwrap().is_none());
        let cpu = dependency_cpu(&[("Leaf.DLL", &a)]).unwrap().unwrap();
        assert!(Rc::ptr_eq(&cpu, &a.mapping.cpu));
        for name in ["../leaf.dll", "leaf", "kernel32.dll", "", "/leaf.dll"] {
            assert!(dependency_cpu(&[(name, &a)]).is_err());
        }
        assert!(dependency_cpu(&[("leaf.dll", &a), ("LEAF.DLL", &a)]).is_err());
        assert!(dependency_cpu(&[("leaf.dll", &a), ("other.dll", &b)]).is_err());
        // The synthetic mappings are never dereferenced or freed.
        let _unused = std::mem::ManuallyDrop::new(std::mem::replace(
            &mut b.mapping.cpu,
            Rc::clone(&a.mapping.cpu),
        ));
        assert!(dependency_cpu(&[("leaf.dll", &a), ("other.dll", &b)]).is_ok());
        assert!(dependency_cpu(&vec![("leaf.dll", &*a); 65]).is_err());
    }
    #[test]
    fn dependency_symbol_resolution_keeps_names_ordinals_data_and_checked_addresses() {
        let mut lib = std::mem::ManuallyDrop::new(LoadedLibrary32 {
            mapping: std::mem::ManuallyDrop::into_inner(layout()),
            attached: false,
        });
        lib.mapping.image.base = 0x100000usize as *mut u8;
        assert!(
            lib.import_address(&ImportSymbol::Name("Function".into()))
                .is_err()
        );
        lib.mapping.exports = Some(ExportTable {
            dll_name: "owned.dll".into(),
            ordinal_base: 17,
            entries: vec![
                crate::Export {
                    ordinal: 17,
                    names: vec!["Function".into()],
                    target: ExportTarget::Address { rva: 0x1000 },
                },
                crate::Export {
                    ordinal: 18,
                    names: vec!["Data".into()],
                    target: ExportTarget::Address { rva: 0x2000 },
                },
            ],
        });
        assert_eq!(
            lib.import_address(&ImportSymbol::Name("Function".into()))
                .unwrap(),
            0x101000
        );
        assert_eq!(
            lib.import_address(&ImportSymbol::Name("Data".into()))
                .unwrap(),
            0x102000
        );
        assert_eq!(
            lib.import_address(&ImportSymbol::Ordinal(18)).unwrap(),
            0x102000
        );
        for symbol in [
            ImportSymbol::Name("function".into()),
            ImportSymbol::Name("Missing".into()),
            ImportSymbol::Ordinal(0),
            ImportSymbol::Ordinal(19),
        ] {
            assert!(lib.import_address(&symbol).is_err());
        }
        lib.mapping.exports.as_mut().unwrap().entries[0].ordinal = 0;
        assert!(lib.import_address(&ImportSymbol::Ordinal(0)).is_err());
        lib.mapping.exports.as_mut().unwrap().entries[0].ordinal = 17;
        lib.mapping.exports.as_mut().unwrap().entries[0].target =
            ExportTarget::Forwarder("other.F".into());
        assert!(lib.import_address(&ImportSymbol::Ordinal(17)).is_err());
        lib.mapping.exports.as_mut().unwrap().entries[0].target =
            ExportTarget::Address { rva: 0xffff0000 };
        assert!(lib.import_address(&ImportSymbol::Ordinal(17)).is_err());
    }
}
