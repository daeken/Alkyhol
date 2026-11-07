"""Synthetic protocol negatives; the actual Windows DLL observer is separate."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import unittest
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe_registered_dll import SOURCES
from run_pe_registered_dll import NATIVE,HOST,EXPECTED,TRACE,decode
from compare_pe_registered_dll import compare


class RegisteredDllProtocolTests(unittest.TestCase):
    def execution(self,case='accepted'):
        value=next(v for v in EXPECTED if v['case']==case)
        return subprocess.CompletedProcess([],0,TRACE,json.dumps({k:value[k] for k in ['loaded','probe','alias']}).encode())

    def reports(self):
        native={'schema_version':1,'backend':NATIVE,'tracked_tree_clean':True,'revision':'a'*40,
                'binaries':{'accepted':'b'*64,'rejected':'c'*64},'sources':{s:'d'*64 for s in SOURCES},
                'normal':copy.deepcopy(EXPECTED),'rebased':None}
        host=copy.deepcopy(native);host.update(backend=HOST,rebased=copy.deepcopy(EXPECTED))
        return [native,host]

    def test_real_lifecycle_protocol_requires_both_statuses_exact_traces_and_exported_calls(self):
        self.assertEqual([decode(self.execution(case),case) for case in ['accepted','rejected']],EXPECTED)
        values=self.reports();self.assertEqual(compare(values,reference=values[0]),40)

    def test_every_raw_trace_byte_crashes_truncation_and_extra_bytes_fail(self):
        for case in ['accepted','rejected']:
            for index in range(len(TRACE)):
                run=self.execution(case);raw=bytearray(run.stdout);raw[index]^=1;run.stdout=bytes(raw)
                with self.assertRaises(ValueError):decode(run,case)
            for code,out in [(-11,TRACE),(78,TRACE),(0,TRACE[:-1]),(0,TRACE+b'\0')]:
                run=self.execution(case);run.returncode=code;run.stdout=out
                with self.assertRaises(ValueError):decode(run,case)

    def test_fake_attach_failures_export_placeholders_and_wrong_scalar_types_fail(self):
        for key,value in [('loaded',1),('loaded',False),('probe',0),('probe',True),('alias',1),('alias',False)]:
            run=self.execution();data=json.loads(run.stderr);data[key]=value;run.stderr=json.dumps(data).encode()
            with self.assertRaises(ValueError):decode(run,'accepted')
        for where in ['normal','rebased']:
            for key,value in [('loaded',1),('probe',True),('alias',1)]:
                values=self.reports();values[1][where][0][key]=value
                with self.assertRaises(ValueError):compare(values,reference=self.reports()[0])

    def test_source_binaries_revision_clean_backend_and_actual_rebase_are_required(self):
        for field,value in [('schema_version',True),('tracked_tree_clean',False),('revision','e'*40),
                            ('backend',NATIVE),('binaries',{}),('sources',{}),('rebased',None)]:
            values=self.reports();values[1][field]=value
            with self.assertRaises(ValueError):compare(values,reference=self.reports()[0])
        values=self.reports();values[0]['rebased']=copy.deepcopy(EXPECTED)
        with self.assertRaises(ValueError):compare(values,reference=self.reports()[0])

    def test_joint_binary_source_or_trace_drift_cannot_replace_native_reference(self):
        for field,value in [('binaries',{'accepted':'e'*64,'rejected':'f'*64}),('sources',{s:'e'*64 for s in SOURCES})]:
            values=self.reports()
            for record in values:record[field]=copy.deepcopy(value)
            with self.assertRaises(ValueError):compare(values,reference=self.reports()[0])


if __name__=='__main__':unittest.main()
