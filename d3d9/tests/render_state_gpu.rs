//! Real GPU checks of state changes; these do not represent original game draws.
use alkyhol_d3d9::{Device, RenderState};
fn triangle() -> Vec<u8> {
    let mut bytes = Vec::new();
    for (x, y) in [(16f32, 2f32), (30., 30.), (2., 30.)] {
        for f in [x, y, 0.5, 1.] {
            bytes.extend(f.to_le_bytes());
        }
        bytes.extend(0x80ff0000u32.to_le_bytes());
    }
    bytes
}
#[test]
#[ignore = "requires an actual GPU"]
fn real_blend_write_masks_culling_sample_mask_and_srgb_clear() {
    let device = Device::new().unwrap();
    let mut target = device.create_target(32, 32).unwrap();
    let vertices = triangle();
    let mut state = RenderState::default();
    let at = |pixels: Vec<u8>| pixels[(20 * 32 + 16) * 4..(20 * 32 + 16) * 4 + 4].to_vec();
    state.set(27, 1).unwrap();
    state.set(19, 5).unwrap();
    state.set(20, 6).unwrap();
    device.clear(&mut target, 0xff0000ff).unwrap();
    device
        .draw_transformed_state(&mut target, &vertices, &state)
        .unwrap();
    assert_eq!(at(device.readback(&target).unwrap()), [127, 0, 128, 191]);
    state.set(27, 0).unwrap();
    state.set(168, 1).unwrap();
    device.clear(&mut target, 0xff0000ff).unwrap();
    device
        .draw_transformed_state(&mut target, &vertices, &state)
        .unwrap();
    assert_eq!(at(device.readback(&target).unwrap()), [255, 0, 255, 255]);
    state.set(168, 15).unwrap();
    state.set(22, 2).unwrap();
    device.clear(&mut target, 0xff0000ff).unwrap();
    device
        .draw_transformed_state(&mut target, &vertices, &state)
        .unwrap();
    assert_eq!(at(device.readback(&target).unwrap()), [255, 0, 0, 255]);
    state.set(22, 1).unwrap();
    state.set(162, 0).unwrap();
    device
        .draw_transformed_state(&mut target, &vertices, &state)
        .unwrap();
    // With a non-multisampled target, the multisample-only mask has no effect.
    assert_eq!(at(device.readback(&target).unwrap()), [0, 0, 255, 128]);
    state.set(162, u32::MAX).unwrap();
    device
        .draw_transformed_state(&mut target, &vertices, &state)
        .unwrap();
    assert_eq!(at(device.readback(&target).unwrap()), [0, 0, 255, 128]);
    state.set(194, 1).unwrap();
    device.clear_state(&mut target, 0xff808080, &state).unwrap();
    assert_eq!(at(device.readback(&target).unwrap()), [188, 188, 188, 255]);
}
