//! Actual trusted i386 DLL execution in a small-PAGEZERO x86-64 Mach-O host.
use alkyhol_pe::{CallingConvention32 as C, LoadedLibrary32, ProcessContext};
use std::io::Write;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = (|| -> alkyhol_pe::Result<()> {
        if args.is_empty()
            || args[1..]
                .iter()
                .any(|a| !["--rebase", "--mismatch"].contains(&a.as_str()))
        {
            return Err("expected fixture directory and optional --rebase/--mismatch".into());
        }
        let dir = std::path::Path::new(&args[0]);
        let process = ProcessContext::install(&["dll-observer32".into()], &[])?;
        let accepted = std::fs::read(dir.join("accepted.dll")).map_err(|e| e.to_string())?;
        let rejected = std::fs::read(dir.join("rejected.dll")).map_err(|e| e.to_string())?;
        let rebase = args.iter().any(|a| a == "--rebase");
        let library = unsafe { LoadedLibrary32::load(&accepted, rebase, &process)? };
        if rebase && !library.relocated() {
            return Err("DLL did not actually relocate".into());
        }
        if args.iter().any(|a| a == "--mismatch") {
            // Deliberately misdescribe a real cdecl export. The CPU return ESP
            // must expose the error; successful host-stack recovery cannot hide it.
            unsafe {
                library.call_name("RunCdecl", &[0xabcdef01, 0x12345678, 19], C::Stdcall)?;
            }
            return Err("mismatched convention was accepted".into());
        }
        let a = unsafe {
            library.call_name(
                "RunStdcall",
                &[0xfffffff0, 7, 0x1234, 11, 0x778899aa],
                C::Stdcall,
            )?
        };
        let b = unsafe { library.call_name("RunCdecl", &[0xabcdef01, 0x12345678, 19], C::Cdecl)? };
        let d = unsafe {
            library.call_name(
                "RunStdcall",
                &[31, 17, 3, 0xfffffff0, 0x11223344],
                C::Stdcall,
            )?
        };
        if unsafe { library.call_name("Data", &[], C::Cdecl) }.is_ok()
            || unsafe { library.call_name("Missing", &[], C::Cdecl) }.is_ok()
            || unsafe { library.call_name("RunCdecl", &[0; 17], C::Cdecl) }.is_ok()
        {
            return Err("unsupported export/arity was accepted".into());
        }
        drop(library); // Real guest PROCESS_DETACH, before mappings/process drop.
        match unsafe { LoadedLibrary32::load(&rejected, rebase, &process) } {
            Err(e) if e == "PE32 DLL PROCESS_ATTACH returned FALSE" => (),
            Err(e) => return Err(e),
            Ok(_) => return Err("rejected DLL escaped initialization".into()),
        }
        let words = [0x44323349u32, 4, a, b, d, 1, 1, 0xa1b2c3d4];
        for word in words {
            std::io::stdout()
                .write_all(&word.to_le_bytes())
                .map_err(|e| e.to_string())?;
        }
        eprintln!(
            "PE32 DLL calls restored selectors and cleared FS leases; forced_rebase={rebase}"
        );
        Ok(())
    })();
    if let Err(e) = result {
        eprintln!("Alkyhol PE32 unsupported: {e}");
        std::process::exit(78);
    }
}
