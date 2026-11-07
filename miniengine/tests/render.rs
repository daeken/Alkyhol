use alkyhol_miniengine::{
    camera::Camera,
    renderer::{RenderMode, Renderer},
};
#[test]
#[ignore = "requires physical Metal RT and pinned Sponza H3D; run --include-ignored"]
fn full_sponza_debug_views_preserve_hit_geometry_and_capture_repeatably() {
    objc::rc::autoreleasepool(|| {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../local/miniengine/assets");
        assert!(Renderer::new(std::path::Path::new("/missing-model"), 129, 73).is_err());
        let mut renderer = Renderer::new(&path, 129, 73).unwrap();
        let camera = Camera::preset(2).unwrap();
        for size in [(0, 1), (1, 0), (4097, 1)] {
            assert!(renderer.resize(size.0, size.1).is_err());
            assert_eq!(renderer.extent(), (129, 73));
        }
        let temp = std::env::temp_dir().join(format!("alkyhol-miniengine-{}", std::process::id()));
        let first = renderer.render(&camera, None).unwrap();
        assert_eq!(first.per_mesh.iter().sum::<u32>() + first.misses, 129 * 73);
        assert!(first.per_mesh.iter().filter(|&&n| n > 0).count() > 20);
        renderer.capture(&temp.join("geometry.png")).unwrap();
        let a = image::open(temp.join("geometry.png")).unwrap().to_rgba8();
        renderer.render(&camera, None).unwrap();
        renderer.capture(&temp.join("repeat.png")).unwrap();
        assert_eq!(a, image::open(temp.join("repeat.png")).unwrap().to_rgba8());
        for mode in [RenderMode::Bary, RenderMode::Normals, RenderMode::Depth] {
            renderer.set_mode(mode);
            assert_eq!(renderer.mode(), mode);
            let stats = renderer.render(&camera, None).unwrap();
            assert_eq!(first, stats);
            renderer.capture(&temp.join("other.png")).unwrap();
            let image = image::open(temp.join("other.png")).unwrap().to_rgba8();
            assert_ne!(image, a);
            assert!(image.pixels().all(|p| p[3] == 255));
        }
        renderer.resize(257, 145).unwrap();
        let changed = renderer.render(&Camera::preset(1).unwrap(), None).unwrap();
        assert_eq!(
            changed.per_mesh.iter().sum::<u32>() + changed.misses,
            257 * 145
        );
        assert_ne!(first.per_mesh, changed.per_mesh);
        assert!(renderer.capture(&temp).is_err());
        std::fs::remove_dir_all(temp).unwrap();
    });
}
