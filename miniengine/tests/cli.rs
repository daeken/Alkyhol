use std::process::Command;
#[test]
fn cli_rejects_unimplemented_modes_and_bad_inputs_and_names_debug_controls() {
    let executable = env!("CARGO_BIN_EXE_alkyhol-miniengine");
    for args in [
        vec!["--mode", "shadow"],
        vec!["--camera", "5"],
        vec!["--width", "0"],
        vec!["--height", "4097"],
        vec!["--frames", "0"],
        vec!["--assets"],
        vec!["--capture"],
        vec!["--unknown"],
    ] {
        let output = Command::new(executable).args(args).output().unwrap();
        assert!(!output.status.success());
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("MiniEngine error:"));
        assert!(!error.contains("MiniEngine adapter:"));
    }
    let output = Command::new(executable).arg("--help").output().unwrap();
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for s in ["Bary Rays", "geometry", "normals", "depth", "unimplemented"] {
        assert!(help.contains(s));
    }
}
