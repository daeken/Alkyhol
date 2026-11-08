"""Synthetic collector checks, separate from Windows API/game observations."""
import struct
import sys
import unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from check_pe32_system_directory import decode,expected

class SystemDirectoryContractTests(unittest.TestCase):
    def fixture(self):
        self.path='Z:\\owned profile λ🌸\\Windows\\System32'
        return struct.pack('<18I',*expected())+struct.pack('<I',len(self.path.encode('utf-16-le'))//2)+(self.path+'\0').encode('utf-16-le')

    def test_full_role_path_and_utf16_units_are_exact(self):
        data=self.fixture();self.assertEqual(decode(data,self.path),self.path)
        with self.assertRaises(ValueError):decode(data,'Z:\\invented\\System32')
        with self.assertRaises(ValueError):decode(data[:72]+struct.pack('<I',len(self.path))+data[76:],self.path)

    def test_guards_errors_and_all_truncations_are_required(self):
        data=self.fixture()
        for n in range(len(data)):
            with self.subTest(length=n),self.assertRaises(ValueError):decode(data[:n],self.path)
        words=expected();words[3]^=1
        with self.assertRaises(ValueError):decode(struct.pack('<18I',*words)+data[72:],self.path)
        with self.assertRaises(ValueError):decode(data+b'\0',self.path)

if __name__=='__main__':unittest.main()
