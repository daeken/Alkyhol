//! Bounded, read-only inspection of PE type/name/language resources. This does
//! not select a UI language, activate a manifest, or make a resource executable.
use crate::Result;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResourceName {
    Id(u16),
    /// Counted UTF-16 units, retained without lossy Unicode conversion.
    Name(Vec<u16>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resource<'a> {
    pub kind: ResourceName,
    pub name: ResourceName,
    pub language: u16,
    pub code_page: u32,
    pub data_rva: usize,
    pub bytes: &'a [u8],
}

fn span(bytes: &[u8], at: usize, len: usize) -> Result<&[u8]> {
    bytes
        .get(at..at.checked_add(len).ok_or("resource offset overflow")?)
        .ok_or_else(|| "truncated resource metadata".into())
}
fn word(bytes: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(span(bytes, at, 2)?.try_into().unwrap()))
}
fn dword(bytes: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(span(bytes, at, 4)?.try_into().unwrap()))
}
fn name(bytes: &[u8], value: u32) -> Result<ResourceName> {
    if value & 0x80000000 == 0 {
        return Ok(ResourceName::Id(
            value.try_into().map_err(|_| "resource ID exceeds WORD")?,
        ));
    }
    let at = (value & 0x7fffffff) as usize;
    if !at.is_multiple_of(2) {
        return Err("unaligned resource name".into());
    }
    let len = word(bytes, at)? as usize;
    if len > 4096 {
        return Err("resource name budget exceeded".into());
    }
    let units = span(bytes, at + 2, len * 2)?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_le_bytes(*b))
        .collect();
    Ok(ResourceName::Name(units))
}

pub(crate) fn parse<'a>(
    directory: (usize, usize),
    read: impl Fn(usize, usize) -> Result<&'a [u8]>,
) -> Result<Vec<Resource<'a>>> {
    if directory == (0, 0) {
        return Ok(Vec::new());
    }
    let metadata = read(directory.0, directory.1)?;
    struct Reader<'a, F> {
        metadata: &'a [u8],
        read: F,
        visits: usize,
        entries: usize,
        result: Vec<Resource<'a>>,
    }
    impl<'a, F: Fn(usize, usize) -> Result<&'a [u8]>> Reader<'a, F> {
        fn walk(&mut self, offset: usize, path: &mut Vec<ResourceName>) -> Result<()> {
            self.visits += 1;
            if self.visits > 4096 {
                return Err("resource directory budget exceeded".into());
            }
            if !offset.is_multiple_of(4) || path.len() > 2 {
                return Err("unsupported resource tree depth/alignment".into());
            }
            span(self.metadata, offset, 16)?;
            let named = word(self.metadata, offset + 12)? as usize;
            let count = named + word(self.metadata, offset + 14)? as usize;
            self.entries += count;
            if self.entries > 65536 {
                return Err("resource entry budget exceeded".into());
            }
            let table = span(self.metadata, offset + 16, count * 8)?;
            let mut seen = BTreeSet::new();
            for (i, entry) in table.as_chunks::<8>().0.iter().enumerate() {
                let raw_name = dword(entry, 0)?;
                if (raw_name & 0x80000000 != 0) != (i < named) {
                    return Err("resource named/ID entry counts disagree".into());
                }
                let key = name(self.metadata, raw_name)?;
                if !seen.insert(key.clone()) {
                    return Err("duplicate resource key".into());
                }
                let target = dword(entry, 4)?;
                if path.len() < 2 {
                    if target & 0x80000000 == 0 {
                        return Err("resource type/name must target a directory".into());
                    }
                    path.push(key);
                    self.walk((target & 0x7fffffff) as usize, path)?;
                    path.pop();
                } else {
                    if target & 0x80000000 != 0 || !target.is_multiple_of(4) {
                        return Err("resource language must target an aligned data entry".into());
                    }
                    let ResourceName::Id(language) = key else {
                        return Err("resource language must be a WORD ID".into());
                    };
                    let data = span(self.metadata, target as usize, 16)?;
                    if dword(data, 12)? != 0 {
                        return Err("nonzero reserved resource data field".into());
                    }
                    let data_rva = dword(data, 0)? as usize;
                    let len = dword(data, 4)? as usize;
                    let bytes = (self.read)(data_rva, len)?;
                    self.result.push(Resource {
                        kind: path[0].clone(),
                        name: path[1].clone(),
                        language,
                        code_page: dword(data, 8)?,
                        data_rva,
                        bytes,
                    });
                }
            }
            Ok(())
        }
    }
    let mut reader = Reader {
        metadata,
        read,
        visits: 0,
        entries: 0,
        result: Vec::new(),
    };
    reader.walk(0, &mut Vec::new())?;
    Ok(reader.result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn put(bytes: &mut [u8], at: usize, n: u32) {
        bytes[at..at + 4].copy_from_slice(&n.to_le_bytes());
    }
    fn fixture() -> Vec<u8> {
        let mut b = vec![0; 0x203];
        for (offset, key, target) in [(0, 24, 32), (32, 1, 64), (64, 1033, 96)] {
            b[offset + 14] = 1;
            put(&mut b, offset + 16, key);
            put(
                &mut b,
                offset + 20,
                target | if offset < 64 { 0x80000000 } else { 0 },
            );
        }
        put(&mut b, 96, 0x200);
        put(&mut b, 100, 3);
        put(&mut b, 104, 65001);
        b[0x200..].copy_from_slice(b"xml");
        b
    }
    fn check(b: &[u8], size: usize) -> Result<Vec<Resource<'_>>> {
        parse((0, size), |at, len| span(b, at, len))
    }
    #[test]
    fn payload_rvas_are_image_relative_and_can_be_outside_directory() {
        let b = fixture();
        let records = check(&b, 112).unwrap();
        assert_eq!(
            records,
            vec![Resource {
                kind: ResourceName::Id(24),
                name: ResourceName::Id(1),
                language: 1033,
                code_page: 65001,
                data_rva: 0x200,
                bytes: b"xml",
            }]
        );
        assert!(
            parse((0, 0), |_, _| -> Result<&[u8]> {
                panic!("absent directory was read")
            })
            .unwrap()
            .is_empty()
        );
    }
    #[test]
    fn names_preserve_counted_utf16_including_surrogates() {
        let mut b = fixture();
        b[12] = 1;
        b[14] = 0;
        put(&mut b, 16, 0x80000080);
        b[128..136].copy_from_slice(&[3, 0, 0xbb, 3, 0x3c, 0xd8, 0x38, 0xdf]);
        assert_eq!(
            check(&b, 136).unwrap()[0].kind,
            ResourceName::Name(vec![0x3bb, 0xd83c, 0xdf38])
        );
        b[128..130].copy_from_slice(&4097u16.to_le_bytes());
        assert!(check(&b, 136).is_err());
    }
    #[test]
    fn every_truncated_table_and_payload_fails() {
        let b = fixture();
        for size in 1..112 {
            assert!(check(&b, size).is_err(), "{size}");
        }
        for len in 0..b.len() {
            assert!(check(&b[..len], 112).is_err(), "{len}");
        }
    }
    #[test]
    fn cycles_extra_levels_bad_offsets_and_reserved_data_fail() {
        let b = fixture();
        for (at, value) in [
            (20, 0x80000000),
            (20, 0x80000021),
            (20, 0xfffffff0),
            (84, 0x80000000),
            (84, 97),
            (96, 0xfffffff0),
            (100, u32::MAX),
            (108, 1),
            (16, 0x10000),
            (80, 0x80000080),
        ] {
            let mut changed = b.clone();
            put(&mut changed, at, value);
            assert!(check(&changed, 112).is_err(), "{at}:{value:x}");
        }
    }
    #[test]
    fn duplicate_keys_and_inconsistent_named_counts_fail() {
        let mut b = fixture();
        b[14] = 2;
        let entry = b[16..24].to_vec();
        b[24..32].copy_from_slice(&entry);
        assert!(check(&b, 112).unwrap_err().contains("duplicate"));
        b[14] = 1;
        b[12] = 1;
        assert!(check(&b, 112).unwrap_err().contains("counts"));
    }
}
