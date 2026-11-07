use alkyhol_sakura::scene::*;
use glam::Vec3;
#[test]
fn obj_retains_face_order_attributes_scale_and_repeated_named_sections() {
    let source = "o tree\nv 1 2 3\nv 4 5 6\nv 7 8 9\nvn 0 1 0\nvt .25 .75\nf 3/1/1 1/1/1 2/1/1 3/1/1\no other\nv 100 100 100\no tree\nv 10 11 12\nf 4//1 2//1 1//1\n";
    let mesh = load_obj(source, "tree").unwrap();
    assert_eq!(mesh.vertices.len(), 6);
    assert_eq!(
        mesh.vertices[0].position,
        [7. * 0.007, 8. * 0.007, 9. * 0.007]
    );
    assert_eq!(mesh.vertices[0].normal, [0., 1., 0.]);
    assert_eq!(mesh.vertices[0].uv, [0.25, 0.75]);
    assert_eq!(
        mesh.vertices[3].position,
        [10. * 0.007, 11. * 0.007, 12. * 0.007]
    );
    assert_eq!(mesh.vertices[3].uv, [0.5; 2]);
    for s in [
        "o tree\nf 0 1 2",
        "o tree\nv NaN 0 0",
        "o tree\nv 0 0 0\nf 1 1 2",
        "o tree\nv 0 0 0\nf 1/2/3 1/2/3 1/2/3",
        "o tree\nf 1 2",
    ] {
        assert!(load_obj(s, "tree").is_err());
    }
    assert!(load_obj(source, "absent").is_err());
}
#[test]
fn mt19937_matches_published_engine_sequence_and_float_range() {
    let mut r = Mt19937::new(5489);
    assert_eq!(
        (0..5).map(|_| r.next_u32()).collect::<Vec<_>>(),
        [3499211612, 581869302, 3890346734, 3586334585, 545404204]
    );
    let mut r = Mt19937::new(42);
    assert_eq!(r.next_u32(), 1608637542);
    for _ in 0..10000 {
        assert!((-0.1..0.7).contains(&r.uniform(-0.1, 0.7)));
    }
}
#[test]
fn scene_uses_all_original_groups_in_hit_table_order_and_pairs_every_tree() {
    let scene = instances();
    assert_eq!(scene.len(), 13525);
    for (start, end, mesh, material, scale) in [
        (0, 441, 0, 0, 2.),
        (441, 1402, 0, 1, 0.01),
        (1402, 2363, 1, 2, 35.),
        (2363, 3324, 2, 3, 35.),
        (3324, 13525, 3, 4, 2.1),
    ] {
        assert!(
            scene[start..end]
                .iter()
                .all(|i| i.mesh == mesh && i.material == material && i.scale == scale)
        );
    }
    assert_eq!(scene[0].position, [-30., 0., -40.]);
    assert_eq!(scene[440].position, [50., 0., 40.]);
    assert_eq!(scene[441].position[0], -1.);
    assert!((scene[441].position[1] - 1.8996321).abs() < 1e-6);
    for i in 0..961 {
        assert_eq!(scene[1402 + i].position, scene[2363 + i].position);
    }
    assert_eq!(scene, instances());
    let cube = cube();
    assert_eq!(cube.vertices.len(), 36);
    assert_eq!(cube.vertices[0].position, [-1., 1., 1.]);
    assert_eq!(cube.vertices[0].normal, [0., 1., 0.]);
}
#[test]
fn camera_preserves_left_handed_basis_motion_height_and_rotation() {
    let mut c = Camera::default();
    assert!((c.eye - Vec3::new(-2f32.sqrt(), 2.2, -2f32.sqrt())).length() < 1e-6);
    let [right, up, forward] = c.basis();
    assert!((right.cross(up) - forward).length() < 1e-6);
    assert!(right.dot(up).abs() < 1e-6);
    let direction = c.target - c.eye;
    c.move_by(Vec3::new(1., -20., 2.));
    assert_eq!(c.eye.y, 2.02);
    assert!(((c.target - c.eye) - direction).length() < 1e-5);
    c.rotate(0.2, -0.1);
    assert!(((c.target - c.eye).length() - direction.length()).abs() < 1e-5);
    assert!((c.target - c.eye).normalize().dot(forward) < 0.999);
}
#[test]
#[ignore = "requires verified upstream assets: python3 tools/fetch_sakura.py"]
fn pinned_assets_have_original_triangle_counts_and_finite_vertices() {
    let assets = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../local/sakura/assets");
    let meshes = load_meshes(&assets).unwrap();
    assert_eq!(
        meshes.iter().map(|m| m.vertices.len()).collect::<Vec<_>>(),
        [36, 55347, 32328, 14238]
    );
    for mesh in meshes {
        assert!(mesh.vertices.iter().all(|v| {
            v.position
                .iter()
                .chain(&v.normal)
                .chain(&v.uv)
                .all(|v| v.is_finite())
        }));
    }
    assert!(load_meshes(std::path::Path::new("/does-not-exist")).is_err());
}
