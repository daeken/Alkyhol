"""Synthetic collector guards; these do not execute the owned PE or any game."""
import hashlib
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe32_exports import SOURCES
from run_pe32_exports import verify,decode


class Pe32ExportProtocolTests(unittest.TestCase):
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
