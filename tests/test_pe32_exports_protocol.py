"""Synthetic collector guards; these do not execute the owned PE or any game."""
import hashlib
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
from build_pe32_exports import SOURCES
from run_pe32_exports import verify,decode,REFERENCE,HOST,EXPECTED_WORDS
from compare_pe32_exports import compare


class Pe32ExportProtocolTests(unittest.TestCase):
    def reports(self):
        native=json.loads(REFERENCE.read_text(encoding='utf-8'))
        host=copy.deepcopy(native);host['backend']=HOST
        host['normal']['stderr_hex']=b'PE32: 4 sections, 7 imports, 49 HIGHLOW fixups\nchecked i386 mapping; relocated=true\n'.hex()
        host['rebased']=copy.deepcopy(host['normal'])
        return native,host

    def test_frozen_native_words_include_distinct_missing_name_and_ordinal_errors(self):
        native,host=self.reports();self.assertEqual(compare([native,host]),26)
        self.assertEqual(native['normal']['words'][17::2],[127,127,182,182,127])

    def test_every_raw_byte_and_decoded_word_crashes_and_missing_rebase_are_checked(self):
        for backend in [0,1]:
            for at in range(104):
                reports=list(self.reports());row=reports[backend]['normal']
                data=bytearray.fromhex(row['stdout_hex']);data[at]^=1;row['stdout_hex']=data.hex()
                with self.assertRaises(ValueError):compare(reports)
            for word in range(26):
                reports=list(self.reports());reports[backend]['normal']['words'][word]^=1
                with self.assertRaises(ValueError):compare(reports)
            for status in [False,81,-11,0xc0000005]:
                reports=list(self.reports());reports[backend]['normal']['status']=status
                with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports());reports[1]['rebased']=None
        with self.assertRaises((ValueError,TypeError)):compare(reports)

    def test_actual_relocation_and_separate_native_diagnostics_are_required(self):
        for backend,key,value in [(0,'normal',b'warning'),(1,'normal',b''),(1,'rebased',b'PE32: 4 sections, 7 imports, 49 HIGHLOW fixups\nchecked i386 mapping; relocated=false\n')]:
            reports=list(self.reports());reports[backend][key]['stderr_hex']=value.hex()
            with self.assertRaises(ValueError):compare(reports)
        raw=struct.pack('<26I',*EXPECTED_WORDS)
        with self.assertRaises(ValueError):decode(subprocess.CompletedProcess([],0,raw,b''),windows=False)
        diagnostic=bytes.fromhex(self.reports()[1]['normal']['stderr_hex'])
        self.assertEqual(decode(subprocess.CompletedProcess([],0,raw,diagnostic),False,True)['words'],EXPECTED_WORDS)
        with self.assertRaises(ValueError):decode(subprocess.CompletedProcess([],0,raw,diagnostic.replace(b'true',b'false')),False,True)

    def test_clean_revision_scalar_types_exact_payload_and_frozen_identity_are_required(self):
        for field,value in [('schema_version',True),('tracked_tree_clean',False),('revision','e'*40),('sources',{}),('pe_sha256','e'*64),('executed_pe_sha256','f'*64)]:
            reports=list(self.reports());reports[1][field]=value
            with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports());reports[1]['normal']['words'][2]=True
        with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports())
        for r in reports:r['pe_sha256']=r['executed_pe_sha256']='f'*64
        with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports())
        for r in reports:r['sources'][SOURCES[0]]='e'*64
        with self.assertRaises(ValueError):compare(reports)
        with tempfile.TemporaryDirectory() as temp:
            with mock.patch('compare_pe32_exports.REFERENCE',Path(temp)/'missing.json'):
                with self.assertRaises(FileNotFoundError):compare(self.reports())

    def test_raw_observer_status_length_and_stderr_cannot_be_ignored(self):
        raw=struct.pack('<26I',*range(26))
        r=decode(subprocess.CompletedProcess([],0,raw,b''))
        self.assertEqual(r['words'],list(range(26)))
        self.assertEqual(bytes.fromhex(r['stdout_hex']),raw)
        for status,out,err in [(81,raw,b''),(-11,raw,b''),(0,raw[:-1],b''),(0,raw+b'X',b''),(0,raw,b'warning')]:
            with self.assertRaises(ValueError):decode(subprocess.CompletedProcess([],status,out,err))

    def test_changed_binary_missing_sources_and_source_drift_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            pe=Path(temp)/'owned.exe';pe.write_bytes(b'owned synthetic input')
            manifest={'schema_version':1,'pe_sha256':hashlib.sha256(pe.read_bytes()).hexdigest(),
                      'sources':{s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
            path=pe.parent/'pe32-exports-fixture.json'
            path.write_text(json.dumps(manifest));self.assertEqual(verify(pe),manifest)
            pe.write_bytes(b'changed input')
            with self.assertRaises(ValueError):verify(pe)
            pe.write_bytes(b'owned synthetic input')
            for field,value in [('schema_version',True),('sources',{}),('sources',{s:'0'*64 for s in SOURCES})]:
                bad={**manifest,field:value};path.write_text(json.dumps(bad))
                with self.assertRaises(ValueError):verify(pe)


if __name__=='__main__':unittest.main()
