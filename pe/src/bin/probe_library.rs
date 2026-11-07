//! Owned frozen-fixture runtime observer, distinct from metadata-only inspection.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    use alkyhol_pe::LoadedLibrary;
    use std::ffi::c_void;
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("usage: probe_library DLL [--rebase]")?;
    let option = args.next();
    let rebase = match option.as_deref() {
        None => false,
        Some(s) if s == "--rebase" => true,
        _ => return Err("invalid library probe option".into()),
    };
    if args.next().is_some() {
        return Err("extra library probe arguments".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    // SAFETY: the caller checks the unchanged owned DLL/source identity against
    // the frozen Windows fixture before invoking this dedicated observer.
    let library = unsafe { LoadedLibrary::load(&bytes, rebase) }?;
    if library.base_address() % 65536 != 0 {
        return Err("DLL base is not 64K aligned".into());
    }
    if rebase && !library.relocated() {
        return Err("DLL did not rebase".into());
    }
    let query = |name: &str, ordinal: u32| -> Result<*mut c_void, String> {
        let address = library.get_ordinal(ordinal).ok_or("missing ordinal")?;
        if !name.is_empty() && library.get_name(name) != Some(address) {
            return Err("named/ordinal DLL lookup differs".into());
        }
        Ok(address)
    };
    let direct = query("Direct", 17)?;
    let alias = query("Alias", 18)?;
    let forwarded = query("Forwarded", 20)?;
    let data = query("Data", 21)?;
    let bss = query("Bss", 22)?;
    let ordinal = query("", 23)?;
    if direct != alias
        || library.get_name("missing_export").is_some()
        || library.get_ordinal(19).is_some()
    {
        return Err("alias/missing DLL lookup differs".into());
    }
    let (initialized, bss_attached, direct_result, ordinal_result, forwarded_error) = unsafe {
        let direct: unsafe extern "win64" fn(u32, u32) -> u32 = std::mem::transmute(direct);
        let ordinal: unsafe extern "win64" fn(u32) -> u32 = std::mem::transmute(ordinal);
        let forwarded: unsafe extern "win64" fn() -> u32 = std::mem::transmute(forwarded);
        let data = data.cast::<u32>().read_unaligned();
        let bss = bss.cast::<u32>().read_unaligned();
        let a = direct(7, 11);
        let b = ordinal(9);
        alkyhol::ffi::SetLastError(0x1234);
        let error = forwarded();
        (data, bss, a, b, error)
    };
    if forwarded_error != 0x1234 {
        return Err("forwarded Win64 API did not execute correctly".into());
    }
    let rva = |address: *mut c_void| address as usize - library.base_address();
    let entries = format!(
        concat!(
            "{{\"ordinal\":17,\"names\":[\"Direct\"],\"target\":{{\"rva\":{}}}}},",
            "{{\"ordinal\":18,\"names\":[\"Alias\"],\"target\":{{\"rva\":{}}}}},",
            "{{\"ordinal\":20,\"names\":[\"Forwarded\"],\"target\":{{\"forwarder\":\"KERNEL32.GetLastError\"}}}},",
            "{{\"ordinal\":21,\"names\":[\"Data\"],\"target\":{{\"rva\":{}}}}},",
            "{{\"ordinal\":22,\"names\":[\"Bss\"],\"target\":{{\"rva\":{}}}}},",
            "{{\"ordinal\":23,\"names\":[],\"target\":{{\"rva\":{}}}}}"
        ),
        rva(direct),
        rva(alias),
        rva(data),
        rva(bss),
        rva(ordinal)
    );
    let relocated = library.relocated();
    drop(library); // Actual DLL_PROCESS_DETACH runs before unmapping.
    println!(
        concat!(
            "{{\"common\":{{\"dll_name\":\"alkyhol_exports.dll\",\"ordinal_base\":17,\"entries\":[{}]}},",
            "\"execution\":{{\"initialized_data\":{},\"bss_after_attach\":{},\"direct_call\":{},\"ordinal_call\":{},\"hole19\":\"NULL\"}},",
            "\"relocated\":{},\"forwarded_last_error\":{}}}"
        ),
        entries,
        initialized,
        bss_attached,
        direct_result,
        ordinal_result,
        relocated,
        forwarded_error
    );
    Ok(())
}
#[cfg(not(all(target_os = "macos", target_arch = "x86_64")))]
fn main() -> Result<(), String> {
    Err("DLL runtime probe requires x86_64 macOS".into())
}
