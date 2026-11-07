use alkyhol_shader::{Stage, compile_hlsl, translate_dxil};
use std::path::Path;

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (bytes, entry) = match args.as_slice() {
        [mode, input, entry] if mode == "dxil" => (std::fs::read(input).map_err(|e|e.to_string())?, entry),
        [mode, dxc, input, entry, stage] if mode == "hlsl" => {
            let stage = match stage.as_str() { "vs" => Stage::Vertex, "ps" => Stage::Pixel, "cs" => Stage::Compute, _ => return Err("stage must be vs, ps or cs".into()) };
            (compile_hlsl(Path::new(dxc),Path::new(input),entry,stage)?,entry)
        }
        _ => return Err("usage: alkyhol-shader dxil INPUT ENTRY | hlsl DXC INPUT ENTRY vs|ps|cs\nPrints JSON containing MSL and runtime binding metadata.".into()),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&translate_dxil(&bytes, entry)?).map_err(|e| e.to_string())?
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
