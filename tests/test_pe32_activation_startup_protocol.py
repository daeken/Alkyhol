"""Synthetic transport checks, not activation or game equivalence evidence."""
import struct
import sys
import unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from observe_pe32_activation_startup import decode,EVENTS,compare,frozen

class ActivationStartupProtocolTests(unittest.TestCase):
    def test_actual_native_scope_and_payload_record_is_required(self):
        import copy
        records=copy.deepcopy(frozen()['records'])
        for row in records:row['path']='validated path on a different host'
        compare(records)
        for field,value in [('current_context_id',0),('current_error',0)]:
            changed=copy.deepcopy(records);changed[0][field]=value
            with self.assertRaises(ValueError):compare(changed)
        changed=copy.deepcopy(records);changed[0]['queries'][0]['data_hex']='00'
        with self.assertRaises(ValueError):compare(changed)

    def fixture(self,first=0x1234,second=0x5678):
        self.paths={1:'C:\\owned\\Alkyhol.OwnedCRT\\ctx-provider.dll',2:'C:\\owned\\ctx-importer.dll',3:'C:\\owned\\ctx-startup.exe'}
        data=bytearray();self.query_offsets=[]
        body=struct.pack('<5I',20,2,0,0,0);section=b'HEAD'+body
        for index,(actor,event) in enumerate(EVENTS):
            current=0 if index in [2,4] else second if index==3 else first
            data+=struct.pack('<7I',0x43545832,actor,event,1,0xface1234,current,4)
            path=self.paths[actor];data+=struct.pack('<I',len(path))+(path+'\0').encode('utf-16-le')
            for flags in [0,1]:
                handle=current if flags else 0
                q=[flags,1,0xface1234,64,1,20,0,len(section),2,0x715e7a11,4,0,int(handle!=0),int(handle==current),1,0x715e7a11,0,handle,0xffffffff]
                self.query_offsets.append(len(data));data+=struct.pack('<19I',*q)
                data+=struct.pack('<I',len(body))+body+struct.pack('<I',len(section))+section
        return bytes(data)

    def test_context_identity_is_normalized_without_dropping_payloads(self):
        a=decode(self.fixture(),self.paths);b=decode(self.fixture(0xaaaa,0xbbbb),self.paths)
        self.assertEqual(a,b)
        self.assertEqual([r['current_context_id'] for r in a],[1,1,0,2,0,1,1])
        self.assertEqual(bytes.fromhex(a[0]['queries'][0]['data_hex']),struct.pack('<5I',20,2,0,0,0))

    def test_truncation_event_loss_extra_data_and_wrong_source_fail(self):
        data=self.fixture()
        for n in [0,1,27,28,len(data)//2,len(data)-1]:
            with self.assertRaises(ValueError):decode(data[:n],self.paths)
        with self.assertRaises(ValueError):decode(data+b'\0',self.paths)
        wrong=dict(self.paths);wrong[1]=self.paths[2]
        with self.assertRaises(ValueError):decode(data,wrong)

    def test_guard_corruption_and_inconsistent_section_data_cannot_pass(self):
        data=self.fixture();offset=self.query_offsets[0]
        changed=bytearray(data);struct.pack_into('<I',changed,offset+15*4,0)
        with self.assertRaises(ValueError):decode(bytes(changed),self.paths)
        changed=bytearray(data);struct.pack_into('<I',changed,offset+10*4,0)
        with self.assertRaises(ValueError):decode(bytes(changed),self.paths)
        changed=bytearray(data);struct.pack_into('<I',changed,offset+19*4,4097)
        with self.assertRaises(ValueError):decode(bytes(changed),self.paths)

if __name__=='__main__':unittest.main()
