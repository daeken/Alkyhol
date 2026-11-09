use std::{env, path::PathBuf, process::Command};
fn run(command: &mut Command) {
    assert!(
        command
            .status()
            .expect("native UI compiler available")
            .success(),
        "native UI glue compilation failed"
    );
}
fn main() {
    println!("cargo:rerun-if-changed=src/screen_capture32.m");
    if env::var("TARGET").unwrap() != "x86_64-apple-darwin" {
        return;
    }
    let out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let object = out.join("screen_capture32.o");
    let library = out.join("libalkyhol_pe_screen.a");
    run(Command::new("xcrun")
        .args([
            "--sdk",
            "macosx",
            "clang",
            "-arch",
            "x86_64",
            "-mmacosx-version-min=14.0",
            "-fobjc-arc",
            "-Wall",
            "-Wextra",
            "-Werror",
            "-c",
            "src/screen_capture32.m",
            "-o",
        ])
        .arg(&object));
    run(Command::new("ar").arg("crs").arg(&library).arg(&object));
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=alkyhol_pe_screen");
    for framework in [
        "ScreenCaptureKit",
        "Foundation",
        "CoreGraphics",
        "CoreFoundation",
    ] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
}
