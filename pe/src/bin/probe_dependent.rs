//! Execute an owned PE64 consumer's real IAT calls and cross-DLL forwarders.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    use alkyhol_pe::{LinkedLibrary, LoadedLibrary, ProcessContext};
    let mut args = std::env::args_os().skip(1);
    let directory = std::path::PathBuf::from(args.next().ok_or("expected fixture directory")?);
    let rebase = match args.next().as_deref() {
        None => false,
        Some(s) if s == "--rebase" => true,
        _ => return Err("invalid dependent probe option".into()),
    };
    if args.next().is_some() {
        return Err("extra arguments".into());
    }
    let dependency_bytes =
        std::fs::read(directory.join("alkyhol_exports.dll")).map_err(|e| e.to_string())?;
    let consumer_bytes =
        std::fs::read(directory.join("alkyhol_dependent.dll")).map_err(|e| e.to_string())?;
    let _context = ProcessContext::install(&["dependent-probe".into()], &[])?;
    // SAFETY: Python verifies this owned fixture's DLL/source identities. Both
    // DLLs obey the bounded trusted-code contract. Context outlives both owners.
    let dependency = unsafe { LoadedLibrary::load(&dependency_bytes, rebase) }?;
    let consumer = unsafe {
        LinkedLibrary::load(
            &consumer_bytes,
            rebase,
            &[("ALKYHOL_EXPORTS.dll", &dependency)],
        )
    }?;
    let query = |name: &str| {
        consumer
            .library()
            .get_name(name)
            .ok_or_else(|| format!("missing {name}"))
    };
    let evaluate = query("Evaluate")?;
    let through = query("Through")?;
    let error = query("Error")?;
    let read_error = query("ReadError")?;
    let through_matches = Some(through) == dependency.get_name("Direct");
    let error_matches = Some(error) == dependency.get_name("Forwarded");
    let (evaluation, through_call, read_error, double_forward_error) = unsafe {
        let evaluate: unsafe extern "win64" fn(u32, u32) -> u32 = std::mem::transmute(evaluate);
        let through: unsafe extern "win64" fn(u32, u32) -> u32 = std::mem::transmute(through);
        let read_error: unsafe extern "win64" fn() -> u32 = std::mem::transmute(read_error);
        let error: unsafe extern "win64" fn() -> u32 = std::mem::transmute(error);
        let a = evaluate(7, 11);
        let b = through(7, 11);
        alkyhol::ffi::SetLastError(0x3456);
        let c = read_error();
        alkyhol::ffi::SetLastError(0x4567);
        (a, b, c, error())
    };
    let relocated = [dependency.relocated(), consumer.library().relocated()];
    drop(consumer); // Its DETACH calls the dependency before it can be dropped.
    drop(dependency);
    eprintln!(
        concat!(
            "{{\"calls\":{{\"evaluation\":{},\"through_call\":{},\"read_error\":{},",
            "\"double_forward_error\":{},\"through_matches_dependency\":{},",
            "\"error_matches_dependency\":{}}},\"relocated\":[{},{}]}}"
        ),
        evaluation,
        through_call,
        read_error,
        double_forward_error,
        through_matches,
        error_matches,
        relocated[0],
        relocated[1]
    );
    Ok(())
}
#[cfg(not(all(target_os = "macos", target_arch = "x86_64")))]
fn main() -> Result<(), String> {
    Err("dependent DLL probe requires x86_64 macOS".into())
}
