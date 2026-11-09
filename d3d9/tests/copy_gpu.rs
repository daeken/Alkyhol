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
