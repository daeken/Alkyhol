"""Synthetic transport checks; original DLL/native runtime observations are separate."""
import struct
import sys
import unittest
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import observe_pe32_crt_state as crt

class CrtStateProtocol(unittest.TestCase):
    def words(self):
        error = 0xface1234
        words = [0x43523332, 4]
        for old in [0, 0, 1, 1, 2, 2, 0]:
            words += [old, 1, error]
        words += ([1] * 8 + [error]) * 2
        words += [1, 0, 1, error] * 5
        words += [sum((j * 17 + i) & 255 for j in range(n))
                  for i, n in enumerate([0, 1, 17, 4099, 65539])]
        return words + [0, 1, 1, 2, 1, error, 73]

    def test_real_storage_checksums_state_stack_flags_and_complete_transport(self):
        words = self.words()
        self.assertEqual(crt.decode(struct.pack('<73I', *words)), words)
        crt.assertions(words)

    def test_truncated_or_wrong_width_transport_cannot_pass(self):
        raw = struct.pack('<73I', *self.words())
        for data in [b'', raw[:-1], raw + b'\0']:
            with self.assertRaises(ValueError):
                crt.decode(data)
        for index in [0, 1, 72]:
            words = self.words()
            words[index] ^= 1
            with self.assertRaises(ValueError):
                crt.decode(struct.pack('<73I', *words))

    def test_bad_stack_cleanup_policy_alignment_nulls_or_storage_cannot_pass(self):
        for index in [*range(2, 23), *range(24, 32), *range(33, 66), *range(67, 72)]:
            words = self.words()
            words[index] ^= 1
            with self.subTest(index=index), self.assertRaises(ValueError):
                crt.assertions(words)

if __name__ == '__main__':
    unittest.main()
