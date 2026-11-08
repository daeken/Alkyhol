"""Transport rejection checks; these synthetic buffers are not native evidence."""
import struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_documents_folder as folder
class DocumentsProtocol(unittest.TestCase):
    def report(self,text):
        data=bytearray(1096);n=len(text.encode('utf-16-le'))//2
        struct.pack_into('<9I',data,0,0x444f4332,4,0,0,123,456,1,n,n)
        for start in [36,564]:
            struct.pack_into('<I',data,start,0x715e7a11)
            data[start+4:start+4+(n+1)*2]=(text+'\0').encode('utf-16-le')
            struct.pack_into('<I',data,start+524,0x715e7a11)
        struct.pack_into('<I',data,1092,0x656e6432)
        return data
    def test_paths_are_unicode_and_platform_specific_errors_remain_observations(self):
        for text in ['C:\\Users\\Guest\\Documents','Z:\\guest\\Documents-桜🌸']:
            row=folder.decode(self.report(text),text)
            self.assertEqual(row['paths'],[text,text])
            self.assertEqual(row['last_errors_observed_only'],[123,456])
            with self.assertRaises(ValueError):folder.decode(self.report(text),text+'other')
    def test_hresult_stack_guards_lengths_termination_and_transport_are_required(self):
        text='Z:\\Documents';data=self.report(text)
        for at in [0,4,8,12,24,28,32,36,560,564,1088,1092,40,568,40+len(text)*2,568+len(text)*2]:
            changed=bytearray(data);changed[at]^=1
            with self.subTest(at=at),self.assertRaises(ValueError):folder.decode(changed,text)
        for changed in [data[:-1],data+b'\0',b'']:
            with self.assertRaises(ValueError):folder.decode(changed,text)
if __name__=='__main__':unittest.main()
