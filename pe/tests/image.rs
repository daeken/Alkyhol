use alkyhol_pe::{Image, ImportSymbol};

fn put16(b: &mut [u8], i: usize, n: u16) {
    b[i..i + 2].copy_from_slice(&n.to_le_bytes());
}
fn put32(b: &mut [u8], i: usize, n: u32) {
    b[i..i + 4].copy_from_slice(&n.to_le_bytes());
}
fn put64(b: &mut [u8], i: usize, n: u64) {
    b[i..i + 8].copy_from_slice(&n.to_le_bytes());
}
const OPT: usize = 0x98;
const TABLE: usize = OPT + 240;
fn directory(b: &mut [u8], i: usize, rva: u32, size: u32) {
    put32(b, OPT + 112 + i * 8, rva);
    put32(b, OPT + 116 + i * 8, size);
}
fn fixture() -> Vec<u8> {
    let mut b = vec![0; 0x800];
    b[..2].copy_from_slice(b"MZ");
    put32(&mut b, 0x3c, 0x80);
    b[0x80..0x84].copy_from_slice(b"PE\0\0");
    put16(&mut b, 0x84, 0x8664);
    put16(&mut b, 0x86, 2);
    put16(&mut b, 0x94, 240);
    put16(&mut b, 0x96, 0x22);
    put16(&mut b, OPT, 0x20b);
    put32(&mut b, OPT + 16, 0x1000);
    put64(&mut b, OPT + 24, 0x140000000);
    put32(&mut b, OPT + 32, 0x1000);
    put32(&mut b, OPT + 36, 0x200);
    put32(&mut b, OPT + 56, 0x3000);
    put32(&mut b, OPT + 60, 0x200);
    put16(&mut b, OPT + 68, 3);
    put32(&mut b, OPT + 108, 16);
    for (i, rva, raw, rights) in [
        (0, 0x1000, 0x200, 0x60000020),
        (1, 0x2000, 0x400, 0xc0000040),
    ] {
        let s = TABLE + i * 40;
        put32(&mut b, s + 8, 0x400);
        put32(&mut b, s + 12, rva);
        put32(&mut b, s + 16, if i == 0 { 0x200 } else { 0x400 });
        put32(&mut b, s + 20, raw);
        put32(&mut b, s + 36, rights);
    }
    b[0x200..0x206].copy_from_slice(&[0xb8, 42, 0, 0, 0, 0xc3]);
    b
}
fn imports_fixture() -> Vec<u8> {
    let mut b = fixture();
    directory(&mut b, 1, 0x2000, 40);
    put32(&mut b, 0x400, 0x2060);
    put32(&mut b, 0x40c, 0x2040);
    put32(&mut b, 0x410, 0x2080);
    b[0x440..0x44d].copy_from_slice(b"kernel32.dll\0");
    put64(&mut b, 0x460, 0x20a0);
    put64(&mut b, 0x480, 0x20a0);
    b[0x4a2..0x4af].copy_from_slice(b"GetLastError\0");
    b
}
fn reloc_fixture() -> Vec<u8> {
    let mut b = fixture();
    directory(&mut b, 5, 0x2100, 12);
    put32(&mut b, 0x500, 0x2000);
    put32(&mut b, 0x504, 12);
    put16(&mut b, 0x508, 0xa180);
    put64(&mut b, 0x580, 0x140001000);
    b
}
#[test]
fn conventional_layout_and_zero_fill_are_preserved() {
    let b = fixture();
    let p = Image::parse(&b).unwrap();
    assert_eq!(p.bitness(), 64);
    assert_eq!(p.declared_size(), p.size);
    assert_eq!(
        (p.base, p.entry, p.size, p.headers),
        (0x140000000, 0x1000, 0x3000, 0x200)
    );
    assert_eq!((p.sections[0].raw_size, p.sections[0].size), (0x200, 0x400));
    assert!(p.imports().unwrap().is_empty());
    assert!(p.relocations().unwrap().is_empty());
    p.check_startup().unwrap();
}
#[test]
fn amd64_certificate_metadata_retains_its_existing_file_backing_gate() {
    let mut bytes = fixture();
    assert!(Image::parse(&bytes).unwrap().static_tls32().is_err());
    assert!(
        Image::parse(&bytes)
            .unwrap()
            .legacy_load_config32()
            .is_err()
    );
    assert_eq!(Image::parse(&bytes).unwrap().certificate_directory(), None);
    directory(&mut bytes, 4, 0x7f8, 8);
    let before = bytes.clone();
    assert_eq!(
        Image::parse(&bytes).unwrap().certificate_directory(),
        Some((0x7f8, 8))
    );
    assert_eq!(bytes, before);
    for (offset, len) in [(0x800, 8), (0xfffffff8, 16), (0, 8), (8, 0)] {
        let mut bad = bytes.clone();
        directory(&mut bad, 4, offset, len);
        assert!(Image::parse(&bad).is_err());
    }
}
#[test]
fn all_truncated_prefixes_fail_without_panicking() {
    let b = fixture();
    for n in 0..b.len() {
        assert!(Image::parse(&b[..n]).is_err(), "{n}");
    }
}
#[test]
fn invalid_layouts_and_unsafe_page_geometry_are_rejected() {
    let cases = [
        (0x84, 0x14c, 2),
        (0x96, 0x2022, 2),
        (OPT, 0x10b, 2),
        (OPT + 32, 0x200, 4),
        (OPT + 36, 3, 4),
        (OPT + 56, 0xfffff000, 4),
        (OPT + 56, 0x2fff, 4),
        (OPT + 60, 0x180, 4),
        (OPT + 68, 1, 2),
        (OPT + 70, 0x4000, 2),
        (OPT + 108, 17, 4),
        (OPT + 16, 0x1200, 4),
        (TABLE + 36, 0xe0000020, 4),
        (TABLE + 40 + 12, 0x1000, 4),
        (TABLE + 40 + 20, 0x200, 4),
        (TABLE + 12, 0xfffff000, 4),
    ];
    for (off, value, width) in cases {
        let mut b = fixture();
        if width == 2 {
            put16(&mut b, off, value as u16);
        } else {
            put32(&mut b, off, value);
        }
        assert!(Image::parse(&b).is_err(), "{off:x}/{value:x}");
    }
}
#[test]
fn named_imports_use_checked_lookup_and_separate_iat() {
    let b = imports_fixture();
    let p = Image::parse(&b).unwrap();
    let i = p.imports().unwrap();
    assert_eq!(i.len(), 1);
    assert_eq!(
        (&*i[0].module, &i[0].symbol, i[0].iat),
        (
            "kernel32.dll",
            &ImportSymbol::Name("GetLastError".into()),
            0x2080
        )
    );
    let mut fallback = b.clone();
    put32(&mut fallback, 0x400, 0);
    assert_eq!(Image::parse(&fallback).unwrap().imports().unwrap(), i);
}
#[test]
fn malformed_imports_never_become_empty_success() {
    for (off, value, width) in [
        (0x400, 0x2ff8, 4),
        (0x404, 1, 4),
        (0x410, 0x1000, 4),
        (0x410, 0x2081, 4),
        (0x460, 0x8000000000010001, 8),
        (0x460, 0x2300, 8),
        (0x44c, 0x80, 1),
    ] {
        let mut b = imports_fixture();
        match width {
            8 => put64(&mut b, off, value),
            4 => put32(&mut b, off, value as u32),
            _ => b[off] = value as u8,
        };
        assert!(Image::parse(&b).unwrap().imports().is_err(), "{off:x}");
    }
    let mut b = imports_fixture();
    directory(&mut b, 1, 0x2000, 20);
    assert!(Image::parse(&b).unwrap().imports().is_err());
}
#[test]
fn named_and_biased_ordinal_thunks_are_distinct_and_keep_order() {
    let mut b = imports_fixture();
    put32(&mut b, 0x400, 0x20c0);
    put64(&mut b, 0x4c0, 0x20a0);
    put64(&mut b, 0x4c8, 0x8000000000000017);
    put64(&mut b, 0x4d0, 0x800000000000ffff);
    put64(&mut b, 0x4d8, 0x8000000000000000);
    let imports = Image::parse(&b).unwrap().imports().unwrap();
    assert_eq!(
        imports.iter().map(|i| &i.symbol).collect::<Vec<_>>(),
        [
            &ImportSymbol::Name("GetLastError".into()),
            &ImportSymbol::Ordinal(23),
            &ImportSymbol::Ordinal(65535),
            &ImportSymbol::Ordinal(0)
        ]
    );
    assert_eq!(
        imports.iter().map(|i| i.iat).collect::<Vec<_>>(),
        [0x2080, 0x2088, 0x2090, 0x2098]
    );
    let mut named_number = imports_fixture();
    named_number[0x4a2..0x4af].fill(0);
    named_number[0x4a2..0x4a6].copy_from_slice(b"#23\0");
    assert_eq!(
        Image::parse(&named_number).unwrap().imports().unwrap()[0].symbol,
        ImportSymbol::Name("#23".into())
    );
    assert_ne!(ImportSymbol::Name("#23".into()), ImportSymbol::Ordinal(23));
}
#[test]
fn ordinal_reserved_bits_and_unbacked_named_hint_are_not_silently_masked() {
    for value in [
        0x8000000000010017,
        0x8000000100000017,
        0xc000000000000017,
        0x80000000,
        0x100000000,
        0x1ffe,
    ] {
        let mut b = imports_fixture();
        put64(&mut b, 0x460, value);
        assert!(Image::parse(&b).unwrap().imports().is_err(), "{value:x}");
    }
    let mut b = imports_fixture();
    put32(&mut b, 0x400, 0); // Valid ILT fallback to ordinal IAT.
    put64(&mut b, 0x480, 0x8000000000000017);
    assert_eq!(
        Image::parse(&b).unwrap().imports().unwrap()[0].symbol,
        ImportSymbol::Ordinal(23)
    );
}
#[test]
fn relocation_blocks_are_bounded_typed_and_disjoint() {
    let mut stripped = reloc_fixture();
    put16(&mut stripped, 0x96, 0x23);
    assert!(Image::parse(&stripped).is_err());

    let b = reloc_fixture();
    assert_eq!(Image::parse(&b).unwrap().relocations().unwrap(), [0x2180]);
    for (off, value, width) in [
        (0x500, 0x1000, 4),
        (0x504, 7, 4),
        (0x504, 14, 4),
        (0x508, 0x3180, 2),
        (0x50a, 0xa184, 2),
        (0x508, 0xafff, 2),
    ] {
        let mut b = reloc_fixture();
        if width == 2 {
            put16(&mut b, off, value as u16);
        } else {
            put32(&mut b, off, value);
        }
        assert!(
            Image::parse(&b).unwrap().relocations().is_err(),
            "{off:x}/{value:x}"
        );
    }
}
#[test]
fn unsupported_startup_is_rejected_even_when_directory_is_valid() {
    for i in [0, 3, 7, 8, 9, 10, 11, 13, 14, 15] {
        let mut b = fixture();
        directory(&mut b, i, 0x2200, 16);
        assert!(Image::parse(&b).unwrap().check_startup().is_err(), "{i}");
    }
}
#[test]
fn directory_backing_and_rva_overflow_are_checked() {
    for (rva, size) in [(0x2400, 1), (0xfffffff0, 64), (0x23ff, 2), (0, 16)] {
        let mut b = fixture();
        directory(&mut b, 1, rva, size);
        assert!(Image::parse(&b).is_err());
    }
}

#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
#[test]
fn mapping_rebases_and_executes_initialized_code() {
    let b = reloc_fixture();
    let m = alkyhol_pe::LoadedImage::load(&b, true).unwrap();
    assert!(m.relocated());
    assert_eq!(unsafe { m.enter() }, 42);
    // A missing import is rejected before any code can run.
    let mut b = imports_fixture();
    b[0x440..0x44d].copy_from_slice(b"vendorr!.dll\0");
    assert!(
        alkyhol_pe::LoadedImage::load(&b, false)
            .err()
            .unwrap()
            .contains("unsupported import")
    );
    assert!(alkyhol_pe::LoadedImage::load(&fixture(), true).is_err());
}

#[test]
fn sparse_bss_extent_accepts_one_gib_but_preserves_bounds_and_overflow_rejection() {
    let mut bytes = fixture();
    let bound = 1024 * 1024 * 1024u32;
    put32(&mut bytes, OPT + 56, bound);
    put32(&mut bytes, TABLE + 40 + 8, bound - 0x2000);
    let image = Image::parse(&bytes).unwrap();
    assert_eq!(image.size, bound as usize);
    assert_eq!(bytes.len(), 0x800);
    assert_eq!(image.sections[1].size, (bound - 0x2000) as usize);
    put32(&mut bytes, OPT + 56, bound + 4096);
    assert!(Image::parse(&bytes).is_err());
    put32(&mut bytes, OPT + 56, bound);
    put32(&mut bytes, TABLE + 40 + 8, bound - 0x1000);
    assert!(Image::parse(&bytes).is_err());
    put32(&mut bytes, TABLE + 40 + 8, bound - 0x2000);
    put64(&mut bytes, OPT + 24, 0xffffffffc0000000);
    assert!(Image::parse(&bytes).is_err());
}

fn static_fixture() -> Vec<u8> {
    let mut b = fixture();
    put32(&mut b, TABLE + 40 + 36, 0x40000040);
    directory(&mut b, 3, 0x2080, 12);
    put32(&mut b, 0x480, 0x1000);
    put32(&mut b, 0x484, 0x1006);
    put32(&mut b, 0x488, 0x20c0);
    b[0x4c0..0x4c4].copy_from_slice(&[1, 0, 0, 0]);
    b
}
#[test]
fn static_runtime_records_are_original_rvas_and_require_explicit_registered_preflight() {
    assert!(
        Image::parse(&fixture())
            .unwrap()
            .runtime_functions()
            .unwrap()
            .is_none()
    );
    let b = static_fixture();
    let p = Image::parse(&b).unwrap();
    let t = p.runtime_functions().unwrap().unwrap();
    assert_eq!(t.rva, 0x2080);
    assert_eq!(
        t.entries,
        vec![alkyhol_pe::RuntimeFunction {
            begin: 0x1000,
            end: 0x1006,
            unwind: 0x20c0
        }]
    );
    assert_eq!(std::mem::size_of::<alkyhol_pe::RuntimeFunction>(), 12);
    assert_eq!(std::mem::align_of::<alkyhol_pe::RuntimeFunction>(), 4);
    assert!(p.check_startup().is_err());
    p.check_registered_startup().unwrap();
    for dir in [7, 8, 9, 10, 11, 13, 14, 15] {
        let mut bad = b.clone();
        directory(&mut bad, dir, 0x2100, 4);
        assert!(
            Image::parse(&bad)
                .unwrap()
                .check_registered_startup()
                .is_err()
        );
    }
    let mut dll = b.clone();
    put16(&mut dll, 0x96, 0x2022);
    let p = Image::parse_dll(&dll).unwrap();
    assert!(p.runtime_functions().unwrap().is_some());
    assert!(p.check_registered_startup().is_err());
}
#[test]
fn registered_dll_preflight_preserves_kind_exports_static_recipe_and_other_startup_gates() {
    assert!(
        Image::parse(&static_fixture())
            .unwrap()
            .check_registered_library_startup()
            .is_err()
    );
    let mut dll = static_fixture();
    put16(&mut dll, 0x96, 0x2022);
    Image::parse_dll(&dll)
        .unwrap()
        .check_registered_library_startup()
        .unwrap();
    put32(&mut dll, OPT + 16, 0);
    Image::parse_dll(&dll)
        .unwrap()
        .check_registered_library_startup()
        .unwrap();
    for dir in [7, 8, 9, 10, 11, 13, 14, 15] {
        let mut bad = dll.clone();
        directory(&mut bad, dir, 0x2100, 4);
        assert!(
            Image::parse_dll(&bad)
                .unwrap()
                .check_registered_library_startup()
                .is_err()
        );
    }
    let mut bad = dll.clone();
    bad[0x4c0] = 9;
    assert!(
        Image::parse_dll(&bad)
            .unwrap()
            .check_registered_library_startup()
            .is_err()
    );
    let mut bad = dll;
    directory(&mut bad, 0, 0x2100, 4);
    assert!(
        Image::parse_dll(&bad)
            .unwrap()
            .check_registered_library_startup()
            .is_err()
    );
}
#[test]
fn static_table_shapes_permissions_ranges_and_recipe_operands_are_checked() {
    for (offset, value) in [
        (OPT + 112 + 3 * 8, 0x2081),
        (OPT + 116 + 3 * 8, 13),
        (TABLE + 40 + 36, 0xc0000040),
        (0x480, 0x1006),
        (0x484, 0x2000),
        (0x488, 0),
        (0x488, 0x20c1),
        (0x488, 0x2400),
    ] {
        let mut b = static_fixture();
        put32(&mut b, offset, value);
        match Image::parse(&b) {
            Err(_) => (),
            Ok(p) => assert!(p.runtime_functions().is_err(), "{offset:x} {value:x}"),
        }
    }
    for header in [
        [2, 0, 0, 0],
        [9, 0, 0, 0],
        [1, 0, 0, 5],
        [1, 7, 0, 0],
        [1, 1, 1, 0],
    ] {
        let mut b = static_fixture();
        b[0x4c0..0x4c4].copy_from_slice(&header);
        b[0x4c4..0x4c8].copy_from_slice(&[1, 3, 0, 0]);
        assert!(
            Image::parse(&b).unwrap().runtime_functions().is_err(),
            "{header:?}"
        );
    }
    let mut b = static_fixture();
    put32(&mut b, 0x488, 0x23fc);
    b[0x7fc..0x800].copy_from_slice(&[1, 1, 1, 0]);
    assert!(Image::parse(&b).unwrap().runtime_functions().is_err());
    let mut b = static_fixture();
    directory(&mut b, 3, 0x2080, 24);
    put32(&mut b, 0x48c, 0x1002);
    put32(&mut b, 0x490, 0x1006);
    put32(&mut b, 0x494, 0x20c0);
    assert!(Image::parse(&b).unwrap().runtime_functions().is_err());
}
#[test]
fn loader_relocations_and_import_writes_cannot_mutate_preflighted_static_metadata() {
    let mut b = static_fixture();
    directory(&mut b, 5, 0x2200, 12);
    put32(&mut b, 0x600, 0x2000);
    put32(&mut b, 0x604, 12);
    put16(&mut b, 0x608, 0xa080);
    assert!(Image::parse(&b).unwrap().runtime_functions().is_err());
    put16(&mut b, 0x608, 0xa0c0);
    assert!(Image::parse(&b).unwrap().runtime_functions().is_err());
    put16(&mut b, 0x608, 0xa180);
    Image::parse(&b).unwrap().runtime_functions().unwrap();
    // A real valid import descriptor whose IAT aliases the static record.
    let mut b = static_fixture();
    directory(&mut b, 1, 0x2100, 40);
    put32(&mut b, 0x500, 0x2160);
    put32(&mut b, 0x50c, 0x2140);
    put32(&mut b, 0x510, 0x2080);
    b[0x540..0x54d].copy_from_slice(b"kernel32.dll\0");
    put64(&mut b, 0x560, 0x21a0);
    b[0x5a2..0x5af].copy_from_slice(b"GetLastError\0");
    assert!(Image::parse(&b).unwrap().runtime_functions().is_err());
}
