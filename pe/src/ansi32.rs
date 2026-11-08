//! Explicit Windows-1252 process ANSI profile, needed by the original legacy
//! CRT. Mapping data/provenance/license: pe/data/windows-1252/README.md.
use std::{collections::BTreeMap, sync::OnceLock};
pub(super) const ACP: u32 = 1252;
struct Mapping {
    mb: [u16; 256],
    wc: BTreeMap<u16, u8>,
}
fn mapping() -> &'static Mapping {
    static TABLE: OnceLock<Mapping> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut table = Mapping {
            mb: [0; 256],
            wc: BTreeMap::new(),
        };
        let mut mode = 0;
        let mut mb_count = 0;
        for line in include_str!("../data/windows-1252/bestfit1252.txt").lines() {
            let parts: Vec<_> = line.split(';').next().unwrap().split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }
            match parts[0] {
                "MBTABLE" => {
                    assert_eq!(parts[1], "256");
                    mode = 1;
                }
                "WCTABLE" => {
                    assert_eq!(parts[1], "698");
                    mode = 2;
                }
                "ENDCODEPAGE" => mode = 0,
                value if value.starts_with("0x") && mode != 0 => {
                    let a = u16::from_str_radix(&parts[0][2..], 16).unwrap();
                    let b = u16::from_str_radix(&parts[1][2..], 16).unwrap();
                    if mode == 1 {
                        assert_eq!(a as usize, mb_count);
                        table.mb[a as usize] = b;
                        mb_count += 1;
                    } else {
                        assert!(table.wc.insert(a, u8::try_from(b).unwrap()).is_none());
                    }
                }
                _ => (),
            }
        }
        assert_eq!(mb_count, 256);
        assert_eq!(table.wc.len(), 698);
        table
    })
}
pub(super) fn decode(bytes: &[u8]) -> Vec<u16> {
    bytes.iter().map(|&b| mapping().mb[b as usize]).collect()
}
pub(super) fn encode(
    units: &[u16],
    no_best_fit: bool,
    default: u8,
) -> crate::Result<(Vec<u8>, bool)> {
    let table = mapping();
    let mut used = false;
    let mut bytes = Vec::with_capacity(units.len());
    for &unit in units {
        // Do not guess legacy API handling of paired/unpaired surrogate units.
        // Explicit UTF-8 still supports supplementary characters independently.
        if (0xd800..=0xdfff).contains(&unit) {
            return Err("PE32 Windows-1252 surrogate conversion is unsupported".into());
        }
        let encoded = table
            .wc
            .get(&unit)
            .copied()
            .filter(|&b| !no_best_fit || table.mb[b as usize] == unit);
        bytes.push(encoded.unwrap_or_else(|| {
            used = true;
            default
        }));
    }
    Ok((bytes, used))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_byte_roundtrips_and_best_fit_is_distinct_from_default_replacement() {
        let bytes: Vec<_> = (0..=255).collect();
        assert_eq!(encode(&decode(&bytes), true, b'?').unwrap(), (bytes, false));
        assert_eq!(decode(&[0x80, 0x81, 0xe9]), [0x20ac, 0x81, 0xe9]);
        assert_eq!(
            encode(&[0x100, 0x20ac, 0x2603], false, b'?').unwrap(),
            (b"A\x80?".to_vec(), true)
        );
        assert_eq!(
            encode(&[0x100, 0x20ac], true, b'!').unwrap(),
            (b"!\x80".to_vec(), true)
        );
        assert!(encode(&[0xd800], false, b'?').is_err());
        assert!(encode(&[0xd83c, 0xdf38], false, b'?').is_err());
    }
}
