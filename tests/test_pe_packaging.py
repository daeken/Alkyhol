import importlib.util
from pathlib import Path
import struct
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('translate_pe', ROOT / 'tools/translate_pe.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def macho():
    b = bytearray(512)
    struct.pack_into('<IIIIIIII', b, 0, 0xfeedfacf, 0x1000007, 3, 2, 1, 152, 0, 0)
    struct.pack_into('<II16sQQQQIIII', b, 32, 0x19, 152, b'__ALKYHOL', 0x100000000, 4096, 0, 512, 1, 1, 1, 0)
    struct.pack_into('<16s16sQQIIIIIIII', b, 104, b'__pefile', b'__ALKYHOL', 0x100000100, 5, 256, 0, 0, 0, 0, 0, 0, 0)
    b[256:261] = b'MZ123'
    return b


class PayloadIdentityTests(unittest.TestCase):
    def test_extracts_actual_section_bytes(self):
        self.assertEqual(module.embedded_payload(macho()), b'MZ123')

    def test_bad_geometry_architecture_permissions_and_missing_payload_fail(self):
        for offset, value in [(0, 0xfeedface), (4, 0x100000c), (20, 153), (36, 8),
                              (88, 3), (92, 3), (96, 2), (152, 511)]:
            with self.subTest(offset=offset):
                b = macho()
                struct.pack_into('<I', b, offset, value)
                with self.assertRaises(ValueError):
                    module.embedded_payload(b)
        for length in [0, 4, 31, 183, 260]:
            with self.assertRaises(ValueError):
                module.embedded_payload(macho()[:length])
        b = macho()
        b[104] = ord('x')
        with self.assertRaises(ValueError):
            module.embedded_payload(b)

    def test_duplicate_embedded_payload_is_rejected(self):
        b = macho()
        struct.pack_into('<I', b, 20, 232)
        struct.pack_into('<I', b, 36, 232)
        struct.pack_into('<I', b, 96, 2)
        b[184:264] = b[104:184]
        with self.assertRaises(ValueError):
            module.embedded_payload(b)


if __name__ == '__main__':
    unittest.main()
