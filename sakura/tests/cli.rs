use std::process::Command;
#[test]
fn cli_rejects_invalid_arguments_before_device_or_asset_access_and_describes_controls() {
    let executable = env!("CARGO_BIN_EXE_alkyhol-sakura");
    for args in [
        vec!["--unknown"],
        vec!["--width"],
        vec!["--height", "no"],
        vec!["--frames", "0"],
        vec!["--time", "NaN"],
        vec!["--width", "0"],
        vec!["--height", "4097"],
        vec!["--assets"],
        vec!["--capture"],
    ] {
        let output = Command::new(executable).args(&args).output().unwrap();
        assert!(!output.status.success(), "{args:?}");
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains("Sakura error:"));
        assert!(!error.contains("Sakura adapter:"));
    }
    let help = Command::new(executable).arg("--help").output().unwrap();
    assert!(help.status.success());
    let text = String::from_utf8(help.stdout).unwrap();
    for s in [
        "SER off",
        "WASD",
        "Q/E",
        "--headless",
        "--capture",
        "--assets",
    ] {
        assert!(text.contains(s));
    }
}
