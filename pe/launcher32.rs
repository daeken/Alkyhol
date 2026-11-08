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
enum Guest32<'a> {
    Plain(alkyhol_pe::LoadedImage32<'a>),
    Exceptions(alkyhol_pe::ExceptionImage32<'a>),
}
impl Guest32<'_> {
    fn relocated(&self) -> bool {
        match self {
            Self::Plain(i) => i.relocated(),
            Self::Exceptions(i) => i.relocated(),
        }
    }
    unsafe fn enter(&self) -> alkyhol_pe::Result<u32> {
        match self {
            Self::Plain(i) => unsafe { i.enter() },
            Self::Exceptions(i) => unsafe { i.enter() },
        }
    }
}
fn main() {
    let all: Vec<_> = std::env::args().skip(1).collect();
    let split = all.iter().position(|a| a == "--").unwrap_or(all.len());
    let args = &all[..split];
    if args.iter().any(|a| {
        ![
            "--inspect",
            "--rebase",
            "--loader-info",
            "--teb-info",
            "--repeat",
            "--seh",
        ]
        .contains(&a.as_str())
    }) {
        process::exit(64);
    }
    let result = (|| -> alkyhol_pe::Result<u32> {
        let mut argv = vec![
            std::env::current_exe()
                .map_err(|e| e.to_string())?
                .to_str()
                .ok_or("non-Unicode executable name")?
                .to_owned(),
        ];
        if split < all.len() {
            argv.extend_from_slice(&all[split + 1..]);
        }
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
        let process = alkyhol_pe::ProcessContext::install(&argv, &environment)?;
        let bytes = embedded_pe();
        let parsed = alkyhol_pe::Image::parse_i386(bytes)?;
        if args.iter().any(|a| a == "--seh") {
            parsed.check_exception_startup32()?;
        } else {
            parsed.check_startup()?;
        }
        if args
            .iter()
            .any(|a| a == "--loader-info" || a == "--inspect")
        {
            eprintln!(
                "PE32: {} sections, {} imports, {} HIGHLOW fixups",
                parsed.sections.len(),
                parsed.imports()?.len(),
                parsed.relocations()?.len()
            );
        }
        let rebase = args.iter().any(|a| a == "--rebase");
        let image = if args.iter().any(|a| a == "--seh") {
            Guest32::Exceptions(alkyhol_pe::ExceptionImage32::load(bytes, rebase, &process)?)
        } else {
            Guest32::Plain(alkyhol_pe::LoadedImage32::load(bytes, rebase, &process)?)
        };
        if args
            .iter()
            .any(|a| a == "--loader-info" || a == "--inspect")
        {
            eprintln!("checked i386 mapping; relocated={}", image.relocated());
        }
        if args.iter().any(|a| a == "--inspect") {
            return Ok(0);
        }
        let mut status = 0;
        // Explicit trusted-observer lifecycle diagnostic, not CRT/process restart.
        for _ in 0..if args.iter().any(|a| a == "--repeat") {
            2
        } else {
            1
        } {
            status = unsafe { image.enter() }?;
            if args.iter().any(|a| a == "--teb-info") {
                // enter() checked the actual before/after selectors and cleared
                // its exclusively leased TEB descriptor before returning.
                eprintln!("host segment selectors restored; owned FS descriptor cleared");
            }
            if status != 0 {
                break;
            }
        }
        Ok(status)
    })();
    match result {
        Ok(status) if status <= 255 => process::exit(status as i32),
        Ok(status) => {
            eprintln!("unsupported PE32 exit status {status}");
            process::exit(78)
        }
        Err(error) => {
            eprintln!("Alkyhol PE32 unsupported: {error}");
            process::exit(78)
        }
    }
}
