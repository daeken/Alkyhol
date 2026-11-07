"""Synthetic virtual COM observation protocol negatives; native evidence is separate."""
import copy
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe_binary_com import SOURCES
from run_pe_binary_com import REFERENCE,EXPECTED_WORDS,decode
from compare_pe_binary_com import compare,NATIVE,HOST


class BinaryComProtocolTests(unittest.TestCase):
    def execution(self,windows=False):
        return subprocess.CompletedProcess([],0,struct.pack('<'+'I'*len(EXPECTED_WORDS),*EXPECTED_WORDS),
            b'' if windows else b'PE64: 4 sections, 4 named imports, 1 DIR64 fixups\nentering Win64 code; relocated=true\n')

    def reports(self):
        native={'schema_version':1,'backend':NATIVE,'tracked_tree_clean':True,'revision':'a'*40,
                'pe_sha256':'b'*64,'executed_pe_sha256':'b'*64,'sources':{s:'c'*64 for s in SOURCES},
                'observation':decode(self.execution(True),windows=True),'forced_rebase_observation':None}
        host=copy.deepcopy(native)
        host.update(backend=HOST,forced_rebase_observation=copy.deepcopy(host['observation']))
        return [native,host]

    def test_exact_payload_bytes_and_virtual_calls_match(self):
        for windows in [False,True]:
            self.assertEqual(decode(self.execution(windows),windows,rebase=not windows),self.reports()[0]['observation'])
        reports=self.reports()
        self.assertEqual(compare(reports,reference=reports[0]),23)

    def test_crashes_truncation_extra_bytes_and_wrong_virtual_results_fail(self):
        for stdout in [b'',self.execution().stdout[:-1],self.execution().stdout+b'\0',b'X'+self.execution().stdout[1:]]:
            run=self.execution()
            run.stdout=stdout
            with self.assertRaises(ValueError): decode(run)
        run=self.execution()
        run.returncode=-11
        with self.assertRaises(ValueError): decode(run)
        for index in [7,8,10,12,13,14,15,16,17,18,21,22]:
            run=self.execution()
            words=EXPECTED_WORDS.copy()
            words[index]^=1
            run.stdout=struct.pack('<'+'I'*len(words),*words)
            with self.assertRaises(ValueError): decode(run)

    def test_diagnostics_and_missing_forced_rebase_are_not_ignored(self):
        for stderr in [b'',b'noise',self.execution().stderr.replace(b'true',b'false')]:
            run=self.execution()
            run.stderr=stderr
            with self.assertRaises(ValueError): decode(run,rebase=True)
        run=self.execution(True)
        run.stderr=b'fake success'
        with self.assertRaises(ValueError): decode(run,windows=True)

    def test_identity_backend_raw_bytes_and_joint_frozen_drift_fail(self):
        for field,value in [('tracked_tree_clean',False),('backend',NATIVE),('revision','d'*40),
            ('pe_sha256','e'*64),('sources',{}),('forced_rebase_observation',None)]:
            reports=self.reports()
            reports[1][field]=value
            with self.assertRaises(ValueError): compare(reports,reference=self.reports()[0])
        reports=self.reports()
        reports[1]['observation']['stdout_hex']='00'
        with self.assertRaises(ValueError): compare(reports,reference=self.reports()[0])
        reports=self.reports()
        for r in reports: r.update(pe_sha256='e'*64,executed_pe_sha256='e'*64)
        with self.assertRaises(ValueError): compare(reports,reference=self.reports()[0])


    def test_default_native_reference_is_required_and_rejects_joint_identity_drift(self):
        native=json.loads(REFERENCE.read_text(encoding='utf-8'))
        host=copy.deepcopy(native)
        host.update(backend=HOST,forced_rebase_observation=copy.deepcopy(host['observation']))
        self.assertEqual(compare([native,host]),23)
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch('compare_pe_binary_com.REFERENCE',Path(directory)/'missing.json'):
                with self.assertRaises(FileNotFoundError): compare([native,host])
        for r in [native,host]: r.update(pe_sha256='e'*64,executed_pe_sha256='e'*64)
        with self.assertRaises(ValueError): compare([native,host])


if __name__=='__main__': unittest.main()
