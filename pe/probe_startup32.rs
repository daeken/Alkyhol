//! Explicit diagnostic execution of a caller-supplied original PE32/DLL graph.
//! External-service imports are inaccessible guards, never successful stubs.
use alkyhol_pe::{ProcessContext, ProcessImage32};
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = (|| -> alkyhol_pe::Result<u32> {
        let path = args.first().ok_or("expected EXE path")?;
        let mut modules = Vec::new();
        let mut external = Vec::new();
        let mut forced = false;
        let mut strict = false;
        let mut file_origins = false;
        let mut options = args[1..].iter();
        while let Some(option) = options.next() {
            match option.as_str() {
                "--dll" => {
                    let value = options.next().ok_or("--dll requires NAME=PATH")?;
                    let (name, path) = value.split_once('=').ok_or("--dll requires NAME=PATH")?;
                    modules.push((name.to_owned(), path.to_owned()));
                }
                "--external" => external.push(
                    options
                        .next()
                        .ok_or("--external requires a module basename")?
                        .clone(),
                ),
                "--rebase" => forced = true,
                "--strict" => strict = true,
                "--file-origins" => file_origins = true,
                _ => return Err("unknown startup diagnostic option".into()),
            }
        }
        let bytes = if file_origins {
            Vec::new()
        } else {
            std::fs::read(path).map_err(|e| e.to_string())?
        };
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("non-Unicode EXE basename")?;
        let environment = std::env::vars_os()
            .map(|(k, v)| {
                Ok((
                    k.into_string()
                        .map_err(|_| "non-Unicode environment name")?,
                    v.into_string()
                        .map_err(|_| "non-Unicode environment value")?,
                ))
            })
            .collect::<alkyhol_pe::Result<Vec<_>>>()?;
        let process = ProcessContext::install(&[path.to_owned()], &environment)?;
        let file_modules: Vec<_> = modules
            .iter()
            .map(|(name, path)| (name.as_str(), std::path::Path::new(path)))
            .collect();
        let byte_modules = if file_origins {
            Vec::new()
        } else {
            modules
                .iter()
                .map(|(name, path)| {
                    Ok((
                        name.clone(),
                        std::fs::read(path).map_err(|e| e.to_string())?,
                    ))
                })
                .collect::<alkyhol_pe::Result<Vec<_>>>()?
        };
        let modules: Vec<_> = byte_modules
            .iter()
            .map(|(name, b)| (name.as_str(), b.as_slice()))
            .collect();
        eprintln!(
            "PE32 diagnostic mode: guarded external imports; every fault stops; no exception recovery"
        );
        let mut image = if file_origins {
            unsafe {
                ProcessImage32::load_files(
                    std::path::Path::new(path),
                    &file_modules,
                    if strict { None } else { Some(&external) },
                    forced,
                    &process,
                )
            }
        } else if strict {
            unsafe { ProcessImage32::load((name, &bytes), &modules, forced, &process) }
        } else {
            unsafe {
                ProcessImage32::load_diagnostic(
                    (name, &bytes),
                    &modules,
                    &external,
                    forced,
                    &process,
                )
            }
        }?;
        unsafe { image.enter() }
    })();
    match result {
        Ok(status) if status <= 255 => std::process::exit(status as i32),
        Ok(_) => {
            eprintln!("Alkyhol PE32 unsupported: exit status exceeds host process width");
            std::process::exit(78)
        }
        Err(e) => {
            eprintln!("Alkyhol PE32 unsupported: {e}");
            std::process::exit(78)
        }
    }
}
