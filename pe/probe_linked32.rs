//! Real PE32 import/dependency lifetime and shared-thread execution observer.
use alkyhol_pe::{CallingConvention32 as C, LoadedLibrary32, ProcessContext};
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = (|| -> alkyhol_pe::Result<()> {
        if args.is_empty() || args.len() > 2 || (args.len() == 2 && args[1] != "--rebase") {
            return Err("expected fixture directory and optional --rebase".into());
        }
        let dir = std::path::Path::new(&args[0]);
        let process = ProcessContext::install(&["linked-observer32".into()], &[])?;
        let a = std::fs::read(dir.join("accepted.dll")).map_err(|e| e.to_string())?;
        let b = std::fs::read(dir.join("consumer.dll")).map_err(|e| e.to_string())?;
        let rebase = args.len() == 2;
        let leaf = unsafe { LoadedLibrary32::load(&a, rebase, &process)? };
        match unsafe { LoadedLibrary32::load(&b, rebase, &process) } {
            Err(error) if error.starts_with("unsupported Win32 import accepted.dll!") => (),
            Err(error) => return Err(error),
            Ok(_) => return Err("missing explicit dependency was accepted".into()),
        }
        match unsafe {
            LoadedLibrary32::load_linked(
                &b,
                rebase,
                &process,
                &[("accepted.dll", &leaf), ("ACCEPTED.DLL", &leaf)],
            )
        } {
            Err(error) if error == "duplicate case-insensitive PE32 DLL dependency" => (),
            Err(error) => return Err(error),
            Ok(_) => return Err("duplicate dependency was accepted".into()),
        }
        let consumer = unsafe {
            LoadedLibrary32::load_linked(&b, rebase, &process, &[("AcCePtEd.DlL", &leaf)])?
        };
        if rebase && (!leaf.relocated() || !consumer.relocated()) {
            return Err("both DLLs must relocate".into());
        }
        let result = unsafe { consumer.call_name("Run", &[0xabcdef01, 0x12345678, 19], C::Cdecl)? };
        if result != (0xb9f9b98cu32 ^ 0xeeddcca5 ^ 0x2468ace0) {
            return Err("cross-DLL integer result differs".into());
        }
        drop(consumer); // Its detach calls into the still-live dependency.
        drop(leaf);
        eprintln!("PE32 linked DLLs restored shared-thread selectors; forced_rebase={rebase}");
        Ok(())
    })();
    if let Err(e) = result {
        eprintln!("Alkyhol PE32 unsupported: {e}");
        std::process::exit(78);
    }
}
