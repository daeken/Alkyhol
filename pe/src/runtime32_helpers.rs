//! Shared PE32 name validation without the AMD64 executable runtime.
use crate::Result;
pub(crate) fn dll_basename(value: &str) -> Result<String> {
    if value.is_empty()
        || value.len() > 255
        || !value.is_ascii()
        || value.contains(['/', '\\', ':'])
        || !value.to_ascii_lowercase().ends_with(".dll")
    {
        return Err("invalid explicit DLL basename".into());
    }
    Ok(value.to_ascii_lowercase())
}
