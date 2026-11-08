//! Trusted unchanged owned PE32 under the explicit bounded exception profile.
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = (|| -> alkyhol_pe::Result<u32> {
        if args.is_empty() || args.len() > 2 || (args.len() == 2 && args[1] != "--rebase") {
            return Err("expected PE32 fixture and optional --rebase".into());
        }
        let p = alkyhol_pe::ProcessContext::install(&["seh32".into()], &[])?;
        let b = std::fs::read(&args[0]).map_err(|e| e.to_string())?;
        let i = alkyhol_pe::ExceptionImage32::load(&b, args.len() == 2, &p)?;
        if args.len() == 2 && !i.relocated() {
            return Err("forced SEH image did not relocate".into());
        }
        let status = unsafe { i.enter() }?;
        eprintln!(
            "PE32 bounded UD2 SEH returned; restored host signals, alternate stack and selectors; forced_rebase={}",
            args.len() == 2
        );
        Ok(status)
    })();
    match result {
        Ok(n) if n <= 255 => std::process::exit(n as i32),
        Ok(n) => {
            eprintln!("unsupported PE32 result {n}");
            std::process::exit(78);
        }
        Err(e) => {
            eprintln!("Alkyhol PE32 unsupported: {e}");
            std::process::exit(78);
        }
    }
}
