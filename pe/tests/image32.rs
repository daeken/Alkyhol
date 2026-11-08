//! Independent synthetic PE32 layouts; real native execution evidence is separate.
use alkyhol_pe::{Image, ImportSymbol};
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
fn fixture() -> Vec<u8> {
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
#[test]
fn i386_header_width_extents_and_all_truncated_prefixes_are_checked() {
    let b = fixture();
    let image = Image::parse_i386(&b).unwrap();
    assert_eq!(image.bitness(), 32);
    assert_eq!(image.base, 0x400000);
    assert_eq!(image.entry, 0x1000);
    image.check_startup().unwrap();
    assert!(Image::parse(&b).is_err());
    assert!(Image::parse_dll(&b).is_err());
    assert!(image.check_registered_startup().is_err());
    assert!(image.runtime_functions().is_err());
    for end in 0..b.len() {
        assert!(Image::parse_i386(&b[..end]).is_err(), "prefix {end}");
    }
    for (at, value) in [(OPT + 56, 0x2000), (OPT + 28, 0xffff0000), (OPT + 92, 17)] {
        let mut bad = b.clone();
        p32(&mut bad, at, value);
        if at == OPT + 28 {
            p32(&mut bad, OPT + 56, 0x20000);
        }
        assert!(Image::parse_i386(&bad).is_err());
    }
    let mut bad = b.clone();
    p16(&mut bad, 0x84, 0x8664);
    assert!(Image::parse_i386(&bad).is_err());
    let mut bad = b.clone();
    p16(&mut bad, OPT, 0x20b);
    assert!(Image::parse_i386(&bad).is_err());
    for index in [0, 3, 7, 8, 9, 10, 11, 13, 14, 15] {
        let mut bad = b.clone();
        directory(&mut bad, index, 0x2100, 4);
        assert!(Image::parse_i386(&bad).unwrap().check_startup().is_err());
    }
}
#[test]
fn four_byte_import_slots_keep_original_names_ordinals_and_reject_malformed_shapes() {
    let b = fixture();
    let items = Image::parse_i386(&b).unwrap().imports().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].iat, 0x20a0);
    assert_eq!(items[0].symbol, ImportSymbol::Name("GetLastError".into()));
    let mut ordinal = b.clone();
    p32(&mut ordinal, 0x680, 0x80000011);
    assert_eq!(
        Image::parse_i386(&ordinal).unwrap().imports().unwrap()[0].symbol,
        ImportSymbol::Ordinal(17)
    );
    for (at, value) in [
        (0x680, 0x80010011),
        (0x680, 0x40000000),
        (0x610, 0x20a2),
        (0x610, 0x1000),
        (0x604, 1),
        (0x60c, 0),
    ] {
        let mut bad = b.clone();
        p32(&mut bad, at, value);
        assert!(
            Image::parse_i386(&bad).unwrap().imports().is_err(),
            "at {at:x}"
        );
    }
    let mut bad = b.clone();
    directory(&mut bad, 1, 0x2000, 20);
    assert!(Image::parse_i386(&bad).unwrap().imports().is_err());
    let mut bad = b.clone();
    let copy = bad[0x600..0x614].to_vec();
    bad[0x614..0x628].copy_from_slice(&copy);
    directory(&mut bad, 1, 0x2000, 60);
    assert!(Image::parse_i386(&bad).unwrap().imports().is_err());
    let mut fallback = b;
    p32(&mut fallback, 0x600, 0);
    p32(&mut fallback, 0x6a0, 0x20c0);
    assert_eq!(
        Image::parse_i386(&fallback).unwrap().imports().unwrap(),
        items
    );
}
#[test]
fn highlow_fixups_allow_initialized_instruction_operands_but_no_header_bss_or_overlap() {
    let b = fixture();
    assert_eq!(
        Image::parse_i386(&b).unwrap().relocations().unwrap(),
        vec![0x1001]
    );
    let mut data = b.clone();
    p32(&mut data, 0x800, 0x2000);
    p16(&mut data, 0x808, 0x3100);
    assert_eq!(
        Image::parse_i386(&data).unwrap().relocations().unwrap(),
        vec![0x2100]
    );
    for item in [0xa001, 0x31fe, 0x3500] {
        let mut bad = b.clone();
        p16(&mut bad, 0x808, item);
        assert!(Image::parse_i386(&bad).unwrap().relocations().is_err());
    }
    let mut bad = b.clone();
    p32(&mut bad, 0x800, 0);
    assert!(Image::parse_i386(&bad).unwrap().relocations().is_err());
    for second in [0x3001, 0x3002, 0x3004] {
        let mut bad = b.clone();
        p16(&mut bad, 0x80a, second);
        assert!(Image::parse_i386(&bad).unwrap().relocations().is_err());
    }
    let mut adjacent = b;
    p16(&mut adjacent, 0x80a, 0x3005);
    assert_eq!(
        Image::parse_i386(&adjacent).unwrap().relocations().unwrap(),
        vec![0x1001, 0x1005]
    );
}
#[test]
fn partial_final_i386_page_retains_header_size_and_bounds_the_rounded_mapping() {
    let original = fixture();
    for declared in [0x3000, 0x2fff, 0x2400, 0x2001] {
        let mut bytes = original.clone();
        p32(&mut bytes, OPT + 56, declared);
        let before = bytes.clone();
        let image = Image::parse_i386(&bytes).unwrap();
        assert_eq!(image.declared_size(), declared as usize);
        assert_eq!(image.size, 0x3000);
        assert_eq!(image.sections[1].rva + image.sections[1].size, 0x2400);
        image.check_startup().unwrap();
        assert_eq!(bytes, before); // Inspection never rewrites the header.
    }
    for declared in [0, 1, 0x1fff, 0x2000, 0x40000001, u32::MAX] {
        let mut bytes = original.clone();
        p32(&mut bytes, OPT + 56, declared);
        assert!(Image::parse_i386(&bytes).is_err(), "declared {declared:x}");
    }
    // Rounding must not hide a preferred-base overflow beyond 32-bit addresses.
    let mut bytes = original.clone();
    p32(&mut bytes, OPT + 28, 0xffff0000);
    p32(&mut bytes, OPT + 56, 0x10001);
    assert!(Image::parse_i386(&bytes).is_err());
    // Other section alignments retain their previous gate, rather than infer
    // semantics from the finite native 4-KiB geometry observations.
    let mut bytes = original;
    p32(&mut bytes, OPT + 32, 8192);
    p32(&mut bytes, OPT + 56, 0x4001);
    assert!(Image::parse_i386(&bytes).is_err());
}
