use alkyhol_miniengine::{camera::Camera, h3d};
use glam::Vec3;
fn put(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn fixture() -> Vec<u8> {
    let vertex = 64 + 336 + 992;
    let indices = vertex + 3 * 56;
    let mut b = vec![0; indices + 12];
    for (offset, value) in [
        (0, 1),
        (4, 1),
        (8, 168),
        (12, 6),
        (16, 0),
        (64 + 32, 0),
        (64 + 36, 31),
        (64 + 44, 56),
        (64 + 308, 0),
        (64 + 312, 3),
        (64 + 316, 0),
        (64 + 320, 3),
    ] {
        put(&mut b, offset, value);
    }
    for i in 0..3 {
        put(&mut b, 32 + i * 4, (-2f32).to_bits());
        put(&mut b, 48 + i * 4, 2f32.to_bits());
    }
    for (i, (offset, count)) in [(0, 3), (12, 2), (20, 3), (32, 3), (44, 3)]
        .into_iter()
        .enumerate()
    {
        put(&mut b, 64 + 52 + i * 8, offset);
        put(&mut b, 64 + 56 + i * 8, count | 5 << 16);
    }
    for i in 0..4 {
        put(&mut b, 64 + 336 + i * 4, 1f32.to_bits());
    }
    let positions = [[0f32, 0., 0.], [1., 0., 0.], [0., 1., 0.]];
    for (i, p) in positions.iter().enumerate() {
        for (j, v) in p.iter().enumerate() {
            put(&mut b, vertex + i * 56 + j * 4, v.to_bits());
        }
        put(&mut b, vertex + i * 56 + 28, 1f32.to_bits());
    }
    for (i, index) in [2u16, 0, 1].into_iter().enumerate() {
        b[indices + i * 2..indices + i * 2 + 2].copy_from_slice(&index.to_le_bytes());
    }
    b
}
#[test]
fn h3d_parses_simd_padding_local_indices_attributes_materials_and_preserves_unused_nan_bits() {
    let mut b = fixture();
    let base = 64 + 336 + 992;
    put(&mut b, base + 32, 0x7fc12345);
    let model = h3d::parse(&b).unwrap();
    assert_eq!(model.meshes.len(), 1);
    assert_eq!(model.materials.len(), 1);
    let v = &model.meshes[0].vertices;
    assert_eq!(v[0].position, [0., 1., 0.]);
    assert_eq!(v[1].position, [0.; 3]);
    assert_eq!(v[2].position, [1., 0., 0.]);
    assert_eq!(v[1].tangent[0].to_bits(), 0x7fc12345);
    assert_eq!(v[0].normal, [0., 0., 1.]);
    assert_eq!(model.materials[0].diffuse, [1.; 4]);
    assert_eq!(model.bounds, [[-2.; 3], [2.; 3]]);
}
#[test]
fn h3d_rejects_truncation_overflow_bad_layout_indices_strings_and_nonfinite_geometry() {
    let good = fixture();
    for end in 0..good.len() {
        assert!(
            h3d::parse(&good[..end]).is_err(),
            "accepted truncated prefix {end}"
        );
    }
    for (offset, value) in [
        (0, u32::MAX),
        (4, 0),
        (8, u32::MAX),
        (12, u32::MAX),
        (16, 1),
        (64 + 32, 1),
        (64 + 36, 0),
        (64 + 44, 12),
        (64 + 52, 4),
        (64 + 56, 0),
        (64 + 308, u32::MAX),
        (64 + 312, 0),
        (64 + 316, u32::MAX),
        (64 + 320, 2),
        (32, f32::NAN.to_bits()),
        (64 + 336 + 992, f32::INFINITY.to_bits()),
    ] {
        let mut bad = good.clone();
        put(&mut bad, offset, value);
        assert!(h3d::parse(&bad).is_err(), "accepted invalid field {offset}");
    }
    let mut bad = good.clone();
    let indices = 64 + 336 + 992 + 168;
    bad[indices..indices + 2].copy_from_slice(&3u16.to_le_bytes());
    assert!(h3d::parse(&bad).is_err());
    let mut bad = good.clone();
    bad[64 + 336 + 92..64 + 336 + 92 + 128].fill(b'A');
    assert!(h3d::parse(&bad).is_err());
    let mut bad = good;
    bad[64 + 336 + 860] = 255;
    assert!(h3d::parse(&bad).is_err());
}
#[test]
fn camera_preserves_upstream_presets_right_handed_basis_and_look_direction() {
    assert!(Camera::preset(5).is_err());
    let mut c = Camera::preset(2).unwrap();
    assert_eq!(c.eye, Vec3::new(-1237.61, 80.60, -26.02));
    let [r, u, f] = c.basis();
    assert!((r.cross(u) + f).length() < 1e-6);
    assert!(f.x > 0.96);
    assert!((f.y - 0.268f32.sin()).abs() < 1e-6);
    c.rotate(0.1, 0.2);
    assert!(c.heading < -1.57);
    assert!((c.pitch - 0.068).abs() < 1e-6);
    c.rotate(0., -100.);
    assert_eq!(c.pitch, 1.56);
}
#[test]
#[ignore = "requires pinned public H3D: python3 tools/fetch_miniengine.py"]
fn original_sponza_has_all_meshes_materials_triangles_and_raw_tangent_anomalies() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../local/miniengine/assets/Sponza/sponza.h3d");
    let model = h3d::parse(&std::fs::read(path).unwrap()).unwrap();
    assert_eq!((model.meshes.len(), model.materials.len()), (34, 27));
    assert_eq!(
        model
            .meshes
            .iter()
            .map(|m| m.vertices.len() / 3)
            .sum::<usize>(),
        278919
    );
    assert!(model.meshes.iter().all(|m| m.material < 27
        && m.vertices.iter().all(|v| {
            v.position
                .iter()
                .chain(&v.normal)
                .chain(&v.uv)
                .all(|f| f.is_finite())
        })));
    assert!(
        model.meshes.iter().flat_map(|m| &m.vertices).any(|v| v
            .tangent
            .iter()
            .chain(&v.bitangent)
            .any(|f| !f.is_finite()))
    );
}
