"""Transport tests preserve actual machine observations; not native evidence."""
import struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_system_info as info

class SystemInfoProtocol(unittest.TestCase):
    def packet(self):
        return [0x53493332,4,36,1,0x715e7a11,0x715e7a11,0xface1234,25,0,4096,65536,0x7ffeffff,3,2,586,65536,0x2c000006,13,0x756e6547,0x6c65746e,0x49656e69,0x206c0,0x10800,0,0]
    def test_all_raw_fields_cpuid_and_error_are_preserved(self):
        words=self.packet();r=info.decode(struct.pack('<25I',*words));self.assertEqual(r['words'],words)
        self.assertEqual(r['information_words'],words[8:17]);self.assertEqual(r['cpuid1'],words[21:]);self.assertEqual(r['last_error'],0xface1234)
    def test_transport_guards_layout_and_required_fields_cannot_be_dropped(self):
        words=self.packet();data=struct.pack('<25I',*words)
        for n in [0,1,2,3,4,5,7]:
            changed=words[:];changed[n]^=1
            with self.subTest(n=n),self.assertRaises(ValueError):info.decode(struct.pack('<25I',*changed))
        for changed in [b'',data[:-1],data+b'\0']:
            with self.assertRaises(ValueError):info.decode(changed)
if __name__=='__main__':unittest.main()
