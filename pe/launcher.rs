//! Thin standalone x86-64 Mach-O host. The original PE file is a separate,
//! read-only __ALKYHOL,__pefile section supplied by the translation tool.
use std::process;
unsafe extern "C" {
    fn _dyld_get_image_header(index: u32) -> *const u8;
    fn _dyld_get_image_vmaddr_slide(index: u32) -> isize;
}
fn embedded_pe() -> &'static [u8] {
    // SAFETY: dyld owns a valid native Mach-O header/load-command list. Only the
    // linker-created read-only section with our exact segment/name is selected.
    unsafe {
        let header = _dyld_get_image_header(0);
        assert!(!header.is_null());
        assert_eq!(header.cast::<u32>().read_unaligned(), 0xfeedfacf);
        let ncmds = header.add(16).cast::<u32>().read_unaligned() as usize;
        let sizeofcmds = header.add(20).cast::<u32>().read_unaligned() as usize;
        let mut offset = 32usize;
        for _ in 0..ncmds {
            assert!(offset + 8 <= 32 + sizeofcmds);
            let cmd = header.add(offset);
            let kind = cmd.cast::<u32>().read_unaligned();
            let size = cmd.add(4).cast::<u32>().read_unaligned() as usize;
            assert!(size >= 8 && offset + size <= 32 + sizeofcmds);
            if kind == 0x19 {
                // LC_SEGMENT_64
                assert!(size >= 72);
                let count = cmd.add(64).cast::<u32>().read_unaligned() as usize;
                assert!(72 + count * 80 <= size);
                for i in 0..count {
                    let s = cmd.add(72 + i * 80);
                    let name = std::slice::from_raw_parts(s, 16);
                    let segment = std::slice::from_raw_parts(s.add(16), 16);
                    if name == b"__pefile\0\0\0\0\0\0\0\0" && segment == b"__ALKYHOL\0\0\0\0\0\0\0"
                    {
                        let addr = s.add(32).cast::<u64>().read_unaligned();
                        let len = s.add(40).cast::<u64>().read_unaligned();
                        let slide = _dyld_get_image_vmaddr_slide(0);
                        let start = (addr as usize).wrapping_add_signed(slide);
                        return std::slice::from_raw_parts(start as *const u8, len as usize);
                    }
                }
            }
            offset += size;
        }
    }
    panic!("missing embedded PE payload")
}
fn main() {
    let raw: Vec<_> = std::env::args().skip(1).collect();
    let split = raw.iter().position(|a| a == "--").unwrap_or(raw.len());
    let args = &raw[..split];
    if args
        .iter()
        .any(|a| !["--inspect", "--rebase", "--loader-info"].contains(&a.as_str()))
    {
        eprintln!(
            "usage: translated-program [--inspect] [--rebase] [--loader-info] [-- guest-arguments...]"
        );
        process::exit(64);
    }
    let mut guest_args = vec![
        std::env::current_exe()
            .expect("host executable path")
            .to_string_lossy()
            .into_owned(),
    ];
    guest_args.extend(raw.iter().skip(split + 1).cloned());
    let info = args
        .iter()
        .any(|a| ["--inspect", "--loader-info"].contains(&a.as_str()));
    let bytes = embedded_pe();
    let result = (|| -> alkyhol_pe::Result<()> {
        let environment = std::env::vars_os()
            .map(|(k, v)| {
                Ok((
                    k.into_string()
                        .map_err(|_| "non-Unicode host environment key")?,
                    v.into_string()
                        .map_err(|_| "non-Unicode host environment value")?,
                ))
            })
            .collect::<alkyhol_pe::Result<Vec<_>>>()?;
        let process_scope = alkyhol_pe::ProcessContext::install(&guest_args, &environment)?;
        let image = alkyhol_pe::Image::parse(bytes)?;
        image.check_registered_startup()?;
        let imports = image.imports()?;
        let relocs = image.relocations()?;
        if info {
            let ordinal_count = imports
                .iter()
                .filter(|i| matches!(i.symbol, alkyhol_pe::ImportSymbol::Ordinal(_)))
                .count();
            let kinds = if ordinal_count == 0 {
                format!("{} named imports", imports.len())
            } else {
                format!(
                    "{} named imports, {ordinal_count} ordinal imports",
                    imports.len() - ordinal_count
                )
            };
            eprintln!(
                "PE64: {} sections, {}, {} DIR64 fixups",
                image.sections.len(),
                kinds,
                relocs.len()
            );
        }
        if args.iter().any(|a| a == "--inspect") {
            // load() also preflights exact import resolution before mapping.
            let mapped = alkyhol_pe::RegisteredImage::load(
                bytes,
                args.iter().any(|a| a == "--rebase"),
                &process_scope,
            )?;
            eprintln!("checked mapping; relocated={}", mapped.relocated());
            return Ok(());
        }
        let mapped = alkyhol_pe::RegisteredImage::load(
            bytes,
            args.iter().any(|a| a == "--rebase"),
            &process_scope,
        )?;
        if info {
            eprintln!("entering Win64 code; relocated={}", mapped.relocated());
        }
        // SAFETY: user explicitly requested execution of this trusted PE program.
        let status = unsafe { mapped.enter() };
        if status > 255 {
            return Err(format!("unsupported Windows exit status {status:#x}"));
        }
        process::exit(status as i32);
    })();
    if let Err(error) = result {
        eprintln!("Alkyhol PE preflight/runtime: {error}");
        process::exit(78);
    }
}
