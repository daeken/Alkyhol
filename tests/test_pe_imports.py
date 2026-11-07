import importlib.util
from pathlib import Path
import struct
import unittest

spec = importlib.util.spec_from_file_location('pe_imports', Path(__file__).resolve().parents[1] / 'tools/pe_imports.py')
pe = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pe)


def fixture(bits=64, delay=False, legacy=False):
    data = bytearray(0x800)
    def put(fmt, offset, *values): struct.pack_into(fmt, data, offset, *values)
    data[:2] = b'MZ'; put('<I', 0x3c, 0x80); data[0x80:0x84] = b'PE\0\0'
    optional_size = 240 if bits == 64 else 224
    put('<HHIIIHH', 0x84, 0x8664 if bits == 64 else 0x14c, 1, 0, 0, 0, optional_size, 0)
    optional = 0x98
    put('<H', optional, 0x20b if bits == 64 else 0x10b)
    base = 0x400000
    put('<Q' if bits == 64 else '<I', optional + (24 if bits == 64 else 28), base)
    put('<I', optional + 60, 0x200)
    minimum = 112 if bits == 64 else 96
    put('<I', optional + minimum - 4, 16)
    put('<II', optional + minimum + (13 if delay else 1) * 8, 0x1000, 64 if delay else 40)
    section = optional + optional_size
    put('<IIII', section + 8, 0x600, 0x1000, 0x600, 0x200)
    bias = base if legacy else 0
    if delay:
        put('<IIIIIIII', 0x200, 0 if legacy else 1, 0x1080 + bias, 0, 0x1100 + bias, 0x1100 + bias, 0, 0, 0)
    else: put('<IIIII', 0x200, 0x1100, 0, 0, 0x1080, 0x1100)
    data[0x280:0x28d] = b'KERNEL32.dll\0'
    put('<' + ('Q' if bits == 64 else 'I') * 3, 0x300, 0x1180, (1 << (bits - 1)) | 42, 0)
    put('<H', 0x380, 17); data[0x382:0x38e] = b'CloseHandle\0'
    return bytes(data)


class ImportTests(unittest.TestCase):
    def test_normal_32_and_64(self):
        for bits in (32, 64):
            p = pe.PE(fixture(bits)); self.assertEqual(p.bits, bits)
            self.assertEqual(p.imports(), [{'module': 'KERNEL32.dll', 'symbols': [{'name': 'CloseHandle'}, {'ordinal': 42}]}])
            self.assertEqual(p.imports(True), [])
    def test_delay_rva_and_legacy_va(self):
        for bits in (32, 64):
            for legacy in (False, True):
                p = pe.PE(fixture(bits, delay=True, legacy=legacy))
                self.assertEqual(p.imports(True)[0]['symbols'][0]['name'], 'CloseHandle')
                self.assertEqual(p.imports(), [])
    def test_bad_signatures_and_truncation(self):
        for data in (b'', b'not a PE', fixture()[:0x90]):
            with self.assertRaises(pe.PEError): pe.PE(data)
    def test_unbacked_rva(self):
        p = pe.PE(fixture()); p.sections = [(0x1000, 0x1000, 0x200, 0x10)]
        with self.assertRaises(pe.PEError): p.imports()
    def test_unterminated_directory(self):
        p = pe.PE(fixture()); p.directories[1] = (0x1000, 20)
        with self.assertRaises(pe.PEError): p.imports()
    def test_truncated_thunk(self):
        data = bytearray(fixture()); struct.pack_into('<I', data, 0x200, 0x15ff)
        with self.assertRaises(pe.PEError): pe.PE(data).imports()
    def test_header_directory_overflow(self):
        data = bytearray(fixture()); struct.pack_into('<I', data, 0x98 + 108, 0xffff)
        with self.assertRaises(pe.PEError): pe.PE(data)


if __name__ == '__main__': unittest.main()
