//! Unchanged guest EXE uses real module/export lookup against a frozen DLL.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    use alkyhol_pe::{LinkedImage, LoadedLibrary, ModuleContext, ProcessContext};
    let mut args = std::env::args_os().skip(1);
    let directory = std::path::PathBuf::from(args.next().ok_or("expected fixture directory")?);
    let rebase = match args.next().as_deref() {
        None => false,
        Some(s) if s == "--rebase" => true,
        _ => return Err("invalid module lookup probe option".into()),
    };
    if args.next().is_some() {
        return Err("extra arguments".into());
    }
    let exe = std::fs::read(directory.join("dynamic-lookup.exe")).map_err(|e| e.to_string())?;
    let dll = std::fs::read(directory.join("alkyhol_exports.dll")).map_err(|e| e.to_string())?;
    let _process = ProcessContext::install(&["dynamic-lookup.exe".into()], &[])?;
    // SAFETY: driver verifies the owned EXE/frozen DLL. Their instructions obey
    // the bounded runtime contract; this DLL's DllMain performs only data writes.
    let library = unsafe { LoadedLibrary::load(&dll, rebase) }?;
    let modules = ModuleContext::install(&[("alkyhol_exports.dll", &library)])?;
    let image = LinkedImage::load(&exe, rebase, &[("alkyhol_exports.dll", &library)])?;
    let relocated = [image.image().relocated(), library.relocated()];
    let status = unsafe { image.image().enter() };
    drop(image);
    drop(modules);
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
    Err("guest module lookup requires x86_64 macOS".into())
}
