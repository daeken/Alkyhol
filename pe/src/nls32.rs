//! Measured Windows CTYPE1 for the explicitly supported code-page repertoire.
//! Provenance: pe/data/ctype1/README.md. No host-locale or guessed Unicode flags.
use std::{collections::BTreeMap, sync::OnceLock};
fn table() -> &'static BTreeMap<u16, u16> {
    static TABLE: OnceLock<BTreeMap<u16, u16>> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut values = BTreeMap::new();
        for line in include_str!("../data/ctype1/windows-1252.txt").lines() {
            let (unit, flags) = line.split_once(' ').expect("owned CTYPE1 row");
            assert!(
                values
                    .insert(
                        u16::from_str_radix(unit, 16).unwrap(),
                        u16::from_str_radix(flags, 16).unwrap()
                    )
                    .is_none()
            );
        }
        assert_eq!(values.len(), 256);
        values
    })
}
pub(super) fn ctype1(units: &[u16]) -> crate::Result<Vec<u16>> {
    units
        .iter()
        .map(|u| {
            table().get(u).copied().ok_or_else(|| {
                format!("PE32 CTYPE1 code unit U+{u:04X} outside measured repertoire")
            })
        })
        .collect()
}
pub(super) fn case_map(units: &[u16], upper: bool) -> crate::Result<Vec<u16>> {
    static MAP: OnceLock<BTreeMap<u16, (u16, u16)>> = OnceLock::new();
    let data = MAP.get_or_init(|| {
        let mut values = BTreeMap::new();
        for line in include_str!("../data/ctype1/case-map.txt").lines() {
            let row: Vec<_> = line
                .split_whitespace()
                .map(|v| u16::from_str_radix(v, 16).unwrap())
                .collect();
            assert_eq!(row.len(), 3);
            assert!(values.insert(row[0], (row[1], row[2])).is_none());
        }
        assert_eq!(values.len(), 256);
        values
    });
    units
        .iter()
        .map(|u| {
            data.get(u)
                .map(|&(lo, up)| if upper { up } else { lo })
                .ok_or_else(|| {
                    format!("PE32 case-map code unit U+{u:04X} outside measured repertoire")
                })
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lookup_preserves_order_duplicates_and_rejects_unknown_units() {
        assert_eq!(
            ctype1(&[b'a' as u16, b'A' as u16, 0, 9, b'a' as u16]).unwrap(),
            [0x382, 0x381, 0x220, 0x268, 0x382]
        );
        assert!(ctype1(&[b'A' as u16, 0x2603, b'a' as u16]).is_err());
        assert!(ctype1(&[0xd800]).is_err());
        assert!(ctype1(&[]).unwrap().is_empty());
    }
    #[test]
    fn case_mapping_preserves_native_nonexpansion_and_nonrepresentable_uppercase() {
        assert_eq!(
            case_map(&[0xdf, 0x192, 0x178, b'a' as u16, 0], true).unwrap(),
            [0xdf, 0x191, 0x178, 65, 0]
        );
        assert_eq!(
            case_map(&[0xdf, 0x192, 0x178, b'A' as u16, 0], false).unwrap(),
            [0xdf, 0x192, 0xff, 97, 0]
        );
        assert!(case_map(&[0x2603], false).is_err());
    }
}
