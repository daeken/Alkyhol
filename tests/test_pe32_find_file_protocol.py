"""Frozen native transport/metadata checks; not full game/filesystem proof."""
import copy,struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_find_file as observer

class FindFileProtocolTests(unittest.TestCase):
    def test_literal_pe_and_complete_native_guard_handle_and_metadata_record(self):
        r=observer.frozen();observer.compare(r['decoded'])
        self.assertEqual(observer.digest(observer.ROOT/'pe/reference/pe32-find-file-v1.exe'),r['inputs']['files']['find-file.exe'])
        self.assertEqual(r['decoded']['words'][-1],21)
        self.assertEqual(r['decoded']['fields'][9:11],[0xa5a5a5a5]*2)

    def test_changed_holes_names_guards_or_lifecycle_cannot_pass(self):
        r=observer.frozen()
        for offset in [0,36,44,64,566]:
            value=copy.deepcopy(r['decoded']);data=bytearray.fromhex(value['record_hex']);data[offset]^=1;value['record_hex']=data.hex()
            with self.assertRaises(ValueError):observer.compare(value)
        value=copy.deepcopy(r['decoded']);value['words'][7]=0
        with self.assertRaises(ValueError):observer.compare(value)
        data=bytes.fromhex(r['stdout_hex'])
        for n in [0,4,27,100,len(data)-1]:
            with self.assertRaises(ValueError):observer.decode(data[:n],r['metadata_before'],r['metadata_after'])

    def test_file_times_require_independent_actual_filesystem_metadata(self):
        r=observer.frozen();data=bytearray.fromhex(r['stdout_hex']);start=len(data)-592
        struct.pack_into('<Q',data,start+4,0)
        with self.assertRaisesRegex(ValueError,'actual filesystem metadata'):observer.decode(bytes(data),r['metadata_before'],r['metadata_after'])
        # Comparisons normalize times only after each host's metadata check.
        other=copy.deepcopy(r['decoded']);body=bytearray.fromhex(other['record_hex']);body[4:28]=bytes(24);other['record_hex']=body.hex();observer.compare(other)

if __name__=='__main__':unittest.main()
