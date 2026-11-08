//! Checked PE32 filesystem search gateway. Unsupported search semantics stop;
//! observations do not become invented successful handles or missing files.
use super::{LoadedImage32, startup32, unsupported};

pub(super) fn first(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    let units = startup32::read_units::<u16>(image, args[0], -1)
        .unwrap_or_else(|| unsupported("unsupported PE32 file search pattern"));
    if image.cpu.diagnostic.is_some() {
        eprintln!(
            "PE32 diagnostic file search: units_len={}; units_prefix={:x?}",
            units.len(),
            &units[..units.len().min(512)]
        );
    }
    unsupported("PE32 filesystem search semantics are not implemented")
}
