"""Synthetic root report protocol tests, separate from SDK/COM execution."""
import copy
import json
import unittest
from tools.run_cpp_root_signatures import SOURCE,compare_reports,validate_observation
class RootReportTests(unittest.TestCase):
    def reports(self):
        observation={'scenario':'root-signatures-v1','layouts':[[i%256 for i in range(68)] for _ in range(6)],'versioned_identical':True,'metadata':True,'parent_lifetime':True}
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
            for byte in range(68):
                reports=self.reports();reports[1]['observation']['layouts'][row][byte]^=1
                with self.assertRaises(ValueError):compare_reports(reports,'current')
