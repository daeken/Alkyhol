//! Real stencil aspect/effect evidence on the selected GPU, no game data.
use alkyhol_d3d9::{Device, RenderState};
fn triangle(points: &[(f32, f32)], color: u32) -> Vec<u8> {
    points
        .iter()
        .flat_map(|&(x, y)| {
            [
                x.to_le_bytes(),
                y.to_le_bytes(),
                0.5f32.to_le_bytes(),
                1f32.to_le_bytes(),
                color.to_le_bytes(),
            ]
            .concat()
        })
        .collect()
}
#[test]
#[ignore = "requires an actual GPU"]
fn stencil_ops_masks_failure_paths_and_independent_aspects_execute() {
    let gpu = Device::new().unwrap();
    let mut target = gpu.create_target(16, 16).unwrap();
    let depth = gpu.create_depth_stencil(16, 16).unwrap();
    assert!(depth.has_stencil());
    let d16 = gpu.create_depth16(16, 16).unwrap();
    assert!(!d16.has_stencil());
    assert!(gpu.clear_depth_stencil(&d16, None, Some(0)).is_err());
    assert!(gpu.clear_depth_stencil(&depth, None, None).is_err());
    assert!(
        gpu.clear_depth_stencil(&depth, Some(f32::NAN), Some(0))
            .is_err()
    );
    let mut state = RenderState::default();
    state.set(22, 1).unwrap();
    state.set(52, 1).unwrap();
    let full = triangle(&[(0., 0.), (64., 0.), (0., 64.)], 0xffff0000);
    gpu.clear(&mut target, 0xff0000ff).unwrap();
    assert!(
        gpu.draw_transformed_depth_state(&mut target, &full, &state, Some(&depth))
            .is_err()
    );
    assert!(
        gpu.draw_transformed_depth_state(&mut target, &full, &state, Some(&d16))
            .is_err()
    );
    for (operation, initial, reference, expected) in [
        (1, 254, 39, 254),
        (2, 254, 39, 0),
        (3, 254, 39, 39),
        (4, 42, 39, 43),
        (4, 255, 39, 255),
        (5, 42, 39, 41),
        (5, 0, 39, 0),
        (6, 0x55, 39, 0xaa),
        (7, 255, 39, 0),
        (8, 0, 39, 255),
    ] {
        gpu.clear(&mut target, 0xff0000ff).unwrap();
        gpu.clear_depth_stencil(&depth, None, Some(initial))
            .unwrap();
        for (ty, value) in [
            (7, 0),
            (168, 0),
            (56, 8),
            (55, operation),
            (57, reference),
            (58, 255),
            (59, 255),
            (53, 1),
            (54, 1),
        ] {
            state.set(ty, value).unwrap();
        }
        gpu.draw_transformed_depth_state(&mut target, &full, &state, Some(&depth))
            .unwrap();
        for (ty, value) in [(168, 15), (56, 3), (55, 1), (57, expected)] {
            state.set(ty, value).unwrap();
        }
        gpu.draw_transformed_depth_state(&mut target, &full, &state, Some(&depth))
            .unwrap();
        assert!(
            gpu.readback(&target)
                .unwrap()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [0, 0, 255, 255]),
            "operation={operation} initial={initial}"
        );
    }
    // Replace on stencil-test failure, then independently on depth-test failure.
    for zfail in [false, true] {
        gpu.clear(&mut target, 0xff0000ff).unwrap();
        gpu.clear_depth_stencil(&depth, Some(0.), Some(0)).unwrap();
        for (ty, value) in [
            (7, u32::from(zfail)),
            (23, 2),
            (168, 0),
            (56, if zfail { 8 } else { 1 }),
            (53, 3),
            (54, 3),
            (55, 1),
            (57, 9),
        ] {
            state.set(ty, value).unwrap();
        }
        gpu.draw_transformed_depth_state(&mut target, &full, &state, Some(&depth))
            .unwrap();
        for (ty, value) in [(7, 0), (168, 15), (56, 3), (53, 1)] {
            state.set(ty, value).unwrap();
        }
        gpu.draw_transformed_depth_state(&mut target, &full, &state, Some(&depth))
            .unwrap();
        assert!(
            gpu.readback(&target)
                .unwrap()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [0, 0, 255, 255])
        );
    }
    // A partial write mask preserves the high nibble; comparison uses its own mask.
    gpu.clear(&mut target, 0xff0000ff).unwrap();
    gpu.clear_depth_stencil(&depth, None, Some(0xa0)).unwrap();
    for (ty, value) in [(168, 0), (56, 8), (55, 3), (57, 0x17), (59, 15)] {
        state.set(ty, value).unwrap();
    }
    let mask = triangle(&[(2., 2.), (14., 2.), (2., 14.)], 0xffff0000);
    gpu.draw_transformed_depth_state(&mut target, &mask, &state, Some(&depth))
        .unwrap();
    for (ty, value) in [(168, 15), (56, 3), (55, 1), (57, 0xa7), (59, 255)] {
        state.set(ty, value).unwrap();
    }
    gpu.draw_transformed_depth_state(&mut target, &full, &state, Some(&depth))
        .unwrap();
    let pixels = gpu.readback(&target).unwrap();
    let at = |x: usize, y: usize| &pixels[(y * 16 + x) * 4..(y * 16 + x + 1) * 4];
    assert_eq!(at(4, 4), [0, 0, 255, 255]);
    assert_eq!(at(13, 13), [255, 0, 0, 255]);
    // Both windings in a single uncullled pass use independent stencil faces.
    gpu.clear(&mut target, 0xff0000ff).unwrap();
    gpu.clear_depth_stencil(&depth, None, Some(9)).unwrap();
    for (ty, value) in [
        (185, 1),
        (168, 0),
        (56, 8),
        (55, 3),
        (57, 7),
        (58, 255),
        (59, 255),
        (186, 1),
        (187, 1),
        (188, 2),
        (189, 8),
    ] {
        state.set(ty, value).unwrap();
    }
    let both = [
        triangle(&[(1., 1.), (7., 1.), (1., 7.)], 0xffff0000),
        triangle(&[(9., 1.), (9., 7.), (15., 1.)], 0xffff0000),
    ]
    .concat();
    gpu.draw_transformed_depth_state(&mut target, &both, &state, Some(&depth))
        .unwrap();
    for (ty, value) in [(185, 0), (168, 15), (56, 3), (55, 1)] {
        state.set(ty, value).unwrap();
    }
    gpu.draw_transformed_depth_state(&mut target, &full, &state, Some(&depth))
        .unwrap();
    let pixels = gpu.readback(&target).unwrap();
    assert_eq!(
        &pixels[(2 * 16 + 2) * 4..(2 * 16 + 2) * 4 + 4],
        [0, 0, 255, 255]
    );
    assert_eq!(
        &pixels[(2 * 16 + 10) * 4..(2 * 16 + 10) * 4 + 4],
        [255, 0, 0, 255]
    );
}
