"""Synthetic transport/identity negatives, separate from real DLL execution."""
import copy
from pathlib import Path
import struct
import subprocess
import sys
import unittest

sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
from compare_pe32_library import compare
from run_pe32_library import NATIVE,HOST,RESTORED,MISMATCH,SOURCES,FILES,decode


def record(words,status=0,diagnostic=b''):
    return {'status':status,'words':words,'stdout_hex':struct.pack('<'+'I'*len(words),*words).hex(),'stderr_hex':diagnostic.hex()}


def pair():
    # Distinct values expose truncated/reordered lifecycle records. These are
    # protocol test data, not an invented native execution observation.
    words=list(range(80))
    common={'schema_version':1,'revision':'a'*40,'tracked_tree_clean':True,
            'files':{n:'b'*64 for n in FILES},'sources':{n:'c'*64 for n in SOURCES}}
    native={**copy.deepcopy(common),'backend':NATIVE,'normal':record(words),'rebased':None,'mismatch_rejected':None}
    bad=words[:12]+words[20:28]+words[36:48];bad[15]=1;bad[29]=1
    host={**common,'backend':HOST,'normal':record(words,diagnostic=RESTORED+b'false\n'),
          'rebased':record(words,diagnostic=RESTORED+b'true\n'),'mismatch_rejected':record(bad,78,MISMATCH)}
    return native,host


class DllProtocol(unittest.TestCase):
    def test_complete_records_and_frozen_native_identity(self):
        native,host=pair()
        self.assertEqual(compare(native,host,copy.deepcopy(native)),80)
        self.assertEqual(compare(native,host,None),80) # Initial capture only.

    def test_process_failure_truncation_and_fabricated_decoding_never_pass(self):
        for field,value in [('returncode',-11),('returncode',1),('stdout',b'\0'*319),('stderr',b'warning')]:
            data={'returncode':0,'stdout':b'\0'*320,'stderr':b''};data[field]=value
            with self.assertRaises(ValueError):decode(subprocess.CompletedProcess([],**data),windows=True)
        native,host=pair()
        for key,field,value in [('normal','status',False),('normal','status',-4),('normal','stdout_hex','00'),
                                ('normal','words',[0]*80),('rebased','stderr_hex',(RESTORED+b'false\n').hex()),
                                ('mismatch_rejected','status',0),('mismatch_rejected','words',[0]*32)]:
            bad=copy.deepcopy(host);bad[key][field]=value
            with self.assertRaises(ValueError):compare(native,bad,native)

    def test_identity_clean_revision_backend_and_complete_file_rosters_required(self):
        native,host=pair()
        for key,value in [('tracked_tree_clean',False),('tracked_tree_clean',1),('schema_version',True),
                          ('revision','d'*40),('backend',NATIVE),('files',{}),('sources',{})]:
            bad=copy.deepcopy(host);bad[key]=value
            with self.assertRaises(ValueError):compare(native,bad,native)
        for key in ['files','sources']:
            bad=copy.deepcopy(host);bad[key][next(iter(bad[key]))]='e'*64
            with self.assertRaises(ValueError):compare(native,bad,native)

    def test_joint_behavior_or_input_drift_still_fails_frozen_reference(self):
        native,host=pair();reference=copy.deepcopy(native)
        native['normal']['words'][0]=999
        native['normal']=record(native['normal']['words'])
        host['normal']=record(native['normal']['words'],diagnostic=RESTORED+b'false\n')
        host['rebased']=record(native['normal']['words'],diagnostic=RESTORED+b'true\n')
        with self.assertRaises(ValueError):compare(native,host,reference)
        native,host=pair()
        for obj in [native,host]:obj['files']['accepted.dll']='d'*64
        with self.assertRaises(ValueError):compare(native,host,reference)


if __name__=='__main__':unittest.main()
