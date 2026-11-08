"""Synthetic snapshot/raw-collection guards, not native GS/cookie evidence."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe32_load_config import SOURCES,CASES
from run_pe32_load_config import verify,capture


class Pe32LoadConfigProtocolTests(unittest.TestCase):
    def test_snapshot_case_parameters_types_and_source_payload_identity_are_checked(self):
        with tempfile.TemporaryDirectory() as temp:
            directory=Path(temp);rows=[]
            for name,size,initial,absent,no_cookie,fault in CASES:
                data=('synthetic '+name).encode();(directory/(name+'.exe')).write_bytes(data)
                rows.append(dict(case=name,config_size=size,initial_cookie=initial,absent=absent,no_cookie=no_cookie,fault=fault,pe_sha256=hashlib.sha256(data).hexdigest()))
            manifest=dict(schema_version=1,sources={s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES},cases=rows)
            path=directory/'load-config-fixture.json';path.write_text(json.dumps(manifest))
            self.assertEqual(len(verify(directory)[1]),8)
            for change in ['schema','source','dropped','order','size','bool','hash']:
                bad=copy.deepcopy(manifest)
                if change=='schema':bad['schema_version']=True
                elif change=='source':bad['sources'][SOURCES[0]]='f'*64
                elif change=='dropped':bad['cases'].pop()
                elif change=='order':bad['cases'].reverse()
                elif change=='size':bad['cases'][0]['config_size']=False
                elif change=='bool':bad['cases'][0]['absent']=1
                else:bad['cases'][0]['pe_sha256']='e'*64
                path.write_text(json.dumps(bad))
                with self.assertRaises(ValueError):verify(directory)
            path.write_text(json.dumps(manifest));(directory/'absent.exe').write_bytes(b'changed')
            with self.assertRaises(ValueError):verify(directory)

    def test_raw_process_trap_launch_error_and_timeout_remain_distinct(self):
        for status in [0,83,0xc000001d,-4]:
            with mock.patch('run_pe32_load_config.subprocess.run',return_value=subprocess.CompletedProcess([],status,b'raw',b'diagnostic')):
                self.assertEqual(capture([],'.'),dict(kind='process',status=status,stdout_hex=b'raw'.hex(),stderr_hex=b'diagnostic'.hex()))
        error=OSError(8,'invalid executable');error.winerror=193
        with mock.patch('run_pe32_load_config.subprocess.run',side_effect=error):
            self.assertEqual(capture([],'.'),dict(kind='launch_error',winerror=193,errno=8))
        for error in [OSError(8,'other'),subprocess.TimeoutExpired([],30)]:
            with mock.patch('run_pe32_load_config.subprocess.run',side_effect=error):
                with self.assertRaises(type(error)):capture([],'.')


if __name__=='__main__':unittest.main()
