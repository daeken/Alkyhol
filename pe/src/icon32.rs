//! Resource-backed shared icons in the classic 32-logical-pixel guest UI profile.
//! Loading retains real decoded pixels/masks; this module does not draw a window.
use super::{LoadedImage32, unsupported};
use crate::{Image, ResourceName, Result};
use std::collections::BTreeMap;

/// Read-only copy of an icon loaded by actual guest code. Pixels are top-down
/// BGRA8 source values; mask rows are top-down DWORD-aligned, most-significant
/// bit first. This snapshot is not evidence that a native window drew the icon.
#[derive(Clone)]
pub struct IconImage32 {
    pub handle: u32,
    pub module: u32,
    pub width: u32,
    pub height: u32,
    pub bgra: Vec<u8>,
    pub and_mask: Vec<u8>,
}
struct Resource {
    kind: u16,
    id: u16,
    language: u16,
    bytes: Vec<u8>,
}
#[derive(Default)]
pub(super) struct State {
    modules: BTreeMap<u32, Result<Vec<Resource>>>,
    loaded: BTreeMap<(u32, u16), IconImage32>,
}
fn word(b: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(at..at + 2)
            .ok_or("truncated icon metadata")?
            .try_into()
            .unwrap(),
    ))
}
fn dword(b: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(at..at + 4)
            .ok_or("truncated icon metadata")?
            .try_into()
            .unwrap(),
    ))
}
fn selected(group: &[u8]) -> Result<(u16, usize)> {
    if word(group, 0)? != 0 || word(group, 2)? != 1 {
        return Err("invalid group-icon header".into());
    }
    let count = word(group, 4)? as usize;
    if count == 0 || count > 128 || group.len() != 6 + count * 14 {
        return Err("invalid group-icon entry count/size".into());
    }
    let mut result = None;
    for i in 0..count {
        let at = 6 + i * 14;
        if group[at] == 32
            && group[at + 1] == 32
            && word(group, at + 4)? == 1
            && word(group, at + 6)? == 32
        {
            if group[at + 2] != 0 || group[at + 3] != 0 || result.is_some() {
                return Err("ambiguous/invalid exact 32px true-color icon".into());
            }
            result = Some((word(group, at + 12)?, dword(group, at + 8)? as usize));
        }
    }
    result.ok_or("icon requires one exact 32px 32bpp candidate; scaling/other display formats are unsupported".into())
}
fn decode(bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
    // BITMAPINFOHEADER, positive doubled height, BI_RGB; no palette, compression
    // or automatic repair of original color/alpha/mask values.
    if dword(bytes, 0)? != 40
        || dword(bytes, 4)? != 32
        || dword(bytes, 8)? != 64
        || word(bytes, 12)? != 1
        || word(bytes, 14)? != 32
        || dword(bytes, 16)? != 0
        || dword(bytes, 32)? != 0
        || dword(bytes, 36)? != 0
    {
        return Err("icon requires a 32px 32bpp BI_RGB DIB with an AND mask".into());
    }
    let image_size = dword(bytes, 20)?;
    if ![0, 4096, 4224].contains(&image_size) || bytes.len() != 40 + 4096 + 128 {
        return Err("icon DIB pixel/mask extent differs from its header".into());
    }
    let mut pixels = vec![0; 4096];
    let mut mask = vec![0; 128];
    for row in 0..32 {
        pixels[row * 128..(row + 1) * 128]
            .copy_from_slice(&bytes[40 + (31 - row) * 128..40 + (32 - row) * 128]);
        mask[row * 4..(row + 1) * 4]
            .copy_from_slice(&bytes[4136 + (31 - row) * 4..4136 + (32 - row) * 4]);
    }
    Ok((pixels, mask))
}
impl State {
    pub(super) fn register(&mut self, base: u32, image: &Image<'_>) {
        // Defer unsupported/malformed resources until this module is queried.
        let resources = (|| {
            let mut out = Vec::new();
            let mut total = 0usize;
            for r in image.resources()? {
                if let (ResourceName::Id(kind @ (3 | 14)), ResourceName::Id(id)) = (r.kind, r.name)
                {
                    if image.sections.iter().any(|s| {
                        r.data_rva >= s.rva
                            && r.data_rva < s.rva + s.size
                            && s.rights & 0x80000000 != 0
                    }) {
                        return Err("mutable icon resources are unsupported".into());
                    }
                    total = total
                        .checked_add(r.bytes.len())
                        .ok_or("icon resource size overflow")?;
                    if total > 8 * 1024 * 1024 || out.len() >= 256 {
                        return Err("icon resource inventory budget exceeded".into());
                    }
                    out.push(Resource {
                        kind,
                        id,
                        language: r.language,
                        bytes: r.bytes.to_vec(),
                    });
                }
            }
            Ok(out)
        })();
        self.modules.insert(base, resources);
    }
    fn load(&mut self, module: u32, id: u16) -> Result<Option<u32>> {
        if let Some(image) = self.loaded.get(&(module, id)) {
            return Ok(Some(image.handle));
        }
        let resources = self
            .modules
            .get(&module)
            .ok_or("LoadIconW requires a registered guest module")?
            .as_ref()
            .map_err(Clone::clone)?;
        let groups: Vec<_> = resources
            .iter()
            .filter(|r| r.kind == 14 && r.id == id)
            .collect();
        if groups.is_empty() {
            return Ok(None);
        }
        if groups.len() != 1 {
            return Err("icon language selection requires a unique resource language".into());
        }
        let (image_id, size) = selected(&groups[0].bytes)?;
        let images: Vec<_> = resources
            .iter()
            .filter(|r| r.kind == 3 && r.id == image_id && r.language == groups[0].language)
            .collect();
        if images.len() != 1 || images[0].bytes.len() != size {
            return Err("selected group icon does not match a unique image resource".into());
        }
        let (bgra, and_mask) = decode(&images[0].bytes)?;
        if self.loaded.len() >= 128 {
            return Err("loaded icon budget exceeded".into());
        }
        let handle = alkyhol::kernel::reserve_handle_token()
            .map_err(|e| format!("icon handle allocation: {e:?}"))?;
        let handle = u32::try_from(handle).map_err(|_| "icon handle exceeds PE32")?;
        if (0x70000001..=0x70000003).contains(&handle) {
            return Err("icon handle overlaps standard-stream tokens".into());
        }
        self.loaded.insert(
            (module, id),
            IconImage32 {
                handle,
                module,
                width: 32,
                height: 32,
                bgra,
                and_mask,
            },
        );
        Ok(Some(handle))
    }
    pub(super) fn images(&self) -> Vec<IconImage32> {
        self.loaded.values().cloned().collect()
    }
}
pub(super) fn load(image: &LoadedImage32<'_>, args: &[u32; 8]) -> u32 {
    if args[0] == 0 || args[1] == 0 || args[1] > u16::MAX as u32 {
        unsupported("LoadIconW system icons and string names are unsupported");
    }
    match image
        .cpu
        .icons
        .borrow_mut()
        .load(args[0], args[1] as u16)
        .unwrap_or_else(|e| unsupported(&e))
    {
        Some(handle) => handle,
        None => {
            alkyhol::ffi::SetLastError(1814);
            0
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn dib() -> Vec<u8> {
        let mut b = vec![0; 4264];
        b[..4].copy_from_slice(&40u32.to_le_bytes());
        b[4..8].copy_from_slice(&32u32.to_le_bytes());
        b[8..12].copy_from_slice(&64u32.to_le_bytes());
        b[12..14].copy_from_slice(&1u16.to_le_bytes());
        b[14..16].copy_from_slice(&32u16.to_le_bytes());
        b
    }
    #[test]
    fn source_rows_alpha_and_mask_bits_are_preserved_without_repair() {
        let mut b = dib();
        b[40..44].copy_from_slice(&[9, 8, 7, 0]);
        b[40 + 31 * 128..44 + 31 * 128].copy_from_slice(&[1, 2, 3, 85]);
        b[4136] = 0x80;
        b[4136 + 31 * 4] = 0x40;
        let (pixels, mask) = decode(&b).unwrap();
        assert_eq!(&pixels[..4], &[1, 2, 3, 85]);
        assert_eq!(&pixels[31 * 128..31 * 128 + 4], &[9, 8, 7, 0]);
        assert_eq!(mask[0], 0x40);
        assert_eq!(mask[124], 0x80);
        for at in [0, 4, 8, 12, 14, 16, 32, 36] {
            let mut bad = b.clone();
            bad[at] ^= 1;
            assert!(decode(&bad).is_err());
        }
        assert!(decode(&b[..4263]).is_err());
        b.push(0);
        assert!(decode(&b).is_err());
    }
    #[test]
    fn exact_group_choice_rejects_ambiguous_size_and_format() {
        let mut group = vec![0; 20];
        group[2] = 1;
        group[4] = 1;
        group[6] = 32;
        group[7] = 32;
        group[10] = 1;
        group[12] = 32;
        group[14..18].copy_from_slice(&4264u32.to_le_bytes());
        group[18] = 7;
        assert_eq!(selected(&group).unwrap(), (7, 4264));
        let mut duplicate = group.clone();
        duplicate[4] = 2;
        duplicate.extend_from_slice(&group[6..]);
        assert!(selected(&duplicate).is_err());
        group[6] = 16;
        assert!(selected(&group).is_err());
    }
}
