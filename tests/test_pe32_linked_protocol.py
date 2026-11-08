"""Synthetic transport/identity negatives; native DLL execution is separate."""
import copy
import json
from pathlib import Path
import struct
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from compare_pe32_linked import compare
from run_pe32_linked import NATIVE,HOST,RESTORED,SOURCES,FILES,LEAF_REFERENCE


def record(words,diagnostic=b''):
    return {'status':0,'words':words,'stdout_hex':struct.pack('<81I',*words).hex(),'stderr_hex':diagnostic.hex()}


def pair():
    leaf=json.loads(LEAF_REFERENCE.read_text(encoding='utf-8'))
    common={'schema_version':1,'revision':'a'*40,'tracked_tree_clean':True,
            'files':{n:'b'*64 for n in FILES},'sources':{n:'c'*64 for n in SOURCES}}
    common['files']['accepted.dll']=leaf['files']['accepted.dll'];common['sources'].update(leaf['sources'])
    words=list(range(81))
    native={**copy.deepcopy(common),'backend':NATIVE,'normal':record(words),'rebased':None}
    host={**common,'backend':HOST,'normal':record(words,RESTORED+b'false\n'),'rebased':record(words,RESTORED+b'true\n')}
    return native,host


class LinkedProtocol(unittest.TestCase):
    def test_complete_native_and_host_records_and_original_dependency_identity(self):
        w,h=pair();self.assertEqual(compare(w,h,copy.deepcopy(w)),81)
        self.assertEqual(compare(w,h,None),81) # Explicit first-capture mode only.

    def test_missing_failure_truncation_decoding_or_cleanup_does_not_pass(self):
        w,h=pair()
        for key,field,value in [('normal','status',False),('normal','status',-11),('normal','status',1),
                                ('normal','stdout_hex','00'),('normal','words',[0]*81),
                                ('rebased','stderr_hex',(RESTORED+b'false\n').hex())]:
            bad=copy.deepcopy(h);bad[key][field]=value
            with self.assertRaises(ValueError):compare(w,bad,w)

    def test_identity_revision_cleanliness_and_backend_must_match(self):
        w,h=pair()
        for key,value in [('revision','d'*40),('tracked_tree_clean',1),('schema_version',True),
                          ('backend',NATIVE),('sources',{}),('files',{})]:
            bad=copy.deepcopy(h);bad[key]=value
            with self.assertRaises(ValueError):compare(w,bad,w)

    def test_joint_input_and_behavior_drift_fails_original_frozen_inputs(self):
        w,h=pair();frozen=copy.deepcopy(w)
        for value in [w,h]:value['files']['accepted.dll']='e'*64
        with self.assertRaises(ValueError):compare(w,h,None)
        w,h=pair();words=w['normal']['words'];words[0]=999
        w['normal']=record(words);h['normal']=record(words,RESTORED+b'false\n');h['rebased']=record(words,RESTORED+b'true\n')
        with self.assertRaises(ValueError):compare(w,h,frozen)


if __name__=='__main__':unittest.main()
