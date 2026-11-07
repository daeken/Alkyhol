"""Synthetic observer protocol checks; actual Windows evidence is separate."""
import copy
import json
from pathlib import Path
import struct
import subprocess
import sys
import unittest
import tempfile
from unittest import mock
ROOT=Path(__file__).resolve().parents[1];sys.path.insert(0,str(ROOT/'tools'))
import run_pe_text_conversion as runner
import compare_pe_text_conversion as comparator
from build_pe_text_conversion import SOURCES


class TextConversionProtocolTests(unittest.TestCase):
    def observation(self):
        # Structurally valid synthetic records; these are not API observations.
        words=[0x54584655,56,20]
        for api,lengths,strict,seed in [(1,runner.BYTE_LENGTHS,8,0xa55a),(2,runner.WIDE_LENGTHS,128,0xa5)]:
            for index,length in enumerate(lengths):
                for flags in [0,strict]:
                    invalid=length==0 or (flags==strict and (index not in runner.BYTE_VALID if api==1 else index>=5))
                    words += [api,index,flags,length&0xffffffff]
                    if invalid:words += [0,87 if length==0 else 1113,0xffffffff,0xffffffff]+[seed]*16
                    else:words += [1,0x1234abcd,1,0x1234abcd,0]+[seed]*15
        words += [0,122,0,122,0x454e4421]
        return {'status':0,'words':words,'stdout_hex':struct.pack('<1832I',*words).hex()}
    def reports(self):
        native={'schema_version':1,'backend':runner.NATIVE,'pe_sha256':'b'*64,'executed_pe_sha256':'b'*64,'sources':{s:'c'*64 for s in SOURCES},
                'normal':self.observation(),'rebased':None,'revision':'a'*40,'tracked_tree_clean':True}
        host=copy.deepcopy(native);host.update(backend=runner.HOST,rebased=copy.deepcopy(native['normal']))
        return [native,host]
    def test_complete_independent_pair_and_exact_loader_prefix(self):
        reports=self.reports();self.assertEqual(comparator.compare(reports,reference=reports[0]),1832)
        raw=bytes.fromhex(reports[0]['normal']['stdout_hex']);prefix=b'PE64: 4 sections, 7 named imports, 1 DIR64 fixups\nentering Win64 code; relocated=true\n'
        self.assertEqual(runner.decode(subprocess.CompletedProcess([],0,raw,b''),True),reports[0]['normal'])
        self.assertEqual(runner.decode(subprocess.CompletedProcess([],0,raw,prefix),False,True),reports[0]['normal'])
    def test_crashes_incomplete_bytes_diagnostics_and_missing_relocation_fail(self):
        raw=bytes.fromhex(self.observation()['stdout_hex']);prefix=b'PE64: 4 sections, 7 named imports, 1 DIR64 fixups\nentering Win64 code; relocated=false\n'
        for code,data,stderr in [(-11,raw,b''),(0,raw[:-1],b''),(0,raw+b'\0',b''),(0,raw,b'noise'),(0,raw,prefix)]:
            with self.assertRaises(ValueError):runner.decode(subprocess.CompletedProcess([],code,data,stderr),False,True)
    def test_invalid_call_identity_failed_valid_queries_dropped_and_overwritten_tail_fail(self):
        for offset,value in [(0,0),(1,55),(3,2),(4,1),(5,8),(6,12),(7,0),(9,2),(3+8+15,0),(1831,0)]:
            observed=self.observation();observed['words'][offset]=value;observed['stdout_hex']=struct.pack('<1832I',*observed['words']).hex()
            with self.assertRaises(ValueError):runner.validate(observed)
        observed=self.observation();observed['status']=False
        with self.assertRaises(ValueError):runner.validate(observed)
    def test_input_execution_backend_revision_raw_bytes_and_rebase_are_required(self):
        for key,value in [('schema_version',True),('tracked_tree_clean',False),('revision','d'*40),('backend',runner.NATIVE),('pe_sha256','e'*64),('executed_pe_sha256','e'*64),('sources',{}),('rebased',None)]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):comparator.compare(reports,reference=self.reports()[0])
        reports=self.reports();reports[1]['normal']['stdout_hex']='00'
        with self.assertRaises(ValueError):comparator.compare(reports,reference=self.reports()[0])
    def test_different_actual_results_and_joint_input_drift_cannot_replace_reference(self):
        reports=self.reports();reports[1]['normal']['words'][11]=65;reports[1]['normal']['stdout_hex']=struct.pack('<1832I',*reports[1]['normal']['words']).hex()
        with self.assertRaises(ValueError):comparator.compare(reports,reference=self.reports()[0])
        reports=self.reports()
        for r in reports:r.update(pe_sha256='d'*64,executed_pe_sha256='d'*64)
        with self.assertRaises(ValueError):comparator.compare(reports,reference=self.reports()[0])

    def test_default_frozen_native_record_missing_reference_and_joint_input_drift(self):
        native=json.loads(runner.REFERENCE.read_text(encoding='utf-8'))
        host=copy.deepcopy(native);host.update(backend=runner.HOST,rebased=copy.deepcopy(native['normal']))
        self.assertEqual(comparator.compare([native,host]),1832)
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch.object(comparator,'REFERENCE',Path(directory)/'missing.json'):
                with self.assertRaises(FileNotFoundError):comparator.compare([native,host])
        for record in [native,host]:record.update(pe_sha256='d'*64,executed_pe_sha256='d'*64)
        with self.assertRaisesRegex(ValueError,'frozen native'):comparator.compare([native,host])


if __name__=='__main__':unittest.main()
