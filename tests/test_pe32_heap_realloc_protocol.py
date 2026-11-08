"""Integrity checks for the owned heap witness; not application equivalence."""
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import observe_pe32_heap_realloc as resize


class HeapResizeProtocol(unittest.TestCase):
    def test_native_record_pins_the_actual_failure_sequence_and_literal_pe(self):
        native=resize.frozen()
        self.assertEqual(resize.digest(resize.ROOT/'pe/reference/pe32-heap-realloc-v1.exe'),native['inputs']['files']['heap-realloc-0.exe'])
        self.assertEqual(resize.decode(bytes.fromhex(native['reports'][0]['stdout_hex'])),resize.expected_words())

    def test_immediate_error_is_separate_from_the_original_composed_sequence(self):
        data=struct.pack('<10I',0x48463332,4,1,1,8,7,1,1,8,10)
        self.assertEqual(len(resize.decode_immediate(data)),10)
        for index in range(10):
            changed=bytearray(data);changed[index*4]^=1
            with self.subTest(index=index),self.assertRaises(ValueError):resize.decode_immediate(changed)
        with self.assertRaises(ValueError):resize.decode_immediate(data[:-1])
    def test_each_byte_of_the_observed_contract_is_required(self):
        words = resize.expected_words()
        data = struct.pack('<64I', *words)
        self.assertEqual(resize.decode(data), words)
        for index in range(len(data)):
            changed = bytearray(data)
            changed[index] ^= 1
            with self.subTest(index=index), self.assertRaises(ValueError):
                resize.decode(changed)

    def test_dropped_or_extra_words_cannot_pass(self):
        data = struct.pack('<64I', *resize.expected_words())
        for changed in [b'', data[:-1], data[:-4], data + b'\0']:
            with self.assertRaises(ValueError):
                resize.decode(changed)


if __name__ == '__main__':
    unittest.main()
