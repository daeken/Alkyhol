"""Reject incomplete heap ownership observations; not a threading proof."""
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_heap_lock as locking


class HeapLockProtocol(unittest.TestCase):
    def test_every_ownership_allocation_and_error_word_is_required(self):
        words=locking.expected_words();data=struct.pack('<28I',*words)
        self.assertEqual(locking.decode(data),words)
        for index in range(28):
            changed=bytearray(data);changed[index*4]^=1
            with self.subTest(index=index),self.assertRaises(ValueError):locking.decode(changed)

    def test_incomplete_or_extended_transport_cannot_pass(self):
        data=struct.pack('<28I',*locking.expected_words())
        for changed in [b'',data[:-1],data+b'\0']:
            with self.assertRaises(ValueError):locking.decode(changed)


if __name__=='__main__':unittest.main()
