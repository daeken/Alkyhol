use alkyhol_jit32::{ModuleInput, Process, Runner, Stop};
use std::{fs::File, os::fd::FromRawFd, path::Path};
fn execute() -> Result<u32, String> {
    let mut args = std::env::args().skip(1);
    let file = args.next().ok_or(
        "usage: alkyhol-jit32 PE [--rebase] [--process [--dll NAME=PATH] [--startup-only]]",
    )?;
    let mut trace_state = false;
    let mut legacy_nt62 = false;
    let mut rebase = false;
    let mut process = false;
    let mut startup_only = false;
    let mut dlls = vec![];
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rebase" => rebase = true,
            "--state" => trace_state = true,
            "--legacy-nt62" => {
                legacy_nt62 = true;
                process = true;
            }
            "--process" => process = true,
            "--startup-only" => {
                startup_only = true;
                process = true;
            }
            "--dll" => {
                let value = args.next().ok_or("--dll requires NAME=PATH")?;
                let (name, path) = value.split_once('=').ok_or("--dll requires NAME=PATH")?;
                dlls.push((name.to_string(), path.to_string()));
                process = true;
            }
            _ => return Err(format!("unknown option: {arg}")),
        }
    }
    let bytes = std::fs::read(&file).map_err(|e| e.to_string())?;
    let fd = unsafe { libc::dup(libc::STDOUT_FILENO) };
    if fd < 0 {
        return Err(std::io::Error::last_os_error().to_string());
    }
    let stdout = unsafe { File::from_raw_fd(fd) };
    let (stop, instructions, services, blocks) = if process {
        let mut owned = vec![(
            Path::new(&file)
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or("EXE requires Unicode basename")?
                .to_string(),
            bytes,
        )];
        for (name, path) in dlls {
            owned.push((
                name,
                std::fs::read(&path).map_err(|e| format!("{path}: {e}"))?,
            ));
        }
        let inputs = owned
            .iter()
            .map(|(name, bytes)| ModuleInput { name, bytes })
            .collect::<Vec<_>>();
        let mut process = Process::load(&inputs, rebase, stdout)?;
        if legacy_nt62 {
            process.use_legacy_nt62_persona()?;
        }
        let startup = process.run_startup(10_000_000);
        eprintln!(
            "JIT32 startup: phase={:?} modules={} stop={startup:?}",
            process.phase(),
            process.modules().len()
        );
        let stop = if startup == Stop::Returned(0) && !startup_only {
            process.run_application(10_000_000)
        } else {
            startup
        };
        let machine = process.machine();
        if trace_state {
            let registers = machine.registers();
            let return_pc = machine.memory().word(registers[4] as u64).ok();
            eprintln!(
                "JIT32 state: pc={:#010x} registers={registers:08x?} flags={:#010x} return={return_pc:08x?}",
                machine.pc(),
                machine.flags()
            );
            if let Some(pc) = return_pc
                && let Some(module) = process
                    .modules()
                    .iter()
                    .find(|m| pc >= m.base && (pc as u64) < (m.base as u64 + m.size as u64))
            {
                eprintln!("JIT32 caller: {}+{:#x}", module.name, pc - module.base);
            }
        }
        (
            stop,
            machine.instructions,
            machine.service_calls,
            machine.compiled_blocks(),
        )
    } else {
        let mut runner = Runner::load(
            &bytes,
            if rebase { Some(0x1000_0000) } else { None },
            stdout,
        )?;
        let stop = runner.run(10_000_000);
        (
            stop,
            runner.instructions,
            runner.service_calls,
            runner.compiled_blocks(),
        )
    };
    eprintln!(
        "JIT32 ARM64: instructions={instructions} services={services} blocks={blocks} stop={stop:?}"
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
