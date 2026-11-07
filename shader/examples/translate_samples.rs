//! Rebuild checked-in generated shader source and binding metadata, or verify it.
use alkyhol_shader::{
    HlslOptions, MslOptions, Stage, compile_hlsl_with_options, translate_dxil_with_options,
};
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
    for (input, entry, stage, output) in [
        (
            "samples/miniengine/bary.hlsl",
            "AlkyholMain",
            Stage::Compute,
            "samples/miniengine/bary.generated.json",
        ),
        (
            "samples/miniengine/camera.hlsl",
            "AlkyholMain",
            Stage::Compute,
            "samples/miniengine/camera.generated.json",
        ),
        (
            "samples/miniengine/original/ToneMapCS.hlsl",
            "main",
            Stage::Compute,
            "samples/miniengine/tone.generated.json",
        ),
        (
            "samples/miniengine/original/PresentSDRPS.hlsl",
            "main",
            Stage::Pixel,
            "samples/miniengine/display.generated.json",
        ),
        (
            "samples/miniengine/original/ScreenQuadPresentVS.hlsl",
            "main",
            Stage::Vertex,
            "samples/miniengine/screen.generated.json",
        ),
        (
            "samples/miniengine/original/BufferCopyPS.hlsl",
            "main",
            Stage::Pixel,
            "samples/miniengine/copy.generated.json",
        ),
        (
            "samples/miniengine/original/BilinearUpsamplePS.hlsl",
            "main",
            Stage::Pixel,
            "samples/miniengine/upsample.generated.json",
        ),
    ] {
        let bytes = compile_hlsl_with_options(
            Path::new(dxc),
            &root.join(input),
            entry,
            stage,
            &HlslOptions {
                shader_model_6_5: true,
                strip_root_signature: true,
                defines: &[("SUPPORT_TYPED_UAV_LOADS", "1")],
            },
        )?;
        let shader = translate_dxil_with_options(
            &bytes,
            entry,
            &MslOptions {
                fragment_output_components: if stage == Stage::Pixel {
                    &[(0, 4)]
                } else {
                    &[]
                },
            },
        )?;
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
