import struct
import sys
import unittest
from unittest.mock import patch
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "tools"))
import observe_pe32_mouse_capture as observer


class CaptureReportTests(unittest.TestCase):
    def test_unconfined_clip_query_checks_every_guard_bound_and_stack_result(self):
        words=[0x434c5032,1]+[1]*113+[116]
        with patch.object(observer,'CLIP_QUERY',True):
            self.assertEqual(observer.decode(struct.pack('<116I',*words)),words)
            for at in range(116):
                changed=words.copy();changed[at]^=1
                with self.assertRaises(ValueError):observer.decode(struct.pack('<116I',*changed))
    def test_client_button_and_nested_context_observations_require_every_word(self):
        words=[0x4d444633,1,1,1,1]
        for msg in [0x200,0x201,0x202,0x203,0x204,0x206,0x207,0x208,0x209]:words += [msg,0,0,1]
        for point in [0x00040003,0xfff3fff9,0x80007fff,0x00040003]:words += [point,0,1,1,1,1]
        words += [0,1,1,1,70]
        packet=struct.pack('<70I',*words)
        with patch.object(observer,'CLIENT_BUTTONS',True):
            self.assertEqual(observer.decode(packet),words)
            for at in range(70):
                changed=words.copy();changed[at]^=1
                with self.assertRaises(ValueError):observer.decode(struct.pack('<70I',*changed))
            for data in [b'',packet[:-1],packet[:-4],packet+b'\0'*4]:
                with self.assertRaises(ValueError):observer.decode(data)
    def test_background_owner_transfers_and_callback_assertions_are_strict(self):
        words=[0x43415032,2]+[1]*18+[21]
        with patch.object(observer,'BACKGROUND',True):
            self.assertEqual(observer.decode(struct.pack('<21I',*words)),words)
            for at in range(21):
                changed=words.copy();changed[at]^=1
                with self.assertRaises(ValueError):observer.decode(struct.pack('<21I',*changed))
            for raw in [b'',struct.pack('<21I',*words)[:-1],struct.pack('<21I',*words)+b'\0'*4]:
                with self.assertRaises(ValueError):observer.decode(raw)

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
