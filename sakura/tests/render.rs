//! Physical Metal application assertions, not a native DXR rendering comparison.
use alkyhol_sakura::{renderer::Renderer, scene::Camera};
use glam::Vec3;
#[test]
#[ignore = "requires physical Metal RT and verified assets; run --include-ignored"]
fn full_scene_renders_repeatably_moves_reflects_resizes_and_captures() {
    objc::rc::autoreleasepool(|| {
        let assets =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../local/sakura/assets");
        assert!(Renderer::new(std::path::Path::new("/missing-sakura-assets"), 129, 73).is_err());
        let mut renderer = Renderer::new(&assets, 129, 73).unwrap();
        for (w, h) in [(0, 73), (129, 0), (4097, 1), (1, 4097)] {
            assert!(renderer.resize(w, h).is_err());
            assert_eq!(renderer.extent(), (129, 73));
        }
        let temp =
            std::env::temp_dir().join(format!("alkyhol-sakura-render-{}", std::process::id()));
        let c = Camera::default();
        let first = renderer.render(&c, 0., None).unwrap();
        eprintln!("Default frame: {first:?}");
        assert_eq!(first.primary.iter().sum::<u32>(), 129 * 73);
        for i in [0, 2, 3, 4, 5] {
            assert!(first.primary[i] > 0, "missing primary material {i}");
        }
        assert!(first.floor_reflections > 0);
        assert_eq!(
            first.secondary_hits + first.secondary_misses,
            first.floor_reflections + first.cube_reflections
        );
        renderer.capture(&temp.join("a.png")).unwrap();
        let repeated = renderer.render(&c, 0., None).unwrap();
        renderer.capture(&temp.join("b.png")).unwrap();
        assert_eq!(first, repeated);
        let a = image::open(temp.join("a.png")).unwrap().to_rgba8();
        let b = image::open(temp.join("b.png")).unwrap().to_rgba8();
        assert_eq!(a, b);
        assert_eq!(a.dimensions(), (129, 73));
        renderer.set_original_shaders_enabled(false);
        let manual = renderer.render(&c, 0., None).unwrap();
        renderer
            .capture(&temp.join("manual-reference.png"))
            .unwrap();
        let manual_pixels = image::open(temp.join("manual-reference.png"))
            .unwrap()
            .to_rgba8();
        let changed = a
            .pixels()
            .zip(manual_pixels.pixels())
            .filter(|(a, b)| a != b)
            .count();
        eprintln!(
            "Original/manual source-camera and arithmetic observation: original={first:?}; manual={manual:?}; changed RGBA8 pixels={changed}; no full-frame equality claim"
        );
        assert_eq!(manual.primary.iter().sum::<u32>(), 129 * 73);
        renderer.set_original_shaders_enabled(true);
        assert_eq!(renderer.render(&c, 0., None).unwrap(), first);
        renderer
            .capture(&temp.join("restored-original.png"))
            .unwrap();
        assert_eq!(
            a,
            image::open(temp.join("restored-original.png"))
                .unwrap()
                .to_rgba8()
        );
        assert!(
            a.pixels().all(|p| p[3] == 255),
            "default view has no primary reflective cubes; floor reflection alpha stays opaque"
        );
        let advanced = renderer.render(&c, 2., None).unwrap();
        renderer.capture(&temp.join("light.png")).unwrap();
        assert_eq!(first.primary, advanced.primary);
        assert_ne!(a, image::open(temp.join("light.png")).unwrap().to_rgba8());
        renderer.resize(257, 257).unwrap();
        let close = Camera {
            eye: Vec3::new(0.5, 2.6, -0.4),
            target: Vec3::new(0.5, 2., 0.),
            up: Vec3::Y,
        };
        let stats = renderer.render(&close, 0., None).unwrap();
        eprintln!("Reflective cube camera: {stats:?}");
        assert_eq!(stats.primary.iter().sum::<u32>(), 257 * 257);
        assert!(stats.primary[1] > 0);
        assert!(stats.cube_reflections > 0);
        assert_eq!(
            stats.secondary_hits + stats.secondary_misses,
            stats.floor_reflections + stats.cube_reflections
        );
        renderer.capture(&temp.join("reflect.png")).unwrap();
        assert_eq!(image::open(temp.join("reflect.png")).unwrap().width(), 257);
        assert!(renderer.capture(&temp).is_err());
        std::fs::remove_dir_all(temp).unwrap();
    });
}
