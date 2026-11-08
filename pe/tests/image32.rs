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
    assert_eq!(image.static_tls32().unwrap(), None);
    assert_eq!(image.legacy_load_config32().unwrap(), None);
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
fn explicit_i386_dll_parser_keeps_machine_kind_and_entry_contracts_separate() {
    let mut bytes = fixture();
    assert!(Image::parse_i386_dll(&bytes).is_err());
    p16(&mut bytes, 0x96, 0x2102);
    let dll = Image::parse_i386_dll(&bytes).unwrap();
    assert_eq!(dll.bitness(), 32);
    assert_eq!(dll.entry, 0x1000);
    assert!(dll.check_startup().is_err());
    assert!(Image::parse_i386(&bytes).is_err());
    assert!(Image::parse_dll(&bytes).is_err());
    p32(&mut bytes, OPT + 16, 0);
    assert_eq!(Image::parse_i386_dll(&bytes).unwrap().entry, 0);
    for entry in [0x2000, 0x1200, 0x3000] {
        p32(&mut bytes, OPT + 16, entry);
        assert!(Image::parse_i386_dll(&bytes).is_err());
    }
    for (machine, magic) in [(0x8664, 0x10b), (0x14c, 0x20b)] {
        p32(&mut bytes, OPT + 16, 0);
        p16(&mut bytes, 0x84, machine);
        p16(&mut bytes, OPT, magic);
        assert!(Image::parse_i386_dll(&bytes).is_err());
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
#[test]
fn certificate_pairs_are_untrusted_nonmapped_metadata_and_integrity_requests_still_fail() {
    let original = fixture();
    assert_eq!(
        Image::parse_i386(&original)
            .unwrap()
            .certificate_directory(),
        None
    );
    for (offset, len) in [
        (0x800, 8),
        (0x2000, 16),
        (8, 16),
        (0x9f9, 7),
        (0xfffffff8, 16),
    ] {
        let mut bytes = original.clone();
        directory(&mut bytes, 4, offset, len);
        let before = bytes.clone();
        let image = Image::parse_i386(&bytes).unwrap();
        assert_eq!(
            image.certificate_directory(),
            Some((offset as usize, len as usize))
        );
        assert_eq!(image.size, 0x3000);
        assert_eq!(image.sections.len(), 2);
        image.check_startup().unwrap();
        assert_eq!(bytes, before);
        p16(&mut bytes, OPT + 70, 0x80);
        assert!(
            Image::parse_i386(&bytes)
                .unwrap_err()
                .contains("Code Integrity")
        );
    }
    for (offset, len) in [(0, 8), (8, 0)] {
        let mut bytes = original.clone();
        directory(&mut bytes, 4, offset, len);
        assert!(Image::parse_i386(&bytes).is_err());
    }
    let mut bytes = original.clone();
    p16(&mut bytes, OPT + 70, 0x80);
    assert!(Image::parse_i386(&bytes).is_err());
    for dir in [1, 2, 3, 5, 6, 9, 10, 13] {
        let mut bytes = original.clone();
        directory(&mut bytes, dir, 0xfffffff8, 16);
        assert!(Image::parse_i386(&bytes).is_err(), "directory {dir}");
    }
    let mut bytes = original;
    let section = OPT + 224 + 40;
    p32(&mut bytes, section + 16, 0x800);
    assert!(Image::parse_i386(&bytes).is_err());
}

fn self_exports() -> Vec<u8> {
    let mut b = fixture();
    p32(&mut b, OPT + 224 + 40 + 36, 0x40000040);
    directory(&mut b, 0, 0x2300, 0x100);
    for (at, value) in [
        (0x90c, 0x2370),
        (0x910, 17),
        (0x914, 2),
        (0x918, 2),
        (0x91c, 0x2340),
        (0x920, 0x2350),
        (0x924, 0x2360),
        (0x940, 0x1000),
        (0x944, 0x2100),
        (0x950, 0x2380),
        (0x954, 0x2390),
    ] {
        p32(&mut b, at, value);
    }
    p16(&mut b, 0x960, 1);
    p16(&mut b, 0x962, 0);
    b[0x970..0x97a].copy_from_slice(b"owned.exe\0");
    b[0x980..0x985].copy_from_slice(b"Data\0");
    b[0x990..0x99a].copy_from_slice(b"Transform\0");
    b
}

#[test]
fn self_exports_allow_immutable_metadata_code_and_data_without_weakening_startup_gates() {
    let b = self_exports();
    let image = Image::parse_i386(&b).unwrap();
    image.check_startup().unwrap();
    let exports = image.exports().unwrap().unwrap();
    assert_eq!(exports.ordinal_base, 17);
    assert_eq!(exports.entries[0].names, ["Transform"]);
    assert_eq!(exports.entries[1].names, ["Data"]);
    assert_eq!(
        exports.entries[1].target,
        alkyhol_pe::ExportTarget::Address { rva: 0x2100 }
    );
    let mut bad = b.clone();
    p32(&mut bad, OPT + 224 + 40 + 36, 0xc0000040);
    assert!(Image::parse_i386(&bad).unwrap().exports().is_ok());
    assert!(Image::parse_i386(&bad).unwrap().check_startup().is_err());
    for (at, value) in [
        (0x610, 0x2340),
        (0x914, 3),
        (0x950, 0x1010),
        (0x920, 0x21ff),
    ] {
        let mut bad = b.clone();
        p32(&mut bad, at, value);
        bad[0x410..0x415].copy_from_slice(b"Data\0");
        assert!(
            Image::parse_i386(&bad).unwrap().check_startup().is_err(),
            "{at:x}"
        );
    }
    let mut bad = b.clone();
    p32(&mut bad, 0x800, 0x2000);
    p16(&mut bad, 0x808, 0x3340);
    assert!(Image::parse_i386(&bad).unwrap().check_startup().is_err());
    let mut bad = b.clone();
    p32(&mut bad, 0x940, 0x23b0);
    bad[0x9b0..0x9bc].copy_from_slice(b"kernel32.X\0\0");
    assert!(Image::parse_i386(&bad).unwrap().exports().is_ok());
    assert!(Image::parse_i386(&bad).unwrap().check_startup().is_err());
    for index in [3, 7, 8, 9, 10, 11, 13, 14, 15] {
        let mut bad = b.clone();
        directory(&mut bad, index, 0x2100, 4);
        assert!(Image::parse_i386(&bad).unwrap().check_startup().is_err());
    }
}

fn tls_image() -> Vec<u8> {
    let mut b = fixture();
    b.resize(0xc00, 0);
    p16(&mut b, 0x86, 3);
    p32(&mut b, OPT + 56, 0x4000);
    let s = OPT + 224 + 80;
    for (offset, value) in [
        (8, 0x200),
        (12, 0x3000),
        (16, 0x200),
        (20, 0xa00),
        (36, 0x40000040),
    ] {
        p32(&mut b, s + offset, value);
    }
    directory(&mut b, 9, 0x3100, 24);
    for (at, value) in [
        (0xb00, 0x402100),
        (0xb04, 0x40210c),
        (0xb08, 0x402300),
        (0xb0c, 0x403180),
        (0xb10, 16),
        (0xb14, 0x00500000),
        (0x700, 0x31415926),
        (0x708, 0x402310),
    ] {
        p32(&mut b, at, value);
    }
    directory(&mut b, 5, 0x2200, 40);
    p32(&mut b, 0x80c, 0x3000);
    p32(&mut b, 0x810, 16);
    for (at, value) in [
        (0x814, 0x3100),
        (0x816, 0x3104),
        (0x818, 0x3108),
        (0x81a, 0x310c),
    ] {
        p16(&mut b, at, value);
    }
    p32(&mut b, 0x81c, 0x2000);
    p32(&mut b, 0x820, 12);
    p16(&mut b, 0x824, 0x3108);
    b
}

#[test]
fn static_tls_metadata_retains_template_fixups_zero_fill_alignment_and_writable_index() {
    let b = tls_image();
    let image = Image::parse_i386(&b).unwrap();
    let tls = image.static_tls32().unwrap().unwrap();
    assert_eq!(
        tls,
        alkyhol_pe::StaticTls32 {
            template_rva: 0x2100,
            template_size: 12,
            zero_fill: 16,
            total_size: 28,
            index_rva: 0x2300,
            alignment: 16,
            callbacks: vec![]
        }
    );
    assert!(
        image
            .check_startup()
            .unwrap_err()
            .contains("SizeOfZeroFill")
    );
    let mut basic = b.clone();
    p32(&mut basic, 0xb10, 0);
    Image::parse_i386(&basic).unwrap().check_startup().unwrap();
    assert_eq!(image.bytes, b);
    for (flags, alignment) in [(0, 16), (0x00100000, 1), (0x00e00000, 8192)] {
        let mut b = b.clone();
        p32(&mut b, 0xb14, flags);
        assert_eq!(
            Image::parse_i386(&b)
                .unwrap()
                .static_tls32()
                .unwrap()
                .unwrap()
                .alignment,
            alignment
        );
    }
    let mut b = b.clone();
    p32(&mut b, 0xb0c, 0);
    p16(&mut b, 0x81a, 0);
    assert!(
        Image::parse_i386(&b)
            .unwrap()
            .static_tls32()
            .unwrap()
            .unwrap()
            .callbacks
            .is_empty()
    );
    p32(&mut b, 0xb10, 0);
    p32(&mut b, 0xb08, 0x402600);
    p32(&mut b, OPT + 224 + 40 + 8, 0x800); // Index in declared zero-filled data.
    Image::parse_i386(&b).unwrap().check_startup().unwrap();
}

#[test]
fn malformed_tls_shapes_flags_extents_permissions_and_loader_write_aliases_fail() {
    let original = tls_image();
    for (at, value) in [
        (OPT + 100 + 9 * 8, 20),
        (OPT + 100 + 9 * 8, 25),
        (0xb00, 0x3fffff),
        (0xb04, 0x4020ff),
        (0xb04, 0x402500),
        (0xb08, 0),
        (0xb08, 0x402301),
        (0xb08, 0x403000),
        (0xb08, 0x401000),
        (0xb08, 0x402100),
        (0xb08, 0x4020a0),
        (0xb0c, 0x403181),
        (0xb10, u32::MAX),
        (0xb10, 1024 * 1024),
        (0xb14, 1),
        (0xb14, 0x00f00000),
        (OPT + 224 + 80 + 36, 0xc0000040),
        (0x610, 0x3100),
    ] {
        let mut b = original.clone();
        p32(&mut b, at, value);
        assert!(
            Image::parse_i386(&b).unwrap().static_tls32().is_err(),
            "{at:x}/{value:x}"
        );
    }
    for at in [0x814, 0x816, 0x818, 0x81a] {
        let mut b = original.clone();
        p16(&mut b, at, 0);
        assert!(Image::parse_i386(&b).unwrap().static_tls32().is_err());
    }
    let mut b = original.clone();
    p16(&mut b, 0x824, 0x3300); // Index loader-write overlap.
    assert!(Image::parse_i386(&b).unwrap().static_tls32().is_err());
    let mut b = original;
    p32(&mut b, 0xb04, 0x402100);
    p32(&mut b, 0xb10, 0);
    assert!(Image::parse_i386(&b).unwrap().static_tls32().is_err());
}

#[test]
fn callbacks_are_checked_metadata_and_never_silently_skipped_at_startup() {
    let mut b = tls_image();
    p32(&mut b, 0xb80, 0x401000);
    directory(&mut b, 5, 0x2200, 44);
    p32(&mut b, 0x810, 20);
    p16(&mut b, 0x81c, 0x3180);
    p16(&mut b, 0x81e, 0);
    p32(&mut b, 0x820, 0x2000);
    p32(&mut b, 0x824, 12);
    p16(&mut b, 0x828, 0x3108);
    let image = Image::parse_i386(&b).unwrap();
    assert_eq!(image.static_tls32().unwrap().unwrap().callbacks, [0x1000]);
    assert!(image.check_startup().unwrap_err().contains("callbacks"));
    for value in [0x402100, 0x403000, 0x500000] {
        let mut bad = b.clone();
        p32(&mut bad, 0xb80, value);
        assert!(Image::parse_i386(&bad).unwrap().static_tls32().is_err());
    }
    p16(&mut b, 0x81c, 0);
    assert!(Image::parse_i386(&b).unwrap().static_tls32().is_err());
}

fn load_config_image() -> Vec<u8> {
    let mut b = tls_image();
    directory(&mut b, 9, 0, 0);
    directory(&mut b, 10, 0x3000, 72);
    b[0xa00..0xa48].fill(0);
    p32(&mut b, 0xa00, 72);
    p32(&mut b, 0xa3c, 0x402300);
    p32(&mut b, 0x900, 0xbb40e64e);
    for at in [0x814, 0x816, 0x818, 0x81a] {
        p16(&mut b, at, 0);
    }
    p16(&mut b, 0x814, 0x303c);
    b
}

#[test]
fn legacy_load_config_retains_both_lengths_metadata_and_writable_cookie_rva() {
    let original = load_config_image();
    for outer in [64, 72] {
        for inner in [64, 72] {
            let mut b = original.clone();
            directory(&mut b, 10, 0x3000, outer);
            p32(&mut b, 0xa00, inner);
            p32(&mut b, 0xa04, 0x12345678);
            p16(&mut b, 0xa08, 1);
            p16(&mut b, 0xa0a, 2);
            let before = b.clone();
            let image = Image::parse_i386(&b).unwrap();
            let config = image.legacy_load_config32().unwrap().unwrap();
            assert_eq!(
                config,
                alkyhol_pe::LegacyLoadConfig32 {
                    directory_size: outer as usize,
                    declared_size: inner as usize,
                    timestamp: 0x12345678,
                    major_version: 1,
                    minor_version: 2,
                    cookie_rva: Some(0x2300),
                    seh_table_rva: None,
                    seh_handlers: vec![]
                }
            );
            assert_eq!(image.bytes, before);
            image.check_startup().unwrap();
        }
    }
    let mut b = original.clone();
    p32(&mut b, 0xa3c, 0);
    p16(&mut b, 0x814, 0);
    assert_eq!(
        Image::parse_i386(&b)
            .unwrap()
            .legacy_load_config32()
            .unwrap()
            .unwrap()
            .cookie_rva,
        None
    );
    let mut b = original;
    p32(&mut b, 0xa3c, 0x402600);
    p32(&mut b, OPT + 224 + 40 + 8, 0x800);
    assert_eq!(
        Image::parse_i386(&b)
            .unwrap()
            .legacy_load_config32()
            .unwrap()
            .unwrap()
            .cookie_rva,
        Some(0x2600)
    );
}

#[test]
fn load_config_rejects_unknown_shapes_active_fields_permissions_and_loader_aliases() {
    let original = load_config_image();
    for (at, value) in [
        (OPT + 100 + 10 * 8, 60),
        (OPT + 100 + 10 * 8, 76),
        (0xa00, 60),
        (0xa00, 76),
        (0xa3c, 0x3fffff),
        (0xa3c, 0x402301),
        (0xa3c, 0x404000),
        (0xa3c, 0x401000),
        (0xa3c, 0x403000),
        (0xa3c, 0x4020a0),
        (OPT + 224 + 80 + 36, 0xc0000040),
        (0x610, 0x3000),
    ] {
        let mut b = original.clone();
        p32(&mut b, at, value);
        assert!(
            Image::parse_i386(&b)
                .unwrap()
                .legacy_load_config32()
                .is_err(),
            "{at:x}/{value:x}"
        );
    }
    for at in (0xa0c..0xa3c).step_by(4) {
        let mut b = original.clone();
        p32(&mut b, at, 1);
        assert!(
            Image::parse_i386(&b)
                .unwrap()
                .legacy_load_config32()
                .is_err()
        );
    }
    for (at, value) in [(0x814, 0), (0x816, 0x3004), (0x824, 0x3300)] {
        let mut b = original.clone();
        p16(&mut b, at, value);
        assert!(
            Image::parse_i386(&b)
                .unwrap()
                .legacy_load_config32()
                .is_err()
        );
    }
}

#[test]
fn safeseh_suffix_is_checked_beyond_a_short_directory_without_claiming_dispatch() {
    let mut b = load_config_image();
    directory(&mut b, 10, 0x3000, 64);
    p32(&mut b, 0xa40, 0x403180);
    p32(&mut b, 0xa44, 2);
    p16(&mut b, 0x816, 0x3040);
    p32(&mut b, 0xb80, 0x1000);
    p32(&mut b, 0xb84, 0x1010);
    let config = Image::parse_i386(&b)
        .unwrap()
        .legacy_load_config32()
        .unwrap()
        .unwrap();
    assert_eq!(config.seh_table_rva, Some(0x3180));
    assert_eq!(config.seh_handlers, [0x1000, 0x1010]);
    assert!(
        Image::parse_i386(&b)
            .unwrap()
            .check_startup()
            .unwrap_err()
            .contains("SafeSEH")
    );
    for (at, value) in [
        (0xa40, 0),
        (0xa44, 0),
        (0xa44, 65537),
        (0xa40, 0x403181),
        (0xa40, 0x402100),
        (0xb84, 0x1000),
        (0xb84, 0),
        (0xb84, 0x1200),
        (0xb84, 0x5000),
        (0x610, 0x3180),
    ] {
        let mut bad = b.clone();
        p32(&mut bad, at, value);
        assert!(
            Image::parse_i386(&bad)
                .unwrap()
                .legacy_load_config32()
                .is_err(),
            "{at:x}"
        );
    }
    p16(&mut b, 0x818, 0x3180);
    assert!(
        Image::parse_i386(&b)
            .unwrap()
            .legacy_load_config32()
            .is_err()
    );
}

#[test]
fn cookie_cannot_alias_tls_template_or_index_loader_storage() {
    let mut b = tls_image();
    directory(&mut b, 10, 0x3000, 72);
    b[0xa00..0xa48].fill(0);
    p32(&mut b, 0xa00, 72);
    p32(&mut b, 0xa3c, 0x402300);
    directory(&mut b, 5, 0x2200, 44);
    p32(&mut b, 0x810, 20);
    p16(&mut b, 0x81c, 0x303c);
    p16(&mut b, 0x81e, 0);
    p32(&mut b, 0x820, 0x2000);
    p32(&mut b, 0x824, 12);
    p16(&mut b, 0x828, 0x3108);
    Image::parse_i386(&b).unwrap().static_tls32().unwrap();
    assert!(
        Image::parse_i386(&b)
            .unwrap()
            .legacy_load_config32()
            .is_err()
    );
    p32(&mut b, 0xa3c, 0x402100);
    assert!(
        Image::parse_i386(&b)
            .unwrap()
            .legacy_load_config32()
            .is_err()
    );
}
