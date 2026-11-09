//! Independently transformed clip positions versus the fixed XYZ GPU stage.
use alkyhol_d3d9::{Device, LegacyDraw, RenderState, VertexAttribute, VertexFormat, fixed_xyz};
const VS: &[u8] = include_bytes!("fixtures/legacy-color.vs.spv");
const PS: &[u8] = include_bytes!("fixtures/legacy-color.ps.spv");
fn transform(v: [f32; 4], m: &[f32; 16]) -> [f32; 4] {
    std::array::from_fn(|col| {
        (0..4)
            .map(|row| v[row] as f64 * m[row * 4 + col] as f64)
            .sum::<f64>() as f32
    })
}
#[test]
#[ignore = "requires an actual GPU"]
fn xyz_world_view_projection_matches_independent_clip_geometry_and_depth_queries() {
    let gpu = Device::new().unwrap();
    let (vs, ps) = fixed_xyz::shaders().unwrap();
    let world = [
        0.5f32, 0., 0., 0., 0., 0.75, 0., 0., 0., 0., 1., 0., 0.25, 0., 0., 1.,
    ];
    let view = [
        0f32, 1., 0., 0., -1., 0., 0., 0., 0., 0., 1., 0., 0.125, -0.125, 0., 1.,
    ];
    let projection = [
        1f32, 0., 0., 0.25, 0., 0.75, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ];
    let identity = [
        1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ];
    let points = [
        [0f32, 0.75, 0.5, 1.],
        [0.75, -0.75, 0.5, 1.],
        [-0.75, -0.75, 0.5, 1.],
    ];
    let vertices: Vec<_> = points
        .into_iter()
        .flatten()
        .flat_map(f32::to_le_bytes)
        .collect();
    let expected: Vec<_> = points
        .into_iter()
        .flat_map(|v| {
            transform(transform(transform(v, &world), &view), &projection)
                .into_iter()
                .chain([1f32, 0., 0., 1.])
                .flat_map(f32::to_le_bytes)
        })
        .collect();
    let matrices: Vec<_> = [world, view, projection]
        .into_iter()
        .flatten()
        .flat_map(f32::to_le_bytes)
        .collect();
    let constants: Vec<_> = identity.into_iter().flat_map(f32::to_le_bytes).collect();
    let position = [VertexAttribute {
        format: VertexFormat::Float32x4,
        offset: 0,
        shader_location: 0,
    }];
    let attrs = [
        position[0],
        VertexAttribute {
            format: VertexFormat::Float32x4,
            offset: 16,
            shader_location: 1,
        },
    ];
    let mut target = gpu.create_target(32, 32).unwrap();
    let depth = gpu.create_depth16(32, 32).unwrap();
    let mut state = RenderState::default();
    state.set(22, 1).unwrap();
    state.set(7, 1).unwrap();
    state.set(14, 0).unwrap();
    gpu.clear(&mut target, 0xff0000ff).unwrap();
    gpu.clear_depth(&depth, 1.).unwrap();
    gpu.draw_legacy_depth_state(
        &mut target,
        &LegacyDraw {
            indices: None,
            vertex_spirv: VS,
            pixel_spirv: PS,
            vertices: &expected,
            stride: 32,
            attributes: &attrs,
            vertex_constants: &constants,
            textures: &[],
            pixel_constants: &[],
        },
        &state,
        Some(&depth),
    )
    .unwrap();
    let reference = gpu.readback(&target).unwrap();
    let visible = reference
        .as_chunks::<4>()
        .0
        .iter()
        .filter(|&&p| p == [0, 0, 255, 255])
        .count() as u32;
    assert!(visible > 0);
    state.set(168, 0).unwrap();
    for (z, count) in [(1f32, visible), (0., 0)] {
        gpu.clear(&mut target, 0xff0000ff).unwrap();
        gpu.clear_depth(&depth, z).unwrap();
        let query = gpu.create_occlusion_query();
        gpu.begin_occlusion_query(&query).unwrap();
        gpu.draw_legacy_depth_state(
            &mut target,
            &LegacyDraw {
                indices: Some(&[0, 1, 2]),
                vertex_spirv: &vs.spirv,
                pixel_spirv: &ps.spirv,
                vertices: &vertices,
                stride: 16,
                attributes: &position,
                vertex_constants: &matrices,
                textures: &[],
                pixel_constants: &[],
            },
            &state,
            Some(&depth),
        )
        .unwrap();
        gpu.end_occlusion_query(&query).unwrap();
        let unchanged = gpu.readback(&target).unwrap();
        assert!(
            unchanged
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| p == &[255, 0, 0, 255])
        );
        let actual = (0..10000)
            .find_map(|_| gpu.occlusion_data(&query, true).unwrap())
            .expect("real GPU query completed");
        assert_eq!(actual, count);
    }
}
