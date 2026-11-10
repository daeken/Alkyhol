//! These shims use Win64 on their guest side and Alkyhol's native ABI internally.
//! Pointer arguments retain the core ffi contract: guest code must provide valid
//! mapped host memory. Guest callbacks/COM are not passed to native ABI functions.
#![allow(non_snake_case)]
use alkyhol::ffi;
use std::ffi::c_void;

// Kernel handles have only 32 significant bits across Win32/Win64. In
// particular, upstream u-config declares CreateFileW's template as int32, so the
// unused upper stack-slot bytes are not initialized by its released executable.
pub(crate) fn kernel_handle(h: usize) -> usize {
    h as u32 as i32 as isize as usize
}
macro_rules! argument {
    (h, $value:expr) => {
        kernel_handle($value)
    };
    ($other:ident, $value:expr) => {
        $value
    };
}
macro_rules! bridge {
    ($name:ident($($arg:ident: $ty:ty),* $(,)?) -> $ret:ty) => {
        #[alkyhol_host_abi::host_abi]
    unsafe extern "win64" fn $name($($arg: $ty),*) -> $ret {
            // SAFETY: same pointee/width contract; compiler performs the ABI bridge.
            unsafe { ffi::$name($(argument!($arg, $arg)),*) }
        }
    };
}
// Calling safe core exports inside the macro's unsafe block is intentional.
#[allow(unused_unsafe)]
mod shims {
    use super::*;
    bridge!(GetLastError() -> u32);
    bridge!(SetLastError(error: u32) -> ());
    // Windows -1/-2 are current-process/current-thread pseudo-handles. Closing
    // them has no effect; do not send those identities to the owned-handle table.
    #[alkyhol_host_abi::host_abi]
    unsafe extern "win64" fn CloseHandle(h: usize) -> i32 {
        let h = kernel_handle(h);
        if h == usize::MAX || h == usize::MAX - 1 {
            1
        } else {
            crate::process::close_stream(h).unwrap_or_else(|| ffi::CloseHandle(h))
        }
    }
    #[alkyhol_host_abi::host_abi]
    unsafe extern "win64" fn CreateFileW(
        name: *const u16,
        access: u32,
        share: u32,
        security: *const c_void,
        disposition: u32,
        flags: u32,
        template: usize,
    ) -> usize {
        unsafe {
            crate::process::create_file(name, access, share, security, disposition, flags, template)
        }
    }
    #[alkyhol_host_abi::host_abi]
    unsafe extern "win64" fn ReadFile(
        h: usize,
        buffer: *mut c_void,
        count: u32,
        transferred: *mut u32,
        overlapped: *mut c_void,
    ) -> i32 {
        let h = kernel_handle(h);
        unsafe { crate::process::stream_io(h, buffer, count, transferred, overlapped, false) }
            .unwrap_or_else(|| unsafe { ffi::ReadFile(h, buffer, count, transferred, overlapped) })
    }
    #[alkyhol_host_abi::host_abi]
    unsafe extern "win64" fn WriteFile(
        h: usize,
        buffer: *const c_void,
        count: u32,
        transferred: *mut u32,
        overlapped: *mut c_void,
    ) -> i32 {
        let h = kernel_handle(h);
        unsafe {
            crate::process::stream_io(h, buffer.cast_mut(), count, transferred, overlapped, true)
        }
        .unwrap_or_else(|| unsafe { ffi::WriteFile(h, buffer, count, transferred, overlapped) })
    }
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
    #[alkyhol_host_abi::host_abi]
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
    if module.eq_ignore_ascii_case("kernel32.dll") {
        let address=shims::address(name)
            .or_else(|| crate::process::address(name))
            .or_else(|| crate::unicode::address(name));
        #[cfg(target_arch="aarch64")]
        return address;
        #[cfg(target_arch="x86_64")]
        return address
            .or_else(|| crate::function_tables::address(name))
            .or_else(|| crate::virtual_unwind::address(name))
            .or_else(|| crate::module_context::address(name));
    }
    if module.eq_ignore_ascii_case("shell32.dll") {
        return crate::process::shell_address(name);
    }
    #[cfg(target_arch="x86_64")]
    if module.eq_ignore_ascii_case("d3dcompiler_47.dll") {
        return crate::binary_com::address(name);
    }
    #[cfg(target_arch="x86_64")]
    if module.eq_ignore_ascii_case("d3d12.dll") {
        return crate::binary_com::d3d12_address(name);
    }
    None
}
pub(crate) fn is_builtin(module: &str) -> bool {
    [
        "kernel32.dll",
        "shell32.dll",
        "d3dcompiler_47.dll",
        "d3d12.dll",
    ]
    .iter()
    .any(|name| module.eq_ignore_ascii_case(name))
}

#[cfg(all(test,target_arch="x86_64"))]
mod tests {
    use super::*;
    #[test]
    fn kernel_handle_upper_bits_are_not_pointer_bits() {
        assert_eq!(kernel_handle(0x1234567800000000), 0);
        assert_eq!(kernel_handle(0x1234567800000042), 0x42);
        assert_eq!(kernel_handle(0x12345678ffffffff), usize::MAX);
        assert_eq!(kernel_handle(0x12345678fffffffe), usize::MAX - 1);
    }

    #[test]
    fn pseudo_close_has_no_effect_and_null_still_fails() {
        let close: unsafe extern "win64" fn(usize) -> i32 =
            unsafe { std::mem::transmute(resolve("kernel32.dll", "CloseHandle").unwrap()) };
        for pseudo in [usize::MAX, usize::MAX - 1] {
            ffi::SetLastError(0x1234);
            assert_eq!(unsafe { close(pseudo) }, 1);
            assert_eq!(ffi::GetLastError(), 0x1234);
        }
        assert_eq!(unsafe { close(0) }, 0);
        assert_eq!(ffi::GetLastError(), 6);
    }

    #[test]
    fn module_and_symbol_identity_are_not_guessed() {
        assert!(resolve("KERNEL32.dll", "GetLastError").is_some());
        assert!(resolve("SHELL32.DLL", "CommandLineToArgvW").is_some());
        assert!(resolve("kernel32.dll", "CommandLineToArgvW").is_none());
        assert!(resolve("shell32.dll", "LocalFree").is_none());
        assert!(resolve("vendor.dll", "CommandLineToArgvW").is_none());
        assert!(resolve("vendor.dll", "GetLastError").is_none());
        assert!(resolve("kernel32.dll", "getlasterror").is_none());
        assert!(resolve("kernel32.dll", "CreateThread").is_none());
        assert!(resolve("kernel32.dll", "GetProcAddress").is_some());
        assert!(resolve("kernel32.dll", "GetModuleHandleW").is_some());
        assert!(resolve("vendor.dll", "GetProcAddress").is_none());
        assert!(resolve("kernel32.dll", "GetModuleHandleA").is_none());
        assert!(resolve("kernel32.dll", "LoadLibraryW").is_none());
        assert!(resolve("kernel32.dll", "FreeLibrary").is_none());
        for name in [
            "RtlAddFunctionTable",
            "RtlDeleteFunctionTable",
            "RtlLookupFunctionEntry",
        ] {
            assert!(resolve("kernel32.dll", name).is_some());
            assert!(resolve("vendor.dll", name).is_none());
        }
        assert!(resolve("kernel32.dll", "RtlVirtualUnwind").is_some());
        assert!(resolve("vendor.dll", "RtlVirtualUnwind").is_none());
        assert!(resolve("kernel32.dll", "RtlUnwindEx").is_none());
        assert!(resolve("kernel32.dll", "RtlInstallFunctionTableCallback").is_none());
        assert!(resolve("D3DCOMPILER_47.dll", "D3DCreateBlob").is_some());
        assert!(resolve("D3D12.DLL", "D3D12SerializeRootSignature").is_some());
        assert!(resolve("d3d12.dll", "D3D12SerializeVersionedRootSignature").is_some());
        for (module, name) in [
            ("vendor.dll", "D3DCreateBlob"),
            ("kernel32.dll", "D3DCreateBlob"),
            ("d3dcompiler_47.dll", "d3dcreateblob"),
            ("d3dcompiler_47.dll", "D3DCompile"),
            ("d3dcompiler_43.dll", "D3DCreateBlob"),
            ("d3dcompiler_47.dll", "D3D12SerializeRootSignature"),
            ("vendor.dll", "D3D12SerializeRootSignature"),
            ("d3d12.dll", "d3d12serializerootsignature"),
            ("d3d12.dll", "D3D12CreateDevice"),
        ] {
            assert!(resolve(module, name).is_none());
        }
    }
}
