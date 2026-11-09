"""Synthetic transport checks, not execution or game compatibility evidence."""
from pathlib import Path
import sys
import struct
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from observe_pe32_event_capacity import decode

class EventCapacityTransportTests(unittest.TestCase):
    def test_every_state_lifetime_and_stack_result_is_required(self):
        words=[0x45564350,1]+[1]*11+[14]
        pack=lambda w:struct.pack('<'+'I'*len(w),*w)
        self.assertEqual(decode(pack(words)),words)
        for index in range(len(words)):
            bad=words.copy();bad[index]=0
            with self.assertRaises(ValueError):decode(pack(bad))
        for raw in [pack(words)[:-1],pack(words[:-1]),pack(words+[1])]:
            with self.assertRaises(ValueError):decode(raw)

if __name__=='__main__':unittest.main()
