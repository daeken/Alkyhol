import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import observe_pe32_mouse_capture as observer


class CaptureReportTests(unittest.TestCase):
    def test_exact_lifecycle_and_callback_report(self):
        words = [0x43415032, 1] + [1] * 16 + [19]
        self.assertEqual(observer.decode(struct.pack("<19I", *words)), words)

    def test_any_failed_capture_query_callback_or_stack_assertion_is_rejected(self):
        for at in range(19):
            words = [0x43415032, 1] + [1] * 16 + [19]
            words[at] ^= 1
            with self.assertRaises(ValueError):
                observer.decode(struct.pack("<19I", *words))

    def test_truncation_extra_words_and_partial_words_are_rejected(self):
        data = struct.pack("<19I", 0x43415032, 1, *([1] * 16), 19)
        for malformed in [b"", data[:-4], data[:-1], data + b"\0", data + b"\0" * 4]:
            with self.assertRaises(ValueError):
                observer.decode(malformed)
