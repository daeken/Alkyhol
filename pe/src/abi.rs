//! These shims use Win64 on their guest side and Alkyhol's native ABI internally.
//! Pointer arguments retain the core ffi contract: guest code must provide valid
//! mapped host memory. Guest callbacks/COM are not passed to native ABI functions.
#![allow(non_snake_case)]
use alkyhol::ffi;
use std::ffi::c_void;

macro_rules! bridge {
    ($name:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty) => {
        unsafe extern "win64" fn $name($($arg: $ty),*) -> $ret {
            // SAFETY: same pointee/width contract; compiler performs the ABI bridge.
            unsafe { ffi::$name($($arg),*) }
        }
    };
}
// Calling safe core exports inside the macro's unsafe block is intentional.
#[allow(unused_unsafe)]
mod shims {
    use super::*;
    bridge!(GetLastError() -> u32);
    bridge!(SetLastError(error: u32) -> ());
    bridge!(CloseHandle(h: usize) -> i32);
    bridge!(
        CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            security: *const c_void,
            disposition: u32,
            flags: u32,
            template: usize,
        ) -> usize
    );
    bridge!(
        ReadFile(
            h: usize,
            buffer: *mut c_void,
            count: u32,
            transferred: *mut u32,
            overlapped: *mut c_void,
        ) -> i32
    );
    bridge!(
        WriteFile(
            h: usize,
            buffer: *const c_void,
            count: u32,
            transferred: *mut u32,
            overlapped: *mut c_void,
        ) -> i32
    );
    bridge!(GetFileSizeEx(h: usize, size: *mut i64) -> i32);
    bridge!(SetFilePointerEx(h: usize, distance: i64, position: *mut i64, method: u32) -> i32);
    bridge!(SetEndOfFile(h: usize) -> i32);
    bridge!(FlushFileBuffers(h: usize) -> i32);
    bridge!(
        CreateEventW(security: *const c_void, manual: i32, initial: i32, name: *const u16) -> usize
    );
    bridge!(SetEvent(h: usize) -> i32);
    bridge!(ResetEvent(h: usize) -> i32);
    bridge!(
        CreateSemaphoreW(
            security: *const c_void,
            initial: i32,
            maximum: i32,
            name: *const u16,
        ) -> usize
    );
    bridge!(ReleaseSemaphore(h: usize, count: i32, previous: *mut i32) -> i32);
    bridge!(WaitForSingleObject(h: usize, ms: u32) -> u32);
    bridge!(QueryPerformanceCounter(out: *mut i64) -> i32);
    bridge!(QueryPerformanceFrequency(out: *mut i64) -> i32);
    bridge!(Sleep(ms: u32) -> ());
    bridge!(TlsAlloc() -> u32);
    bridge!(TlsFree(index: u32) -> i32);
    bridge!(TlsGetValue(index: u32) -> *mut c_void);
    bridge!(TlsSetValue(index: u32, value: *mut c_void) -> i32);
    unsafe extern "win64" fn ExitProcess(status: u32) -> ! {
        // Unix process status cannot represent the full Windows DWORD.
        if status > 255 {
            eprintln!("unsupported Windows exit status {status:#x} (Unix status is 8-bit)");
            std::process::exit(125);
        }
        std::process::exit(status as i32)
    }
    pub(super) fn address(name: &str) -> Option<usize> {
        match name {
            "GetLastError" => Some(GetLastError as *const () as usize),
            "SetLastError" => Some(SetLastError as *const () as usize),
            "CloseHandle" => Some(CloseHandle as *const () as usize),
            "CreateFileW" => Some(CreateFileW as *const () as usize),
            "ReadFile" => Some(ReadFile as *const () as usize),
            "WriteFile" => Some(WriteFile as *const () as usize),
            "GetFileSizeEx" => Some(GetFileSizeEx as *const () as usize),
            "SetFilePointerEx" => Some(SetFilePointerEx as *const () as usize),
            "SetEndOfFile" => Some(SetEndOfFile as *const () as usize),
            "FlushFileBuffers" => Some(FlushFileBuffers as *const () as usize),
            "CreateEventW" => Some(CreateEventW as *const () as usize),
            "SetEvent" => Some(SetEvent as *const () as usize),
            "ResetEvent" => Some(ResetEvent as *const () as usize),
            "CreateSemaphoreW" => Some(CreateSemaphoreW as *const () as usize),
            "ReleaseSemaphore" => Some(ReleaseSemaphore as *const () as usize),
            "WaitForSingleObject" => Some(WaitForSingleObject as *const () as usize),
            "QueryPerformanceCounter" => Some(QueryPerformanceCounter as *const () as usize),
            "QueryPerformanceFrequency" => Some(QueryPerformanceFrequency as *const () as usize),
            "Sleep" => Some(Sleep as *const () as usize),
            "TlsAlloc" => Some(TlsAlloc as *const () as usize),
            "TlsFree" => Some(TlsFree as *const () as usize),
            "TlsGetValue" => Some(TlsGetValue as *const () as usize),
            "TlsSetValue" => Some(TlsSetValue as *const () as usize),
            "ExitProcess" => Some(ExitProcess as *const () as usize),
            _ => None,
        }
    }
}

pub(crate) fn resolve(module: &str, name: &str) -> Option<usize> {
    // A vendor DLL exporting the same name is not kernel32.
    if !module.eq_ignore_ascii_case("kernel32.dll") {
        return None;
    }
    shims::address(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn module_and_symbol_identity_are_not_guessed() {
        assert!(resolve("KERNEL32.dll", "GetLastError").is_some());
        assert!(resolve("vendor.dll", "GetLastError").is_none());
        assert!(resolve("kernel32.dll", "getlasterror").is_none());
        assert!(resolve("kernel32.dll", "CreateThread").is_none());
        assert!(resolve("kernel32.dll", "GetProcAddress").is_none());
    }
}
