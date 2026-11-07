use crate::Result;
use std::collections::BTreeSet;

const PAGE: usize = 4096;
const MAX_IMAGE: usize = 512 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct Section {
    pub rva: usize,
    pub size: usize,
    pub raw: usize,
    pub raw_size: usize,
    pub rights: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub module: String,
    pub symbol: String,
    pub iat: usize,
}
#[derive(Debug)]
pub struct Image<'a> {
    pub bytes: &'a [u8],
    pub base: u64,
    pub size: usize,
    pub headers: usize,
    pub entry: usize,
    pub sections: Vec<Section>,
    directories: [(usize, usize); 16],
}

fn range(bytes: &[u8], start: usize, len: usize) -> Result<&[u8]> {
    bytes
        .get(start..start.checked_add(len).ok_or("range overflow")?)
        .ok_or_else(|| "truncated file range".into())
}
fn u16_at(bytes: &[u8], off: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        range(bytes, off, 2)?.try_into().unwrap(),
    ))
}
fn u32_at(bytes: &[u8], off: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        range(bytes, off, 4)?.try_into().unwrap(),
    ))
}
fn u64_at(bytes: &[u8], off: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        range(bytes, off, 8)?.try_into().unwrap(),
    ))
}
fn round_page(n: usize) -> usize {
    (n + PAGE - 1) & !(PAGE - 1)
}

impl<'a> Image<'a> {
    /// Validate executable headers, disjoint sections and an initialized code entry.
    /// Supports conventional page-aligned AMD64 PE32+ EXEs, not DLLs/drivers.
    pub fn parse(bytes: &'a [u8]) -> Result<Self> {
        if range(bytes, 0, 2)? != b"MZ" {
            return Err("missing MZ signature".into());
        }
        let pe = u32_at(bytes, 0x3c)? as usize;
        if range(bytes, pe, 4)? != b"PE\0\0" || u16_at(bytes, pe + 4)? != 0x8664 {
            return Err("expected AMD64 PE signature".into());
        }
        let count = u16_at(bytes, pe + 6)? as usize;
        let flags = u16_at(bytes, pe + 22)?;
        if count == 0 || count > 96 || flags & 0x2002 != 2 {
            return Err("expected executable image, not DLL; 1..96 sections".into());
        }
        let opt_size = u16_at(bytes, pe + 20)? as usize;
        let opt = pe + 24;
        range(bytes, opt, opt_size)?;
        if opt_size < 112 || u16_at(bytes, opt)? != 0x20b {
            return Err("expected PE32+ optional header".into());
        }
        let align = u32_at(bytes, opt + 32)? as usize;
        let file_align = u32_at(bytes, opt + 36)? as usize;
        if !align.is_power_of_two()
            || !(PAGE..=65536).contains(&align)
            || !file_align.is_power_of_two()
            || !(512..=65536).contains(&file_align)
            || align < file_align
        {
            return Err("unsupported section/file alignment".into());
        }
        if u16_at(bytes, opt + 70)? & 0x4000 != 0 {
            return Err("unsupported Guard CF image flag".into());
        }
        let subsystem = u16_at(bytes, opt + 68)?;
        if ![2, 3].contains(&subsystem) {
            return Err("only Windows GUI/console subsystems are accepted".into());
        }
        let size = u32_at(bytes, opt + 56)? as usize;
        let headers = u32_at(bytes, opt + 60)? as usize;
        let base = u64_at(bytes, opt + 24)?;
        let entry = u32_at(bytes, opt + 16)? as usize;
        let table_end = opt + opt_size + count * 40;
        if size == 0
            || size > MAX_IMAGE
            || !size.is_multiple_of(align)
            || headers < table_end
            || headers > size
            || !headers.is_multiple_of(file_align)
            || !base.is_multiple_of(65536)
            || base.checked_add(size as u64).is_none()
        {
            return Err("invalid image size/base/headers".into());
        }
        range(bytes, 0, headers)?;
        let mut directories = [(0, 0); 16];
        let dirs = u32_at(bytes, opt + 108)? as usize;
        if dirs > 16 || 112 + dirs * 8 > opt_size {
            return Err("invalid directory count".into());
        }
        for (i, dir) in directories.iter_mut().enumerate().take(dirs) {
            *dir = (
                u32_at(bytes, opt + 112 + i * 8)? as usize,
                u32_at(bytes, opt + 116 + i * 8)? as usize,
            );
            if (dir.0 == 0) != (dir.1 == 0) {
                return Err("partial directory entry".into());
            }
        }
        if flags & 1 != 0 && directories[5].1 != 0 {
            return Err("relocations present in RELOCS_STRIPPED image".into());
        }
        let mut sections: Vec<Section> = Vec::new();
        for i in 0..count {
            let s = opt + opt_size + i * 40;
            range(bytes, s, 40)?;
            let virtual_size = u32_at(bytes, s + 8)? as usize;
            let rva = u32_at(bytes, s + 12)? as usize;
            let raw_size = u32_at(bytes, s + 16)? as usize;
            let raw = u32_at(bytes, s + 20)? as usize;
            let rights = u32_at(bytes, s + 36)?;
            let len = virtual_size.max(raw_size);
            if len == 0
                || !rva.is_multiple_of(align)
                || rva < round_page(headers)
                || rva
                    .checked_add(round_page(len))
                    .is_none_or(|end| end > size)
                || (raw_size != 0
                    && (raw < headers
                        || !raw.is_multiple_of(file_align)
                        || !raw_size.is_multiple_of(file_align)))
                || rights & 0xa0000000 == 0xa0000000
            {
                return Err("invalid section extent/alignment or writable executable code".into());
            }
            range(bytes, raw, raw_size)?;
            for old in &sections {
                if rva < old.rva + round_page(old.size) && old.rva < rva + round_page(len)
                    || raw_size != 0
                        && old.raw_size != 0
                        && raw < old.raw + old.raw_size
                        && old.raw < raw + raw_size
                {
                    return Err("overlapping sections".into());
                }
            }
            sections.push(Section {
                rva,
                size: len,
                raw,
                raw_size,
                rights,
            });
        }
        let image = Self {
            bytes,
            base,
            size,
            headers,
            entry,
            sections,
            directories,
        };
        if !image
            .sections
            .iter()
            .any(|s| s.rights & 0x20000000 != 0 && entry >= s.rva && entry < s.rva + s.raw_size)
        {
            return Err("entry is not initialized executable code".into());
        }
        for (i, &(rva, len)) in image.directories.iter().enumerate() {
            if len == 0 {
                continue;
            }
            if i == 4 {
                range(bytes, rva, len)?;
            }
            // Certificate directory uses file offsets.
            else {
                image.file_range(rva, len)?;
            }
        }
        Ok(image)
    }

    fn file_range(&self, rva: usize, len: usize) -> Result<&'a [u8]> {
        let end = rva.checked_add(len).ok_or("RVA overflow")?;
        if rva < self.headers && end <= self.headers {
            return range(self.bytes, rva, len);
        }
        for s in &self.sections {
            if rva >= s.rva && end <= s.rva + s.raw_size {
                return range(self.bytes, s.raw + rva - s.rva, len);
            }
        }
        Err(format!("unbacked RVA {rva:#x}+{len:#x}"))
    }
    fn data_target(&self, rva: usize, len: usize) -> Result<()> {
        let end = rva.checked_add(len).ok_or("target overflow")?;
        if self
            .sections
            .iter()
            .any(|s| s.rights & 0x20000000 == 0 && rva >= s.rva && end <= s.rva + s.size)
        {
            Ok(())
        } else {
            Err("fixup must target non-executable section data".into())
        }
    }
    fn string(&self, rva: usize) -> Result<String> {
        let mut name = Vec::new();
        for i in 0..4096 {
            let b = self.file_range(rva.checked_add(i).ok_or("name RVA overflow")?, 1)?[0];
            if b == 0 {
                if name.is_empty() {
                    return Err("empty import name".into());
                }
                return String::from_utf8(name).map_err(|_| "non-UTF8 import name".into());
            }
            if !b.is_ascii() {
                return Err("non-ASCII import name".into());
            }
            name.push(b);
        }
        Err("unterminated import name".into())
    }
    /// Parse bounded, named, unbound eager imports. Ordinals are unsupported.
    pub fn imports(&self) -> Result<Vec<Import>> {
        let (rva, len) = self.directories[1];
        let mut result = Vec::new();
        let mut slots = BTreeSet::new();
        if len == 0 {
            return Ok(result);
        }
        for off in (0..len.saturating_sub(19)).step_by(20) {
            let d = self.file_range(rva + off, 20)?;
            if d.iter().all(|&b| b == 0) {
                return Ok(result);
            }
            let lookup = u32_at(d, 0)? as usize;
            if u32_at(d, 4)? != 0 || u32_at(d, 8)? != 0 {
                return Err("bound/forwarded import descriptors unsupported".into());
            }
            let module = self.string(u32_at(d, 12)? as usize)?;
            let iat = u32_at(d, 16)? as usize;
            if iat == 0 || !iat.is_multiple_of(8) {
                return Err("invalid IAT".into());
            }
            let lookup = if lookup == 0 { iat } else { lookup };
            let mut terminated = false;
            for i in 0..65536 {
                let value = u64_at(self.file_range(lookup + i * 8, 8)?, 0)?;
                self.data_target(iat + i * 8, 8)?;
                if value == 0 {
                    terminated = true;
                    break;
                }
                if value >> 32 != 0 {
                    return Err("ordinal/non-RVA import unsupported".into());
                }
                let symbol = self.string(value as usize + 2)?;
                if !slots.insert(iat + i * 8) {
                    return Err("overlapping IAT slots".into());
                }
                result.push(Import {
                    module: module.clone(),
                    symbol,
                    iat: iat + i * 8,
                });
            }
            if !terminated {
                return Err("unterminated thunk table".into());
            }
        }
        Err("unterminated import descriptors".into())
    }
    /// List unique DIR64 fixups. Executable/header and overlapping fixups are rejected.
    pub fn relocations(&self) -> Result<Vec<usize>> {
        let (rva, len) = self.directories[5];
        let mut result = BTreeSet::new();
        let mut off = 0;
        while off < len {
            let header = self.file_range(rva + off, 8)?;
            let page = u32_at(header, 0)? as usize;
            let block = u32_at(header, 4)? as usize;
            if !page.is_multiple_of(PAGE)
                || block < 8
                || !block.is_multiple_of(2)
                || block > len - off
            {
                return Err("invalid relocation block".into());
            }
            for i in (8..block).step_by(2) {
                let item = u16_at(self.file_range(rva + off + i, 2)?, 0)?;
                match item >> 12 {
                    0 => (),
                    10 => {
                        let target = page + (item as usize & 0xfff);
                        self.data_target(target, 8)?;
                        if result
                            .range(target.saturating_sub(7)..target + 8)
                            .next()
                            .is_some()
                        {
                            return Err("overlapping relocation targets".into());
                        }
                        result.insert(target);
                    }
                    _ => return Err("unsupported relocation kind".into()),
                }
            }
            off += block;
        }
        Ok(result.into_iter().collect())
    }
    /// Startup gates are deliberately conservative; no silent TLS/CFG/unwind omission.
    pub fn check_startup(&self) -> Result<()> {
        for (i, name) in [
            (0, "exports"),
            (3, "exception/unwind"),
            (7, "architecture"),
            (8, "global pointer"),
            (9, "static TLS/callbacks"),
            (10, "load config/CFG"),
            (11, "bound imports"),
            (13, "delay imports"),
            (14, "CLR"),
            (15, "reserved directory"),
        ] {
            if self.directories[i].1 != 0 {
                return Err(format!("unsupported startup feature: {name}"));
            }
        }
        Ok(())
    }
}
