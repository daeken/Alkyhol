"""The registration witness tests state replacement, not exception recovery."""
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_exception_filter as filtering


class FilterProtocol(unittest.TestCase):
    def test_every_state_and_error_observation_is_required(self):
        words=filtering.expected_words();data=struct.pack('<11I',*words)
        self.assertEqual(filtering.decode(data),words)
        for index in range(11):
            changed=bytearray(data);changed[index*4]^=1
            with self.subTest(index=index),self.assertRaises(ValueError):filtering.decode(changed)

    def test_transport_extent_cannot_be_dropped_or_extended(self):
        data=struct.pack('<11I',*filtering.expected_words())
        for changed in [b'',data[:-1],data+b'\0']:
            with self.assertRaises(ValueError):filtering.decode(changed)


if __name__=='__main__':unittest.main()
