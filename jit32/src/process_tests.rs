//! Synthetic loader/runtime checks; original-binary/native evidence is separate.
use super::*;
const OPT: usize = 0x98;
fn p16(b: &mut [u8], at: usize, value: u16) {
    b[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn p32(b: &mut [u8], at: usize, value: u32) {
    b[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn directory(b: &mut [u8], index: usize, rva: u32, len: u32) {
    p32(b, OPT + 96 + index * 8, rva);
    p32(b, OPT + 100 + index * 8, len);
}
fn base_fixture() -> Vec<u8> {
    let mut b = vec![0; 0xa00];
    b[..2].copy_from_slice(b"MZ");
    p32(&mut b, 0x3c, 0x80);
    b[0x80..0x84].copy_from_slice(b"PE\0\0");
    p16(&mut b, 0x84, 0x14c);
    p16(&mut b, 0x86, 2);
    p16(&mut b, 0x94, 224);
    p16(&mut b, 0x96, 0x102);
    p16(&mut b, OPT, 0x10b);
    p32(&mut b, OPT + 16, 0x1000);
    p32(&mut b, OPT + 28, 0x400000);
    p32(&mut b, OPT + 32, 4096);
    p32(&mut b, OPT + 36, 512);
    p32(&mut b, OPT + 56, 0x3000);
    p32(&mut b, OPT + 60, 0x400);
    p16(&mut b, OPT + 68, 3);
    p32(&mut b, OPT + 92, 16);
    for (i, rva, raw, len, rights) in [
        (0, 0x1000, 0x400, 0x200, 0x60000020),
        (1, 0x2000, 0x600, 0x400, 0xc0000040),
    ] {
        let at = OPT + 224 + i * 40;
        p32(&mut b, at + 8, len);
        p32(&mut b, at + 12, rva);
        p32(&mut b, at + 16, len);
        p32(&mut b, at + 20, raw);
        p32(&mut b, at + 36, rights);
    }
    b[0x400..0x406].copy_from_slice(&[0xb8, 42, 0, 0, 0, 0xc3]);
    directory(&mut b, 1, 0x2000, 40);
    p32(&mut b, 0x600, 0x2080);
    p32(&mut b, 0x60c, 0x2060);
    p32(&mut b, 0x610, 0x20a0);
    b[0x660..0x66d].copy_from_slice(b"kernel32.dll\0");
    p32(&mut b, 0x680, 0x20c0);
    b[0x6c2..0x6cf].copy_from_slice(b"GetLastError\0");
    directory(&mut b, 5, 0x2200, 12);
    p32(&mut b, 0x800, 0x1000);
    p32(&mut b, 0x804, 12);
    p16(&mut b, 0x808, 0x3001);
    b
}
fn fixture(dll: bool, tls: bool) -> Vec<u8> {
    let mut b = base_fixture();
    b.resize(0xc00, 0);
    p16(&mut b, 0x86, 3);
    p32(&mut b, OPT + 56, 0x4000);
    if dll {
        p16(&mut b, 0x96, 0x2102);
    }
    let section = OPT + 224 + 80;
    for (off, value) in [
        (8, 0x200),
        (12, 0x3000),
        (16, 0x200),
        (20, 0xa00),
        (36, 0x40000040),
    ] {
        p32(&mut b, section + off, value);
    }
    // Remove the base helper's instruction-immediate relocation. A real absolute
    // pointer in data instead gives forced rebasing an observable target.
    p16(&mut b, 0x808, 0x3300);
    p32(&mut b, 0x800, 0x2000);
    p32(&mut b, 0x900, 0x401000);
    if dll {
        b[0x400..0x408].copy_from_slice(&[0xb8, 1, 0, 0, 0, 0xc2, 12, 0]);
    } else {
        b[0x400..0x403].copy_from_slice(&[0x31, 0xc0, 0xc3]);
    }
    b[0x420..0x426].copy_from_slice(&[0xb8, 86, 0, 0, 0, 0xc3]);
    // Named code + ordinal data exports, with metadata independently readonly.
    directory(&mut b, 0, 0x3000, 0x100);
    for (off, value) in [
        (0xc, 0x3060),
        (0x10, 7),
        (0x14, 2),
        (0x18, 2),
        (0x1c, 0x3040),
        (0x20, 0x3048),
        (0x24, 0x3050),
        (0x40, 0x1020),
        (0x44, 0x2300),
        (0x48, 0x3078),
        (0x4c, 0x3070),
    ] {
        p32(&mut b, 0xa00 + off, value);
    }
    p16(&mut b, 0xa50, 1);
    p16(&mut b, 0xa52, 0);
    b[0xa60..0xa68].copy_from_slice(b"dep.dll\0");
    b[0xa70..0xa73].copy_from_slice(b"Fn\0");
    b[0xa78..0xa7d].copy_from_slice(b"Data\0");
    if tls {
        directory(&mut b, 9, 0x3100, 24);
        for (off, value) in [
            (0, 0x402380),
            (4, 0x402384),
            (8, 0x4023c0),
            (12, 0x403120),
            (16, 0),
            (20, 0x00500000),
        ] {
            p32(&mut b, 0xb00 + off, value);
        }
        p32(&mut b, 0x980, 0x31415926);
        p32(&mut b, 0xb20, 0x401040);
        // Callback increments actual module data, then performs stdcall cleanup.
        b[0x440..0x449].copy_from_slice(&[0xff, 0x05, 0xd0, 0x23, 0x40, 0, 0xc2, 12, 0]);
        // Three relocation blocks: callback's address, data pointer, TLS metadata.
        directory(&mut b, 5, 0x2200, 44);
        p32(&mut b, 0x800, 0x1000);
        p32(&mut b, 0x804, 12);
        p16(&mut b, 0x808, 0x3042);
        p16(&mut b, 0x80a, 0);
        p32(&mut b, 0x80c, 0x2000);
        p32(&mut b, 0x810, 12);
        p16(&mut b, 0x814, 0x3300);
        p16(&mut b, 0x816, 0);
        p32(&mut b, 0x818, 0x3000);
        p32(&mut b, 0x81c, 20);
        for (i, off) in [0x3100, 0x3104, 0x3108, 0x310c, 0x3120, 0]
            .iter()
            .enumerate()
        {
            p16(&mut b, 0x820 + i * 2, *off);
        }
    }
    b
}
fn stdout() -> File {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "alkyhol-process-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let f = File::options()
        .create_new(true)
        .write(true)
        .open(&p)
        .unwrap();
    std::fs::remove_file(p).unwrap();
    f
}
#[test]
fn mapped_tls_and_callbacks_resume_once_at_both_bases() {
    let b = fixture(false, true);
    for rebase in [false, true] {
        let mut p = Process::load(
            &[ModuleInput {
                name: "owned.exe",
                bytes: &b,
            }],
            rebase,
            stdout(),
        )
        .unwrap();
        let base = p.modules()[0].base;
        let teb = p.machine.state[22];
        assert_eq!(p.machine.memory.word(teb + 0x18).unwrap(), teb as u32);
        assert_eq!(
            p.machine.memory.word(teb + 4).unwrap(),
            STACK + STACK_SIZE as u32
        );
        let vector = p.machine.memory.word(teb + 0x2c).unwrap();
        let block = p.machine.memory.word(vector as u64).unwrap();
        assert_eq!(p.machine.memory.word(block as u64).unwrap(), 0x31415926);
        assert_eq!(block & 15, 0);
        assert_eq!(p.machine.memory.word(base as u64 + 0x23c0).unwrap(), 0);
        assert_eq!(
            p.machine.memory.word(base as u64 + 0x2300).unwrap(),
            base + 0x1000
        );
        assert!(matches!(p.run_application(10), Stop::Unsupported { .. }));
        assert!(matches!(p.run_startup(1), Stop::Budget { .. }));
        assert_eq!(p.machine.memory.word(base as u64 + 0x23d0).unwrap(), 1);
        assert_eq!(p.run_startup(100), Stop::Returned(0));
        assert_eq!(p.phase(), &Phase::Ready);
        assert_eq!(p.run_startup(100), Stop::Returned(0));
        assert_eq!(p.machine.memory.word(base as u64 + 0x23d0).unwrap(), 1);
        assert_eq!(p.run_application(100), Stop::Returned(0));
        assert_eq!(p.phase(), &Phase::Finished);
        assert!(matches!(p.run_application(100), Stop::Unsupported { .. }));
    }
}
#[test]
fn static_dll_binding_named_ordinal_data_and_dependency_attach() {
    let mut exe = fixture(false, false);
    let dll = fixture(true, false);
    exe[0x660..0x680].fill(0);
    exe[0x660..0x668].copy_from_slice(b"dep.dll\0");
    exe[0x6c2..0x6e0].fill(0);
    exe[0x6c2..0x6c5].copy_from_slice(b"Fn\0");
    // EXE calls original dependency function via its real patched IAT.
    exe[0x400..0x407].copy_from_slice(&[0xff, 0x15, 0xa0, 0x20, 0x40, 0, 0xc3]);
    // Its absolute IAT operand must also be relocated in the rebased case.
    directory(&mut exe, 5, 0x2200, 24);
    p32(&mut exe, 0x80c, 0x1000);
    p32(&mut exe, 0x810, 12);
    p16(&mut exe, 0x814, 0x3002);
    for rebase in [false, true] {
        let mut p = Process::load(
            &[
                ModuleInput {
                    name: "owned.exe",
                    bytes: &exe,
                },
                ModuleInput {
                    name: "DeP.DlL",
                    bytes: &dll,
                },
            ],
            rebase,
            stdout(),
        )
        .unwrap();
        assert_eq!(p.modules.len(), 2);
        assert_ne!(p.modules[0].base, p.modules[1].base);
        assert!(matches!(p.run_application(10), Stop::Unsupported { .. }));
        assert!(matches!(p.run_startup(0), Stop::Budget { .. }));
        let sp = p.machine.state[4];
        assert_eq!(p.machine.memory.word(sp).unwrap(), RETURN);
        assert_eq!(p.machine.memory.word(sp + 4).unwrap(), p.modules[1].base);
        assert_eq!(p.machine.memory.word(sp + 8).unwrap(), 1);
        assert_ne!(p.machine.memory.word(sp + 12).unwrap(), 0);
        assert_eq!(
            p.run_startup(1),
            Stop::Budget {
                pc: p.modules[1].base + 0x1005
            }
        );
        assert_eq!(p.run_startup(10), Stop::Returned(0));
        assert_eq!(p.run_application(20), Stop::Returned(86));
    }
    let images = [Image::parse_i386_diagnostic(&dll, true).unwrap()];
    let bases = [0x10000000];
    let names = BTreeMap::from([("dep.dll".into(), 0)]);
    let mut binding = Binding {
        images: &images,
        bases: &bases,
        names: &names,
        gates: BTreeMap::new(),
        external: BTreeMap::new(),
        keys: BTreeMap::new(),
    };
    assert_eq!(
        binding
            .resolve(
                "DEP.DLL",
                &ImportSymbol::Name("Fn".into()),
                &mut BTreeSet::new()
            )
            .unwrap(),
        0x10001020
    );
    assert_eq!(
        binding
            .resolve("dep.dll", &ImportSymbol::Ordinal(8), &mut BTreeSet::new())
            .unwrap(),
        0x10002300
    );
    assert!(
        binding
            .resolve(
                "dep.dll",
                &ImportSymbol::Name("fn".into()),
                &mut BTreeSet::new()
            )
            .is_err()
    );
    assert!(
        Process::load(
            &[ModuleInput {
                name: "owned.exe",
                bytes: &exe
            }],
            false,
            stdout()
        )
        .is_err()
    );
}
#[test]
fn unresolved_import_exit_is_stable_and_never_returns_success() {
    let mut exe = fixture(false, false);
    exe[0x6c2..0x6e0].fill(0);
    exe[0x6c2..0x6d0].copy_from_slice(b"Unimplemented\0");
    exe[0x400..0x407].copy_from_slice(&[0xff, 0x15, 0xa0, 0x20, 0x40, 0, 0xc3]);
    let mut p = Process::load(
        &[ModuleInput {
            name: "owned.exe",
            bytes: &exe,
        }],
        false,
        stdout(),
    )
    .unwrap();
    assert_eq!(p.run_startup(10), Stop::Returned(0));
    let stop = p.run_application(10);
    assert!(matches!(stop, Stop::Import { .. }));
    let regs = p.machine.registers();
    assert_eq!(p.run_application(10), stop);
    assert_eq!(p.machine.registers(), regs);
    assert_eq!(p.machine.instructions, 1);
    assert_eq!(p.machine.service_calls, 0);
}
#[test]
fn attach_rejection_and_bad_stack_cleanup_cannot_reach_application() {
    for code in [
        &[0x31, 0xc0, 0xc2, 12, 0][..],
        &[0xb8, 1, 0, 0, 0, 0xc3][..],
    ] {
        let mut exe = fixture(false, false);
        let mut dll = fixture(true, false);
        exe[0x660..0x680].fill(0);
        exe[0x660..0x668].copy_from_slice(b"dep.dll\0");
        exe[0x6c2..0x6e0].fill(0);
        exe[0x6c2..0x6c5].copy_from_slice(b"Fn\0");
        dll[0x400..0x400 + code.len()].copy_from_slice(code);
        let mut p = Process::load(
            &[
                ModuleInput {
                    name: "owned.exe",
                    bytes: &exe,
                },
                ModuleInput {
                    name: "dep.dll",
                    bytes: &dll,
                },
            ],
            false,
            stdout(),
        )
        .unwrap();
        assert!(matches!(p.run_startup(100), Stop::Unsupported { .. }));
        assert_eq!(p.phase(), &Phase::Failed);
        assert!(matches!(p.run_startup(100), Stop::Unsupported { .. }));
        assert!(matches!(p.run_application(100), Stop::Unsupported { .. }));
    }
}
#[test]
fn graph_names_forwarders_and_address_collisions_are_checked() {
    assert_eq!(order(0, &[vec![1, 2], vec![2], vec![]]).unwrap(), [2, 1, 0]);
    assert!(order(0, &[vec![1], vec![0]]).is_err());
    assert!(name("../wrong.dll").is_err());
    assert!(name("Z:bad.dll").is_err());
    assert_eq!(place(0x400000, 0x10000, false, &[]).unwrap(), 0x400000);
    assert_ne!(
        place(0x400000, 0x10000, false, &[(0x400000, 0x10000)]).unwrap(),
        0x400000
    );
    let mut dll = fixture(true, false);
    p32(&mut dll, 0xa40, 0x3080);
    let text = b"kernel32.GetLastError\0";
    dll[0xa80..0xa80 + text.len()].copy_from_slice(text);
    let images = [Image::parse_i386_diagnostic(&dll, true).unwrap()];
    let bases = [0x10000000];
    let names = BTreeMap::from([("dep.dll".into(), 0)]);
    let mut binding = Binding {
        images: &images,
        bases: &bases,
        names: &names,
        gates: BTreeMap::new(),
        external: BTreeMap::new(),
        keys: BTreeMap::new(),
    };
    let gate = binding
        .resolve(
            "dep.dll",
            &ImportSymbol::Name("Fn".into()),
            &mut BTreeSet::new(),
        )
        .unwrap();
    assert_eq!(gate, GATE);
    assert!(binding.gates.contains_key(&gate));
    let same = binding
        .resolve(
            "kernel32.dll",
            &ImportSymbol::Name("GetLastError".into()),
            &mut BTreeSet::new(),
        )
        .unwrap();
    assert_eq!(same, gate);
    assert!(
        binding
            .resolve(
                "missing.dll",
                &ImportSymbol::Name("Fn".into()),
                &mut BTreeSet::new()
            )
            .is_err()
    );
}

#[test]
fn direct_teb_last_error_writes_survive_preserving_kernel_calls() {
    let exe = fixture(false, false);
    let mut p = Process::load(
        &[ModuleInput {
            name: "owned.exe",
            bytes: &exe,
        }],
        false,
        stdout(),
    )
    .unwrap();
    let teb = p.machine.teb.unwrap();
    p.machine
        .memory
        .write(teb as u64 + 0x34, &0x13572468u32.to_le_bytes())
        .unwrap();
    p.begin(p.entry, &[]).unwrap();
    p.machine
        .service(Service::Kernel(crate::system::Call::GetProcessHeap))
        .unwrap();
    assert_eq!(
        p.machine.memory.word(teb as u64 + 0x34).unwrap(),
        0x13572468
    );
    p.machine.state[4] -= 4;
    p.machine
        .memory
        .write(p.machine.state[4], &RETURN.to_le_bytes())
        .unwrap();
    p.machine.service(Service::GetLastError).unwrap();
    assert_eq!(p.machine.registers()[0], 0x13572468);
}

#[test]
fn dormant_dlls_do_not_acquire_imports_tls_or_initializers() {
    let exe = fixture(false, false);
    let mut dormant = fixture(true, true);
    dormant[0x660..0x680].fill(0);
    dormant[0x660..0x66c].copy_from_slice(b"missing.dll\0");
    let mut p = Process::load(
        &[
            ModuleInput {
                name: "owned.exe",
                bytes: &exe,
            },
            ModuleInput {
                name: "dormant.dll",
                bytes: &dormant,
            },
        ],
        false,
        stdout(),
    )
    .unwrap();
    assert_eq!(p.modules().len(), 1);
    assert!(p.startup.is_empty());
    assert!(p.modules()[0].tls_index.is_none());
    assert_eq!(p.run_startup(1), Stop::Returned(0));
    assert_eq!(p.run_application(10), Stop::Returned(0));
}
