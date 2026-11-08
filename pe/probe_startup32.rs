//! Explicit diagnostic execution of a caller-supplied original PE32/DLL graph.
//! External-service imports are inaccessible guards, never successful stubs.
use alkyhol_pe::{ProcessContext, ProcessImage32, ProcessOptions32, TimeZone32};
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let result = (|| -> alkyhol_pe::Result<u32> {
        let path = args.first().ok_or("expected EXE path")?;
        let mut modules = Vec::new();
        let mut external = Vec::new();
        let mut forced = false;
        let mut strict = false;
        let mut file_origins = false;
        let mut profile = ProcessOptions32::default();
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
                "--host-machine-info" => profile.host_machine_info = true,
                "--timezone" => {
                    if options.next().map(String::as_str) != Some("UTC") {
                        return Err(
                            "only the explicitly selected UTC guest timezone is supported".into(),
                        );
                    }
                    profile.time_zone = Some(TimeZone32::Utc);
                }
                "--activation" => {
                    profile.activation_assemblies.get_or_insert_with(Vec::new);
                }
                "--assembly-manifest" => {
                    profile
                        .activation_assemblies
                        .get_or_insert_with(Vec::new)
                        .push(
                            options
                                .next()
                                .ok_or("--assembly-manifest requires a manifest path")?
                                .into(),
                        );
                }
                "--assembly-policy" => {
                    profile.activation_assemblies.get_or_insert_with(Vec::new);
                    profile.activation_policies.push(options.next().ok_or("--assembly-policy requires an explicitly trusted installed policy manifest")?.into());
                }
                "--system-directory" => {
                    profile.system_directory = Some(
                        options
                            .next()
                            .ok_or("--system-directory requires a host directory")?
                            .into(),
                    )
                }
                "--system-windows-directory" => {
                    profile.system_windows_directory = Some(
                        options
                            .next()
                            .ok_or("--system-windows-directory requires a host directory")?
                            .into(),
                    );
                }
                "--windows-directory" => {
                    profile.windows_directory = Some(
                        options
                            .next()
                            .ok_or("--windows-directory requires a host directory")?
                            .into(),
                    );
                }
                _ => return Err("unknown startup diagnostic option".into()),
            }
        }
        if (profile.system_directory.is_some()
            || profile.system_windows_directory.is_some()
            || profile.windows_directory.is_some()
            || profile.host_machine_info
            || profile.time_zone.is_some()
            || profile.activation_assemblies.is_some())
            && !file_origins
        {
            return Err("guest directory/assembly profile requires --file-origins".into());
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
                ProcessImage32::load_files_configured(
                    std::path::Path::new(path),
                    &file_modules,
                    if strict { None } else { Some(&external) },
                    forced,
                    &process,
                    &profile,
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
