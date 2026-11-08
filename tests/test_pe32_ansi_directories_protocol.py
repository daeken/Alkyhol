"""Directory contract checks retain per-host source paths and full byte extents."""
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_ansi_directories as directory


class AnsiDirectories(unittest.TestCase):
    def packet(self,path='Z:\\PrivateWindows'):
        return struct.pack('<10I',0x57443332,4,len(path),1,0x715e7a11,
                           0x715e7a11,1,1,0xface1234,10)+path.encode('ascii')+b'\0'

    def test_actual_path_identity_is_separate_from_the_abi_contract(self):
        first=directory.decode(self.packet(),'Z:\\PrivateWindows')
        second=directory.decode(self.packet('C:\\Windows\\System32'),'C:\\Windows\\System32')
        self.assertEqual(first['contract'],second['contract'])
        self.assertNotEqual(first['path'],second['path'])
        with self.assertRaises(ValueError):directory.decode(self.packet(),'Z:\\SharedWindows')

    def test_every_field_extent_and_termination_is_required(self):
        data=self.packet()
        for i in range(10):
            changed=bytearray(data);changed[i*4]^=1
            with self.subTest(word=i),self.assertRaises(ValueError):directory.decode(changed)
        for changed in [b'',data[:-1],data+b'\0',data[:-1]+b'x',data[:41]+b'\0'+data[42:]]:
            with self.assertRaises(ValueError):directory.decode(changed)


if __name__=='__main__':unittest.main()
