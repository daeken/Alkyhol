//! Real fixed/translated pixel rejection; synthetic comparator tests are separate.
use alkyhol_d3d9::{Device, LegacyDraw, RenderState, VertexAttribute, VertexFormat};
const VS: &[u8] = include_bytes!("fixtures/legacy-color.vs.spv");
const PS: &[u8] = include_bytes!("fixtures/legacy-color.ps.spv");
fn passes(function: u32, alpha: f32, reference: u32) -> bool {
    let reference = reference as f32 / 255.;
    match function {
        1 => false,
        2 => alpha < reference,
        3 => alpha == reference,
        4 => alpha <= reference,
        5 => alpha > reference,
        6 => alpha != reference,
        7 => alpha >= reference,
        8 => true,
        _ => unreachable!(),
    }
}
#[test]
#[ignore = "requires an actual GPU"]
fn all_alpha_comparisons_reject_pixels_on_fixed_and_original_shader_paths() {
    let device = Device::new().unwrap();
    for legacy in [false, true] {
        for (byte, reference) in [(0u32, 0u32), (128, 128), (128, 127), (255, 255)] {
            let alpha = if legacy && byte == 128 {
                0.5
            } else {
                byte as f32 / 255.
            };
            let vertices: Vec<_> = if legacy {
                [(0f32, 0.75f32), (0.75, -0.75), (-0.75, -0.75)]
                    .into_iter()
                    .flat_map(|(x, y)| {
                        [x, y, 0.5, 1., 1., 0., 0., alpha]
                            .into_iter()
                            .flat_map(f32::to_le_bytes)
                    })
                    .collect()
            } else {
                [(16f32, 4f32), (28., 28.), (4., 28.)]
                    .into_iter()
                    .flat_map(|(x, y)| {
                        [
                            x.to_le_bytes(),
                            y.to_le_bytes(),
                            0.5f32.to_le_bytes(),
                            1f32.to_le_bytes(),
                            (byte << 24 | 0xff0000).to_le_bytes(),
                        ]
                        .concat()
                    })
                    .collect()
            };
            let constants: Vec<_> = [
                1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
            ]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect();
            let attributes = [
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 16,
                    shader_location: 1,
                },
            ];
            let draw = |target: &mut _, state: &_| {
                if legacy {
                    device.draw_legacy_state(
                        target,
                        &LegacyDraw {
                            indices: None,
                            vertex_spirv: VS,
                            pixel_spirv: PS,
                            vertices: &vertices,
                            stride: 32,
                            attributes: &attributes,
                            vertex_constants: &constants,
                            textures: &[],
                            pixel_constants: &[],
                        },
                        state,
                    )
                } else {
                    device.draw_transformed_depth_state(target, &vertices, state, None)
                }
                .unwrap();
            };
            let mut target = device.create_target(32, 32).unwrap();
            device.clear(&mut target, 0xff0000ff).unwrap();
            draw(&mut target, &RenderState::default());
            let baseline = device.readback(&target).unwrap();
            assert!(
                baseline
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .any(|p| p[2] == 255 && p[0] == 0)
            );
            for function in 1..=8 {
                let mut state = RenderState::default();
                state.set(15, 1).unwrap();
                state.set(24, reference).unwrap();
                state.set(25, function).unwrap();
                device.clear(&mut target, 0xff0000ff).unwrap();
                let query = device.create_occlusion_query();
                device.begin_occlusion_query(&query).unwrap();
                draw(&mut target, &state);
                device.end_occlusion_query(&query).unwrap();
                let actual = device.readback(&target).unwrap();
                let visible = passes(function, alpha, reference);
                for (before, after) in baseline
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .zip(actual.as_chunks::<4>().0)
                {
                    assert_eq!(
                        after,
                        if visible { before } else { &[255, 0, 0, 255] },
                        "legacy={legacy} alpha={alpha} ref={reference} function={function}"
                    );
                }
                let count = (0..10000)
                    .find_map(|_| device.occlusion_data(&query, true).unwrap())
                    .expect("GPU query completes");
                let covered = baseline
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|p| p[2] == 255 && p[0] == 0)
                    .count() as u32;
                assert_eq!(count, if visible { covered } else { 0 });
            }
        }
    }
}
