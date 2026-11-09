//! Finite bias visibility and full-frame coverage on real depth attachments.
use alkyhol_d3d9::{Device, RenderState};
fn triangle(sloped: bool, color: u32) -> Vec<u8> {
    [(16f32, 2f32, 0.25f32), (30., 30., 0.75), (2., 30., 0.75)]
        .into_iter()
        .flat_map(|(x, y, z)| {
            [
                x.to_le_bytes(),
                y.to_le_bytes(),
                (if sloped { z } else { 0.5 }).to_le_bytes(),
                1f32.to_le_bytes(),
                color.to_le_bytes(),
            ]
            .concat()
        })
        .collect()
}
#[test]
#[ignore = "requires an actual GPU"]
fn constant_and_slope_bias_keep_coverage_and_change_d16_d24_occlusion() {
    let gpu = Device::new().unwrap();
    for stencil in [false, true] {
        let depth = if stencil {
            gpu.create_depth_stencil(32, 32)
        } else {
            gpu.create_depth16(32, 32)
        }
        .unwrap();
        for (constant, slope, sloped, visible) in [
            (0.125f32, 0f32, false, false),
            (-0.125, 0., false, true),
            (0., 1., true, false),
            (0., -1., true, true),
        ] {
            let mut target = gpu.create_target(32, 32).unwrap();
            let mut state = RenderState::default();
            state.set(7, 1).unwrap();
            state.set(23, 2).unwrap();
            gpu.clear(&mut target, 0xff0000ff).unwrap();
            gpu.clear_depth(&depth, 1.).unwrap();
            gpu.draw_transformed_depth_state(
                &mut target,
                &triangle(sloped, 0xffff0000),
                &state,
                Some(&depth),
            )
            .unwrap();
            let original = gpu.readback(&target).unwrap();
            state.set(195, constant.to_bits()).unwrap();
            state.set(175, slope.to_bits()).unwrap();
            gpu.draw_transformed_depth_state(
                &mut target,
                &triangle(sloped, 0xff00ff00),
                &state,
                Some(&depth),
            )
            .unwrap();
            let actual = gpu.readback(&target).unwrap();
            for (before, after) in original
                .as_chunks::<4>()
                .0
                .iter()
                .zip(actual.as_chunks::<4>().0)
            {
                assert_eq!(
                    after,
                    if before == &[0, 0, 255, 255] && visible {
                        &[0, 255, 0, 255]
                    } else {
                        before
                    },
                    "stencil={stencil} bias={constant} slope={slope}"
                );
            }
            assert_eq!(
                &actual[(20 * 32 + 16) * 4..(20 * 32 + 16) * 4 + 4],
                if visible {
                    &[0, 255, 0, 255]
                } else {
                    &[0, 0, 255, 255]
                }
            );
        }
    }
}
