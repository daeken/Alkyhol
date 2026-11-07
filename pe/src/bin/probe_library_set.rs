//! Shared named/ordinal consumers loaded by one closed DLL bundle owner.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    use alkyhol_pe::{LibrarySet, LoadedLibrary, ProcessContext};
    use std::ffi::c_void;
    let mut args = std::env::args_os().skip(1);
    let directory = std::path::PathBuf::from(args.next().ok_or("expected fixture directory")?);
    let rebase = match args.next().as_deref() {
        None => false,
        Some(s) if s == "--rebase" => true,
        _ => return Err("invalid bundle probe option".into()),
    };
    if args.next().is_some() {
        return Err("extra arguments".into());
    }
    let names = [
        "alkyhol_dependent.dll",
        "alkyhol_ordinal.dll",
        "alkyhol_exports.dll",
    ];
    let bytes = names
        .iter()
        .map(|n| std::fs::read(directory.join(n)).map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    let inputs: Vec<_> = names
        .iter()
        .zip(&bytes)
        .map(|(n, b)| (*n, b.as_slice()))
        .collect();
    let _context = ProcessContext::install(&["library-set-probe".into()], &[])?;
    // SAFETY: the driver verifies all three unchanged frozen DLLs/sources. The
    // complete bundle is checked before initialization; context outlives it.
    let set = unsafe { LibrarySet::load(&inputs, rebase) }?;
    let named = set.get(names[0]).ok_or("missing named consumer")?;
    let ordinal = set.get(names[1]).ok_or("missing ordinal consumer")?;
    let leaf = set
        .get("ALKYHOL_EXPORTS.DLL")
        .ok_or("missing shared leaf")?;
    fn query(library: &LoadedLibrary, name: &str) -> Result<*mut c_void, String> {
        library
            .get_name(name)
            .ok_or_else(|| format!("missing {name}"))
    }
    let (
        evaluation,
        through_call,
        read_error,
        double_error,
        ord_evaluation,
        initial,
        ord_call,
        data,
    ) = unsafe {
        let evaluate: unsafe extern "win64" fn(u32, u32) -> u32 =
            std::mem::transmute(query(named, "Evaluate")?);
        let through: unsafe extern "win64" fn(u32, u32) -> u32 =
            std::mem::transmute(query(named, "Through")?);
        let read: unsafe extern "win64" fn() -> u32 =
            std::mem::transmute(query(named, "ReadError")?);
        let error: unsafe extern "win64" fn() -> u32 = std::mem::transmute(query(named, "Error")?);
        let ord_evaluate: unsafe extern "win64" fn(u32, u32) -> u32 =
            std::mem::transmute(query(ordinal, "Evaluate")?);
        let ord_through: unsafe extern "win64" fn(u32) -> u32 =
            std::mem::transmute(query(ordinal, "Through")?);
        let a = evaluate(7, 11);
        let b = through(7, 11);
        alkyhol::ffi::SetLastError(0x3456);
        let c = read();
        alkyhol::ffi::SetLastError(0x4567);
        (
            a,
            b,
            c,
            error(),
            ord_evaluate(7, 11),
            query(ordinal, "Initial")?.cast::<u32>().read_unaligned(),
            ord_through(9),
            query(ordinal, "ForwardData")?
                .cast::<u32>()
                .read_unaligned(),
        )
    };
    let named_code = Some(query(named, "Through")?) == leaf.get_name("Direct");
    let named_error = Some(query(named, "Error")?) == leaf.get_name("Forwarded");
    let ord_code = Some(query(ordinal, "Through")?) == leaf.get_ordinal(23);
    let ord_data = Some(query(ordinal, "ForwardData")?) == leaf.get_ordinal(21);
    let ord_direct = Some(query(ordinal, "ForwardDirect")?) == leaf.get_ordinal(17);
    let shared = query(named, "Through")? == query(ordinal, "ForwardDirect")?;
    let flags = [named.relocated(), ordinal.relocated(), leaf.relocated()];
    drop(set); // Consumer DETACH still calls shared leaf before leaf is unmapped.
    eprintln!(
        concat!(
            "{{\"common\":{{\"dependent\":{{\"evaluation\":{},\"through_call\":{},\"read_error\":{},",
            "\"double_forward_error\":{},\"through_matches_dependency\":{},\"error_matches_dependency\":{}}},",
            "\"ordinal\":{{\"evaluation\":{},\"initial\":{},\"through_call\":{},\"forward_data\":{},",
            "\"through_matches_ordinal\":{},\"data_matches_ordinal\":{},\"direct_matches_ordinal\":{}}},",
            "\"shared_leaf_targets\":{}}},\"relocated\":[{},{},{}]}}"
        ),
        evaluation,
        through_call,
        read_error,
        double_error,
        named_code,
        named_error,
        ord_evaluation,
        initial,
        ord_call,
        data,
        ord_code,
        ord_data,
        ord_direct,
        shared,
        flags[0],
        flags[1],
        flags[2]
    );
    Ok(())
}
#[cfg(not(all(target_os = "macos", target_arch = "x86_64")))]
fn main() -> Result<(), String> {
    Err("closed DLL bundle requires x86_64 macOS".into())
}
