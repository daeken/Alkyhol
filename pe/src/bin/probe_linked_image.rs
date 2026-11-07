//! Enter an unchanged Windows EXE linked to an explicitly supplied frozen DLL.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    use alkyhol_pe::{LinkedImage, LoadedLibrary, ProcessContext};
    let mut args = std::env::args_os().skip(1);
    let directory = std::path::PathBuf::from(args.next().ok_or("expected fixture directory")?);
    let rebase = match args.next().as_deref() {
        None => false,
        Some(s) if s == "--rebase" => true,
        _ => return Err("invalid linked EXE probe option".into()),
    };
    if args.next().is_some() {
        return Err("extra arguments".into());
    }
    let exe = std::fs::read(directory.join("linked-image.exe")).map_err(|e| e.to_string())?;
    let dll = std::fs::read(directory.join("alkyhol_exports.dll")).map_err(|e| e.to_string())?;
    let _context = ProcessContext::install(&["linked-image.exe".into()], &[])?;
    // SAFETY: driver verifies the owned EXE and unchanged frozen DLL, whose code
    // obeys the bounded runtime contract. Context outlives both mappings.
    let library = unsafe { LoadedLibrary::load(&dll, rebase) }?;
    let image = LinkedImage::load(&exe, rebase, &[("ALKYHOL_EXPORTS.DLL", &library)])?;
    let relocated = [image.image().relocated(), library.relocated()];
    let status = unsafe { image.image().enter() };
    drop(image); // Dependency stays attached until the EXE has unmapped.
    drop(library);
    eprintln!(
        "{{\"entry_status\":{status},\"relocated\":[{},{}]}}",
        relocated[0], relocated[1]
    );
    if status != 0 {
        return Err(format!("unexpected guest entry return {status}"));
    }
    Ok(())
}
#[cfg(not(all(target_os = "macos", target_arch = "x86_64")))]
fn main() -> Result<(), String> {
    Err("linked Windows EXE requires x86_64 macOS".into())
}
