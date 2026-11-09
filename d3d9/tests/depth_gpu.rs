//! Owned D16 occlusion/state evidence on the selected actual GPU, not game proof.
use alkyhol_d3d9::{Device, RenderState};
fn triangle(depth: f32, color: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    for (x, y) in [(16f32, 2f32), (30., 30.), (2., 30.)] {
        for f in [x, y, depth, 1.] {
            bytes.extend(f.to_le_bytes());
        }
        bytes.extend(color.to_le_bytes());
    }
    bytes
}
#[test]
#[ignore = "requires an actual GPU"]
fn actual_d16_clear_occlusion_writes_comparisons_and_attachment_lifetime() {
    let device = Device::new().unwrap();
    let mut target = device.create_target(32, 32).unwrap();
    let original = device.create_depth16(32, 32).unwrap();
    let depth = original.clone();
    drop(original);
    assert_eq!(depth.dimensions(), (32, 32));
    assert!(device.create_depth16(0, 32).is_err());
    let mut state = RenderState::default();
    state.set(7, 1).unwrap();
    device.clear(&mut target, 0xff0000ff).unwrap();
    let red = triangle(0.25, 0xffff0000);
    let green = triangle(0.75, 0xff00ff00);
    assert!(
        device
            .draw_transformed_depth_state(&mut target, &red, &state, None)
            .is_err()
    );
    assert!(
        device
            .draw_transformed_depth_state(&mut target, &red, &state, Some(&depth))
            .is_err()
    );
    let at = |pixels: Vec<u8>| pixels[(20 * 32 + 16) * 4..(20 * 32 + 16) * 4 + 4].to_vec();
    let draw = |target: &mut _, vertices: &[u8], state: &_| {
        device
            .draw_transformed_depth_state(target, vertices, state, Some(&depth))
            .unwrap()
    };
    device.clear_depth(&depth, 1.).unwrap();
    draw(&mut target, &red, &state);
    draw(&mut target, &green, &state);
    assert_eq!(at(device.readback(&target).unwrap()), [0, 0, 255, 255]);
    // Clearing just depth preserves rendered color, then allows the farther draw.
    device.clear_depth(&depth, 1.).unwrap();
    assert_eq!(at(device.readback(&target).unwrap()), [0, 0, 255, 255]);
    draw(&mut target, &green, &state);
    assert_eq!(at(device.readback(&target).unwrap()), [0, 255, 0, 255]);
    device.clear_depth(&depth, 1.).unwrap();
    state.set(14, 0).unwrap();
    draw(&mut target, &red, &state);
    state.set(14, 1).unwrap();
    draw(&mut target, &green, &state);
    assert_eq!(at(device.readback(&target).unwrap()), [0, 255, 0, 255]);
    device.clear_depth(&depth, 0.).unwrap();
    state.set(23, 5).unwrap();
    draw(&mut target, &red, &state);
    assert_eq!(at(device.readback(&target).unwrap()), [0, 0, 255, 255]);
    state.set(23, 1).unwrap();
    draw(&mut target, &green, &state);
    assert_eq!(at(device.readback(&target).unwrap()), [0, 0, 255, 255]);
    state.set(23, 8).unwrap();
    draw(&mut target, &green, &state);
    assert_eq!(at(device.readback(&target).unwrap()), [0, 255, 0, 255]);
    for invalid in [f32::NAN, -0.1, 1.1] {
        assert!(device.clear_depth(&depth, invalid).is_err());
    }
    let other = device.create_depth16(16, 16).unwrap();
    assert!(
        device
            .draw_transformed_depth_state(&mut target, &red, &state, Some(&other))
            .is_err()
    );
    state.set(7, 0).unwrap();
    device
        .draw_transformed_depth_state(&mut target, &red, &state, None)
        .unwrap();
    assert_eq!(at(device.readback(&target).unwrap()), [0, 0, 255, 255]);
}
