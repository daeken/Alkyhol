//! Exhaustive real sampling of owned alpha and 4-bit-channel texture contents.
use alkyhol_d3d9::{
    Device, LegacyDraw, SampleFormat, SamplerState, TextureBinding, compiler::Input, fixed_vertex,
};
#[test]
#[ignore = "requires an actual GPU"]
fn every_alpha_byte_and_argb4444_word_samples_without_channel_loss() {
    let device = Device::new().unwrap();
    let shader = fixed_vertex::vertex_shader(&[Input {
        semantic: "COLOR".into(),
        index: 0,
        register: 0,
    }])
    .unwrap();
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
    for (format, size, data) in [
        (SampleFormat::Alpha8, 16u32, (0..=255u8).collect::<Vec<_>>()),
        (
            SampleFormat::Argb4444,
            256,
            (0..=u16::MAX).flat_map(u16::to_le_bytes).collect(),
        ),
    ] {
        let mut texture = device
            .create_sampled_texture(size, size, 1, format)
            .unwrap();
        device.upload_texture(&mut texture, 0, &data).unwrap();
        let mut target = device.create_target(size, size).unwrap();
        device.clear(&mut target, 0xff0000ff).unwrap();
        let s = size as f32;
        let half = 0.5 / s;
        let vertices: Vec<_> = [
            [0f32, 0., 0.5, 1., half, half, 0., 1.],
            [s, 0., 0.5, 1., 1. + half, half, 0., 1.],
            [0., s, 0.5, 1., half, 1. + half, 0., 1.],
            [s, s, 0.5, 1., 1. + half, 1. + half, 0., 1.],
        ]
        .into_iter()
        .flatten()
        .flat_map(f32::to_le_bytes)
        .collect();
        let constants: Vec<_> = [s, s, 0., 0.]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect();
        let mut sampler = SamplerState::default();
        sampler.set(1, 3).unwrap();
        sampler.set(2, 3).unwrap();
        device
            .draw_legacy(
                &mut target,
                &LegacyDraw {
                    indices: Some(&[0, 1, 2, 1, 3, 2]),
                    vertex_spirv: &shader.spirv,
                    pixel_spirv: include_bytes!("fixtures/legacy-texture.ps.spv"),
                    vertices: &vertices,
                    stride: 32,
                    attributes: &attributes,
                    vertex_constants: &constants,
                    pixel_constants: &[],
                    textures: &[TextureBinding {
                        slot: 0,
                        texture: &texture,
                        sampler,
                    }],
                },
            )
            .unwrap();
        let pixels = device.readback(&target).unwrap();
        for (n, pixel) in pixels.as_chunks::<4>().0.iter().enumerate() {
            let expected = if format == SampleFormat::Alpha8 {
                [0, 0, 0, n as u8]
            } else {
                [
                    (n & 15) as u8 * 17,
                    ((n >> 4) & 15) as u8 * 17,
                    ((n >> 8) & 15) as u8 * 17,
                    ((n >> 12) & 15) as u8 * 17,
                ]
            };
            assert_eq!(*pixel, expected, "format={format:?} pixel={n}");
        }
    }
}
