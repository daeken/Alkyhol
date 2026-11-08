"""Synthetic certificate fixture checks; loader/trust observations are separate."""
from pathlib import Path
import struct
import sys
import unittest
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe32_certificates import variants,CASES
from test_pe32_geometry import sample


class Pe32CertificateTests(unittest.TestCase):
    def test_only_directory_and_explicit_certificate_append_change(self):
        raw=bytearray(sample());struct.pack_into('<I',raw,0x98+92,16);raw=bytes(raw)
        offset,items=variants(raw)
        self.assertEqual([row['case'] for row,_ in items],CASES)
        self.assertEqual(items[0][1],raw)
        for row,data in items:
            self.assertEqual(data[:offset],raw[:offset]);self.assertEqual(data[offset+8:len(raw)],raw[offset+8:])
            self.assertEqual(struct.unpack_from('<II',data,offset),(row['certificate_offset'],row['certificate_size']))
            self.assertEqual(len(data),row['file_size'])
        self.assertEqual(items[1][1][len(raw):],struct.pack('<IHH',8,0x200,2))
        self.assertEqual(items[2][1][len(raw):],b'notacert')

    def test_code_integrity_and_existing_certificate_directory_cannot_enter_observer(self):
        original=bytearray(sample());struct.pack_into('<I',original,0x98+92,16)
        for at,fmt,value in [(0x98+70,'H',0x80),(0x98+96+32,'I',8),(0x98+92,'I',4), (0x84,'H',0x8664)]:
            raw=original.copy();struct.pack_into('<'+fmt,raw,at,value)
            with self.assertRaises(ValueError):variants(raw)
        with self.assertRaises(ValueError):variants(original[:-1])
        with self.assertRaises(ValueError):variants(b'MZ')


if __name__=='__main__':unittest.main()
