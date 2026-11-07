use std::process::Command;

#[test]
fn cli_rejects_bad_modes_stages_and_missing_inputs_without_success_output() {
    for args in [
        vec![],
        vec!["unknown"],
        vec!["dxil", "/nonexistent.dxil", "main"],
        vec!["hlsl", "dxc", "x.hlsl", "main", "rt"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_alkyhol-shader"))
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}

#[test]
fn cli_emits_msl_and_binding_metadata_for_actual_hlsl() {
    let dxc = std::env::var_os("DXC").unwrap_or_else(|| "dxc".into());
    let result = Command::new(env!("CARGO_BIN_EXE_alkyhol-shader"))
        .arg("hlsl")
        .arg(&dxc)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/compute.hlsl"
        ))
        .args(["CSMain", "cs"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(report["stage"], "Compute");
    assert_eq!(report["threads_per_group"], serde_json::json!([8, 1, 1]));
    assert_eq!(report["resources"].as_array().unwrap().len(), 4);
    assert!(report["msl"].as_str().unwrap().contains("kernel"));
    let temp = tempfile::tempdir().unwrap();
    let dxil = temp.path().join("compiled.dxil");
    std::fs::write(
        &dxil,
        alkyhol_shader::compile_hlsl(
            std::path::Path::new(&dxc),
            std::path::Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/compute.hlsl"
            )),
            "CSMain",
            alkyhol_shader::Stage::Compute,
        )
        .unwrap(),
    )
    .unwrap();
    let binary_result = Command::new(env!("CARGO_BIN_EXE_alkyhol-shader"))
        .arg("dxil")
        .arg(dxil)
        .arg("CSMain")
        .output()
        .unwrap();
    assert!(
        binary_result.status.success(),
        "{}",
        String::from_utf8_lossy(&binary_result.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&binary_result.stdout).unwrap(),
        report
    );
}
