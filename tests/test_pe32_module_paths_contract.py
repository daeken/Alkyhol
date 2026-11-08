"""Synthetic format negatives for the collector, not game/native evidence."""
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from check_pe32_module_paths import decode, expected_words


class ModulePathContractTests(unittest.TestCase):
    def fixture(self):
        self.paths=['Z:\\owned λ🌸\\program.exe','Z:\\owned λ🌸\\program.exe','Z:\\owned λ🌸\\module-path.dll']
        self.header=struct.pack('<105I',*expected_words())
        self.tail=b''.join(struct.pack('<I',len(p.encode('utf-16-le'))//2)+(p+'\0').encode('utf-16-le') for p in self.paths)
        return self.header+self.tail

    def test_full_utf16_paths_are_checked_against_each_independent_source(self):
        self.assertEqual(decode(self.fixture(),self.paths),self.paths)
        wrong=self.paths[:];wrong[2]=self.paths[0]
        with self.assertRaises(ValueError):decode(self.header+self.tail,wrong)

    def test_missing_records_tail_and_every_truncated_prefix_fail(self):
        data=self.fixture()
        for n in range(len(data)):
            with self.subTest(length=n),self.assertRaises(ValueError):decode(data[:n],self.paths)
        with self.assertRaises(ValueError):decode(data+b'\0',self.paths)
        with self.assertRaises(ValueError):decode(self.header,[])

    def test_guards_errors_and_utf16_unit_counts_cannot_be_relaxed(self):
        data=self.fixture()
        words=expected_words();words[4]^=1
        with self.assertRaises(ValueError):decode(struct.pack('<105I',*words)+self.tail,self.paths)
        # Python's code-point length is shorter than this real UTF-16 length.
        changed=self.header+struct.pack('<I',len(self.paths[0]))+self.tail[4:]
        with self.assertRaises(ValueError):decode(changed,self.paths)
        with self.assertRaises(ValueError):decode(data[:420]+struct.pack('<I',2048)+data[424:],self.paths)


if __name__=='__main__':unittest.main()
