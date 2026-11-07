"""Synthetic root report protocol tests, separate from SDK/COM execution."""
import copy
import hashlib
import json
import unittest
from tools.run_cpp_root_signatures import ROOT,SOURCE,compare_reports,validate_observation
class RootReportTests(unittest.TestCase):
    def reports(self):
        # Measured payload in synthetic envelopes, not a GPU/SDK execution.
        observation=json.loads((ROOT/'cpp/reference/root-signatures-windows-v1.json').read_text())['observation']
        native={'schema_version':1,'backend':'native Windows SDK WARP root signatures','source_revision':'current','tracked_tree_clean':True,'artifacts':{SOURCE:'source'},'observation':observation}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM root signatures';return [native,host]
    def test_requires_complete_unsigned_bytes_and_each_com_assertion(self):
        validate_observation(json.dumps(self.reports()[0]['observation']))
        for key,value in [('scenario','other'),('layouts',[]),('versioned_identical',False),('metadata',1),('parent_lifetime',False)]:
            observation=self.reports()[0]['observation'];observation[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
        for value in [[],[0]*67,[-1]*68,[256]*68,[True]*68]:
            observation=self.reports()[0]['observation'];observation['layouts'][0]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
    def test_requires_independent_clean_same_revision_and_identical_source(self):
        compare_reports(self.reports(),'current')
        for key,value in [('schema_version',0),('backend','native Windows SDK WARP root signatures'),('source_revision','old'),('tracked_tree_clean',False)]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        reports=self.reports();reports[1]['artifacts'][SOURCE]='changed'
        with self.assertRaises(ValueError):compare_reports(reports,'current')
        with self.assertRaises(ValueError):compare_reports(self.reports()[:1],'current')
    def test_checksum_header_body_sampler_and_every_other_byte_remain_comparison_fields(self):
        for row in range(6):
            for byte in range(len(self.reports()[0]['observation']['layouts'][row])):
                reports=self.reports();reports[1]['observation']['layouts'][row][byte]^=1
                with self.assertRaises(ValueError):compare_reports(reports,'current')

    def test_joint_backend_drift_cannot_replace_frozen_native_bytes(self):
        reports=self.reports()
        for report in reports:report['observation']['layouts'][0][0]^=1
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_frozen_native_source_revision_and_complete_observation_are_immutable(self):
        report=json.loads((ROOT/'cpp/reference/root-signatures-windows-v1.json').read_text())
        self.assertEqual(report['backend'],'native Windows SDK WARP root signatures')
        self.assertIs(report['tracked_tree_clean'],True)
        self.assertEqual(report['source_revision'],'5473a5894b198aaa06d9ed4ce19b8077213a3228')
        self.assertEqual(report['artifacts'][SOURCE],hashlib.sha256((ROOT/SOURCE).read_bytes()).hexdigest())
        canonical=json.dumps(report['observation'],sort_keys=True,separators=(',',':')).encode()
        self.assertEqual(hashlib.sha256(canonical).hexdigest(),'f0c59ed82431c950da3780d21b233c267d5b204cfc641ed5a67d7cc46fcdc3fe')
