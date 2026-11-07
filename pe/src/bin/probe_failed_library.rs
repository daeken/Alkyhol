//! Actual failed-init DLL observer. Guest trace goes to stdout; result to stderr.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    use alkyhol_pe::{LoadedLibrary, ProcessContext};
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("expected DLL path")?;
    let rebase = match args.next().as_deref() {
        None => false,
        Some(s) if s == "--rebase" => true,
        _ => return Err("invalid failed-library option".into()),
    };
    if args.next().is_some() {
        return Err("extra arguments".into());
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let _context = ProcessContext::install(&["failed-library-probe".into()], &[])?;
    // SAFETY: the Python driver verifies this unchanged owned source/binary
    // before execution. Its only imports are the three declared kernel32 APIs.
    let error = match unsafe { LoadedLibrary::load(&bytes, rebase) } {
        Ok(_) => return Err("failed-attach DLL unexpectedly loaded".into()),
        Err(e) => e,
    };
    if !error.contains("PROCESS_ATTACH returned FALSE") {
        return Err(error);
    }
    eprintln!(
        "{{\"loaded\":false,\"attach_rejected\":true,\"host_last_error\":{}}}",
        alkyhol::ffi::GetLastError()
    );
    Ok(())
}
#[cfg(not(all(target_os = "macos", target_arch = "x86_64")))]
fn main() -> Result<(), String> {
    Err("failed DLL runtime probe requires x86_64 macOS".into())
}
