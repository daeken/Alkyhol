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
        assert_eq!(ctype1(&[]).unwrap(), []);
    }
}
