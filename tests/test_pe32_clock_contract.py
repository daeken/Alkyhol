"""Synthetic clock-record validation; actual Windows/M4 execution is separate."""
from pathlib import Path
import struct
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from check_pe32_startup import validate_clock

class ClockRecordContract(unittest.TestCase):
    def record(self,ticks):
        return struct.pack('<10I',0x54483332,ticks&0xffffffff,ticks>>32,
                           0x715e7a11,37,128,128,1,1,4)

    def test_each_machine_uses_its_own_clock_bracket(self):
        first=self.record(116444736010000000)
        second=self.record(116444736020000000)
        self.assertNotEqual(first,second)
        validate_clock(first,1_000_000_000,1_010_000_000)
        validate_clock(second,2_000_000_000,2_010_000_000)
        # A coarse native timestamp may trail the precise parent clock.
        validate_clock(self.record(116444736009500000),1_000_000_000,1_010_000_000)

    def test_zero_wrong_epoch_and_unbounded_tolerance_cannot_pass(self):
        for ticks in [0,10_000_000,116444736000000000,116444736030000000]:
            with self.subTest(ticks=ticks),self.assertRaises(ValueError):
                validate_clock(self.record(ticks),1_000_000_000,1_010_000_000)

    def test_layout_guard_heap_size_and_error_preservation_are_required(self):
        valid=self.record(116444736010000000)
        for changed in [valid[:-1],valid+b'\0']:
            with self.assertRaises(ValueError):validate_clock(changed,1_000_000_000,1_010_000_000)
        for index in [0,3,4,5,6,7,8,9]:
            words=list(struct.unpack('<10I',valid));words[index]^=1
            with self.subTest(index=index),self.assertRaises(ValueError):
                validate_clock(struct.pack('<10I',*words),1_000_000_000,1_010_000_000)

if __name__=='__main__':unittest.main()
