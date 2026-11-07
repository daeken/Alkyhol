//! DXBC retail checksum, ported from Microsoft's open DXC DxilHash.cpp.
//! Copyright (C) Microsoft Corporation. NCSA license: cpp/licenses/DXC-NCSA.txt.
//! Pin/source attribution is in cpp/root-signatures.md. Not a security hash.

const K: [u32; 64] = [
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391,
];
fn compress(state: &mut [u32; 4], block: &[u8]) {
    let x: [u32; 16] =
        std::array::from_fn(|i| u32::from_le_bytes(block[i * 4..i * 4 + 4].try_into().unwrap()));
    let [mut a, mut b, mut c, mut d] = *state;
    for (i, &k) in K.iter().enumerate() {
        let (f, g, shifts) = match i {
            0..16 => ((b & c) | (!b & d), i, [7, 12, 17, 22]),
            16..32 => ((b & d) | (c & !d), (5 * i + 1) % 16, [5, 9, 14, 20]),
            32..48 => (b ^ c ^ d, (3 * i + 5) % 16, [4, 11, 16, 23]),
            _ => (c ^ (b | !d), (7 * i) % 16, [6, 10, 15, 21]),
        };
        let next = b.wrapping_add(
            a.wrapping_add(f)
                .wrapping_add(k)
                .wrapping_add(x[g])
                .rotate_left(shifts[i % 4]),
        );
        (a, b, c, d) = (d, next, b, c);
    }
    for (s, v) in state.iter_mut().zip([a, b, c, d]) {
        *s = s.wrapping_add(v);
    }
}
pub(super) fn checksum(bytes: &[u8]) -> [u8; 16] {
    let count = bytes.len() as u32;
    let mut state = [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476];
    let (chunks, tail) = bytes.as_chunks::<64>();
    for block in chunks {
        compress(&mut state, block);
    }
    let mut last = [0u8; 64];
    if tail.len() >= 56 {
        last[..tail.len()].copy_from_slice(tail);
        last[tail.len()] = 0x80;
        compress(&mut state, &last);
        last.fill(0);
    } else {
        last[4..4 + tail.len()].copy_from_slice(tail);
        last[4 + tail.len()] = 0x80;
    }
    last[..4].copy_from_slice(&count.wrapping_shl(3).to_le_bytes());
    last[60..].copy_from_slice(&(count.wrapping_shl(1) | 1).to_le_bytes());
    compress(&mut state, &last);
    let mut result = [0; 16];
    for (chunk, word) in result.as_chunks_mut::<4>().0.iter_mut().zip(state) {
        chunk.copy_from_slice(&word.to_le_bytes());
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pinned_dxc_retail_vectors_cover_empty_and_multiple_blocks() {
        // DxilHashTest.cpp FixHashValueTest, same pinned NCSA-licensed source.
        for (input, expected) in [
            (
                "".to_owned(),
                [0xf6600d14u32, 0xbae275b7, 0xd4be4a4e, 0xa1e9b201],
            ),
            (
                "a".repeat(160),
                [0xa5e5a0bb, 0xd96dd9f8, 0x0b4b7191, 0xd63aa54a],
            ),
            (
                "z".repeat(160),
                [0x58ed1b7a, 0x90ede58b, 0xf9ad857c, 0x5440e613],
            ),
            (
                "ab".repeat(80),
                [0xf4fc06fc, 0x0bbd9ef7, 0x765ae0f7, 0x52a55925],
            ),
        ] {
            let actual = checksum(input.as_bytes());
            assert_eq!(
                actual,
                expected
                    .into_iter()
                    .flat_map(u32::to_le_bytes)
                    .collect::<Vec<_>>()
                    .as_slice()
            );
        }
    }
}
