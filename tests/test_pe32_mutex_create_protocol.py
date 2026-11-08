"""Named-object transport tests are not evidence of a working mutex backend."""
import struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_mutex_create as mutex
class MutexProtocol(unittest.TestCase):
    def test_existence_lifetime_stack_and_error_words_are_required(self):
        e=0xface1234
        words=[0x4d583332,4,1,0,1,183,1,e,1,e,1,0,1,e,1,16];data=struct.pack('<16I',*words)
        self.assertEqual(mutex.decode(data,'lifecycle'),words)
        for i in range(16):
            changed=bytearray(data);changed[i*4]^=1
            with self.subTest(word=i),self.assertRaises(ValueError):mutex.decode(changed,'lifecycle')
        for changed in [b'',data[:-1],data+b'\0']:
            with self.assertRaises(ValueError):mutex.decode(changed,'lifecycle')
    def test_live_other_process_and_closed_last_reference_are_distinct(self):
        live=struct.pack('<8I',0x4d583332,4,1,183,1,0xface1234,1,8)
        closed=struct.pack('<8I',0x4d583332,4,1,0,1,0xface1234,1,8)
        mutex.decode(live,'other-process-live');mutex.decode(closed,'after-other-process-close')
        with self.assertRaises(ValueError):mutex.decode(live,'after-other-process-close')
        with self.assertRaises(ValueError):mutex.decode(closed,'other-process-live')
        with self.assertRaises(ValueError):mutex.decode(closed,'unknown-case')
    def test_frozen_native_record_covers_all_three_cases(self):
        report=mutex.frozen()
        self.assertEqual(len(report['reports']),3)
        self.assertEqual([len(row['words']) for row in report['reports']],[16,8,8])
if __name__=='__main__':unittest.main()
