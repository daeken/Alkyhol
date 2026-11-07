//! Trusted owned DLL observer. Guest lifecycle trace is stdout; status is stderr.
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    use alkyhol_pe::{ProcessContext, RegisteredLibrary};
    let mut args = std::env::args_os().skip(1);
    let path = args.next().ok_or("expected DLL path")?;
    let (mut rebase, mut reject) = (false, false);
    for arg in args {
        if arg == "--rebase" && !rebase {
            rebase = true;
        } else if arg == "--expect-reject" && !reject {
            reject = true;
        } else {
            return Err("invalid registered-library probe option".into());
        }
    }
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let process = ProcessContext::install(&["registered-dll-probe".into()], &[])?;
    // SAFETY: Python checks the owned DLL/source identities and snapshots bytes.
    match unsafe { RegisteredLibrary::load(&bytes, rebase, &process) } {
        Ok(dll) => {
            if reject {
                return Err("rejected DLL unexpectedly attached".into());
            }
            if rebase && !dll.library().relocated() {
                return Err("DLL did not rebase".into());
            }
            let named = dll.library().get_name("Probe").ok_or("missing Probe")?;
            let alias = dll.library().get_ordinal(17) == Some(named);
            let probe: unsafe extern "win64" fn() -> u32 = unsafe { std::mem::transmute(named) };
            let result = unsafe { probe() };
            drop(dll);
            eprintln!("{{\"loaded\":true,\"probe\":{result},\"alias\":{alias}}}");
        }
        Err(error) => {
            if !reject || error != "DLL PROCESS_ATTACH returned FALSE" {
                return Err(error);
            }
            eprintln!("{{\"loaded\":false,\"probe\":null,\"alias\":null}}");
        }
    }
    Ok(())
}
#[cfg(not(all(target_os = "macos", target_arch = "x86_64")))]
fn main() -> Result<(), String> {
    Err("registered DLL observer requires x86_64 macOS".into())
}
