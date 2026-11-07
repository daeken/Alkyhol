//! Actual owned mixed named/ordinal imports and forwarders, not a metadata-only probe.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    use alkyhol_pe::{Image, ImportSymbol, LinkedLibrary, LoadedLibrary};
    let mut args = std::env::args_os().skip(1);
    let directory = std::path::PathBuf::from(args.next().ok_or("expected fixture directory")?);
    let rebase = match args.next().as_deref() {
        None => false,
        Some(s) if s == "--rebase" => true,
        _ => return Err("invalid ordinal probe option".into()),
    };
    if args.next().is_some() {
        return Err("extra arguments".into());
    }
    let leaf_bytes =
        std::fs::read(directory.join("alkyhol_exports.dll")).map_err(|e| e.to_string())?;
    let bytes = std::fs::read(directory.join("alkyhol_ordinal.dll")).map_err(|e| e.to_string())?;
    let imports = Image::parse_dll(&bytes)?.imports()?;
    if imports.iter().map(|i| &i.symbol).collect::<Vec<_>>()
        != [
            &ImportSymbol::Ordinal(21),
            &ImportSymbol::Name("Direct".into()),
            &ImportSymbol::Ordinal(23),
        ]
        || imports.iter().any(|i| i.module != "alkyhol_exports.dll")
    {
        return Err("actual mixed import thunks differ".into());
    }
    // SAFETY: Python verifies this unchanged owned source/binary fixture. The
    // original leaf is attached first and borrowed through consumer teardown.
    let leaf = unsafe { LoadedLibrary::load(&leaf_bytes, rebase) }?;
    let consumer =
        unsafe { LinkedLibrary::load(&bytes, rebase, &[("alkyhol_exports.dll", &leaf)]) }?;
    let query = |name| {
        consumer
            .library()
            .get_name(name)
            .ok_or("missing ordinal probe export")
    };
    let through = query("Through")?;
    let data = query("ForwardData")?;
    let direct = query("ForwardDirect")?;
    let same_code = Some(through) == leaf.get_ordinal(23);
    let same_data = Some(data) == leaf.get_ordinal(21);
    let same_direct = Some(direct) == leaf.get_ordinal(17);
    let (evaluation, initial, through_call, forward_data) = unsafe {
        let evaluate: unsafe extern "win64" fn(u32, u32) -> u32 =
            std::mem::transmute(query("Evaluate")?);
        let through: unsafe extern "win64" fn(u32) -> u32 = std::mem::transmute(through);
        (
            evaluate(7, 11),
            query("Initial")?.cast::<u32>().read_unaligned(),
            through(9),
            data.cast::<u32>().read_unaligned(),
        )
    };
    let flags = [leaf.relocated(), consumer.library().relocated()];
    drop(consumer);
    drop(leaf);
    println!(
        concat!(
            "{{\"common\":{{\"evaluation\":{},\"initial\":{},\"through_call\":{},\"forward_data\":{},",
            "\"through_matches_ordinal\":{},\"data_matches_ordinal\":{},\"direct_matches_ordinal\":{}}},",
            "\"relocated\":[{},{}]}}"
        ),
        evaluation,
        initial,
        through_call,
        forward_data,
        same_code,
        same_data,
        same_direct,
        flags[0],
        flags[1]
    );
    Ok(())
}
#[cfg(not(all(target_os = "macos", target_arch = "x86_64")))]
fn main() -> Result<(), String> {
    Err("ordinal DLL execution requires x86_64 macOS".into())
}
