"""Synthetic snapshot/raw-collection guards, not native GS/cookie evidence."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import struct
import unittest
from unittest import mock
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe32_load_config import SOURCES,CASES
from run_pe32_load_config import verify,capture,REFERENCE,HOST
from run_pe32_load_config_geometry import REFERENCE as GREFERENCE,HOST as GHOST
from compare_pe32_load_config import compare


class Pe32LoadConfigProtocolTests(unittest.TestCase):
    def reports(self,geometry=False):
        native=json.loads((GREFERENCE if geometry else REFERENCE).read_text())
        host=copy.deepcopy(native);host['backend']=GHOST if geometry else HOST
        for row in host['normal']:
            fault=row['case']=='corrupt_frame';imports=2 if fault else 4
            fixups=10 if fault else 29 if row['case']=='absent' else 37 if row['case']=='no_cookie' else 38
            diagnostic=f'PE32: 4 sections, {imports} imports, {fixups} HIGHLOW fixups\nchecked i386 mapping; relocated=true\n'
            if fault:row['status']=-4
            else:diagnostic+='host segment selectors restored; owned FS descriptor cleared\n'
            row['stderr_hex']=diagnostic.encode().hex()
        host['rebased']=copy.deepcopy(host['normal'])
        return native,host

    def test_actual_frozen_properties_and_deliberate_trap_are_separate(self):
        self.assertEqual(compare(self.reports()),(7,1));self.assertEqual(compare(self.reports(True)),(4,0))
        native,host=self.reports()
        self.assertEqual(native['normal'][-1]['status'],0xc000001d)
        self.assertEqual(host['normal'][-1]['status'],-4)
        words=[struct.unpack('<16I',bytes.fromhex(r['stdout_hex'])) for r in native['normal'][:-1]]
        self.assertEqual([r[3] for r in words],[0,0,0,1,0,0,0])
        self.assertTrue(all(r[10]==0 for r in words)) # Compiler's valid-frame inline fast path.

    def test_every_case_raw_outputs_unexplained_crashes_and_probe_failures_are_rejected(self):
        for geometry in [False,True]:
            for backend in [0,1]:
                for case in range(len(self.reports(geometry)[0]['normal'])):
                    reports=list(self.reports(geometry));row=reports[backend]['normal'][case]
                    data=bytearray.fromhex(row['stdout_hex']);data[-1]^=1;row['stdout_hex']=data.hex()
                    with self.assertRaises(ValueError):compare(reports)
                for status in [False,78,83,-11,0xc0000005]:
                    reports=list(self.reports(geometry));reports[backend]['normal'][0]['status']=status
                    with self.assertRaises(ValueError):compare(reports)
        for status in [0,78,-11,-5]:
            reports=list(self.reports());reports[1]['normal'][-1]['status']=status
            with self.assertRaises(ValueError):compare(reports)

    def test_exact_payload_sources_clean_revision_and_frozen_inputs_are_required(self):
        for field,value in [('schema_version',True),('tracked_tree_clean',False),('revision','f'*40),('sources',{})]:
            reports=list(self.reports());reports[1][field]=value
            with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports());reports[1]['cases'][0]['config_size']=False
        with self.assertRaises(ValueError):compare(reports)
        for field in ['sources','cases']:
            reports=list(self.reports())
            for report in reports:
                if field=='sources':report[field][SOURCES[0]]='e'*64
                else:report[field][0]['pe_sha256']='e'*64
            with self.assertRaises(ValueError):compare(reports)
        with tempfile.TemporaryDirectory() as temp:
            with mock.patch('compare_pe32_load_config.REFERENCE',Path(temp)/'missing.json'):
                with self.assertRaises(FileNotFoundError):compare(self.reports())

    def test_actual_forced_relocation_cleanup_and_all_observation_rows_are_required(self):
        for change in ['missing','dropped','relocation','cleanup','native_diagnostic','trap_marker']:
            reports=list(self.reports());host=reports[1]
            if change=='missing':host['rebased']=None
            elif change=='dropped':host['normal'].pop()
            elif change=='relocation':host['rebased'][0]['stderr_hex']=bytes.fromhex(host['rebased'][0]['stderr_hex']).replace(b'true',b'false').hex()
            elif change=='cleanup':host['normal'][0]['stderr_hex']=''
            elif change=='native_diagnostic':reports[0]['normal'][0]['stderr_hex']='ff'
            else:host['normal'][-1]['stdout_hex']=''
            with self.assertRaises(ValueError):compare(reports)

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
