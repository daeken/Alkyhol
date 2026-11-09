//! Owned legacy sampler3D bytecode and actual three-axis/mip filtering.
use alkyhol_d3d9::{
    Device, LegacyDraw, RenderState, SamplerState, TextureBinding, VertexAttribute, VertexFormat,
};
const VS: &[u8] = include_bytes!("fixtures/legacy-volume-linked.vs.spv");
const PS: &[u8] = include_bytes!("fixtures/legacy-volume.ps.spv");
#[test]
#[ignore = "requires an actual GPU"]
fn actual_volume_slices_linear_z_and_mip_clamps_sample_distinct_storage() {
    let gpu = Device::new().unwrap();
    for dims in [
        (0, 2, 2, 1),
        (2, 0, 2, 1),
        (2, 2, 0, 1),
        (2, 2, 2, 3),
        (512, 512, 512, 1),
    ] {
        assert!(
            gpu.create_volume_texture(dims.0, dims.1, dims.2, dims.3)
                .is_err()
        );
    }
    let mut volume = gpu.create_volume_texture(2, 2, 2, 0).unwrap();
    assert!(volume.is_volume());
    assert!(!volume.is_cube());
    assert_eq!(volume.volume_dimensions(1).unwrap(), (1, 1, 1));
    assert!(volume.volume_dimensions(2).is_err());
    assert!(gpu.upload_volume(&mut volume, 0, &[0; 31]).is_err());
    assert!(gpu.upload_texture(&mut volume, 0, &[0; 32]).is_err());
    let mut bytes = Vec::new();
    for color in [0xffff0000u32, 0xff0000ff] {
        for _ in 0..4 {
            bytes.extend(color.to_le_bytes());
        }
    }
    gpu.upload_volume(&mut volume, 0, &bytes).unwrap();
    let identity: Vec<_> = [
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
    let mut target = gpu.create_target(32, 32).unwrap();
    for (index, (z, linear, mip, expected)) in [
        (0.25f32, false, 0, [0, 0, 255, 255]),
        (0.75, false, 0, [255, 0, 0, 255]),
        (0.5, true, 0, [128, 0, 128, 255]),
        (0.5, false, 1, [0, 255, 0, 255]),
    ]
    .into_iter()
    .enumerate()
    {
        let vertices: Vec<_> = [(0f32, 0.75f32), (0.75, -0.75), (-0.75, -0.75)]
            .into_iter()
            .flat_map(|(x, y)| {
                [x, y, 0.5, 1., 0.5, 0.5, z, 1.]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
            })
            .collect();
        let mut sampler = SamplerState::default();
        for key in [1, 2, 3] {
            sampler.set(key, 3).unwrap();
        }
        sampler.set(5, if linear { 2 } else { 1 }).unwrap();
        sampler.set(6, if linear { 2 } else { 1 }).unwrap();
        sampler.set(9, mip).unwrap();
        let render = |target: &mut _| {
            gpu.draw_legacy_state(
                target,
                &LegacyDraw {
                    indices: None,
                    vertex_spirv: VS,
                    pixel_spirv: PS,
                    vertices: &vertices,
                    stride: 32,
                    attributes: &attributes,
                    vertex_constants: &identity,
                    textures: &[TextureBinding {
                        slot: 0,
                        texture: &volume,
                        sampler,
                    }],
                    pixel_constants: &[],
                },
                &RenderState::default(),
            )
        };
        gpu.clear(&mut target, 0xff00ffff).unwrap();
        if index == 0 {
            assert!(render(&mut target).is_err());
            gpu.upload_volume(&mut volume, 1, &0xff00ff00u32.to_le_bytes())
                .unwrap();
        }
        let render = |target: &mut _| {
            gpu.draw_legacy_state(
                target,
                &LegacyDraw {
                    indices: None,
                    vertex_spirv: VS,
                    pixel_spirv: PS,
                    vertices: &vertices,
                    stride: 32,
                    attributes: &attributes,
                    vertex_constants: &identity,
                    textures: &[TextureBinding {
                        slot: 0,
                        texture: &volume,
                        sampler,
                    }],
                    pixel_constants: &[],
                },
                &RenderState::default(),
            )
        };
        render(&mut target).unwrap();
        let pixels = gpu.readback(&target).unwrap();
        let mut covered = 0;
        for p in pixels.as_chunks::<4>().0 {
            if p == &[255, 255, 0, 255] {
                continue;
            }
            covered += 1;
            for (a, b) in p.iter().zip(expected) {
                assert!(
                    a.abs_diff(b) <= u8::from(linear),
                    "sample={index} pixel={p:?} expected={expected:?}"
                );
            }
        }
        assert!(covered > 200);
        assert_eq!(
            &pixels[(20 * 32 + 22) * 4..(20 * 32 + 22) * 4 + 4],
            &expected
        );
    }
}
#[test]
#[ignore = "requires an actual GPU"]
fn linear_luminance_volumes_keep_raw_values_and_expand_only_for_gpu_sampling() {
    use alkyhol_d3d9::SampleFormat;
    let gpu = Device::new().unwrap();
    assert!(
        gpu.create_sampled_volume(2, 2, 2, 1, SampleFormat::Bc1)
            .is_err()
    );
    let mut texture = gpu
        .create_sampled_volume(2, 2, 2, 1, SampleFormat::Luminance8)
        .unwrap();
    assert_eq!(texture.byte_layout(0).unwrap(), (2, 8));
    gpu.upload_volume(&mut texture, 0, &[64, 64, 64, 64, 192, 192, 192, 192])
        .unwrap();
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
    let mut target = gpu.create_target(32, 32).unwrap();
    for (z, linear, value) in [(0.25f32, false, 64u8), (0.75, false, 192), (0.5, true, 128)] {
        let vertices: Vec<_> = [(0f32, 0.75f32), (0.75, -0.75), (-0.75, -0.75)]
            .into_iter()
            .flat_map(|(x, y)| {
                [x, y, 0.5, 1., 0.5, 0.5, z, 1.]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
            })
            .collect();
        let mut sampler = SamplerState::default();
        sampler.set(5, if linear { 2 } else { 1 }).unwrap();
        sampler.set(6, if linear { 2 } else { 1 }).unwrap();
        gpu.clear(&mut target, 0xff00ffff).unwrap();
        let draw = |sampler| {
            gpu.draw_legacy(
                &mut target,
                &LegacyDraw {
                    indices: None,
                    vertex_spirv: VS,
                    pixel_spirv: PS,
                    vertices: &vertices,
                    stride: 32,
                    attributes: &attributes,
                    vertex_constants: &constants,
                    textures: &[TextureBinding {
                        slot: 0,
                        texture: &texture,
                        sampler,
                    }],
                    pixel_constants: &[],
                },
            )
        };
        let mut draw = draw;
        draw(sampler).unwrap();
        let pixels = gpu.readback(&target).unwrap();
        let mut visible = 0;
        for p in pixels.as_chunks::<4>().0 {
            if p == &[255, 255, 0, 255] {
                continue;
            }
            visible += 1;
            assert_eq!(p[3], 255);
            for &channel in &p[..3] {
                assert!(channel.abs_diff(value) <= u8::from(linear));
            }
        }
        assert!(visible > 200);
        sampler.set(11, 1).unwrap();
        assert!(
            gpu.draw_legacy(
                &mut target,
                &LegacyDraw {
                    indices: None,
                    vertex_spirv: VS,
                    pixel_spirv: PS,
                    vertices: &vertices,
                    stride: 32,
                    attributes: &attributes,
                    vertex_constants: &constants,
                    textures: &[TextureBinding {
                        slot: 0,
                        texture: &texture,
                        sampler
                    }],
                    pixel_constants: &[]
                }
            )
            .is_err()
        );
    }
}
