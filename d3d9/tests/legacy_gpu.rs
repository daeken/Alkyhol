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
