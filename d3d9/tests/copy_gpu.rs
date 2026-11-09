//! Actual attachment copy/alpha evidence; no original game pixels.
use alkyhol_d3d9::Device;
#[test]
#[ignore = "requires an actual GPU"]
fn attachment_copy_preserves_all_pixels_alpha_and_initialization_lifetimes() {
    let device = Device::new().unwrap();
    let mut source = device.create_target(7, 3).unwrap();
    let mut destination = device.create_target(7, 3).unwrap();
    device.clear(&mut destination, 0x12345678).unwrap();
    let prior = device.readback(&destination).unwrap();
    assert!(
        device
            .copy_target(&source, &mut destination, false)
            .is_err()
    );
    assert_eq!(device.readback(&destination).unwrap(), prior);
    device.clear(&mut source, 0x80332211).unwrap();
    let original = device.readback(&source).unwrap();
    device
        .copy_target(&source, &mut destination, false)
        .unwrap();
    assert_eq!(device.readback(&destination).unwrap(), original);
    let (texture, mut sampled) = device.create_render_texture(7, 3).unwrap();
    device.copy_target(&source, &mut sampled, true).unwrap();
    drop(texture);
    let copied = device.readback(&sampled).unwrap();
    assert!(
        copied
            .as_chunks::<4>()
            .0
            .iter()
            .all(|p| *p == [0x11, 0x22, 0x33, 255])
    );
    assert_eq!(device.readback(&source).unwrap(), original);
    let mut different = device.create_target(3, 7).unwrap();
    assert!(device.copy_target(&source, &mut different, false).is_err());
    assert!(device.readback(&different).is_err());
    let mut alias = source.clone();
    assert!(device.copy_target(&source, &mut alias, false).is_err());
    assert_eq!(device.readback(&source).unwrap(), original);
}

#[test]
#[ignore = "requires an actual GPU"]
fn partial_fill_and_pitched_upload_preserve_neighbors_and_reject_invalid_storage() {
    let gpu = Device::new().unwrap();
    let mut target = gpu.create_target(7, 5).unwrap();
    assert!(
        gpu.fill_target_region(&mut target, [1, 1, 3, 3], 0)
            .is_err()
    );
    gpu.fill_target_region(&mut target, [0, 0, 7, 5], 0x80203040)
        .unwrap();
    gpu.fill_target_region(&mut target, [2, 1, 6, 4], 0x507090b0)
        .unwrap();
    let bytes = [
        1, 2, 3, 4, 5, 6, 7, 8, 99, 99, 99, 99, 9, 10, 11, 12, 13, 14, 15, 16,
    ];
    gpu.write_target_region(&mut target, [3, 2, 5, 4], &bytes, 12)
        .unwrap();
    let result = gpu.readback(&target).unwrap();
    for y in 0..5 {
        for x in 0..7 {
            let expected = if (3..5).contains(&x) && (2..4).contains(&y) {
                let offset = (y - 2) * 12 + (x - 3) * 4;
                bytes[offset..offset + 4].to_vec()
            } else if (2..6).contains(&x) && (1..4).contains(&y) {
                vec![0xb0, 0x90, 0x70, 0x50]
            } else {
                vec![0x40, 0x30, 0x20, 0x80]
            };
            assert_eq!(&result[(y * 7 + x) * 4..(y * 7 + x + 1) * 4], expected);
        }
    }
    for (rect, pitch, length) in [
        ([3, 2, 5, 4], 7, 20),
        ([3, 2, 5, 4], 12, 19),
        ([0, 0, 8, 5], 32, 160),
        ([0, 0, 1, 3], u32::MAX, 20),
        ([2, 1, 2, 3], 4, 20),
    ] {
        assert!(
            gpu.write_target_region(&mut target, rect, &vec![0; length], pitch)
                .is_err()
        );
        assert_eq!(gpu.readback(&target).unwrap(), result);
    }
}
