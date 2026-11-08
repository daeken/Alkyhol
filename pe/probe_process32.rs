//! Closed PE32 process with real static DLL initialization and termination.
use alkyhol_pe::{ProcessContext, ProcessImage32};
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = (|| -> alkyhol_pe::Result<u32> {
        if args.is_empty()
            || args.len() > 2
            || (args.len() == 2
                && ![
                    "--rebase",
                    "--missing",
                    "--duplicate",
                    "--unused",
                    "--symbol",
                    "--failed",
                ]
                .contains(&args[1].as_str()))
        {
            return Err("expected process fixture directory and optional --rebase".into());
        }
        let dir = std::path::Path::new(&args[0]);
        let read = |name: &str| std::fs::read(dir.join(name)).map_err(|e| e.to_string());
        let exe = read("program.exe")?;
        let leaf = read("accepted.dll")?;
        let consumer = read("consumer.dll")?;
        let terminate = read("terminate.dll")?;
        let process = ProcessContext::install(&["program.exe".into()], &[])?;
        let forced = args.len() == 2;
        let mode = args.get(1).map(String::as_str).unwrap_or("");
        let forced = forced && mode == "--rebase";
        // Deliberately supplied out of dependency order; the same leaf has both
        // a direct EXE import and an indirect consumer import, and initializes once.
        let failed = read("failed.dll")?;
        let mut dlls = vec![
            (
                "CoNsUmEr.DlL",
                if mode == "--symbol" {
                    leaf.as_slice()
                } else if mode == "--failed" {
                    failed.as_slice()
                } else {
                    consumer.as_slice()
                },
            ),
            ("terminate.dll", terminate.as_slice()),
            ("AcCePtEd.DlL", leaf.as_slice()),
        ];
        match mode {
            "--missing" => {
                dlls.remove(1);
            }
            "--duplicate" => dlls.push(("accepted.dll", &leaf)),
            "--unused" => dlls.push(("unused.dll", &leaf)),
            _ => (),
        }
        let mut image =
            unsafe { ProcessImage32::load(("program.exe", &exe), &dlls, forced, &process) }?;
        let relocated = image.relocations();
        if relocated.len() != 4 || (forced && relocated.iter().any(|v| !*v)) {
            return Err("missing process graph forced-relocation evidence".into());
        }
        let status = unsafe { image.enter() }?;
        if unsafe { image.enter() }.unwrap_err() != "PE32 process cannot be restarted" {
            return Err("process restart was accepted".into());
        }
        drop(image);
        eprintln!(
            "PE32 process statically attached once, terminated and restored host selectors; forced_rebase={forced}"
        );
        Ok(status)
    })();
    match result {
        Ok(status) if status <= 255 => std::process::exit(status as i32),
        Ok(_) => {
            eprintln!("Alkyhol PE32 unsupported: full DWORD exit status");
            std::process::exit(78)
        }
        Err(e) => {
            eprintln!("Alkyhol PE32 unsupported: {e}");
            std::process::exit(78)
        }
    }
}
