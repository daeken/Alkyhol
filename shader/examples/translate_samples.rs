//! Rebuild checked-in generated shader source and binding metadata, or verify it.
use alkyhol_shader::{HlslOptions, Stage, compile_hlsl_with_options, translate_dxil};
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let dxc = args
        .first()
        .ok_or("usage: translate_samples DXC [--check]")?;
    let check = args.get(1).is_some_and(|s| s == "--check");
    if args.len() > 2 || (args.len() == 2 && !check) {
        return Err("unexpected arguments".into());
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (input, entry, output) in [
        (
            "samples/miniengine/bary.hlsl",
            "AlkyholMain",
            "samples/miniengine/bary.generated.json",
        ),
        (
            "samples/miniengine/camera.hlsl",
            "AlkyholMain",
            "samples/miniengine/camera.generated.json",
        ),
    ] {
        let bytes = compile_hlsl_with_options(
            Path::new(dxc),
            &root.join(input),
            entry,
            Stage::Compute,
            &HlslOptions {
                shader_model_6_5: true,
                ..Default::default()
            },
        )?;
        let shader = translate_dxil(&bytes, entry)?;
        let text = serde_json::to_string_pretty(&shader)? + "\n";
        let path = root.join(output);
        if check {
            if std::fs::read_to_string(&path)? != text {
                return Err(format!("generated shader differs: {output}").into());
            }
        } else {
            std::fs::write(path, text)?;
        }
        println!("{output}: {}", if check { "verified" } else { "generated" });
    }
    Ok(())
}
