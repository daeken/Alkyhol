//! Independent synthetic PE/COFF layouts, not DLL execution/native parity.
use alkyhol_pe::{ExportTarget, Image};
fn p16(b: &mut [u8], i: usize, n: u16) {
    b[i..i + 2].copy_from_slice(&n.to_le_bytes());
}
fn p32(b: &mut [u8], i: usize, n: u32) {
    b[i..i + 4].copy_from_slice(&n.to_le_bytes());
}
const O: usize = 0x98;
fn library() -> Vec<u8> {
    let mut b = vec![0; 0x600];
    b[..2].copy_from_slice(b"MZ");
    p32(&mut b, 0x3c, 0x80);
    b[0x80..0x84].copy_from_slice(b"PE\0\0");
    p16(&mut b, 0x84, 0x8664);
    p16(&mut b, 0x86, 2);
    p16(&mut b, 0x94, 240);
    p16(&mut b, 0x96, 0x2022);
    p16(&mut b, O, 0x20b);
    p32(&mut b, O + 16, 0x1000);
    b[O + 24..O + 32].copy_from_slice(&0x180000000u64.to_le_bytes());
    p32(&mut b, O + 32, 0x1000);
    p32(&mut b, O + 36, 0x200);
    p32(&mut b, O + 56, 0x3000);
    p32(&mut b, O + 60, 0x200);
    p16(&mut b, O + 68, 3);
    p32(&mut b, O + 108, 16);
    p32(&mut b, O + 112, 0x2000);
    p32(&mut b, O + 116, 0x180);
    for (i, rva, raw, rights) in [
        (0, 0x1000, 0x200, 0x60000020),
        (1, 0x2000, 0x400, 0x40000040),
    ] {
        let s = O + 240 + i * 40;
        p32(&mut b, s + 8, 0x400);
        p32(&mut b, s + 12, rva);
        p32(&mut b, s + 16, 0x200);
        p32(&mut b, s + 20, raw);
        p32(&mut b, s + 36, rights);
    }
    b[0x200] = 0xc3;
    p32(&mut b, 0x40c, 0x2050);
    p32(&mut b, 0x410, 17);
    p32(&mut b, 0x414, 5);
    p32(&mut b, 0x418, 4);
    p32(&mut b, 0x41c, 0x2080);
    p32(&mut b, 0x420, 0x20a0);
    p32(&mut b, 0x424, 0x20b0);
    b[0x450..0x45b].copy_from_slice(b"helper.dll\0");
    for (i, address) in [0x1000, 0, 0x2140, 0x2300, 0x1010].into_iter().enumerate() {
        p32(&mut b, 0x480 + i * 4, address);
    }
    for (i, name) in [0x20c0, 0x20d0, 0x20e0, 0x20f0].into_iter().enumerate() {
        p32(&mut b, 0x4a0 + i * 4, name);
    }
    for (i, index) in [0, 0, 2, 3].into_iter().enumerate() {
        p16(&mut b, 0x4b0 + i * 2, index);
    }
    for (off, name) in [
        (0x4c0, &b"Alias\0"[..]),
        (0x4d0, &b"Direct\0"[..]),
        (0x4e0, &b"Forwarded\0"[..]),
        (0x4f0, &b"Global\0"[..]),
        (0x540, &b"KERNEL32.Sleep\0"[..]),
    ] {
        b[off..off + name.len()].copy_from_slice(name);
    }
    b
}
#[test]
fn dll_kind_and_optional_entry_preserve_normal_exe_startup_gates() {
    let mut b = library();
    assert!(Image::parse(&b).is_err());
    let dll = Image::parse_dll(&b).unwrap();
    assert!(dll.check_startup().is_err());
    p32(&mut b, O + 16, 0);
    assert_eq!(Image::parse_dll(&b).unwrap().entry, 0);
    p32(&mut b, O + 16, 0x2000);
    assert!(Image::parse_dll(&b).is_err());
    p16(&mut b, 0x96, 0x22);
    p32(&mut b, O + 16, 0x1000);
    assert!(Image::parse_dll(&b).is_err());
    assert!(Image::parse(&b).unwrap().check_startup().is_err());
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    {
        p16(&mut b, 0x96, 0x2022);
        assert!(alkyhol_pe::LoadedImage::load(&b, false).is_err());
    }
}
#[test]
fn biased_ordinals_aliases_sparse_holes_data_and_forwarders_are_preserved() {
    let b = library();
    let e = Image::parse_dll(&b).unwrap().exports().unwrap().unwrap();
    assert_eq!((e.dll_name.as_str(), e.ordinal_base), ("helper.dll", 17));
    assert_eq!(e.entries.len(), 4);
    assert_eq!(
        e.entries.iter().map(|e| e.ordinal).collect::<Vec<_>>(),
        [17, 19, 20, 21]
    );
    assert_eq!(e.entries[0].names, ["Alias", "Direct"]);
    assert_eq!(e.entries[0].target, ExportTarget::Address { rva: 0x1000 });
    assert_eq!(
        e.entries[1].target,
        ExportTarget::Forwarder("KERNEL32.Sleep".into())
    );
    assert_eq!(e.entries[2].target, ExportTarget::Address { rva: 0x2300 }); // zero-filled data is still mapped.
    assert!(e.entries[3].names.is_empty());
}
#[test]
fn empty_and_ordinal_only_exports_are_distinct_from_absent_directory() {
    let mut b = library();
    p32(&mut b, 0x418, 0);
    p32(&mut b, 0x420, 0);
    p32(&mut b, 0x424, 0);
    assert_eq!(
        Image::parse_dll(&b)
            .unwrap()
            .exports()
            .unwrap()
            .unwrap()
            .entries
            .len(),
        4
    );
    p32(&mut b, 0x414, 0);
    p32(&mut b, 0x41c, 0);
    assert!(
        Image::parse_dll(&b)
            .unwrap()
            .exports()
            .unwrap()
            .unwrap()
            .entries
            .is_empty()
    );
    p32(&mut b, O + 112, 0);
    p32(&mut b, O + 116, 0);
    assert_eq!(Image::parse_dll(&b).unwrap().exports().unwrap(), None);
}
#[test]
fn malformed_tables_counts_order_ordinals_and_targets_fail_honestly() {
    for (off, value, width) in [
        (0x400, 1, 4),
        (0x40c, 0, 4),
        (O + 116, 39, 4),
        (0x410, u32::MAX, 4),
        (0x414, 65537, 4),
        (0x418, 65537, 4),
        (0x41c, 0, 4),
        (0x420, 0, 4),
        (0x424, 0, 4),
        (0x4a0, 0x4fff, 4),
        (0x4a4, 0x20c0, 4),
        (0x4b0, 5, 2),
        (0x4b0, 1, 2),
        (0x480, 0x2fff, 4),
        (0x41c, 0x21ff, 4),
    ] {
        let mut b = library();
        if width == 2 {
            p16(&mut b, off, value as u16);
        } else {
            p32(&mut b, off, value);
        }
        assert!(
            Image::parse_dll(&b).unwrap().exports().is_err(),
            "{off:x}/{value:x}"
        );
    }
    let mut b = library();
    p32(&mut b, 0x4a0, 0x20d0);
    p32(&mut b, 0x4a4, 0x20c0);
    assert!(Image::parse_dll(&b).unwrap().exports().is_err());
}
#[test]
fn forwarder_must_terminate_inside_declared_directory_not_adjacent_bytes() {
    for (target, contents) in [
        (0x217f, &b"X\0"[..]),
        (0x2140, &b"\xff\0"[..]),
        (0x2140, &b"\0"[..]),
    ] {
        let mut b = library();
        p32(&mut b, 0x488, target);
        let off = target as usize - 0x2000 + 0x400;
        b[off..off + contents.len()].copy_from_slice(contents);
        assert!(Image::parse_dll(&b).unwrap().exports().is_err());
    }
    let mut b = library();
    b[0x540..0x550].fill(0);
    b[0x540..0x54c].copy_from_slice(b"MYDLL.#1234\0");
    assert_eq!(
        Image::parse_dll(&b)
            .unwrap()
            .exports()
            .unwrap()
            .unwrap()
            .entries[1]
            .target,
        ExportTarget::Forwarder("MYDLL.#1234".into())
    );
}
#[test]
fn every_truncated_dll_prefix_fails_without_entry_or_export_access() {
    let b = library();
    for n in 0..b.len() {
        assert!(Image::parse_dll(&b[..n]).is_err(), "{n}");
    }
}
