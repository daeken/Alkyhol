use alkyhol_pe::Image;

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
        (&*i[0].module, &*i[0].symbol, i[0].iat),
        ("kernel32.dll", "GetLastError", 0x2080)
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
        (0x460, 0x8000000000000001, 8),
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
