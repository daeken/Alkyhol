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
from build_pe32_tls_data import SOURCES
from run_pe32_tls_data import verify,decode,REFERENCE,HOST,EXPECTED_WORDS
from compare_pe32_tls_data import compare


class Pe32TlsDataProtocolTests(unittest.TestCase):
    def reports(self):
        native=json.loads(REFERENCE.read_text(encoding='utf-8'))
        host=copy.deepcopy(native);host['backend']=HOST
        host['normal']['stderr_hex']=b'PE32: 5 sections, 4 imports, 44 HIGHLOW fixups\nchecked i386 mapping; relocated=true\nhost segment selectors restored; owned FS descriptor cleared\n'.hex()
        host['rebased']=copy.deepcopy(host['normal'])
        host['repeat_rejected']=copy.deepcopy(host['normal']);host['repeat_rejected']['status']=78
        host['repeat_rejected']['stderr_hex']+=b'Alkyhol PE32 unsupported: PE32 static TLS images support one entry\n'.hex()
        return native,host

    def test_compiler_tls_data_and_original_template_independence_match_frozen_native(self):
        native,host=self.reports();self.assertEqual(compare([native,host]),24)
        self.assertEqual(native['normal']['words'][8],0)
        self.assertEqual(native['normal']['words'][11],0x31415926)
        self.assertEqual(native['normal']['words'][22],0x31415926)

    def test_every_raw_byte_and_decoded_word_crashes_and_missing_rebase_are_checked(self):
        for backend in [0,1]:
            for at in range(96):
                reports=list(self.reports());row=reports[backend]['normal']
                data=bytearray.fromhex(row['stdout_hex']);data[at]^=1;row['stdout_hex']=data.hex()
                with self.assertRaises(ValueError):compare(reports)
            for word in range(24):
                reports=list(self.reports());reports[backend]['normal']['words'][word]^=1
                with self.assertRaises(ValueError):compare(reports)
            for status in [False,81,-11,0xc0000005]:
                reports=list(self.reports());reports[backend]['normal']['status']=status
                with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports());reports[1]['rebased']=None
        with self.assertRaises((ValueError,TypeError)):compare(reports)

    def test_actual_relocation_and_separate_native_diagnostics_are_required(self):
        for backend,key,value in [(0,'normal',b'warning'),(1,'normal',b''),(1,'rebased',b'PE32: 5 sections, 4 imports, 44 HIGHLOW fixups\nchecked i386 mapping; relocated=false\nhost segment selectors restored; owned FS descriptor cleared\n')]:
            reports=list(self.reports());reports[backend][key]['stderr_hex']=value.hex()
            with self.assertRaises(ValueError):compare(reports)
        raw=struct.pack('<24I',*EXPECTED_WORDS)
        with self.assertRaises(ValueError):decode(subprocess.CompletedProcess([],0,raw,b''),windows=False)
        diagnostic=bytes.fromhex(self.reports()[1]['normal']['stderr_hex'])
        self.assertEqual(decode(subprocess.CompletedProcess([],0,raw,diagnostic),False,True)['words'],EXPECTED_WORDS)
        with self.assertRaises(ValueError):decode(subprocess.CompletedProcess([],0,raw,diagnostic.replace(b'true',b'false')),False,True)

    def test_repeat_is_a_failed_second_entry_after_one_complete_real_body(self):
        row=self.reports()[1]['repeat_rejected'];raw=bytes.fromhex(row['stdout_hex']);diagnostic=bytes.fromhex(row['stderr_hex'])
        self.assertEqual(decode(subprocess.CompletedProcess([],78,raw,diagnostic),False,False,True)['words'],EXPECTED_WORDS)
        for status,out,err in [(0,raw,diagnostic),(78,b'',diagnostic),(78,raw*2,diagnostic),(78,raw,diagnostic[:-1])]:
            with self.assertRaises(ValueError):decode(subprocess.CompletedProcess([],status,out,err),False,False,True)
        for key,value in [('status',0),('stdout_hex',''),('stderr_hex','')]:
            reports=list(self.reports());reports[1]['repeat_rejected'][key]=value
            with self.assertRaises(ValueError):compare(reports)

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
            with mock.patch('compare_pe32_tls_data.REFERENCE',Path(temp)/'missing.json'):
                with self.assertRaises(FileNotFoundError):compare(self.reports())

    def test_raw_observer_status_length_and_stderr_cannot_be_ignored(self):
        raw=struct.pack('<24I',*range(24))
        r=decode(subprocess.CompletedProcess([],0,raw,b''))
        self.assertEqual(r['words'],list(range(24)))
        self.assertEqual(bytes.fromhex(r['stdout_hex']),raw)
        for status,out,err in [(81,raw,b''),(-11,raw,b''),(0,raw[:-1],b''),(0,raw+b'X',b''),(0,raw,b'warning')]:
            with self.assertRaises(ValueError):decode(subprocess.CompletedProcess([],status,out,err))

    def test_changed_binary_missing_sources_and_source_drift_fail(self):
        with tempfile.TemporaryDirectory() as temp:
            pe=Path(temp)/'owned.exe';pe.write_bytes(b'owned synthetic input')
            manifest={'schema_version':1,'pe_sha256':hashlib.sha256(pe.read_bytes()).hexdigest(),
                      'sources':{s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
            path=pe.parent/'pe32-tls-data-fixture.json'
            path.write_text(json.dumps(manifest));self.assertEqual(verify(pe),manifest)
            pe.write_bytes(b'changed input')
            with self.assertRaises(ValueError):verify(pe)
            pe.write_bytes(b'owned synthetic input')
            for field,value in [('schema_version',True),('sources',{}),('sources',{s:'0'*64 for s in SOURCES})]:
                bad={**manifest,field:value};path.write_text(json.dumps(bad))
                with self.assertRaises(ValueError):verify(pe)


if __name__=='__main__':unittest.main()
