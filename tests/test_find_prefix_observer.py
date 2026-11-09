"""Synthetic record rejection is transport validation, not execution evidence."""
from pathlib import Path
import sys
import struct
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from observe_pe32_find_prefix import decode

class PrefixTransportTests(unittest.TestCase):
    def test_missing_metadata_iteration_error_or_stack_checks_cannot_pass(self):
        words=[0x46505246,1]+[1]*13+[16]
        pack=lambda w:struct.pack('<'+'I'*len(w),*w)
        self.assertEqual(decode(pack(words)),words)
        for index in range(len(words)):
            bad=words.copy();bad[index]=0
            with self.assertRaises(ValueError):decode(pack(bad))
        for raw in [pack(words)[:-1],pack(words[:-1]),pack(words+[1])]:
            with self.assertRaises(ValueError):decode(raw)

if __name__=='__main__':unittest.main()
