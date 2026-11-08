"""Synthetic transport negatives, separate from actual native exception evidence."""
import copy
from pathlib import Path
import struct
import sys
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from compare_pe32_seh import compare
from run_pe32_seh import NATIVE,HOST,SOURCES,CASES,RESTORED,REJECTIONS


def record(words,diagnostic=b''):
    return {'status':0,'words':words,'stdout_hex':struct.pack('<32I',*words).hex(),'stderr_hex':diagnostic.hex()}


def pair():
    common={'schema_version':1,'revision':'a'*40,'tracked_tree_clean':True,
            'sources':{n:'b'*64 for n in SOURCES},
            'cases':[{'case':name,'mode':mode,'pe_sha256':str(mode)*64} for name,mode in CASES]}
    words=list(range(32))
    native={**copy.deepcopy(common),'backend':NATIVE,'executed_cases':['pe32-seh'],
            'normal':record(words),'rebased':None,'rejected':None}
    host={**common,'backend':HOST,'executed_cases':[n for n,_ in CASES],
          'normal':record(words,RESTORED+b'false\n'),'rebased':record(words,RESTORED+b'true\n'),
          'rejected':{n:{'status':78,'stdout_hex':'','stderr_hex':d.hex()} for n,d in REJECTIONS.items()}}
    return native,host


class SehProtocol(unittest.TestCase):
    def test_actual_records_and_initial_frozen_comparison(self):
        w,h=pair();self.assertEqual(compare(w,h,copy.deepcopy(w)),32)
        self.assertEqual(compare(w,h,None),32)

    def test_crashes_dropped_rejections_and_missing_cleanup_never_count_as_recovery(self):
        w,h=pair()
        for key,field,value in [('normal','status',-4),('normal','status',1),('normal','status',False),
                                ('normal','stdout_hex','00'),('normal','words',[0]*32),
                                ('rebased','stderr_hex',(RESTORED+b'false\n').hex())]:
            bad=copy.deepcopy(h);bad[key][field]=value
            with self.assertRaises(ValueError):compare(w,bad,w)
        for name in REJECTIONS:
            for status in [-4,-11,0,True]:
                bad=copy.deepcopy(h);bad['rejected'][name]['status']=status
                with self.assertRaises(ValueError):compare(w,bad,w)
        bad=copy.deepcopy(h);bad['rejected'].pop('nested')
        with self.assertRaises(ValueError):compare(w,bad,w)

    def test_clean_revision_sources_payloads_and_executed_case_rosters_required(self):
        w,h=pair()
        for key,value in [('tracked_tree_clean',1),('schema_version',True),('backend',NATIVE),
                          ('revision','c'*40),('sources',{}),('executed_cases',['pe32-seh']),('cases',[])]:
            bad=copy.deepcopy(h);bad[key]=value
            with self.assertRaises(ValueError):compare(w,bad,w)
        bad=copy.deepcopy(w);bad['rejected']=h['rejected']
        with self.assertRaises(ValueError):compare(bad,h,w)

    def test_joint_semantic_or_binary_drift_is_rejected_by_frozen_native_evidence(self):
        w,h=pair();frozen=copy.deepcopy(w)
        words=w['normal']['words'];words[0]=999
        w['normal']=record(words);h['normal']=record(words,RESTORED+b'false\n');h['rebased']=record(words,RESTORED+b'true\n')
        with self.assertRaises(ValueError):compare(w,h,frozen)
        w,h=pair()
        for value in [w,h]:value['cases'][0]['pe_sha256']='d'*64
        with self.assertRaises(ValueError):compare(w,h,frozen)


if __name__=='__main__':unittest.main()
