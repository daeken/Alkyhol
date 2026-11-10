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
    println!("cargo:rerun-if-changed=src/audio_host32.c");
    println!("cargo:rerun-if-changed=src/sound_host32.c");
    println!("cargo:rerun-if-changed=src/filesystem_host32.c");
    println!("cargo:rerun-if-changed=src/network_host32.c");
    println!("cargo:rerun-if-changed=src/memory_host32.c");
    let target = env::var("TARGET").unwrap();
    let native =
        target == "aarch64-apple-darwin" && env::var_os("CARGO_FEATURE_NATIVE_RUNTIME32").is_some();
    if target != "x86_64-apple-darwin" && !native {
        return;
    }
    let arch = if native { "arm64" } else { "x86_64" };
    if native {
        println!("cargo:rerun-if-changed=src");
        for name in ["ci_sqrt32", "ci_atan32", "raise32"] {
            println!("cargo:rerun-if-changed=src/{name}.S");
        }
        println!("cargo:rerun-if-changed=tools/extract_guest_templates.py");
        run(Command::new("python3")
            .arg("tools/extract_guest_templates.py")
            .arg(env::var_os("OUT_DIR").unwrap()));
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
            arch,
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
    let mut native = Vec::new();
    for name in [
        "audio_host32",
        "sound_host32",
        "filesystem_host32",
        "network_host32",
        "memory_host32",
    ] {
        let object = out.join(format!("{name}.o"));
        run(Command::new("xcrun")
            .args([
                "--sdk",
                "macosx",
                "clang",
                "-arch",
                arch,
                "-mmacosx-version-min=14.0",
                "-Wall",
                "-Wextra",
                "-Werror",
                "-c",
            ])
            .arg(format!("src/{name}.c"))
            .arg("-o")
            .arg(&object));
        native.push(object);
    }
    run(Command::new("ar")
        .arg("crs")
        .arg(&library)
        .arg(&object)
        .args(&native));
    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-lib=static=alkyhol_pe_screen");
    for framework in [
        "ScreenCaptureKit",
        "Foundation",
        "CoreGraphics",
        "CoreFoundation",
        "CoreAudio",
        "AudioToolbox",
    ] {
        println!("cargo:rustc-link-lib=framework={framework}");
    }
}
