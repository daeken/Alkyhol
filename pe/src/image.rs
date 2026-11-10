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
/// Checked PE32 static TLS metadata in image-relative coordinates. Inspection
/// neither initializes TLS nor calls callbacks. Runtime support is separate.
/// Requested zero-fill/total extents describe the file, not a native heap query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticTls32 {
    pub template_rva: usize,
    pub template_size: usize,
    pub zero_fill: usize,
    pub total_size: usize,
    pub index_rva: usize,
    pub alignment: usize,
    pub callbacks: Vec<usize>,
}
/// Inspected legacy PE32 load configuration. RVAs remain guest-image offsets;
/// inspection does not seed cookies, register handlers or dispatch exceptions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LegacyLoadConfig32 {
    pub directory_size: usize,
    pub declared_size: usize,
    pub timestamp: u32,
    pub major_version: u16,
    pub minor_version: u16,
    pub cookie_rva: Option<usize>,
    pub seh_table_rva: Option<usize>,
    pub seh_handlers: Vec<usize>,
}
#[derive(Debug)]
pub struct Image<'a> {
    pub bytes: &'a [u8],
    pub base: u64,
    pub size: usize,
    declared_size: usize,
    pub headers: usize,
    pub entry: usize,
    pub sections: Vec<Section>,
    directories: [(usize, usize); 16],
    is_dll: bool,
    i386: bool,
    dll_characteristics: u16,
    pub(crate) large_address_aware: bool,
    diagnostic: bool,
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
    /// Parse an explicit diagnostic PE32 view. Immutable executable metadata and
    /// initialized executable IAT storage are admitted; normal constructors keep
    /// their stricter contract. This constructor executes no code.
    pub fn parse_i386_diagnostic(bytes: &'a [u8], dll: bool) -> Result<Self> {
        let mut image = Self::parse_format(bytes, dll, true)?;
        image.diagnostic = true;
        Ok(image)
    }

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
    /// Inspect a conventional i386 PE32 EXE for the separate experimental runtime.
    /// This does not enable PE32 in the existing AMD64 loader APIs.
    pub fn parse_i386(bytes: &'a [u8]) -> Result<Self> {
        Self::parse_format(bytes, false, true)
    }
    /// Inspect an i386 PE32 DLL without initializing it. DLL startup requires
    /// the explicit PE32 library runtime; EXE and AMD64 APIs remain separate.
    pub fn parse_i386_dll(bytes: &'a [u8]) -> Result<Self> {
        Self::parse_format(bytes, true, true)
    }
    /// Machine pointer width selected by the validated PE header.
    pub fn bitness(&self) -> u8 {
        if self.i386 { 32 } else { 64 }
    }
    /// Original SizeOfImage header field, retained without modifying PE bytes.
    /// The allocation extent (`size`) rounds supported PE32 4-KiB layouts up.
    pub fn declared_size(&self) -> usize {
        self.declared_size
    }
    /// Raw certificate file-offset/size metadata, never a signature/trust result.
    /// PE32 ordinary loading may retain a pair outside the input file. No bytes
    /// are read through this pair; callers must bound any separate inspection.
    pub fn certificate_directory(&self) -> Option<(usize, usize)> {
        (self.directories[4].1 != 0).then_some(self.directories[4])
    }
    fn parse_kind(bytes: &'a [u8], is_dll: bool) -> Result<Self> {
        Self::parse_format(bytes, is_dll, false)
    }
    fn parse_format(bytes: &'a [u8], is_dll: bool, i386: bool) -> Result<Self> {
        if range(bytes, 0, 2)? != b"MZ" {
            return Err("missing MZ signature".into());
        }
        let pe = u32_at(bytes, 0x3c)? as usize;
        if range(bytes, pe, 4)? != b"PE\0\0"
            || u16_at(bytes, pe + 4)? != if i386 { 0x14c } else { 0x8664 }
        {
            return Err("unexpected PE machine/signature".into());
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
        let directory_offset = if i386 { 96 } else { 112 };
        if opt_size < directory_offset || u16_at(bytes, opt)? != if i386 { 0x10b } else { 0x20b } {
            return Err("unexpected PE optional header format".into());
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
        if i386 && u16_at(bytes, opt + 70)? & 0x80 != 0 {
            return Err("PE32 Code Integrity enforcement is unsupported".into());
        }
        let subsystem = u16_at(bytes, opt + 68)?;
        if ![2, 3].contains(&subsystem) {
            return Err("only Windows GUI/console subsystems are accepted".into());
        }
        let declared_size = u32_at(bytes, opt + 56)? as usize;
        let size = if i386 && align == PAGE {
            // Native geometry observations accept a partial final image page.
            // Keep the header bytes intact; only the checked mapping span rounds.
            declared_size
                .checked_add(PAGE - 1)
                .ok_or("PE32 image extent overflow")?
                & !(PAGE - 1)
        } else {
            if i386 && !declared_size.is_multiple_of(align) {
                return Err("unaligned PE32 SizeOfImage requires 4-KiB section alignment".into());
            }
            declared_size
        };
        let headers = u32_at(bytes, opt + 60)? as usize;
        let base = if i386 {
            u64::from(u32_at(bytes, opt + 28)?)
        } else {
            u64_at(bytes, opt + 24)?
        };
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
            || (i386 && base + size as u64 > 0x100000000)
        {
            return Err("invalid image size/base/headers".into());
        }
        range(bytes, 0, headers)?;
        let mut directories = [(0, 0); 16];
        let dirs = u32_at(bytes, opt + directory_offset - 4)? as usize;
        if dirs > 16 || directory_offset + dirs * 8 > opt_size {
            return Err("invalid directory count".into());
        }
        for (i, dir) in directories.iter_mut().enumerate().take(dirs) {
            *dir = (
                u32_at(bytes, opt + directory_offset + i * 8)? as usize,
                u32_at(bytes, opt + directory_offset + 4 + i * 8)? as usize,
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
            declared_size,
            headers,
            entry,
            sections,
            directories,
            is_dll,
            i386,
            dll_characteristics: u16_at(bytes, opt + 70)?,
            large_address_aware: u16_at(bytes, pe + 22)? & 0x20 != 0,
            diagnostic: false,
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
                // Certificates are file metadata, not mapped image data. Native
                // PE32 loading without FORCE_INTEGRITY accepts malformed and
                // out-of-file pairs; retain the raw fields without dereferencing.
                // Signature verification APIs remain unsupported, not successful.
                if !i386 {
                    range(bytes, rva, len)?;
                }
            }
            // Certificate directory uses file offsets.
            else {
                image.file_range(rva, len)?;
            }
        }
        Ok(image)
    }

    /// Inspect the bounded type/name/language resource tree and borrow its exact
    /// file-backed payloads. No language fallback or manifest activation occurs.
    pub fn resources(&self) -> Result<Vec<crate::Resource<'a>>> {
        crate::resource::parse(self.directories[2], |rva, len| self.file_range(rva, len))
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
        let width = if self.i386 { 4 } else { 8 };
        let ordinal_flag = if self.i386 { 1u64 << 31 } else { 1u64 << 63 };
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
            let module_rva = u32_at(d, 12)? as usize;
            if module_rva == 0 {
                return Err("null import module name RVA".into());
            }
            let module = self.string(module_rva)?;
            let iat = u32_at(d, 16)? as usize;
            if iat == 0 || !iat.is_multiple_of(width) {
                return Err("invalid IAT".into());
            }
            let lookup = if lookup == 0 { iat } else { lookup };
            let mut terminated = false;
            for i in 0..65536 {
                let item = self.file_range(lookup + i * width, width)?;
                let value = if self.i386 {
                    u64::from(u32_at(item, 0)?)
                } else {
                    u64_at(item, 0)?
                };
                if self.i386 && self.diagnostic {
                    let at = iat + i * width;
                    if !self.sections.iter().any(|s| {
                        at >= s.rva
                            && at
                                .checked_add(width)
                                .is_some_and(|end| end <= s.rva + s.raw_size)
                            && s.rights & 0x40000000 != 0
                    }) {
                        return Err(
                            "PE32 diagnostic IAT must target initialized readable section bytes"
                                .into(),
                        );
                    }
                } else {
                    self.data_target(iat + i * width, width)?;
                }
                if value == 0 {
                    terminated = true;
                    break;
                }
                let symbol = if value & ordinal_flag != 0 {
                    if value & !(ordinal_flag | 0xffff) != 0 {
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
                if !slots.insert(iat + i * width) {
                    return Err("overlapping IAT slots".into());
                }
                result.push(Import {
                    module: module.clone(),
                    symbol,
                    iat: iat + i * width,
                });
            }
            if !terminated {
                return Err("unterminated thunk table".into());
            }
        }
        Err("unterminated import descriptors".into())
    }
    /// List unique DIR64 (AMD64) or HIGHLOW (i386) fixups. Headers and
    /// overlapping writes are rejected; only i386 admits initialized code targets.
    pub fn relocations(&self) -> Result<Vec<usize>> {
        let width = if self.i386 { 4 } else { 8 };
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
                    kind if kind == if self.i386 { 3 } else { 10 } => {
                        let target = page + (item as usize & 0xfff);
                        if self.i386 {
                            // PE32 absolute instruction operands need HIGHLOW
                            // fixups before final W^X protections. No header/BSS writes.
                            if !self
                                .sections
                                .iter()
                                .any(|s| target >= s.rva && target + width <= s.rva + s.raw_size)
                            {
                                return Err(
                                    "HIGHLOW fixup must target initialized section bytes".into()
                                );
                            }
                        } else {
                            self.data_target(target, width)?;
                        }
                        if result
                            .range(target.saturating_sub(width - 1)..target + width)
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
        self.export_table(false)
    }
    fn export_table(&self, stable_i386: bool) -> Result<Option<ExportTable>> {
        let (rva, length) = self.directories[0];
        if length == 0 {
            return Ok(None);
        }
        if length < 40 {
            return Err("truncated export directory".into());
        }
        let mut writes = if stable_i386 {
            self.relocations()?
                .into_iter()
                .chain(self.imports()?.into_iter().map(|i| i.iat))
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        writes.sort_unstable();
        let read = |address: usize, len: usize| -> Result<&'a [u8]> {
            let bytes = self.file_range(address, len)?;
            if stable_i386 {
                let end = address.checked_add(len).ok_or("export metadata overflow")?;
                let first = writes.partition_point(|&w| w + 4 <= address);
                if !self.sections.iter().any(|s| {
                    address >= s.rva
                        && end <= s.rva + s.raw_size
                        && s.rights
                            & (if self.diagnostic {
                                0xc0000000
                            } else {
                                0xe0000000
                            })
                            == 0x40000000
                }) || writes.get(first).is_some_and(|&w| w < end)
                {
                    return Err(
                        "PE32 self-export metadata must be readonly and free of loader writes"
                            .into(),
                    );
                }
            }
            Ok(bytes)
        };
        let string = |address: usize| -> Result<String> {
            let mut text = Vec::new();
            for i in 0..4096 {
                let byte = read(address.checked_add(i).ok_or("export name overflow")?, 1)?[0];
                if byte == 0 {
                    if text.is_empty() {
                        return Err("empty export name".into());
                    }
                    return String::from_utf8(text).map_err(|_| "non-UTF8 export name".into());
                }
                if !byte.is_ascii() {
                    return Err("non-ASCII export name".into());
                }
                text.push(byte);
            }
            Err("unterminated export name".into())
        };
        let directory = read(rva, 40)?;
        if u32_at(directory, 0)? != 0 {
            return Err("nonzero reserved export flags".into());
        }
        let name_rva = u32_at(directory, 12)? as usize;
        if name_rva == 0 {
            return Err("missing DLL export name".into());
        }
        let dll_name = string(name_rva)?;
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
            read(address, count * 4)?
        };
        let mut names: Vec<Vec<String>> = vec![Vec::new(); count];
        if names_count != 0 {
            let names_rva = u32_at(directory, 32)? as usize;
            let ordinals_rva = u32_at(directory, 36)? as usize;
            if names_rva == 0 || ordinals_rva == 0 {
                return Err("missing export name/ordinal table".into());
            }
            let pointers = read(names_rva, names_count * 4)?;
            let ordinals = read(ordinals_rva, names_count * 2)?;
            let mut previous: Option<String> = None;
            for i in 0..names_count {
                let name = string(u32_at(pointers, i * 4)? as usize)?;
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
                    let byte = read(address + offset, 1)?[0];
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
        if stable_i386
            && (entries.len() != count
                || entries
                    .iter()
                    .any(|e| matches!(e.target, ExportTarget::Forwarder(_))))
        {
            return Err("PE32 sparse/forwarded self exports are unsupported".into());
        }
        Ok(Some(ExportTable {
            dll_name,
            ordinal_base,
            entries,
        }))
    }
    /// Inspect bounded, relocatable PE32 static TLS without executing callbacks.
    /// Metadata must be readonly, pointer fields have exact HIGHLOW fixups, and
    /// the writable index cannot overlap loader writes or template bytes.
    pub fn static_tls32(&self) -> Result<Option<StaticTls32>> {
        if !self.i386 {
            return Err("PE32 TLS inspection requires i386".into());
        }
        let (rva, length) = self.directories[9];
        if length == 0 {
            return Ok(None);
        }
        if length != 24 || !rva.is_multiple_of(4) {
            return Err("invalid PE32 TLS directory shape".into());
        }
        let directory = self.file_range(rva, 24)?;
        let fixups = self.relocations()?;
        let iats = self
            .imports()?
            .into_iter()
            .map(|i| i.iat)
            .collect::<Vec<_>>();
        let overlaps = |a: usize, n: usize, b: usize, m: usize| a < b + m && b < a + n;
        let metadata = |at: usize, len: usize, expected: &[usize]| -> Result<&'a [u8]> {
            let bytes = self.file_range(at, len)?;
            if !self.sections.iter().any(|s| {
                at >= s.rva
                    && at + len <= s.rva + s.raw_size
                    && s.rights
                        & (if self.diagnostic {
                            0xc0000000
                        } else {
                            0xe0000000
                        })
                        == 0x40000000
            }) || expected.iter().any(|w| fixups.binary_search(w).is_err())
                || fixups
                    .iter()
                    .any(|&w| overlaps(at, len, w, 4) && !expected.contains(&w))
                || iats.iter().any(|&w| overlaps(at, len, w, 4))
            {
                return Err("PE32 TLS metadata must be readonly with exact pointer fixups and no IAT writes".into());
            }
            Ok(bytes)
        };
        let address = |va: u32| -> Result<usize> {
            let offset = u64::from(va)
                .checked_sub(self.base)
                .ok_or("TLS VA precedes image")?;
            if offset > self.size as u64 {
                return Err("TLS VA exceeds image".into());
            }
            Ok(offset as usize)
        };
        let fields = [
            u32_at(directory, 0)?,
            u32_at(directory, 4)?,
            u32_at(directory, 8)?,
            u32_at(directory, 12)?,
        ];
        let expected = fields
            .iter()
            .enumerate()
            .filter(|(_, value)| **value != 0)
            .map(|(i, _)| rva + i * 4)
            .collect::<Vec<_>>();
        metadata(rva, 24, &expected)?;
        let (template_rva, template_size) = if fields[0] == 0 && fields[1] == 0 {
            (0, 0)
        } else {
            if fields[0] == 0 || fields[1] < fields[0] {
                return Err("invalid TLS template VA extent".into());
            }
            let start = address(fields[0])?;
            let end = address(fields[1])?;
            let len = end - start;
            self.file_range(start, len)?;
            if !self.sections.iter().any(|s| {
                start >= s.rva && end <= s.rva + s.raw_size && s.rights & 0x60000000 == 0x40000000
            }) {
                return Err(
                    "TLS template must be initialized readable non-executable section data".into(),
                );
            }
            (start, len)
        };
        let zero_fill = u32_at(directory, 16)? as usize;
        let total_size = template_size
            .checked_add(zero_fill)
            .ok_or("TLS allocation overflow")?;
        if total_size == 0 || total_size > 1024 * 1024 {
            return Err("TLS allocation exceeds supported 1-byte..1-MiB extent".into());
        }
        let flags = u32_at(directory, 20)?;
        let code = (flags >> 20) & 15;
        if flags & !0x00f00000 != 0 || code == 15 {
            return Err("unsupported TLS characteristics/alignment".into());
        }
        let alignment = if code == 0 { 16 } else { 1usize << (code - 1) };
        let index_rva = address(fields[2])?;
        if fields[2] == 0
            || !index_rva.is_multiple_of(4)
            || !self.sections.iter().any(|s| {
                index_rva >= s.rva
                    && index_rva + 4 <= s.rva + s.size
                    && s.rights & 0xe0000000 == 0xc0000000
            })
            || fixups
                .iter()
                .chain(&iats)
                .any(|&w| overlaps(index_rva, 4, w, 4))
            || overlaps(index_rva, 4, template_rva, template_size)
        {
            return Err(
                "TLS index must be aligned writable data without template/loader-write overlap"
                    .into(),
            );
        }
        let mut callbacks = Vec::new();
        if fields[3] != 0 {
            let array = address(fields[3])?;
            if !array.is_multiple_of(4) {
                return Err("unaligned TLS callback array".into());
            }
            let mut terminated = false;
            for i in 0..256 {
                let at = array + i * 4;
                let va = u32_at(self.file_range(at, 4)?, 0)?;
                if va == 0 {
                    metadata(at, 4, &[])?;
                    terminated = true;
                    break;
                }
                metadata(at, 4, &[at])?;
                let target = address(va)?;
                if !self.sections.iter().any(|s| {
                    target >= s.rva
                        && target < s.rva + s.raw_size
                        && s.rights & 0x60000000 == 0x60000000
                }) {
                    return Err("TLS callback must target initialized executable code".into());
                }
                callbacks.push(target);
            }
            if !terminated {
                return Err("unterminated bounded TLS callback array".into());
            }
        }
        Ok(Some(StaticTls32 {
            template_rva,
            template_size,
            zero_fill,
            total_size,
            index_rva,
            alignment,
            callbacks,
        }))
    }
    /// Inspect the supported 64/72-byte PE32 legacy record, including a longer
    /// declared structure behind a shorter directory. Other active fields fail.
    pub fn legacy_load_config32(&self) -> Result<Option<LegacyLoadConfig32>> {
        if !self.i386 {
            return Err("legacy PE32 load-config inspection requires i386".into());
        }
        let (rva, directory_size) = self.directories[10];
        if directory_size == 0 {
            return Ok(None);
        }
        if ![64, 72].contains(&directory_size) || !rva.is_multiple_of(4) {
            return Err("unsupported legacy load-config directory size/alignment".into());
        }
        let declared_size = u32_at(self.file_range(rva, 4)?, 0)? as usize;
        if ![64, 72].contains(&declared_size) {
            return Err("unsupported legacy load-config structure size".into());
        }
        let span = directory_size.max(declared_size);
        let fixups = self.relocations()?;
        let iats = self
            .imports()?
            .into_iter()
            .map(|i| i.iat)
            .collect::<Vec<_>>();
        let overlaps = |a: usize, n: usize, b: usize, m: usize| a < b + m && b < a + n;
        let readonly = |at: usize, len: usize, expected: &[usize]| -> Result<&'a [u8]> {
            let bytes = self.file_range(at, len)?;
            if !self.sections.iter().any(|s| {
                at >= s.rva
                    && at + len <= s.rva + s.raw_size
                    && s.rights
                        & (if self.diagnostic {
                            0xc0000000
                        } else {
                            0xe0000000
                        })
                        == 0x40000000
            }) || expected.iter().any(|w| fixups.binary_search(w).is_err())
                || fixups
                    .iter()
                    .any(|&w| overlaps(at, len, w, 4) && !expected.contains(&w))
                || iats.iter().any(|&w| overlaps(at, len, w, 4))
            {
                return Err("legacy load-config metadata requires readonly storage and exact pointer fixups".into());
            }
            Ok(bytes)
        };
        let record = self.file_range(rva, span)?;
        if record[12..60].iter().any(|&b| b != 0) {
            return Err("unsupported active legacy load-config fields".into());
        }
        let cookie_va = u32_at(record, 60)?;
        let seh_va = if span == 72 { u32_at(record, 64)? } else { 0 };
        let seh_count = if span == 72 {
            u32_at(record, 68)? as usize
        } else {
            0
        };
        if (seh_va == 0) != (seh_count == 0) || seh_count > 65536 {
            return Err("invalid bounded SafeSEH table/count pair".into());
        }
        let mut expected = Vec::new();
        if cookie_va != 0 {
            expected.push(rva + 60);
        }
        if seh_va != 0 {
            expected.push(rva + 64);
        }
        readonly(rva, span, &expected)?;
        let address = |va: u32| -> Result<usize> {
            let offset = u64::from(va)
                .checked_sub(self.base)
                .ok_or("load-config VA precedes image")?;
            if offset >= self.size as u64 {
                return Err("load-config VA exceeds image".into());
            }
            Ok(offset as usize)
        };
        let cookie_rva = if cookie_va == 0 {
            None
        } else {
            let target = address(cookie_va)?;
            if !target.is_multiple_of(4)
                || !self.sections.iter().any(|s| {
                    target >= s.rva
                        && target + 4 <= s.rva + s.size
                        && s.rights & 0xe0000000 == 0xc0000000
                })
                || fixups
                    .iter()
                    .chain(&iats)
                    .any(|&w| overlaps(target, 4, w, 4))
            {
                return Err(
                    "security cookie requires aligned writable data without loader-write overlap"
                        .into(),
                );
            }
            if let Some(tls) = self.static_tls32()?
                && (overlaps(target, 4, tls.index_rva, 4)
                    || overlaps(target, 4, tls.template_rva, tls.template_size))
            {
                return Err("security cookie overlaps TLS loader storage".into());
            }
            Some(target)
        };
        let mut seh_handlers = Vec::new();
        let seh_table_rva = if seh_va == 0 {
            None
        } else {
            let table = address(seh_va)?;
            if !table.is_multiple_of(4) {
                return Err("unaligned SafeSEH table".into());
            }
            let bytes = readonly(table, seh_count * 4, &[])?;
            for entry in bytes.as_chunks::<4>().0 {
                let target = u32_at(entry, 0)? as usize;
                if seh_handlers
                    .last()
                    .is_some_and(|&previous| target <= previous)
                    || !self.sections.iter().any(|s| {
                        target >= s.rva
                            && target < s.rva + s.raw_size
                            && s.rights & 0x60000000 == 0x60000000
                    })
                {
                    return Err(
                        "SafeSEH RVAs must be unique sorted initialized code targets".into(),
                    );
                }
                seh_handlers.push(target);
            }
            Some(table)
        };
        Ok(Some(LegacyLoadConfig32 {
            directory_size,
            declared_size,
            timestamp: u32_at(record, 4)?,
            major_version: u16_at(record, 8)?,
            minor_version: u16_at(record, 10)?,
            cookie_rva,
            seh_table_rva,
            seh_handlers,
        }))
    }
    /// Inspect a bounded static AMD64 table and supported v1 body recipes.
    /// Records/code/unwind slots must be backed by readonly/executable image data.
    /// This does not map/register code or enable exception dispatch.
    pub fn runtime_functions(&self) -> Result<Option<RuntimeFunctionTable>> {
        if self.i386 {
            return Err("AMD64 static function records do not apply to PE32".into());
        }
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
                address >= s.rva
                    && end <= s.rva + s.raw_size
                    && s.rights
                        & (if self.diagnostic {
                            0xc0000000
                        } else {
                            0xe0000000
                        })
                        == 0x40000000
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
            if !self.i386 {
                return Err("unsupported startup feature: exports".into());
            }
            self.export_table(true)?;
        }
        self.check_auxiliary_startup()
    }
    /// Validate EXE metadata for the separate bounded PE32 exception profile.
    /// Retains checked SafeSEH targets; this inspection does not install handlers
    /// or establish support for other exception kinds, nesting or unwind.
    pub fn check_exception_startup32(&self) -> Result<()> {
        if !self.i386 || self.is_dll || self.dll_characteristics & 0x400 != 0 {
            return Err("bounded PE32 exceptions require an i386 EXE without NO_SEH".into());
        }
        self.export_table(true)?;
        if self.directories[3].1 != 0 {
            return Err("PE32 exception directory is unsupported".into());
        }
        self.check_auxiliary_startup_mode(true, false)
    }
    // Handler-registration metadata is retained here without claiming recovery:
    // the diagnostic profile stops on every fault instead of ignoring it.
    pub fn check_diagnostic_startup32(&self) -> Result<()> {
        if !self.i386 {
            return Err("diagnostic startup requires PE32".into());
        }
        self.export_table(true)?;
        if self.is_dll && self.directories[9].1 != 0 {
            return Err("PE32 DLL static TLS/callbacks are unsupported".into());
        }
        if self.directories[3].1 != 0 {
            return Err("PE32 exception directory is unsupported".into());
        }
        self.check_auxiliary_startup_mode(true, false)
    }
    /// Inspect startup metadata for an emulator which implements checked TLS
    /// callbacks itself. This does not invoke callbacks or register SEH handlers;
    /// fault delivery remains the emulator's explicit responsibility. Other
    /// auxiliary directories keep the diagnostic profile's conservative gates.
    pub fn check_emulated_startup32(&self) -> Result<()> {
        if !self.i386 {
            return Err("emulated startup requires PE32".into());
        }
        self.export_table(true)?;
        if self.directories[3].1 != 0 {
            return Err("PE32 exception directory is unsupported".into());
        }
        self.check_auxiliary_startup_mode(true, true)
    }
    fn diagnostic_delay_imports32(&self) -> Result<()> {
        let (rva, length) = self.directories[13];
        let relocations = self.relocations()?;
        if !length.is_multiple_of(32) {
            return Err("invalid PE32 delay descriptor extent".into());
        }
        for offset in (0..length).step_by(32) {
            let descriptor = self.file_range(rva + offset, 32)?;
            let mut f = [0u32; 8];
            for (i, v) in f.iter_mut().enumerate() {
                *v = u32_at(descriptor, i * 4)?;
            }
            if f == [0; 8] {
                return Ok(());
            }
            if f[0] != 1 {
                return Err("unsupported PE32 delay descriptor attributes".into());
            }
            let pointer = |value: u32| -> Result<usize> {
                if value == 0 {
                    return Err("null PE32 delay metadata pointer".into());
                }
                Ok(value as usize)
            };
            self.string(pointer(f[1])?)?;
            let handle = pointer(f[2])?;
            self.data_target(handle, 4)?;
            if !handle.is_multiple_of(4)
                || !self.sections.iter().any(|s| {
                    handle >= s.rva
                        && handle + 4 <= s.rva + s.size
                        && s.rights & 0xc0000000 == 0xc0000000
                })
            {
                return Err("delay module handle storage must be writable".into());
            }
            let iat = pointer(f[3])?;
            let names = pointer(f[4])?;
            if !iat.is_multiple_of(4) || !names.is_multiple_of(4) {
                return Err("unaligned PE32 delay thunk table".into());
            }
            let mut count = None;
            for i in 0..65536 {
                let name = u32_at(self.file_range(names + i * 4, 4)?, 0)?;
                let thunk = u32_at(self.file_range(iat + i * 4, 4)?, 0)?;
                if name == 0 {
                    if thunk != 0 {
                        return Err("nonzero delay IAT terminator".into());
                    }
                    count = Some(i + 1);
                    break;
                }
                if name & 0x80000000 != 0 {
                    if name & 0x7fff0000 != 0 {
                        return Err("reserved PE32 delay ordinal bits".into());
                    }
                } else {
                    self.file_range(name as usize, 2)?;
                    self.string(name as usize + 2)?;
                }
                let entry = u64::from(thunk)
                    .checked_sub(self.base)
                    .ok_or("delay thunk precedes image")? as usize;
                if !self.sections.iter().any(|s| {
                    entry >= s.rva && entry < s.rva + s.raw_size && s.rights & 0x20000000 != 0
                }) || relocations.binary_search(&(iat + i * 4)).is_err()
                {
                    return Err("delay IAT requires relocated original executable thunks".into());
                }
            }
            let count = count.ok_or("unterminated PE32 delay thunk table")?;
            // Bound/unload snapshots remain the original guest helper's data.
            // No cache shortcut or host-side successful loader is invented.
            for value in [f[5], f[6]] {
                if value != 0 {
                    self.file_range(pointer(value)?, count * 4)?;
                }
            }
        }
        Err("unterminated PE32 delay descriptors".into())
    }
    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    pub(crate) fn check_library_startup(&self) -> Result<()> {
        if !self.is_dll {
            return Err("expected DLL image".into());
        }
        if self.i386 {
            self.export_table(true)?;
            // DLL TLS needs a process-wide slot/module lifetime. The isolated
            // EXE's single slot must not masquerade as a DLL TLS implementation.
            if self.directories[9].1 != 0 {
                return Err("PE32 DLL static TLS/callbacks are unsupported".into());
            }
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
        self.check_auxiliary_startup_mode(false, false)
    }
    fn check_auxiliary_startup_mode(
        &self,
        allow_seh32: bool,
        allow_tls_callbacks: bool,
    ) -> Result<()> {
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
                if i == 13 && self.i386 && self.diagnostic {
                    self.diagnostic_delay_imports32()?;
                    continue;
                }
                if i == 9 && self.i386 {
                    let tls = self.static_tls32()?.ok_or("missing PE32 TLS metadata")?;
                    if !allow_tls_callbacks && !tls.callbacks.is_empty() {
                        return Err("PE32 TLS callbacks are unsupported".into());
                    }
                    if tls.zero_fill != 0 {
                        return Err("PE32 nonzero TLS SizeOfZeroFill is unsupported pending native investigation".into());
                    }
                    continue;
                }
                if i == 10 && self.i386 {
                    let config = self
                        .legacy_load_config32()?
                        .ok_or("missing legacy load configuration")?;
                    if !allow_seh32 && !config.seh_handlers.is_empty() {
                        return Err("PE32 SafeSEH exception dispatch is unsupported".into());
                    }
                    continue;
                }
                return Err(format!("unsupported startup feature: {name}"));
            }
        }
        Ok(())
    }
}
