//! Real GPU NULL sampler semantics, distinct from broken bound resources.
use alkyhol_d3d9::{
    Device, LegacyDraw, SampleFormat, SamplerState, TextureBinding, compiler::Input, fixed_vertex,
};
#[test]
#[ignore = "requires an actual GPU"]
fn null_2d_cube_volume_samples_are_opaque_black_and_real_bindings_still_validate() {
    let gpu = Device::new().unwrap();
    let attributes = [
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 16,
            shader_location: 1,
        },
    ];
    let vertices: Vec<_> = [
        [0f32, 0., 0.5, 1., 0.75, 0.25, 0.75, 1.],
        [8., 0., 0.5, 1., 0.75, 0.25, 0.75, 1.],
        [0., 8., 0.5, 1., 0.75, 0.25, 0.75, 1.],
        [8., 8., 0.5, 1., 0.75, 0.25, 0.75, 1.],
    ]
    .into_iter()
    .flatten()
    .flat_map(f32::to_le_bytes)
    .collect();
    let constants: Vec<_> = [8f32, 8., 0., 0.]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    for (kind, pixel, register) in [
        (
            0,
            include_bytes!("fixtures/legacy-texture.ps.spv").as_slice(),
            0,
        ),
        (
            1,
            include_bytes!("fixtures/legacy-cube.ps.spv").as_slice(),
            8,
        ),
        (
            2,
            include_bytes!("fixtures/legacy-volume.ps.spv").as_slice(),
            8,
        ),
    ] {
        let vertex = fixed_vertex::vertex_shader(&[Input {
            semantic: "COLOR".into(),
            index: 0,
            register,
        }])
        .unwrap();
        let mut target = gpu.create_target(8, 8).unwrap();
        gpu.clear(&mut target, 0x12345678).unwrap();
        let render = |target: &mut _, textures: &[TextureBinding<'_>]| {
            gpu.draw_legacy(
                target,
                &LegacyDraw {
                    indices: Some(&[0, 1, 2, 1, 3, 2]),
                    vertex_spirv: &vertex.spirv,
                    pixel_spirv: pixel,
                    vertices: &vertices,
                    stride: 32,
                    attributes: &attributes,
                    vertex_constants: &constants,
                    pixel_constants: &[],
                    textures,
                },
            )
        };
        render(&mut target, &[]).unwrap();
        assert!(
            gpu.readback(&target)
                .unwrap()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [0, 0, 0, 255])
        );
        let mut texture = match kind {
            0 => gpu.create_texture(1, 1, 1),
            1 => gpu.create_cube_texture(1, 1, SampleFormat::Bgra8),
            _ => gpu.create_volume_texture(1, 1, 1, 1),
        }
        .unwrap();
        let result = render(
            &mut target,
            &[TextureBinding {
                slot: 0,
                texture: &texture,
                sampler: SamplerState::default(),
            }],
        );
        assert!(result.unwrap_err().contains("uninitialized"));
        let color = [32, 64, 128, 64];
        match kind {
            0 => gpu.upload_texture(&mut texture, 0, &color).unwrap(),
            1 => {
                for face in 0..6 {
                    gpu.upload_cube_face(&mut texture, face, 0, &color).unwrap();
                }
            }
            _ => gpu.upload_volume(&mut texture, 0, &color).unwrap(),
        };
        render(
            &mut target,
            &[TextureBinding {
                slot: 0,
                texture: &texture,
                sampler: SamplerState::default(),
            }],
        )
        .unwrap();
        assert!(
            gpu.readback(&target)
                .unwrap()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == color)
        );
        render(&mut target, &[]).unwrap();
        assert!(
            gpu.readback(&target)
                .unwrap()
                .as_chunks::<4>()
                .0
                .iter()
                .all(|p| *p == [0, 0, 0, 255])
        );
    }
}
