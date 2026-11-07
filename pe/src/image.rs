use crate::Result;
use std::collections::BTreeSet;

const PAGE: usize = 4096;
// Bounded virtual address extent; sparse BSS is demand-zero, not a Vec allocation.
const MAX_IMAGE: usize = 1024 * 1024 * 1024;

/// Direct AMD64 runtime-function record. All three DWORDs are image-relative.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeFunction {
    pub begin: u32,
    pub end: u32,
    pub unwind: u32,
}
/// Validated static table; `rva` identifies the original image records.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFunctionTable {
    pub rva: usize,
    pub entries: Vec<RuntimeFunction>,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub rva: usize,
    pub size: usize,
    pub raw: usize,
    pub raw_size: usize,
    pub rights: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ImportSymbol {
    Name(String),
    /// Low 16-bit biased ordinal. Zero is preserved as metadata, but the
    /// bounded runtime does not resolve it without native evidence.
    Ordinal(u16),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Import {
    pub module: String,
    pub symbol: ImportSymbol,
    pub iat: usize,
}
/// An export can address code or data. Forwarder text is retained verbatim;
/// metadata parsing does not resolve it or make its target callable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ExportTarget {
    Address { rva: usize },
    Forwarder(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Export {
    pub ordinal: u32,
    pub names: Vec<String>,
    pub target: ExportTarget,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExportTable {
    pub dll_name: String,
    pub ordinal_base: u32,
    /// Sparse ordinal holes are omitted; explicit ordinals retain their bias.
    pub entries: Vec<Export>,
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
    is_dll: bool,
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
        Self::parse_kind(bytes, false)
    }
    /// Inspect a bounded AMD64 DLL without mapping or calling its entry. A zero
    /// DLL entry is allowed; a nonzero entry must still be initialized code.
    /// Parsing never initializes the DLL; runtime loading is a separate API.
    pub fn parse_dll(bytes: &'a [u8]) -> Result<Self> {
        Self::parse_kind(bytes, true)
    }
    fn parse_kind(bytes: &'a [u8], is_dll: bool) -> Result<Self> {
        if range(bytes, 0, 2)? != b"MZ" {
            return Err("missing MZ signature".into());
        }
        let pe = u32_at(bytes, 0x3c)? as usize;
        if range(bytes, pe, 4)? != b"PE\0\0" || u16_at(bytes, pe + 4)? != 0x8664 {
            return Err("expected AMD64 PE signature".into());
        }
        let count = u16_at(bytes, pe + 6)? as usize;
        let flags = u16_at(bytes, pe + 22)?;
        let expected = if is_dll { 0x2002 } else { 2 };
        if count == 0 || count > 96 || flags & 0x2002 != expected {
            return Err("wrong EXE/DLL image kind or section count outside 1..96".into());
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
            is_dll,
        };
        if !(is_dll && entry == 0)
            && !image
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
    /// Parse bounded named/ordinal unbound eager imports. Reserved thunk bits
    /// must be zero; named hints and names must be file-backed. No resolution
    /// or promise about native sparse-ordinal behavior follows from parsing.
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
                let symbol = if value & (1 << 63) != 0 {
                    if value & !((1 << 63) | 0xffff) != 0 {
                        return Err("nonzero reserved ordinal thunk bits".into());
                    }
                    ImportSymbol::Ordinal(value as u16)
                } else {
                    if value >> 31 != 0 {
                        return Err("nonzero reserved name thunk bits".into());
                    }
                    self.file_range(value as usize, 2)?;
                    ImportSymbol::Name(self.string(value as usize + 2)?)
                };
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
    /// Read bounded named/ordinal export metadata without loading dependencies.
    /// Code and mapped data RVAs are retained; absolute/out-of-image targets,
    /// unsorted/duplicate names, named holes and more than 65536 slots/names fail.
    /// A forwarder's entire ASCII string must lie inside the export directory.
    pub fn exports(&self) -> Result<Option<ExportTable>> {
        let (rva, length) = self.directories[0];
        if length == 0 {
            return Ok(None);
        }
        if length < 40 {
            return Err("truncated export directory".into());
        }
        let directory = self.file_range(rva, 40)?;
        if u32_at(directory, 0)? != 0 {
            return Err("nonzero reserved export flags".into());
        }
        let name_rva = u32_at(directory, 12)? as usize;
        if name_rva == 0 {
            return Err("missing DLL export name".into());
        }
        let dll_name = self.string(name_rva)?;
        let ordinal_base = u32_at(directory, 16)?;
        let count = u32_at(directory, 20)? as usize;
        let names_count = u32_at(directory, 24)? as usize;
        if count > 65536 || names_count > 65536 || (count == 0 && names_count != 0) {
            return Err("unsupported export table counts".into());
        }
        if count != 0 && ordinal_base.checked_add((count - 1) as u32).is_none() {
            return Err("export ordinal overflow".into());
        }
        let table = if count == 0 {
            &[]
        } else {
            let address = u32_at(directory, 28)? as usize;
            if address == 0 {
                return Err("missing export address table".into());
            }
            self.file_range(address, count * 4)?
        };
        let mut names: Vec<Vec<String>> = vec![Vec::new(); count];
        if names_count != 0 {
            let names_rva = u32_at(directory, 32)? as usize;
            let ordinals_rva = u32_at(directory, 36)? as usize;
            if names_rva == 0 || ordinals_rva == 0 {
                return Err("missing export name/ordinal table".into());
            }
            let pointers = self.file_range(names_rva, names_count * 4)?;
            let ordinals = self.file_range(ordinals_rva, names_count * 2)?;
            let mut previous: Option<String> = None;
            for i in 0..names_count {
                let name = self.string(u32_at(pointers, i * 4)? as usize)?;
                if previous.as_ref().is_some_and(|p| p >= &name) {
                    return Err("export names must be unique and lexically sorted".into());
                }
                let ordinal_index = u16_at(ordinals, i * 2)? as usize;
                if ordinal_index >= count || u32_at(table, ordinal_index * 4)? == 0 {
                    return Err("export name refers to missing address slot".into());
                }
                previous = Some(name.clone());
                names[ordinal_index].push(name);
            }
        }
        let mut entries = Vec::new();
        for (i, names) in names.into_iter().enumerate() {
            let address = u32_at(table, i * 4)? as usize;
            if address == 0 {
                continue;
            }
            let target = if (rva..rva + length).contains(&address) {
                let max = (rva + length - address).min(4096);
                let mut text = Vec::new();
                let mut terminated = false;
                for offset in 0..max {
                    let byte = self.file_range(address + offset, 1)?[0];
                    if byte == 0 {
                        terminated = true;
                        break;
                    }
                    if !byte.is_ascii() {
                        return Err("non-ASCII export forwarder".into());
                    }
                    text.push(byte);
                }
                if !terminated || text.is_empty() {
                    return Err("unterminated/empty bounded export forwarder".into());
                }
                ExportTarget::Forwarder(String::from_utf8(text).unwrap())
            } else {
                if !(address < self.headers
                    || self
                        .sections
                        .iter()
                        .any(|s| address >= s.rva && address < s.rva + s.size))
                {
                    return Err("export address is not in mapped image bytes".into());
                }
                ExportTarget::Address { rva: address }
            };
            entries.push(Export {
                ordinal: ordinal_base + i as u32,
                names,
                target,
            });
        }
        Ok(Some(ExportTable {
            dll_name,
            ordinal_base,
            entries,
        }))
    }
    /// Inspect a bounded static AMD64 table and supported v1 body recipes.
    /// Records/code/unwind slots must be backed by readonly/executable image data.
    /// This does not map/register code or enable exception dispatch.
    pub fn runtime_functions(&self) -> Result<Option<RuntimeFunctionTable>> {
        let (rva, length) = self.directories[3];
        if length == 0 {
            return Ok(None);
        }
        if !rva.is_multiple_of(4) || !length.is_multiple_of(12) || length / 12 > 65536 {
            return Err("invalid static runtime-function table shape".into());
        }
        // Loader fixups must not mutate the metadata after this preflight.
        let mut writes: Vec<usize> = self
            .imports()?
            .into_iter()
            .map(|i| i.iat)
            .chain(self.relocations()?)
            .collect();
        writes.sort_unstable();
        let readonly = |address: usize, length: usize| -> Result<()> {
            let end = address
                .checked_add(length)
                .ok_or("static metadata extent overflow")?;
            let first = writes.partition_point(|&w| w + 8 <= address);
            if writes.get(first).is_some_and(|&w| w < end) {
                return Err("loader fixup overlaps static function metadata".into());
            }
            if !self.sections.iter().any(|s| {
                address >= s.rva && end <= s.rva + s.raw_size && s.rights & 0xe0000000 == 0x40000000
            }) {
                return Err(
                    "static metadata must be initialized readonly non-executable data".into(),
                );
            }
            Ok(())
        };
        readonly(rva, length)?;
        let bytes = self.file_range(rva, length)?;
        let mut entries = Vec::with_capacity(length / 12);
        let mut previous = 0;
        for row in bytes.as_chunks::<12>().0 {
            let e = RuntimeFunction {
                begin: u32_at(row, 0)?,
                end: u32_at(row, 4)?,
                unwind: u32_at(row, 8)?,
            };
            if e.begin >= e.end || e.begin < previous || e.unwind == 0 || e.unwind & 3 != 0 {
                return Err("invalid sorted/direct static function record".into());
            }
            let begin = e.begin as usize;
            let end = e.end as usize;
            if !self.sections.iter().any(|s| {
                begin >= s.rva && end <= s.rva + s.raw_size && s.rights & 0x60000000 == 0x60000000
            }) {
                return Err("static function range is not initialized RX code".into());
            }
            readonly(e.unwind as usize, 4)?;
            let header: [u8; 4] = self.file_range(e.unwind as usize, 4)?.try_into().unwrap();
            let slots = usize::from(header[2]) * 2;
            let padded = (slots + 3) & !3;
            readonly(e.unwind as usize, 4 + padded)?;
            if u32::from(header[1]) > e.end - e.begin {
                return Err("static prologue exceeds function".into());
            }
            crate::unwind_info::decode(
                header,
                self.file_range(e.unwind as usize + 4, slots)?,
                u64::from(header[1]),
                true,
            )
            .map_err(|e| format!("unsupported static unwind recipe: {e}"))?;
            previous = e.end;
            entries.push(e);
        }
        Ok(Some(RuntimeFunctionTable { rva, entries }))
    }
    /// Preflight an EXE for the context-borrowing RegisteredImage API.
    /// Only validated static body recipes are added; TLS/CFG/handlers still fail.
    pub fn check_registered_startup(&self) -> Result<()> {
        if self.is_dll {
            return Err("registered EXE API does not initialize DLLs".into());
        }
        if self.directories[0].1 != 0 {
            return Err("unsupported startup feature: exports".into());
        }
        self.check_auxiliary_startup_except_static()?;
        self.runtime_functions()?;
        Ok(())
    }
    /// Preflight an isolated DLL for RegisteredLibrary, including static records.
    /// This inspects metadata only; imports/forwarders are resolved before attach.
    pub fn check_registered_library_startup(&self) -> Result<()> {
        if !self.is_dll {
            return Err("registered library API requires a DLL".into());
        }
        self.check_auxiliary_startup_except_static()?;
        self.runtime_functions()?;
        self.exports()?;
        Ok(())
    }
    /// Startup gates are deliberately conservative; no silent TLS/CFG/unwind omission.
    pub fn check_startup(&self) -> Result<()> {
        if self.is_dll {
            return Err("DLL image requires the explicit isolated library API".into());
        }
        if self.directories[0].1 != 0 {
            return Err("unsupported startup feature: exports".into());
        }
        self.check_auxiliary_startup()
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    pub(crate) fn check_library_startup(&self) -> Result<()> {
        if !self.is_dll {
            return Err("expected DLL image".into());
        }
        self.check_auxiliary_startup()
    }
    fn check_auxiliary_startup(&self) -> Result<()> {
        if self.directories[3].1 != 0 {
            return Err(
                "unsupported startup feature: exception/unwind (use registered EXE API)".into(),
            );
        }
        self.check_auxiliary_startup_except_static()
    }
    fn check_auxiliary_startup_except_static(&self) -> Result<()> {
        for (i, name) in [
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
