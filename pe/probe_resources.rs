//! Inspect resources without mapping or executing any supplied image.
use alkyhol_pe::{Image, ResourceName};
use std::io::Write;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("expected PE32 path")?;
    let bytes = std::fs::read(path)?;
    let image = Image::parse_i386(&bytes)?;
    let resources = image.resources()?;
    let keys = [
        (ResourceName::Id(24), ResourceName::Id(1), 1033),
        (ResourceName::Id(24), ResourceName::Id(2), 1033),
        (
            ResourceName::Id(10),
            ResourceName::Name("Λ🌸".encode_utf16().collect()),
            1041,
        ),
    ];
    let mut out = std::io::stdout().lock();
    for (kind, name, language) in keys {
        let resource = resources
            .iter()
            .find(|r| r.kind == kind && r.name == name && r.language == language)
            .ok_or("missing owned resource")?;
        out.write_all(&(resource.bytes.len() as u32).to_le_bytes())?;
        out.write_all(resource.bytes)?;
    }
    Ok(())
}
