"""Protect the literal number-format evidence and strict observation comparator."""
import copy
import hashlib
import json
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import check_pe32_number_format as witness


class NumberFormatProtocol(unittest.TestCase):
    def setUp(self):
        self.record = json.loads((witness.ROOT/'pe/reference/pe32-number-format-windows-v1.json').read_text())
        with zipfile.ZipFile(witness.ROOT/'pe/reference/pe32-number-format-v1.zip') as archive:
            self.data = archive.read('native-observations.bin')
        self.rows = witness.decode(self.data)

    def test_literal_fixture_identity_and_native_machine_are_preserved(self):
        self.assertEqual(self.record['native_machine'], 0x8664)
        self.assertEqual(self.record['observer_process_machine'], 0)
        self.assertEqual(len(self.rows), 548)
        self.assertEqual(hashlib.sha256(self.data).hexdigest(), self.record['observations'][0]['stdout_sha256'])
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder)
            self.assertEqual(witness.frozen(directory).read_bytes(), self.data)
            target = directory/'number-format.exe'
            self.assertEqual(witness.digest(target), self.record['identity']['files'][target.name])
            target.write_bytes(b'different candidate')
            with self.assertRaises(ValueError):
                witness.frozen(directory)
            self.assertEqual(target.read_bytes(), b'different candidate')

    def test_every_output_code_unit_and_unwritten_tail_is_compared(self):
        reference = [self.rows[0]]
        self.assertEqual(witness.compare(reference, reference), [])
        for slot in range(128):
            changed = copy.deepcopy(reference)
            changed[0]['output'][slot] ^= 1
            self.assertEqual(len(witness.compare(changed, reference)), 1, slot)

    def test_sizes_and_defined_failure_codes_cannot_be_masked_by_last_error_rules(self):
        success = [self.rows[0]]
        changed = copy.deepcopy(success)
        changed[0]['error'] ^= 1
        self.assertEqual(witness.compare(changed, success), [])
        changed[0]['count'] += 1
        self.assertEqual(len(witness.compare(changed, success)), 1)
        failure = [next(row for row in self.rows if not row['count'])]
        changed = copy.deepcopy(failure)
        changed[0]['error'] ^= 1
        self.assertEqual(len(witness.compare(changed, failure)), 1)
        with self.assertRaises(ValueError):
            witness.compare([], success)

    def test_incomplete_transport_serial_numbers_and_guard_corruption_fail(self):
        for malformed in [b'', self.data[:-1], self.data+b'\0', self.data[4:]]:
            with self.assertRaises(ValueError):
                witness.decode(malformed)
        for offset in [0, 4, 8, 8+12, 8+270, len(self.data)-8, len(self.data)-4]:
            changed = bytearray(self.data)
            changed[offset] ^= 1
            with self.subTest(offset=offset), self.assertRaises(ValueError):
                witness.decode(changed)


if __name__ == '__main__':
    unittest.main()
