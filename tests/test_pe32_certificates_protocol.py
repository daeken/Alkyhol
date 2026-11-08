"""Synthetic comparison negatives using a frozen native record, not execution proof."""
import copy
import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest import mock
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from run_pe32_certificates import REFERENCE,HOST,verify
from build_pe32_certificates import SOURCES,variants
from compare_pe32_certificates import compare
from test_pe32_geometry import sample


class Pe32CertificatesProtocolTests(unittest.TestCase):
    def reports(self):
        native=json.loads(REFERENCE.read_text(encoding='utf-8'))
        host=copy.deepcopy(native);host['backend']=HOST
        for row in host['normal']:
            if row['kind']=='process':
                row['stderr_hex']=b'PE32: 4 sections, 4 imports, 19 HIGHLOW fixups\nchecked i386 mapping; relocated=true\n'.hex()
            else:
                name=row['case'];row.clear();row.update(case=name,kind='process',status=78,stdout_hex='',stderr_hex=b'Alkyhol PE32 unsupported: bounded image extent\n'.hex())
        host['rebased']=copy.deepcopy(host['normal'])
        return native,host

    def test_all_eight_acceptances_and_exact_raw_outputs_agree(self):
        native,host=self.reports();self.assertEqual(compare([native,host]),(8,0))

    def test_crashes_probe_failures_unknown_os_errors_and_every_success_byte_cannot_pass(self):
        for backend in [0,1]:
            for byte in range(32):
                reports=list(self.reports());row=reports[backend]['normal'][0]
                changed=bytearray.fromhex(row['stdout_hex']);changed[byte]^=1;row['stdout_hex']=changed.hex()
                with self.assertRaises(ValueError):compare(reports)
            for status in [81,82,-11,0xc0000005]:
                reports=list(self.reports());reports[backend]['normal'][0]['status']=status
                with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports());reports[0]['normal'][-1]={'case':'overflowing_range','kind':'launch_error','winerror':577,'errno':22}
        with self.assertRaises(ValueError):compare(reports)

    def test_actual_forced_rebase_raw_diagnostics_and_accepted_case_body_are_required(self):
        for change in ['missing','normal','forced','dropped','unsupported']:
            reports=list(self.reports());host=reports[1]
            if change=='missing':host['rebased']=None
            elif change in ['normal','forced']:
                key='normal' if change=='normal' else 'rebased';host[key][0]['stderr_hex']=''
            elif change=='dropped':host['normal'].pop()
            else:host['normal'][1].update(status=78,stdout_hex='',stderr_hex=b'Alkyhol PE32 unsupported: not implemented\n'.hex())
            with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports());reports[1]['rebased'][0]['stderr_hex']=bytes.fromhex(reports[1]['rebased'][0]['stderr_hex']).replace(b'true',b'false').hex()
        with self.assertRaises(ValueError):compare(reports)

    def test_identity_scalar_types_revision_and_default_frozen_inputs_are_required(self):
        for field,value in [('schema_version',True),('tracked_tree_clean',False),('revision','f'*40),('sources',{}),('base_pe_sha256','e'*64)]:
            reports=list(self.reports());reports[1][field]=value
            with self.assertRaises(ValueError):compare(reports)
        for kind in ['status','size','winerror']:
            reports=list(self.reports())
            if kind=='status':reports[1]['normal'][0]['status']=False
            elif kind=='size':reports[1]['cases'][-1]['certificate_size']=False
            else:reports[0]['normal'][-1]={'case':'overflowing_range','kind':'launch_error','winerror':True,'errno':8}
            with self.assertRaises(ValueError):compare(reports)
        reports=list(self.reports())
        for record in reports:record['cases'][1]['pe_sha256']='e'*64
        with self.assertRaises(ValueError):compare(reports)
        with tempfile.TemporaryDirectory() as temp:
            with mock.patch('compare_pe32_certificates.REFERENCE',Path(temp)/'missing.json'):
                with self.assertRaises(FileNotFoundError):compare(self.reports())

    def test_snapshot_verification_rejects_mutation_outside_header_and_joint_manifest_drift(self):
        with tempfile.TemporaryDirectory() as temp:
            directory=Path(temp);raw=bytearray(sample());struct.pack_into('<I',raw,0x98+92,16);raw=bytes(raw);offset,items=variants(raw)
            sources={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in SOURCES}
            digest=hashlib.sha256(raw).hexdigest()
            frozen={'pe_sha256':digest,'sources':{k:v for k,v in sources.items() if k.startswith('pe/')}}
            reference=directory/'reference.json';reference.write_text(json.dumps(frozen))
            (directory/'base.exe').write_bytes(raw)
            for row,data in items:(directory/(row['case']+'.exe')).write_bytes(data)
            manifest={'schema_version':1,'base_pe_sha256':digest,'directory_offset':offset,'sources':sources,'cases':[row for row,_ in items]}
            path=directory/'certificates-fixture.json';path.write_text(json.dumps(manifest))
            with mock.patch('run_pe32_certificates.BASE_REFERENCE',reference):
                self.assertEqual(len(verify(directory)[1]),8)
                bad=bytearray(items[1][1]);bad[-1]^=1;(directory/'empty_certificate.exe').write_bytes(bad)
                with self.assertRaises(ValueError):verify(directory)
                manifest['cases'][1]['pe_sha256']=hashlib.sha256(bad).hexdigest();path.write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):verify(directory)


if __name__=='__main__':unittest.main()
