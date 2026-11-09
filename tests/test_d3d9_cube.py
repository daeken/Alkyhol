import struct
import sys
import unittest
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import observe_d3d9_cube as observer

class CubeResourceReportTests(unittest.TestCase):
    def test_com_header_rollover_report_rejects_each_altered_word_and_failed_stage(self):
        words = [0x434f4d48, 1, 0, 1, 1, 1, 2, 8]
        self.assertEqual(observer.decode(struct.pack('<8I', *words), True), words)
        for n in range(8):
            changed = words.copy(); changed[n] ^= 1
            with self.assertRaises(ValueError): observer.decode(struct.pack('<8I', *changed), True)
        for stage in range(1, 8):
            changed = words.copy(); changed[2] = stage
            with self.assertRaises(ValueError): observer.decode(struct.pack('<8I', *changed), True)
        with self.assertRaises(ValueError): observer.decode(struct.pack('<8I', *words))
    def test_complete_resource_contract(self):
        words = [0x43554245, 1, 0, 1, 1, 1, 1, 8]
        self.assertEqual(observer.decode(struct.pack('<8I', *words)), words)

    def test_each_failure_stage_and_each_corrupted_word_is_rejected(self):
        words = [0x43554245, 1, 0, 1, 1, 1, 1, 8]
        for stage in range(1, 22):
            changed = words.copy(); changed[2] = stage
            with self.assertRaises(ValueError): observer.decode(struct.pack('<8I', *changed))
        for n in range(8):
            changed = words.copy(); changed[n] ^= 1
            with self.assertRaises(ValueError): observer.decode(struct.pack('<8I', *changed))

    def test_wrong_report_length_is_rejected(self):
        raw = struct.pack('<8I', 0x43554245, 1, 0, 1, 1, 1, 1, 8)
        for value in [b'', raw[:-1], raw[:-4], raw + b'\0', raw + b'\0'*4]:
            with self.assertRaises(ValueError): observer.decode(value)
