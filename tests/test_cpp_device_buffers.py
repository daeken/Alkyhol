"""Synthetic report gates; actual native/Metal execution supplies buffer behavior."""
import copy
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from tools.run_cpp_device_buffers import ROOT,run,validate_observation,compare_reports

class DeviceBufferReportTests(unittest.TestCase):
    def valid(self):
        return json.loads((ROOT/'cpp/reference/device-shared-buffers-windows-v1.json').read_text())['observation']
    def reports(self):
        report={'schema_version':1,'backend':'native Windows SDK WARP device buffers','source_revision':'current','tracked_tree_clean':True,'artifacts':{'cpp/tests/device_buffers.cpp':'source'},'observation':self.valid()}
        other=copy.deepcopy(report);other['backend']='Rust restricted Metal COM device buffers';return [report,other]
    def test_preserves_returned_descriptor_fields_without_inventing_native_values(self):
        value=self.valid();value['resources'][0]['heap'][1]=2;self.assertEqual(validate_observation(json.dumps(value)),value)
    def test_rejects_incomplete_invalid_or_failed_observations(self):
        for key,value in [('resources',[]),('parent_lifetime',False),('gpu_addresses_nonzero',False),('scenario','unknown')]:
            report=self.valid();report[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(report))
        for key,value in [('desc',[1]),('heap',[0]*4),('checksum',True),('heap_flags',-1)]:
            report=self.valid();report['resources'][0][key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(report))
    def test_requires_independent_clean_identical_revision_and_source(self):
        compare_reports(self.reports(),'current')
        with self.assertRaises(ValueError):compare_reports([self.reports()[0]]*2,'current')
        for key,value in [('source_revision','old'),('tracked_tree_clean',False),('schema_version',0)]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        reports=self.reports();reports[1]['artifacts']['cpp/tests/device_buffers.cpp']='changed'
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_any_descriptor_or_mapped_content_difference_fails(self):
        for field in ['heap','desc','checksum','heap_flags']:
            reports=self.reports();row=reports[1]['observation']['resources'][0]
            if isinstance(row[field],list):row[field][0]+=1
            else:row[field]+=1
            with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_live_backends_cannot_jointly_drift_from_frozen_reference(self):
        reports=self.reports()
        for report in reports:report['observation']['resources'][0]['heap_flags']=0
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_frozen_native_buffer_evidence_and_source_identity_are_preserved(self):
        import hashlib
        reference=json.loads((ROOT/'cpp/reference/device-shared-buffers-windows-v1.json').read_text())
        self.assertTrue(reference['tracked_tree_clean'])
        self.assertEqual(reference['backend'],'native Windows SDK WARP device buffers')
        self.assertEqual(reference['source_revision'],'2b5bbe39d90f2c973ffd7277b395aea7ea81296e')
        self.assertEqual(reference['artifacts']['cpp/tests/device_buffers.cpp'],hashlib.sha256((ROOT/'cpp/reference/probes/device_buffers_v1.cpp').read_bytes()).hexdigest())
        observed=validate_observation(json.dumps(reference['observation']))
        self.assertEqual(hashlib.sha256(json.dumps(observed,sort_keys=True,separators=(',',':')).encode()).hexdigest(),'b321745cbdcf68bc226f55a51d80adddb4757fe6ad4688a97e3fe3d0c3607167')
    def test_rejects_unsupported_hosts_before_any_build(self):
        with patch('tools.run_cpp_device_buffers.os.name','posix'),patch('tools.run_cpp_device_buffers.platform.system',return_value='Linux'),patch('tools.run_cpp_device_buffers.subprocess.run') as command:
            with self.assertRaises(RuntimeError):run(Path('unused'))
            command.assert_not_called()
