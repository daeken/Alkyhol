import struct
import sys
import unittest
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import observe_pe32_c_locale as observer

class CLocaleReportTests(unittest.TestCase):
    def test_exact_all_category_report(self):
        words = [0x434c4f43, 1] + [1]*19 + [22]
        self.assertEqual(observer.decode(struct.pack('<22I', *words)), words)

    def test_failed_category_change_preservation_or_stack_cannot_pass(self):
        for n in range(22):
            words = [0x434c4f43, 1] + [1]*19 + [22]; words[n] ^= 1
            with self.assertRaises(ValueError): observer.decode(struct.pack('<22I', *words))

    def test_extra_or_incomplete_output_cannot_pass(self):
        raw = struct.pack('<22I', 0x434c4f43, 1, *([1]*19), 22)
        for changed in [b'', raw[:-1], raw[:-4], raw + b'\0', raw + b'\0'*4]:
            with self.assertRaises(ValueError): observer.decode(changed)
