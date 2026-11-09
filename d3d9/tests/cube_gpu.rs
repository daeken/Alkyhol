//! Real cube face/mip uploads and original owned SM2 cube sampling on the GPU.
use alkyhol_d3d9::{
    Device, LegacyDraw, SampleFormat, SamplerState, TextureBinding, compiler::Input, fixed_vertex,
};
#[test]
#[ignore = "requires an actual GPU"]
fn cube_faces_and_mips_are_independent_and_sample_through_legacy_bytecode() {
    let device = Device::new().unwrap();
    let vertex = fixed_vertex::vertex_shader(&[Input {
        semantic: "COLOR".into(),
        index: 0,
        register: 8,
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
    let directions = [
        [1., 0., 0.],
        [-1., 0., 0.],
        [0., 1., 0.],
        [0., -1., 0.],
        [0., 0., 1.],
        [0., 0., -1.],
    ];
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (face, direction) in directions.iter().enumerate() {
        let x = face as f32 * 8.;
        for (x, y) in [(x, 0.), (x + 8., 0.), (x, 8.), (x + 8., 8.)] {
            vertices.extend(
                [x, y, 0.5, 1., direction[0], direction[1], direction[2], 1.]
                    .into_iter()
                    .flat_map(f32::to_le_bytes),
            );
        }
        indices.extend([0, 1, 2, 1, 3, 2].map(|i| i + face as u32 * 4));
    }
    let constants: Vec<_> = [48f32, 8., 0., 0.]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let mut cube = device
        .create_cube_texture(4, 0, SampleFormat::Bgra8)
        .unwrap();
    assert!(cube.is_cube());
    assert_eq!(cube.levels(), 3);
    let mut plain = device.create_texture(4, 4, 1).unwrap();
    assert!(!plain.is_cube());
    assert!(device.upload_cube_face(&mut plain, 0, 0, &[0; 64]).is_err());
    assert!(device.upload_texture(&mut cube, 0, &[0; 64]).is_err());
    assert!(device.upload_cube_face(&mut cube, 6, 0, &[0; 64]).is_err());
    assert!(device.upload_cube_face(&mut cube, 0, 3, &[0; 4]).is_err());
    assert!(device.upload_cube_face(&mut cube, 0, 0, &[0; 63]).is_err());
    let mut target = device.create_target(48, 8).unwrap();
    device.clear(&mut target, 0xff000000).unwrap();
    let draw = |target: &mut _, cube: &_, sampler| {
        device.draw_legacy(
            target,
            &LegacyDraw {
                indices: Some(&indices),
                vertex_spirv: &vertex.spirv,
                pixel_spirv: include_bytes!("fixtures/legacy-cube.ps.spv"),
                vertices: &vertices,
                stride: 32,
                attributes: &attributes,
                vertex_constants: &constants,
                pixel_constants: &[],
                textures: &[TextureBinding {
                    slot: 0,
                    texture: cube,
                    sampler,
                }],
            },
        )
    };
    let color = |face: u32, mip: u32| {
        [
            23 + face as u8 * 31,
            17 + mip as u8 * 61,
            111,
            200 + face as u8,
        ]
    };
    for face in 0..6 {
        for mip in 0..3 {
            let (w, h) = cube.dimensions(mip).unwrap();
            assert_eq!(cube.byte_layout(mip).unwrap(), (w * 4, w * h * 4));
            let bytes: Vec<_> = (0..w * h).flat_map(|_| color(face, mip)).collect();
            if face == 5 && mip == 2 {
                assert!(draw(&mut target, &cube, SamplerState::default()).is_err());
            }
            device
                .upload_cube_face(&mut cube, face, mip, &bytes)
                .unwrap();
        }
    }
    for mip in 0..3 {
        let mut sampler = SamplerState::default();
        sampler.set(9, mip).unwrap();
        draw(&mut target, &cube, sampler).unwrap();
        let pixels = device.readback(&target).unwrap();
        for y in 0..8 {
            for x in 0..48 {
                assert_eq!(
                    &pixels[(y * 48 + x) * 4..(y * 48 + x) * 4 + 4],
                    &color(x as u32 / 8, mip),
                    "x={x} y={y} mip={mip}"
                );
            }
        }
    }
    assert!(
        device
            .create_cube_texture(0, 1, SampleFormat::Bgra8)
            .is_err()
    );
    assert!(
        device
            .create_cube_texture(4, 4, SampleFormat::Bgra8)
            .is_err()
    );
}
