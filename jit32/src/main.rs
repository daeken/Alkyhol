use alkyhol_jit32::{Runner, Stop};
use std::{fs::File, os::fd::FromRawFd};
fn execute() -> Result<u32, String> {
    let mut args = std::env::args().skip(1);
    let file = args.next().ok_or("usage: alkyhol-jit32 PE [--rebase]")?;
    let base = match args.next().as_deref() {
        None => None,
        Some("--rebase") => Some(0x1000_0000),
        _ => return Err("unknown option".into()),
    };
    if args.next().is_some() {
        return Err("extra argument".into());
    }
    let bytes = std::fs::read(file).map_err(|e| e.to_string())?;
    let fd = unsafe { libc::dup(libc::STDOUT_FILENO) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let stdout = unsafe { File::from_raw_fd(fd) };
    let mut runner = Runner::load(&bytes, base, stdout)?;
    let stop = runner.run(10_000_000);
    eprintln!(
        "JIT32 ARM64: instructions={} services={} blocks={} stop={stop:?}",
        runner.instructions,
        runner.service_calls,
        runner.compiled_blocks()
    );
    match stop {
        Stop::Returned(status) => Ok(status),
        _ => Err(format!("guest stopped: {stop:?}")),
    }
}
fn main() {
    match execute() {
        Ok(code) => std::process::exit(code.min(255) as i32),
        Err(e) => {
            eprintln!("JIT32 error: {e}");
            std::process::exit(70);
        }
    }
}
