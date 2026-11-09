//! Backed D3D9 transform state. Programmable and POSITIONT stages bypass it.
use std::collections::BTreeMap;
const IDENTITY: [u32; 16] = [
    0x3f800000, 0, 0, 0, 0, 0x3f800000, 0, 0, 0, 0, 0x3f800000, 0, 0, 0, 0, 0x3f800000,
];
#[derive(Default)]
pub(super) struct Transforms(BTreeMap<u32, [u32; 16]>);
impl Transforms {
    pub(super) fn valid(state: u32) -> bool {
        matches!(state,2|3|16..=23|256..=511)
    }
    pub(super) fn get(&self, state: u32) -> Option<[u32; 16]> {
        Self::valid(state).then(|| *self.0.get(&state).unwrap_or(&IDENTITY))
    }
    pub(super) fn set(&mut self, state: u32, matrix: [u32; 16]) -> bool {
        if !Self::valid(state) {
            return false;
        }
        self.0.insert(state, matrix);
        true
    }
    pub(super) fn multiply(&mut self, state: u32, left: [u32; 16]) -> bool {
        let Some(right) = self.get(state) else {
            return false;
        };
        if left
            .iter()
            .chain(&right)
            .any(|n| !f32::from_bits(*n).is_finite())
        {
            return false;
        }
        let mut output = [0; 16];
        for row in 0..4 {
            for col in 0..4 {
                let product =
                    |k| f32::from_bits(left[row * 4 + k]) * f32::from_bits(right[k * 4 + col]);
                output[row * 4 + col] =
                    (((product(0) + product(1)) + product(2)) + product(3)).to_bits();
            }
        }
        self.set(state, output)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matrix_storage_keeps_bits_independent_defaults_and_invalid_requests_atomic() {
        let mut state = Transforms::default();
        for key in [2, 3, 16, 23, 256, 511] {
            assert_eq!(state.get(key), Some(IDENTITY));
        }
        let mut bits = IDENTITY;
        bits[1] = 0x80000000;
        bits[2] = 0x7fc01234;
        assert!(state.set(3, bits));
        assert_eq!(state.get(3), Some(bits));
        assert_eq!(state.get(256), Some(IDENTITY));
        for key in [0, 1, 4, 15, 24, 255, 512, u32::MAX] {
            assert!(!state.set(key, bits));
            assert_eq!(state.get(key), None);
        }
        assert!(!state.multiply(3, IDENTITY));
        assert_eq!(state.get(3), Some(bits));
    }
    #[test]
    fn multiplication_premultiplies_and_preserves_the_other_transforms() {
        let mut state = Transforms::default();
        let mut scale = IDENTITY;
        for (n, v) in [(0, 2f32), (5, 3.), (10, 4.)] {
            scale[n] = v.to_bits();
        }
        let mut translation = IDENTITY;
        for (n, v) in [(12, 5f32), (13, 7.), (14, 11.)] {
            translation[n] = v.to_bits();
        }
        state.set(256, scale);
        assert!(state.multiply(256, translation));
        let result = state.get(256).unwrap();
        assert_eq!(
            result[12..15],
            [10f32.to_bits(), 21f32.to_bits(), 44f32.to_bits()]
        );
        assert_eq!(state.get(3), Some(IDENTITY));
    }
}
