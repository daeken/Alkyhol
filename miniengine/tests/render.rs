use alkyhol_miniengine::{
    camera::Camera,
    renderer::{RenderMode, Renderer},
};
#[test]
#[ignore = "requires physical Metal RT and original Sponza assets"]
fn original_hlsl_bary_pipeline_agrees_with_manual_reference_on_all_camera_presets() {
    objc::rc::autoreleasepool(|| {
        let assets =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../local/miniengine/assets");
        let mut renderer = Renderer::new(&assets, 129, 73).unwrap();
        renderer.set_mode(RenderMode::Bary);
        let temp =
            std::env::temp_dir().join(format!("alkyhol-original-hlsl-{}", std::process::id()));
        for preset in 0..5 {
            let camera = Camera::preset(preset).unwrap();
            renderer.set_original_bary_enabled(true);
            let original = renderer.render(&camera, None).unwrap();
            renderer.capture(&temp.join("original.png")).unwrap();
            let pixels = image::open(temp.join("original.png")).unwrap().to_rgba8();
            renderer.set_original_bary_enabled(false);
            let manual = renderer.render(&camera, None).unwrap();
            renderer.capture(&temp.join("manual.png")).unwrap();
            assert_eq!(original, manual, "camera {preset} traversal");
            assert_eq!(
                pixels,
                image::open(temp.join("manual.png")).unwrap().to_rgba8(),
                "camera {preset} exact RGBA8"
            );
        }
    });
}
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
        renderer.set_original_shading_enabled(false);
        let reference = renderer.render(&camera, None).unwrap();
        assert_eq!(first.per_mesh, reference.per_mesh);
        assert_eq!(first.misses, reference.misses);
        renderer.set_original_shading_enabled(true);
        assert_eq!(first, renderer.render(&camera, None).unwrap());
        renderer.capture(&temp.join("restored-source.png")).unwrap();
        assert_eq!(
            a,
            image::open(temp.join("restored-source.png"))
                .unwrap()
                .to_rgba8()
        );
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
        renderer.set_mode(RenderMode::LitNoShadows);
        let unshadowed = renderer.render(&camera, None).unwrap();
        assert_eq!(unshadowed.per_mesh, first.per_mesh);
        assert_eq!(unshadowed.shadow_rays, 0);
        renderer.capture(&temp.join("unshadowed.png")).unwrap();
        let unshadowed_pixels = image::open(temp.join("unshadowed.png")).unwrap().to_rgba8();
        renderer.set_mode(RenderMode::Lit);
        let lit = renderer.render(&camera, None).unwrap();
        assert_eq!(lit.per_mesh, first.per_mesh);
        assert!(lit.shadow_rays > 0 && lit.shadow_hits > 0 && lit.shadow_hits < lit.shadow_rays);
        renderer.capture(&temp.join("lit.png")).unwrap();
        let lit_pixels = image::open(temp.join("lit.png")).unwrap().to_rgba8();
        assert_ne!(lit_pixels, unshadowed_pixels);
        renderer.set_mode(RenderMode::LitShadowRays);
        let diffuse = renderer.render(&camera, None).unwrap();
        assert_eq!(diffuse.per_mesh, lit.per_mesh);
        assert_eq!(diffuse.shadow_rays, lit.shadow_rays);
        renderer.set_mode(RenderMode::Reflections);
        let reflections = renderer.render(&camera, None).unwrap();
        assert_eq!(reflections.per_mesh, first.per_mesh);
        assert!(reflections.reflection_rays > 0 && reflections.reflection_hits > 0);
        assert_eq!(reflections.short_self_reflections, 0);
        renderer.capture(&temp.join("reflections.png")).unwrap();
        let reflected_pixels = image::open(temp.join("reflections.png"))
            .unwrap()
            .to_rgba8();
        let changed = reflected_pixels
            .pixels()
            .zip(lit_pixels.pixels())
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            changed > 0 && changed <= reflections.reflection_rays as usize,
            "reflections must change only eligible floor pixels"
        );
        eprintln!(
            "Sponza shaded frame: {reflections:?}; {changed} floor pixels changed by reflections"
        );
        renderer.set_original_shading_enabled(false);
        let reference = renderer.render(&camera, None).unwrap();
        assert_eq!(reference.per_mesh, reflections.per_mesh);
        assert_eq!(reference.reflection_rays, reflections.reflection_rays);
        renderer.set_original_shading_enabled(true);
        assert_eq!(reflections, renderer.render(&camera, None).unwrap());
        renderer
            .capture(&temp.join("restored-reflections.png"))
            .unwrap();
        assert_eq!(
            reflected_pixels,
            image::open(temp.join("restored-reflections.png"))
                .unwrap()
                .to_rgba8()
        );
        renderer.set_mode(RenderMode::ReflectionBary);
        let reflection_bary = renderer.render(&camera, None).unwrap();
        assert_eq!(reflection_bary.reflection_rays, reflections.reflection_rays);
        assert_eq!(reflection_bary.short_self_reflections, 0);
        renderer.set_mode(RenderMode::Shadows);
        renderer.render(&camera, None).unwrap();
        renderer.capture(&temp.join("shadow-mask.png")).unwrap();
        let mask = image::open(temp.join("shadow-mask.png"))
            .unwrap()
            .to_rgba8();
        assert!(
            mask.pixels()
                .all(|p| (p[0] == 0 || p[0] == 255) && p[0] == p[1] && p[1] == p[2] && p[3] == 255)
        );
        renderer.resize(257, 145).unwrap();
        let changed = renderer.render(&Camera::preset(1).unwrap(), None).unwrap();
        assert_eq!(
            changed.per_mesh.iter().sum::<u32>() + changed.misses,
            257 * 145
        );
        assert_ne!(first.per_mesh, changed.per_mesh);
        renderer.set_mode(RenderMode::Reflections);
        for preset in 0..5 {
            let stats = renderer
                .render(&Camera::preset(preset).unwrap(), None)
                .unwrap();
            assert_eq!(stats.per_mesh.iter().sum::<u32>() + stats.misses, 257 * 145);
            assert_eq!(stats.short_self_reflections, 0, "preset {preset}");
        }
        assert!(renderer.capture(&temp).is_err());
        std::fs::remove_dir_all(temp).unwrap();
    });
}
