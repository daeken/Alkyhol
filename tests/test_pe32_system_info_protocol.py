"""Transport tests preserve actual machine observations; not native evidence."""
import struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_system_info as info

class SystemInfoProtocol(unittest.TestCase):
    def test_frozen_native_cpu_and_laa_records_pin_literal_pe_bytes(self):
        native=info.frozen()
        for i,row in enumerate(native['reports']):
            literal='pe32-system-info-v1.exe' if i==0 else 'pe32-system-info-large-v1.exe'
            self.assertEqual(info.digest(info.ROOT/'pe/reference'/literal),native['inputs']['files'][row['file']])
            self.assertEqual(row['record']['information_words'][3],0xfffeffff if i else 0x7ffeffff)
            self.assertEqual(row['record']['information_words'][8],0x01010019)

    def test_every_machine_field_is_checked_against_actual_cpu_and_parent_facts(self):
        record=info.decode(struct.pack('<25I',*self.packet()));info.validate_machine(record,False,2)
        for index in range(9):
            changed={**record,'information_words':record['information_words'][:]};changed['information_words'][index]^=1
            with self.subTest(index=index),self.assertRaises(ValueError):info.validate_machine(changed,False,2)
        with self.assertRaises(ValueError):info.validate_machine(record,True,2)
        with self.assertRaises(ValueError):info.validate_machine(record,False,1)
        with self.assertRaises(ValueError):info.validate_machine({**record,'last_error':0},False,2)

    def test_real_storage_addresses_and_effects_are_required_not_just_profile_numbers(self):
        words=[0x424d3332,4,65536,0x7ffeffff,0x400000,0x50000,0x403004,0x60000,65540,0x90000,0x71,0x19,1,1,0xface1234,0x715e7a11,0x715e7a11,18]
        data=struct.pack('<18I',*words);self.assertEqual(info.decode_storage(data,False)['words'],words)
        for index in [4,5,6,7,9]:
            changed=words[:];changed[index]=0xffffffff
            with self.subTest(pointer=index),self.assertRaises(ValueError):info.decode_storage(struct.pack('<18I',*changed),False)
        for index in [0,1,2,3,8,10,11,12,13,14,15,16,17]:
            changed=words[:];changed[index]^=1
            with self.subTest(field=index),self.assertRaises(ValueError):info.decode_storage(struct.pack('<18I',*changed),False)
        with self.assertRaises(ValueError):info.decode_storage(data,True)
        with self.assertRaises(ValueError):info.decode_storage(data[:-1],False)
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
