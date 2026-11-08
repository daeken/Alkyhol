"""Transport checks; synthetic words are not native filesystem evidence."""
import struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_create_directory as directory
class DirectoryProtocol(unittest.TestCase):
    def words(self):return [0x4d4b4432,4,1,123,0,183,1,456,0,3,0,183,0,183,0,3,0,183,1,789,0,183,1,24]
    def test_failures_and_stack_are_required_but_success_last_error_is_raw(self):
        words=self.words();row=directory.decode(struct.pack('<24I',*words))
        self.assertEqual(row['success_last_errors_observed_only'],[123,456,789])
        for i in [0,1,2,4,5,6,8,9,10,11,12,13,14,15,16,17,18,20,21,22,23]:
            changed=words.copy();changed[i]^=1
            with self.subTest(word=i),self.assertRaises(ValueError):directory.decode(struct.pack('<24I',*changed))
    def test_transport_must_cover_every_call_and_no_trailing_bytes(self):
        data=struct.pack('<24I',*self.words())
        for changed in [b'',data[:-1],data+b'\0']:
            with self.assertRaises(ValueError):directory.decode(changed)
if __name__=='__main__':unittest.main()
