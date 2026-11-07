use alkyhol_shader::*;
use std::path::{Path, PathBuf};

fn dxc() -> PathBuf {
    std::env::var_os("DXC")
        .map(PathBuf::from)
        .unwrap_or_else(|| "dxc".into())
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}
fn compile(name: &str, entry: &str, stage: Stage) -> Vec<u8> {
    compile_hlsl(&dxc(), &fixture(name), entry, stage).expect("DXC must be available (set DXC)")
}

#[test]
fn compute_preserves_namespaces_spaces_and_group_size() {
    let bytes = compile("compute.hlsl", "CSMain", Stage::Compute);
    let shader = translate_dxil(&bytes, "CSMain").unwrap();
    assert_eq!(shader.stage, Stage::Compute);
    assert_eq!(shader.threads_per_group, Some([8, 1, 1]));
    assert_eq!(shader.resources.len(), 4);
    for (i, (class, space)) in [
        (RegisterClass::Cbv, 0),
        (RegisterClass::Srv, 0),
        (RegisterClass::Srv, 2),
        (RegisterClass::Uav, 0),
    ]
    .into_iter()
    .enumerate()
    {
        let resource = &shader.resources[i];
        assert_eq!(
            (resource.class, resource.space, resource.register),
            (class, space, 0)
        );
        assert_eq!(resource.namespace, MetalNamespace::Buffer);
        assert_eq!(resource.metal_index, i as u32);
    }
    assert!(shader.msl.contains("threadgroup"));
    assert!(translate_dxil(&bytes, "Missing").is_err());
    assert!(translate_dxil(&bytes, "CSMain\0bad").is_err());
}

#[test]
fn compute_pso_selects_the_declared_entry_without_guessing_its_name() {
    let bytes = compile("compute.hlsl", "CSMain", Stage::Compute);
    let translated = translate_compute_pipeline(&bytes).unwrap();
    let explicit = translate_dxil(&bytes, "CSMain").unwrap();
    assert_eq!(translated.msl, explicit.msl);
    assert_eq!(translated.entry_point, explicit.entry_point);
    assert_eq!(translated.threads_per_group, Some([8, 1, 1]));
    assert_eq!(translated.resources.len(), 4);
}

#[test]
fn compute_pso_rejects_graphics_programs_and_corrupt_envelopes() {
    let vertex = compile("microsoft-hello-triangle.hlsl", "VSMain", Stage::Vertex);
    assert!(
        translate_compute_pipeline(&vertex)
            .unwrap_err()
            .contains("compute entry")
    );
    let bytes = compile("compute.hlsl", "CSMain", Stage::Compute);
    for length in [0, 4, 31, bytes.len() - 1] {
        assert!(translate_compute_pipeline(&bytes[..length]).is_err());
    }
}

#[test]
fn original_microsoft_vertex_and_pixel_shaders_translate_without_edits() {
    for (entry, stage) in [("VSMain", Stage::Vertex), ("PSMain", Stage::Pixel)] {
        let shader = translate_dxil(
            &compile("microsoft-hello-triangle.hlsl", entry, stage),
            entry,
        )
        .unwrap();
        assert_eq!(shader.stage, stage);
        assert_eq!(shader.threads_per_group, None);
        assert!(shader.resources.is_empty());
        assert!(!shader.entry_point.is_empty());
    }
}

#[test]
fn truncated_and_corrupt_containers_never_reach_conversion() {
    let bytes = compile("checker.hlsl", "PSMain", Stage::Pixel);
    for end in 0..bytes.len() {
        assert!(
            translate_dxil(&bytes[..end], "PSMain").is_err(),
            "truncation {end}"
        );
    }
    for offset in [0, 20, 24, 28, 32] {
        let mut corrupt = bytes.clone();
        corrupt[offset..offset + 4].fill(0xff);
        assert!(
            translate_dxil(&corrupt, "PSMain").is_err(),
            "corruption {offset}"
        );
    }
    let mut trailing = bytes;
    trailing.extend_from_slice(&[0; 4]);
    assert!(translate_dxil(&trailing, "PSMain").is_err());
}

#[test]
fn compiler_diagnostics_and_invalid_parameters_fail() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("broken.hlsl");
    std::fs::write(&file, "[numthreads(1,1,1)] void main() { undefined(); }").unwrap();
    assert!(
        compile_hlsl(&dxc(), &file, "main", Stage::Compute)
            .unwrap_err()
            .contains("DXC failed")
    );
    assert!(compile_hlsl(&dxc(), &file, "9bad", Stage::Compute).is_err());
    assert!(
        compile_hlsl(
            Path::new("/nonexistent/alkyhol-dxc"),
            &file,
            "main",
            Stage::Compute
        )
        .is_err()
    );
}

#[test]
fn explicit_compile_options_preserve_defines_and_require_root_signature_opt_in() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("options.hlsl");
    std::fs::write(&file,"RWStructuredBuffer<uint> o:register(u0); [RootSignature(\"DescriptorTable(UAV(u0))\")] [numthreads(1,1,1)] void main() {o[0]=TEST_VALUE;}").unwrap();
    let options = HlslOptions {
        shader_model_6_5: true,
        strip_root_signature: false,
        defines: &[("TEST_VALUE", "42")],
    };
    assert!(
        compile_hlsl_with_options(&dxc(), &file, "main", Stage::Compute, &options)
            .unwrap_err()
            .contains("root signatures")
    );
    let options = HlslOptions {
        strip_root_signature: true,
        ..options
    };
    let bytes = compile_hlsl_with_options(&dxc(), &file, "main", Stage::Compute, &options).unwrap();
    let result = translate_dxil(&bytes, "main").unwrap();
    assert!(!result.requires_ray_query);
    assert_eq!(result.resources.len(), 1);
    assert!(result.msl.contains("42u"));
    let bad = HlslOptions {
        defines: &[("-bad", "1")],
        ..Default::default()
    };
    assert!(compile_hlsl_with_options(&dxc(), &file, "main", Stage::Compute, &bad).is_err());
}

#[test]
fn original_sponza_adapter_declares_real_ray_queries_and_original_entry_bodies() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/miniengine/bary.hlsl");
    assert!(compile_hlsl(&dxc(), &file, "AlkyholMain", Stage::Compute).is_err());
    let bytes = compile_hlsl_with_options(
        &dxc(),
        &file,
        "AlkyholMain",
        Stage::Compute,
        &HlslOptions {
            shader_model_6_5: true,
            ..Default::default()
        },
    )
    .unwrap();
    let shader = translate_dxil(&bytes, "AlkyholMain").unwrap();
    assert!(shader.requires_ray_query);
    assert_eq!(shader.threads_per_group, Some([8, 8, 1]));
    let acceleration = shader
        .resources
        .iter()
        .find(|r| r.kind == "RtAccelerationStructure")
        .unwrap();
    assert_eq!(acceleration.namespace, MetalNamespace::Buffer);
    assert!(shader.msl.contains("intersection_query"));
}

#[test]
fn bounded_original_sakura_shader_uses_ray_queries_and_preserves_buffer_classes() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/sakura/ser_off.hlsl");
    let bytes = compile_hlsl_with_options(
        &dxc(),
        &file,
        "AlkyholMain",
        Stage::Compute,
        &HlslOptions {
            shader_model_6_5: true,
            ..Default::default()
        },
    )
    .unwrap();
    let shader = translate_dxil(&bytes, "AlkyholMain").unwrap();
    assert!(shader.requires_ray_query);
    assert_eq!(shader.resources.len(), 16);
    for register in [2, 4, 6, 8] {
        assert!(
            shader
                .resources
                .iter()
                .any(|r| r.class == RegisterClass::Srv
                    && r.register == register
                    && r.kind == "StructuredBuffer"
                    && r.namespace == MetalNamespace::Buffer)
        );
    }
    assert!(
        !shader
            .resources
            .iter()
            .any(|r| r.class == RegisterClass::Srv && r.register == 3),
        "the original trunk index declaration is unused, not a substituted leaf/trunk binding"
    );
}

#[test]
fn explicit_metal_attachment_contract_widens_rgb_and_rejects_invalid_requests() {
    let file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/miniengine/original/PresentSDRPS.hlsl");
    let bytes = compile_hlsl_with_options(
        &dxc(),
        &file,
        "main",
        Stage::Pixel,
        &HlslOptions {
            strip_root_signature: true,
            ..Default::default()
        },
    )
    .unwrap();
    let shader = translate_dxil_with_options(
        &bytes,
        "main",
        &MslOptions {
            fragment_output_components: &[(0, 4)],
        },
    )
    .unwrap();
    assert!(shader.msl.contains("float4"));
    for output in [
        &[(0, 0)][..],
        &[(0, 5)][..],
        &[(8, 4)][..],
        &[(0, 4), (0, 3)][..],
    ] {
        assert!(
            translate_dxil_with_options(
                &bytes,
                "main",
                &MslOptions {
                    fragment_output_components: output
                }
            )
            .is_err()
        );
    }
    let compute = compile("compute.hlsl", "CSMain", Stage::Compute);
    assert!(
        translate_dxil_with_options(
            &compute,
            "CSMain",
            &MslOptions {
                fragment_output_components: &[(0, 4)]
            }
        )
        .is_err()
    );
}

#[test]
fn unsupported_descriptor_arrays_counters_and_geometry_stage_fail() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("unsupported.hlsl");
    for source in [
        "StructuredBuffer<uint> t[2] : register(t0); RWStructuredBuffer<uint> o : register(u0); [numthreads(1,1,1)] void main(uint3 id:SV_DispatchThreadID) { o[id.x] = t[id.x&1][0]; }",
        "AppendStructuredBuffer<uint> o : register(u0); [numthreads(1,1,1)] void main() { o.Append(42); }",
    ] {
        std::fs::write(&file, source).unwrap();
        let bytes = compile_hlsl(&dxc(), &file, "main", Stage::Compute).unwrap();
        assert!(
            translate_dxil(&bytes, "main")
                .unwrap_err()
                .contains("unsupported")
        );
    }
    // Compile a real unsupported stage with DXC, rather than fabricating stage metadata.
    std::fs::write(&file,"struct V { float4 position : SV_Position; }; [maxvertexcount(3)] void GSMain(triangle V v[3], inout TriangleStream<V> s) { s.Append(v[0]); s.Append(v[1]); s.Append(v[2]); }").unwrap();
    let output = temp.path().join("gs.dxil");
    let result = std::process::Command::new(dxc())
        .args(["-T", "gs_6_0", "-E", "GSMain", "-Fo"])
        .arg(&output)
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}: {}{}",
        result.status,
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(
        translate_dxil(&std::fs::read(output).unwrap(), "GSMain")
            .unwrap_err()
            .contains("unsupported shader stage")
    );
}

#[test]
fn texture_slot_budget_counts_entire_arrays_and_both_srv_uav_namespaces() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("texture_budget.hlsl");
    for texture_output in [false, true] {
        let output = if texture_output {
            "RWTexture2D<uint> o:register(u0);"
        } else {
            "RWStructuredBuffer<uint> o:register(u0);"
        };
        let target = if texture_output { "id.xy" } else { "id.x" };
        std::fs::write(&file, format!("Texture2D<uint> t[128]:register(t0); {output} [numthreads(1,1,1)] void main(uint3 id:SV_DispatchThreadID) {{o[{target}]=t[NonUniformResourceIndex(id.x&127)].Load(int3(0,0,0));}}")).unwrap();
        let bytes = compile_hlsl(&dxc(), &file, "main", Stage::Compute).unwrap();
        let shader = translate_dxil(&bytes, "main");
        if texture_output {
            assert!(shader.unwrap_err().contains("too many Texture resources"));
        } else {
            let shader = shader.unwrap();
            let textures = shader
                .resources
                .iter()
                .find(|r| r.class == RegisterClass::Srv)
                .unwrap();
            assert_eq!(
                (textures.metal_index, textures.count, textures.namespace),
                (0, 128, MetalNamespace::Texture)
            );
            let output = shader
                .resources
                .iter()
                .find(|r| r.class == RegisterClass::Uav)
                .unwrap();
            assert_eq!(
                (output.metal_index, output.namespace),
                (0, MetalNamespace::Buffer)
            );
        }
    }
}

#[test]
fn buffer_dimension_translation_reports_a_separate_reserved_binding() {
    let shader = translate_dxil(
        &compile("buffer_dimensions.hlsl", "CSMain", Stage::Compute),
        "CSMain",
    )
    .unwrap();
    let sizes = shader.buffer_sizes.as_ref().unwrap();
    assert_eq!((sizes.metal_index, sizes.word_count), (25, 24));
    assert_eq!(sizes.resource_indices, vec![0, 1, 2, 3]);
    assert!(
        shader
            .resources
            .iter()
            .filter(|r| r.namespace == MetalNamespace::Buffer)
            .all(|r| r.metal_index < 24)
    );
    assert!(
        translate_dxil(&compile("compute.hlsl", "CSMain", Stage::Compute), "CSMain")
            .unwrap()
            .buffer_sizes
            .is_none()
    );
}
