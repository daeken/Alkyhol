//! Read an owned SPIR-V compiler probe, validate it and print the actual WGSL.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("expected SPIR-V input")?;
    let bytes = std::fs::read(path)?;
    let mut module = alkyhol_d3d9::legacy::lower_spirv(
        &bytes,
        if std::env::args().nth(2).as_deref() == Some("ps") {
            alkyhol_d3d9::legacy::Stage::Pixel
        } else {
            alkyhol_d3d9::legacy::Stage::Vertex
        },
    )?;
    if std::env::args().nth(2).as_deref() != Some("ps") {
        alkyhol_d3d9::legacy::vertex_pixel_centers(&mut module)?;
    }
    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    )
    .validate(&module)?;
    let text = naga::back::wgsl::write_string(
        &module,
        &info,
        naga::back::wgsl::WriterFlags::EXPLICIT_TYPES,
    )?;
    println!("{text}");
    Ok(())
}
