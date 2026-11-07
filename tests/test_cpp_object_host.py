"""Synthetic metadata report gates; execution evidence comes from C++ and WARP."""
import copy
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from tools.run_cpp_object_host import ROOT,compare_reports,run

class MetadataReportTests(unittest.TestCase):
    def reports(self):
        native=json.loads((ROOT/'cpp/reference/object-private-data-windows-v1.json').read_text())
        native.update(source_revision='current',matches_frozen_contract=True)
        host=copy.deepcopy(native);host['backend']='Rust ID3D12Object metadata base (CPU; no GPU)'
        return [native,host]
    def test_accepts_complete_independent_reports(self):compare_reports(self.reports(),'current')
    def test_requires_two_different_backends(self):
        for reports in [[],self.reports()[:1],[self.reports()[0]]*2]:
            with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_rejects_dirty_stale_and_unverified_execution(self):
        for key,value in [('schema_version',0),('source_revision','older'),('tracked_tree_clean',False),('matches_frozen_contract',False)]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_rejects_changed_source_bytes_and_foreign_reference_events(self):
        reports=self.reports();reports[1]['artifacts']['cpp/tests/object_private_data.cpp']='different'
        with self.assertRaises(ValueError):compare_reports(reports,'current')
        reports=self.reports();reports[1]['observation']['records'][-1]['destroyed']=0
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_refuses_an_unsupported_host_before_building(self):
        with patch('tools.run_cpp_object_host.platform.system',return_value='Linux'),patch('tools.run_cpp_object_host.subprocess.run') as command:
            with self.assertRaises(RuntimeError):run(Path('unused'))
            command.assert_not_called()
