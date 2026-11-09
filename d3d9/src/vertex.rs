//! D3D9 vertex fetch for the packed declarations requested by DAO.
//! Expansion preserves original float bits; only integer formats are converted.
use crate::Result;
/// Size of each admitted source declaration element. Other formats fail explicitly.
pub fn element_size(ty: u32) -> Result<usize> {
    match ty {
        0..=3 => Ok((ty as usize + 1) * 4),
        4 | 6 | 8 => Ok(4), // D3DCOLOR, SHORT2, UBYTE4N
        _ => Err("vertex element format is not implemented".into()),
    }
}
/// Expand one source element into the four float registers specified by D3D9.
/// D3DCOLOR reorders BGRA bytes to RGBA; UBYTE4N keeps source component order.
pub fn expand(ty: u32, source: &[u8]) -> Result<[u8; 16]> {
    if source.len() != element_size(ty)? {
        return Err("vertex element byte extent differs from its declaration".into());
    }
    let mut output = [0; 16];
    output[12..16].copy_from_slice(&1f32.to_le_bytes());
    match ty {
        0..=3 => output[..source.len()].copy_from_slice(source),
        4 | 8 => {
            let order = if ty == 4 { [2, 1, 0, 3] } else { [0, 1, 2, 3] };
            for (component, index) in order.into_iter().enumerate() {
                output[component * 4..component * 4 + 4]
                    .copy_from_slice(&(source[index] as f32 / 255.).to_le_bytes());
            }
        }
        6 => {
            for component in 0..2 {
                let value = i16::from_le_bytes(
                    source[component * 2..component * 2 + 2].try_into().unwrap(),
                );
                output[component * 4..component * 4 + 4]
                    .copy_from_slice(&(value as f32).to_le_bytes());
            }
        }
        _ => unreachable!(),
    }
    Ok(output)
}
#[cfg(test)]
mod tests {
    use super::*;
    fn floats(bytes: [u8; 16]) -> [f32; 4] {
        std::array::from_fn(|i| f32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap()))
    }
    #[test]
    fn float_expansion_preserves_nan_signed_zero_and_defaults() {
        for ty in 0..=3 {
            let words = [0x7fc12abcu32, 0x80000000, 0xbf800000, 0x3f000000];
            let input: Vec<_> = words[..ty as usize + 1]
                .iter()
                .flat_map(|w| w.to_le_bytes())
                .collect();
            let actual = expand(ty, &input).unwrap();
            assert_eq!(actual[..input.len()], input);
            let expected = [0f32, 0., 0., 1.];
            for i in ty as usize + 1..4 {
                assert_eq!(&actual[i * 4..i * 4 + 4], &expected[i].to_le_bytes());
            }
        }
    }
    #[test]
    fn short2_is_signed_unnormalized_and_has_zero_one_defaults() {
        assert_eq!(
            floats(expand(6, &[0, 128, 255, 127]).unwrap()),
            [-32768., 32767., 0., 1.]
        );
        assert_eq!(
            floats(expand(6, &[255, 255, 1, 0]).unwrap()),
            [-1., 1., 0., 1.]
        );
    }
    #[test]
    fn packed_color_and_ubyte4n_have_distinct_order_and_normalization() {
        let data = [0u8, 17, 128, 255];
        assert_eq!(
            floats(expand(4, &data).unwrap()),
            [128. / 255., 17. / 255., 0., 1.]
        );
        assert_eq!(
            floats(expand(8, &data).unwrap()),
            [0., 17. / 255., 128. / 255., 1.]
        );
        for byte in 0..=255u8 {
            let expected = [byte as f32 / 255.; 4];
            assert_eq!(floats(expand(4, &[byte; 4]).unwrap()), expected);
            assert_eq!(floats(expand(8, &[byte; 4]).unwrap()), expected);
        }
    }
    #[test]
    fn unsupported_formats_and_partial_elements_are_errors() {
        for ty in [5, 7, 9, 17, u32::MAX] {
            assert!(element_size(ty).is_err());
            assert!(expand(ty, &[0; 4]).is_err());
        }
        for ty in [0, 1, 2, 3, 4, 6, 8] {
            let size = element_size(ty).unwrap();
            assert!(expand(ty, &vec![0; size - 1]).is_err());
            assert!(expand(ty, &vec![0; size + 1]).is_err());
        }
    }
}
