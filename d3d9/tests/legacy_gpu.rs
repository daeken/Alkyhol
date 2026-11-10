//! Owned input shaders compiled by vkd3d 2.1; not original game code/evidence.
use alkyhol_d3d9::{
    Device, LegacyDraw,
    legacy::{self, Stage},
};
const VS: &[u8] = include_bytes!("fixtures/legacy-color.vs.spv");
const PS: &[u8] = include_bytes!("fixtures/legacy-color.ps.spv");
#[test]
fn actual_legacy_modules_validate_after_triangle_and_pixel_center_adapters() {
    let mut vertex = legacy::lower_spirv(VS, Stage::Vertex).unwrap();
    legacy::vertex_pixel_centers(&mut vertex).unwrap();
    legacy::wgsl(&vertex).unwrap();
    let pixel = legacy::lower_spirv(PS, Stage::Pixel).unwrap();
    legacy::wgsl(&pixel).unwrap();
    assert!(legacy::lower_spirv(&VS[..VS.len() - 1], Stage::Vertex).is_err());
    assert!(legacy::lower_spirv(PS, Stage::Vertex).is_err());
    assert!(legacy::lower_spirv(VS, Stage::Pixel).is_err());
}
#[test]
#[ignore = "requires an actual native GPU"]
fn gpu_executes_legacy_shader_matrix_math_and_runtime_constant_changes() {
    let device = Device::new().unwrap();
    let mut target = device.create_target(32, 32).unwrap();
    let mut vertices = Vec::new();
    for (x, y) in [(0f32, 0.75f32), (0.75, -0.75), (-0.75, -0.75)] {
        for f in [x, y, 0.5, 1., 1., 0., 0., 1.] {
            vertices.extend(f.to_le_bytes());
        }
    }
    let mut constants: Vec<_> = [
        1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
    .iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();
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
    for shift in [0f32, 0.75] {
        constants[12..16].copy_from_slice(&shift.to_le_bytes());
        device.clear(&mut target, 0xff0000ff).unwrap();
        device
            .draw_legacy(
                &mut target,
                &LegacyDraw {
                    indices: None,
                    textures: &[],
                    vertex_spirv: VS,
                    pixel_spirv: PS,
                    vertices: &vertices,
                    stride: 32,
                    attributes: &attributes,
                    vertex_constants: &constants,
                    pixel_constants: &[],
                },
            )
            .unwrap();
        let stats = device.legacy_pipeline_cache_stats();
        assert_eq!(stats.builds, 1);
        assert_eq!(stats.hits, if shift == 0. { 0 } else { 1 });
        assert_eq!(stats.entries, 1);
        let pixels = device.readback(&target).unwrap();
        let at = |x: usize, y: usize| &pixels[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4];
        assert_eq!(at(0, 0), [255, 0, 0, 255]);
        assert_eq!(
            at(10, 20),
            if shift == 0. {
                [0, 0, 255, 255]
            } else {
                [255, 0, 0, 255]
            }
        );
        assert_eq!(at(22, 20), [0, 0, 255, 255]);
    }
}

#[test]
#[ignore = "requires an actual GPU"]
fn real_legacy_texture_sampling_applies_srgb_to_rgb_and_preserves_alpha() {
    use alkyhol_d3d9::{SamplerState, TextureBinding};
    let device = Device::new().unwrap();
    let mut target = device.create_target(32, 32).unwrap();
    let mut texture = device.create_texture(1, 1, 1).unwrap();
    assert_eq!(texture.dimensions(0).unwrap(), (1, 1));
    assert!(texture.dimensions(1).is_err());
    assert!(device.upload_texture(&mut texture, 0, &[128; 3]).is_err());
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
        0f32, 0.75, 0.5, 1., 1., 0., 0., 1., 0.75, -0.75, 0.5, 1., 1., 0., 0., 1., -0.75, -0.75,
        0.5, 1., 1., 0., 0., 1.,
    ]
    .iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();
    let constants: Vec<_> = [
        1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
    .iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();
    let pixel = include_bytes!("fixtures/legacy-texture.ps.spv");
    let mut sampler = SamplerState::default();
    device.clear(&mut target, 0xff0000ff).unwrap();
    let draw = |texture: &alkyhol_d3d9::Texture2D, sampler, target: &mut alkyhol_d3d9::Target| {
        device.draw_legacy(
            target,
            &LegacyDraw {
                indices: None,
                vertex_spirv: VS,
                pixel_spirv: pixel,
                vertices: &vertices,
                stride: 32,
                attributes: &attributes,
                vertex_constants: &constants,
                pixel_constants: &[],
                textures: &[TextureBinding {
                    slot: 0,
                    texture,
                    sampler,
                }],
            },
        )
    };
    assert!(draw(&texture, sampler, &mut target).is_err()); // no zero-filled successful read
    device
        .upload_texture(&mut texture, 0, &[128, 128, 128, 128])
        .unwrap();
    for srgb in [0, 1] {
        sampler.set(11, srgb).unwrap();
        draw(&texture, sampler, &mut target).unwrap();
        let pixels = device.readback(&target).unwrap();
        let at = (20 * 32 + 22) * 4;
        assert_eq!(
            &pixels[at..at + 4],
            if srgb == 0 {
                &[128, 128, 128, 128]
            } else {
                &[55, 55, 55, 128]
            }
        );
    }
    device
        .upload_texture(&mut texture, 0, &[10, 60, 120, 200])
        .unwrap();
    for srgb in [0, 1] {
        sampler.set(11, srgb).unwrap();
        draw(&texture, sampler, &mut target).unwrap();
        let pixels = device.readback(&target).unwrap();
        let at = (20 * 32 + 22) * 4;
        assert_eq!(
            &pixels[at..at + 4],
            if srgb == 0 {
                &[10, 60, 120, 200]
            } else {
                &[1, 12, 48, 200]
            }
        );
    }
    let mut luminance = device
        .create_sampled_texture(3, 2, 0, alkyhol_d3d9::SampleFormat::Luminance8)
        .unwrap();
    assert_eq!(luminance.byte_layout(0).unwrap(), (3, 6));
    assert_eq!(luminance.byte_layout(1).unwrap(), (1, 1));
    assert!(draw(&luminance, SamplerState::default(), &mut target).is_err());
    device
        .upload_texture(&mut luminance, 0, &[0, 64, 128, 192, 254, 255])
        .unwrap();
    let mut mip_sampler = SamplerState::default();
    mip_sampler.set(9, 1).unwrap();
    for value in [0, 1, 127, 128, 254, 255] {
        device.upload_texture(&mut luminance, 1, &[value]).unwrap();
        draw(&luminance, mip_sampler, &mut target).unwrap();
        let pixels = device.readback(&target).unwrap();
        let at = (20 * 32 + 22) * 4;
        assert_eq!(&pixels[at..at + 4], &[value, value, value, 255]);
    }
    mip_sampler.set(11, 1).unwrap();
    assert!(draw(&luminance, mip_sampler, &mut target).is_err());
    let (sampled, mut rendered) = device.create_render_texture(32, 32).unwrap();
    assert!(sampled.shares_target(&rendered));
    assert!(!texture.shares_target(&rendered));
    assert!(device.readback(&rendered).is_err());
    assert!(draw(&sampled, alkyhol_d3d9::SamplerState::default(), &mut target).is_err());
    device.clear(&mut rendered, 0xff0000ff).unwrap();
    draw(
        &texture,
        alkyhol_d3d9::SamplerState::default(),
        &mut rendered,
    )
    .unwrap();
    let pixels = device.readback(&rendered).unwrap();
    let at = (20 * 32 + 22) * 4;
    assert_eq!(&pixels[at..at + 4], &[10, 60, 120, 200]);
    assert_eq!(&pixels[..4], &[255, 0, 0, 255]);
    assert!(
        draw(
            &sampled,
            alkyhol_d3d9::SamplerState::default(),
            &mut rendered
        )
        .is_err()
    );
    let alias = rendered.clone();
    drop(rendered);
    assert_eq!(
        &device.readback(&alias).unwrap()[at..at + 4],
        &[10, 60, 120, 200]
    );
    drop(alias); // Sampled storage survives every render-surface handle.
    let mut centered = vertices.clone();
    for vertex in centered.as_chunks_mut::<32>().0 {
        vertex[16..20].copy_from_slice(&0.5f32.to_le_bytes());
        vertex[20..24].copy_from_slice(&0.5f32.to_le_bytes());
    }
    device
        .draw_legacy(
            &mut target,
            &LegacyDraw {
                indices: None,
                vertex_spirv: VS,
                pixel_spirv: pixel,
                vertices: &centered,
                stride: 32,
                attributes: &attributes,
                vertex_constants: &constants,
                pixel_constants: &[],
                textures: &[TextureBinding {
                    slot: 0,
                    texture: &sampled,
                    sampler: alkyhol_d3d9::SamplerState::default(),
                }],
            },
        )
        .unwrap();
    assert_eq!(
        &device.readback(&target).unwrap()[at..at + 4],
        &[10, 60, 120, 200]
    );
    let mut mips = device.create_texture(4, 4, 0).unwrap();
    assert_eq!(mips.levels(), 3);
    assert!(device.create_texture(4, 4, 4).is_err());
    for level in 0..3 {
        let (w, h) = mips.dimensions(level).unwrap();
        let color = [64, 128, 192][level as usize];
        device
            .upload_texture(&mut mips, level, &vec![color; (w * h * 4) as usize])
            .unwrap();
    }
    sampler.set(11, 0).unwrap();
    sampler.set(9, 2).unwrap();
    draw(&mips, sampler, &mut target).unwrap();
    let pixels = device.readback(&target).unwrap();
    assert_eq!(
        &pixels[(20 * 32 + 22) * 4..(20 * 32 + 22) * 4 + 4],
        &[192; 4]
    );
}

#[test]
#[ignore = "requires an actual GPU"]
fn shader_model2_executes_all_128_arithmetic_slots() {
    let device = Device::new().unwrap();
    let mut target = device.create_target(32, 32).unwrap();
    let vertices: Vec<_> = [
        0f32, 0.75, 0.5, 1., 1., 0., 0., 1., 0.75, -0.75, 0.5, 1., 1., 0., 0., 1., -0.75, -0.75,
        0.5, 1., 1., 0., 0., 1.,
    ]
    .iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();
    let constants: Vec<_> = [
        1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
    .iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();
    let attributes = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4];
    device.clear(&mut target, 0xff0000ff).unwrap();
    device
        .draw_legacy(
            &mut target,
            &LegacyDraw {
                indices: None,
                vertex_spirv: include_bytes!("fixtures/legacy-model2.vs.spv"),
                pixel_spirv: include_bytes!("fixtures/legacy-slots128.ps.spv"),
                vertices: &vertices,
                stride: 32,
                attributes: &attributes,
                vertex_constants: &constants,
                pixel_constants: &[],
                textures: &[],
            },
        )
        .unwrap();
    let pixels = device.readback(&target).unwrap();
    assert_eq!(
        &pixels[(20 * 32 + 22) * 4..(20 * 32 + 22) * 4 + 4],
        &[235, 71, 163, 143]
    );
}

#[test]
#[ignore = "requires an actual GPU"]
fn indexed_gpu_quad_uses_four_vertices_and_checked_index_ranges() {
    let device = Device::new().unwrap();
    let mut target = device.create_target(32, 32).unwrap();
    let mut vertices = Vec::new();
    for (x, y) in [
        (-0.75f32, 0.75f32),
        (0.75, 0.75),
        (0.75, -0.75),
        (-0.75, -0.75),
    ] {
        for f in [x, y, 0.5, 1., 1., 0., 0., 1.] {
            vertices.extend(f.to_le_bytes());
        }
    }
    let constants: Vec<_> = [
        1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
    .iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();
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
    device.clear(&mut target, 0xff0000ff).unwrap();
    let draw = |target: &mut _, indices: &[u32]| {
        device.draw_legacy(
            target,
            &LegacyDraw {
                indices: Some(indices),
                vertex_spirv: VS,
                pixel_spirv: PS,
                vertices: &vertices,
                stride: 32,
                attributes: &attributes,
                vertex_constants: &constants,
                pixel_constants: &[],
                textures: &[],
            },
        )
    };
    for invalid in [&[][..], &[0, 1][..], &[0, 1, 4][..]] {
        assert!(draw(&mut target, invalid).is_err());
    }
    draw(&mut target, &[0, 1, 2, 0, 2, 3]).unwrap();
    let pixels = device.readback(&target).unwrap();
    for (x, y) in [(8, 8), (24, 8), (8, 24), (24, 24)] {
        assert_eq!(
            &pixels[(y * 32 + x) * 4..(y * 32 + x) * 4 + 4],
            &[0, 0, 255, 255]
        );
    }
    assert_eq!(&pixels[..4], &[255, 0, 0, 255]);
}

#[test]
#[ignore = "requires actual BC texture support"]
fn bc123_native_blocks_keep_mip_tails_color_alpha_and_srgb_views() {
    use alkyhol_d3d9::{SampleFormat, SamplerState, TextureBinding};
    let device = Device::new().unwrap();
    assert!(device.supports_bc());
    let mut target = device.create_target(32, 32).unwrap();
    let vertices: Vec<_> = [
        0f32, 0.75, 0.5, 1., 1., 0., 0., 1., 0.75, -0.75, 0.5, 1., 1., 0., 0., 1., -0.75, -0.75,
        0.5, 1., 1., 0., 0., 1.,
    ]
    .iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();
    let constants: Vec<_> = [
        1f32, 0., 0., 0., 0., 1., 0., 0., 0., 0., 1., 0., 0., 0., 0., 1.,
    ]
    .iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();
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
    let draw = |target: &mut _, texture: &_, sampler| {
        device.draw_legacy(
            target,
            &LegacyDraw {
                indices: None,
                vertex_spirv: VS,
                pixel_spirv: include_bytes!("fixtures/legacy-texture.ps.spv"),
                vertices: &vertices,
                stride: 32,
                attributes: &attributes,
                vertex_constants: &constants,
                pixel_constants: &[],
                textures: &[TextureBinding {
                    slot: 0,
                    texture,
                    sampler,
                }],
            },
        )
    };
    let block = |format: SampleFormat, color: u16| {
        let mut bytes = match format {
            SampleFormat::Bc1 => vec![],
            SampleFormat::Bc2 => vec![0x88; 8],
            SampleFormat::Bc3 => vec![128, 255, 0, 0, 0, 0, 0, 0],
            _ => unreachable!(),
        };
        bytes.extend(color.to_le_bytes());
        bytes.extend([0; 6]);
        bytes
    };
    let at = |pixels: Vec<u8>| pixels[(20 * 32 + 22) * 4..(20 * 32 + 22) * 4 + 4].to_vec();
    for format in [SampleFormat::Bc1, SampleFormat::Bc2, SampleFormat::Bc3] {
        let mut texture = device.create_sampled_texture(8, 8, 0, format).unwrap();
        assert_eq!(texture.levels(), 4);
        device.clear(&mut target, 0xff0000ff).unwrap();
        assert!(draw(&mut target, &texture, SamplerState::default()).is_err());
        for (level, color) in [0xf800u16, 0x07e0, 0x001f, 0xffff].into_iter().enumerate() {
            let (_, length) = texture.byte_layout(level as u32).unwrap();
            let block = block(format, color);
            let data: Vec<_> = block
                .iter()
                .copied()
                .cycle()
                .take(length as usize)
                .collect();
            device
                .upload_texture(&mut texture, level as u32, &data)
                .unwrap();
        }
        let alpha = if format == SampleFormat::Bc1 {
            255
        } else if format == SampleFormat::Bc2 {
            136
        } else {
            128
        };
        for (level, rgb) in [[0, 0, 255], [0, 255, 0], [255, 0, 0], [255, 255, 255]]
            .into_iter()
            .enumerate()
        {
            let mut sampler = SamplerState::default();
            sampler.set(9, level as u32).unwrap();
            draw(&mut target, &texture, sampler).unwrap();
            assert_eq!(
                at(device.readback(&target).unwrap()),
                [rgb[0], rgb[1], rgb[2], alpha]
            );
        }
    }
    let mut gray = device
        .create_sampled_texture(4, 4, 1, SampleFormat::Bc3)
        .unwrap();
    device
        .upload_texture(&mut gray, 0, &block(SampleFormat::Bc3, 0x8410))
        .unwrap();
    for (srgb, expected) in [(0, [132, 130, 132, 128]), (1, [59, 57, 59, 128])] {
        let mut sampler = SamplerState::default();
        sampler.set(11, srgb).unwrap();
        draw(&mut target, &gray, sampler).unwrap();
        assert_eq!(at(device.readback(&target).unwrap()), expected);
    }
}

#[test]
#[ignore = "requires an actual native GPU"]
fn pretransformed_rhw_preserves_screen_positions_and_subviewport_clipping() {
    use alkyhol_d3d9::{RenderState, SamplerState, TextureBinding, compiler::Input, fixed_vertex};
    let device = Device::new().unwrap();
    let mut target = device.create_target(32, 32).unwrap();
    let mut texture = device.create_texture(2, 2, 1).unwrap();
    device
        .upload_texture(
            &mut texture,
            0,
            &[
                0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255, 255, 255, 255, 255,
            ],
        )
        .unwrap();
    let shader = fixed_vertex::vertex_shader(&[Input {
        semantic: "COLOR".into(),
        index: 0,
        register: 0,
    }])
    .unwrap();
    let vertices: Vec<u8> = [
        [4f32, 4., 0.5, 0.5, 0., 0., 0., 1.],
        [28., 4., 0.5, 2., 1., 0., 0., 1.],
        [4., 28., 0.5, 0.5, 0., 1., 0., 1.],
        [28., 28., 0.5, 2., 1., 1., 0., 1.],
    ]
    .into_iter()
    .flatten()
    .flat_map(f32::to_le_bytes)
    .collect();
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
    for viewport in [[0f32, 0., 32., 32., 0., 1.], [8., 4., 16., 20., 0., 1.]] {
        let constants: Vec<_> = [viewport[2], viewport[3], viewport[0], viewport[1]]
            .into_iter()
            .flat_map(f32::to_le_bytes)
            .collect();
        let bindings = [TextureBinding {
            slot: 0,
            texture: &texture,
            sampler: SamplerState::default(),
        }];
        let draw = LegacyDraw {
            indices: Some(&[0, 1, 2, 1, 3, 2]),
            vertex_spirv: &shader.spirv,
            pixel_spirv: include_bytes!("fixtures/legacy-texture.ps.spv"),
            vertices: &vertices,
            stride: 32,
            attributes: &attributes,
            vertex_constants: &constants,
            textures: &bindings,
            pixel_constants: &[],
        };
        device.clear(&mut target, 0xff0000ff).unwrap();
        device
            .draw_legacy_viewport_depth_state(
                &mut target,
                &draw,
                &RenderState::default(),
                None,
                viewport,
            )
            .unwrap();
        let pixels = device.readback(&target).unwrap();
        let at = |x: usize, y: usize| &pixels[(y * 32 + x) * 4..(y * 32 + x + 1) * 4];
        assert_eq!(at(0, 0), [255, 0, 0, 255]);
        assert_eq!(at(10, 8), [0, 255, 0, 255]); // Affine UV would incorrectly select red.
        assert_eq!(at(22, 20), [255, 255, 255, 255]);
        assert_eq!(
            at(6, 8),
            if viewport[0] == 0. {
                [0, 0, 255, 255]
            } else {
                [255, 0, 0, 255]
            }
        );
        assert_eq!(
            at(26, 8),
            if viewport[0] == 0. {
                [0, 255, 0, 255]
            } else {
                [255, 0, 0, 255]
            }
        );
        assert!(
            device
                .draw_legacy_viewport_depth_state(
                    &mut target,
                    &draw,
                    &RenderState::default(),
                    None,
                    [31., 0., 2., 32., 0., 1.]
                )
                .is_err()
        );
        assert_eq!(device.readback(&target).unwrap(), pixels);
    }
}
