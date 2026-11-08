"""Synthetic collector/fixture checks, not Windows activation evidence."""
import struct
import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1]/'tools'))
from check_pe32_activation import compare_records, decode_native, expected_resources, frozen, resource_record
from check_pe32_activation_miss import frozen as frozen_miss


class ActivationContractTests(unittest.TestCase):
    def synthetic_report(self):
        query = [0]*20
        query[17] = 0x715e7a11
        self.records = [[0x41433332, 32, 64, 4], query[:], query[:],
                        [1, 1, 0, 1], [2, 1, 1], *[query[:] for _ in range(5)],
                        [3, 1, 0], query[:]]
        return self.encode()

    def encode(self):
        return expected_resources()+b''.join(struct.pack('<'+'I'*len(row), *row) for row in self.records)

    def test_numeric_and_unicode_resource_headers_have_exact_alignment(self):
        record = resource_record(24, 1, 1033, b'xml')
        self.assertEqual(struct.unpack_from('<II', record), (3, 32))
        self.assertEqual(struct.unpack_from('<HHHH', record, 8), (0xffff, 24, 0xffff, 1))
        self.assertEqual(struct.unpack_from('<H', record, 22), (1033,))
        self.assertEqual(record[32:], b'xml\0')
        named = resource_record(10, 'λ🌸', 1041, b'\x00\xff')
        size, header = struct.unpack_from('<II', named)
        self.assertEqual((size, header % 4), (2, 0))
        self.assertEqual(named[12:20], 'λ🌸\0'.encode('utf-16-le'))
        self.assertEqual(named[header:], b'\x00\xff\0\0')

    def test_collector_preserves_query_observations_without_guessing_outputs(self):
        data = self.synthetic_report()
        self.assertEqual(decode_native(data), self.records)
        self.records[5][5] = 14007
        self.assertEqual(decode_native(self.encode())[5][5], 14007)

    def test_collector_rejects_width_resource_and_abi_changes(self):
        data = self.synthetic_report()
        for changed in [data[:-1], data+b'\0', b'X'+data[1:]]:
            with self.assertRaises(ValueError): decode_native(changed)
        self.records[0][1] = 56
        with self.assertRaises(ValueError): decode_native(self.encode())

    def test_failed_lifecycle_and_corrupt_guards_cannot_pass(self):
        for row, field in [(3, 1), (3, 3), (4, 1), (4, 2), (10, 1), (5, 17)]:
            self.synthetic_report()
            self.records[row][field] = 0
            with self.assertRaises(ValueError): decode_native(self.encode())

    def test_native_comparison_requires_the_unchanged_real_capture(self):
        capture = frozen()
        compare_records(decode_native(bytes.fromhex(capture['stdout_hex'])))
        with self.assertRaises(ValueError): compare_records(decode_native(self.synthetic_report()))
        changed = [row[:] for row in capture['records']]
        changed[5][7] = 2
        with self.assertRaises(ValueError): compare_records(changed)

    def test_bounded_bridge_contract_is_backed_by_a_separate_native_capture(self):
        capture = frozen_miss()
        words = struct.unpack('<33I', bytes.fromhex(capture['stdout_hex']))
        self.assertEqual(words[0], 0x414d3332)
        self.assertEqual([(words[i], words[i+1]) for i in [1, 9, 17, 25]], [(40, 0), (64, 0), (40, 1), (64, 1)])
        self.assertEqual(capture['inputs']['files']['activation-miss0.exe'], 'd67f11dcb64c5c5ada0bfd81983e33e23d8191fcb7f0bb58f7fcb21acaf3bda1')


if __name__ == '__main__':
    unittest.main()
