"""Synthetic protocol rejection checks; actual Windows/M4 execution is separate."""
import copy
import json
from pathlib import Path
import struct
import sys
import unittest
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'tools'))
import run_pe32_process as runner
import compare_pe32_process as comparator

def rec(words,status=7,stderr=b''):
    return {'words':words,'stdout_hex':struct.pack('<'+'I'*len(words),*words).hex(),'status':status,'stderr_hex':stderr.hex()}

class Process32ProtocolTests(unittest.TestCase):
    def reports(self):
        native={'schema_version':1,'backend':runner.NATIVE,'files':{n:'a'*64 for n in runner.FILES},'sources':{n:'b'*64 for n in runner.SOURCES},'revision':'c'*40,'tracked_tree_clean':True,'normal':rec(list(range(105))),'rebased':None,'rejected':None}
        host=copy.deepcopy(native);host['backend']=runner.HOST
        host['normal']['stderr_hex']=(runner.RESTORED+b'false\n').hex()
        host['rebased']=rec(list(range(105)),stderr=runner.RESTORED+b'true\n')
        host['rejected']={n:{'status':78,'stdout_hex':'','stderr_hex':s.hex()} for n,s in runner.REJECTIONS.items()}
        words=[0x4c323349,0,1,0,1,4,1,1,0x12345678,0,0x31415926,1,
               0x43323349,3,4,1,0x31415926,0x11223344,0x55667788,0x99aabbcc,
               0x4641494c,1,1,0x31415926,4,1,
               0x43323349,3,3,2,0x31415926,0x11223344,0x55667788,0x99aabbcc,
               0x4641494c,0,0,0x31415926,3,1,
               0x4c323349,0,0,1,1,4,1,1,0x12345678,2,0x31415926,1]
        host['rejected']['failed']['stdout_hex']=struct.pack('<52I',*words).hex()
        return native,host

    def test_complete_records_include_all_words_and_host_only_restrictions(self):
        native,host=self.reports();self.assertEqual(comparator.compare(native,host,native),105)
        for field,value in [('rebased',host['normal']),('rejected',host['rejected'])]:
            altered=copy.deepcopy(native);altered[field]=value
            with self.assertRaises(ValueError):comparator.compare(altered,host,native)

    def test_every_raw_word_and_both_actual_relocation_runs_are_required(self):
        native,host=self.reports()
        for key in ['normal','rebased']:
            for i in range(105):
                altered=copy.deepcopy(host);altered[key]['words'][i]^=1
                altered[key]['stdout_hex']=struct.pack('<105I',*altered[key]['words']).hex()
                with self.assertRaises(ValueError):comparator.compare(native,altered,native)
            for field,value in [('status',0),('status',True),('status',-11),('words',[]),('stdout_hex','00'),('stderr_hex','')]:
                altered=copy.deepcopy(host);altered[key][field]=value
                with self.assertRaises(ValueError):comparator.compare(native,altered,native)

    def test_identity_clean_revision_and_frozen_evidence_cannot_drift_together(self):
        native,host=self.reports()
        for key,value in [('schema_version',True),('backend',runner.NATIVE),('revision','d'*40),('tracked_tree_clean',False),('files',{}),('sources',{})]:
            altered=copy.deepcopy(host);altered[key]=value
            with self.assertRaises(ValueError):comparator.compare(native,altered,native)
        changed=copy.deepcopy(native);other=copy.deepcopy(host)
        for value in [changed,other]:value['files']['program.exe']='d'*64
        with self.assertRaises(ValueError):comparator.compare(changed,other,native)
        with self.assertRaises(ValueError):comparator.compare(native,host,None)

    def test_preflight_never_emits_guest_bytes_and_failed_attach_requires_real_rollback(self):
        native,host=self.reports()
        for mode in runner.REJECTIONS:
            for field,value in [('status',0),('status',-4),('status',False),('stdout_hex','00'),('stderr_hex','')]:
                altered=copy.deepcopy(host);altered['rejected'][mode][field]=value
                with self.assertRaises(ValueError):comparator.compare(native,altered,native)
            altered=copy.deepcopy(host);del altered['rejected'][mode]
            with self.assertRaises(ValueError):comparator.compare(native,altered,native)

    def test_actual_frozen_native_record_rejects_joint_body_drift(self):
        native=json.loads(runner.REFERENCE.read_text(encoding='utf-8'))
        host=copy.deepcopy(native);host['backend']=runner.HOST
        host['normal']['stderr_hex']=(runner.RESTORED+b'false\n').hex()
        host['rebased']=copy.deepcopy(host['normal']);host['rebased']['stderr_hex']=(runner.RESTORED+b'true\n').hex()
        host['rejected']=self.reports()[1]['rejected']
        reference=json.loads(runner.REFERENCE.read_text(encoding='utf-8'))
        self.assertEqual(comparator.compare(native,host,reference),105)
        for value in [native['normal'],host['normal'],host['rebased']]:
            value['words'][54]^=1
            value['stdout_hex']=struct.pack('<105I',*value['words']).hex()
        with self.assertRaisesRegex(ValueError,'frozen process evidence'):comparator.compare(native,host,reference)

if __name__=='__main__':unittest.main()
