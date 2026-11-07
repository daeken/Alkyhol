"""Synthetic report validation, separate from actual native/Metal GPU execution."""
import copy
import json
import unittest
from tools.run_cpp_command_copy import SOURCE,compare_reports,validate_observation

class CommandCopyReportTests(unittest.TestCase):
    def reports(self):
        observation={'scenario':'command-copy-v1','results':[0x80004005]*2+[0]*9,
                     'buffers':['a5'*1024,'07'*1024],'deferred':True,'parent_lifetime':True}
        native={'schema_version':1,'backend':'native Windows SDK WARP command copies',
                'source_revision':'current','tracked_tree_clean':True,
                'artifacts':{SOURCE:'source'},'observation':observation}
        metal=copy.deepcopy(native);metal['backend']='Rust restricted Metal COM command copies'
        return [native,metal]
    def test_preserves_returned_bytes_and_results_without_manufacturing_native_output(self):
        reports=self.reports();compare_reports(reports,'current')
        report=reports[0]['observation'];report['results'][0]=1;report['buffers'][0]='ff'*1024
        self.assertEqual(validate_observation(json.dumps(report)),report)
    def test_rejects_incomplete_failed_and_invalid_observations(self):
        for key,value in [('scenario','other'),('results',[0]),('results',[True]*11),('buffers',['a5']),('buffers',['xx'*1024]*2),('deferred',False),('parent_lifetime',1)]:
            report=self.reports()[0]['observation'];report[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(report))
    def test_requires_two_independent_clean_same_revision_and_source_reports(self):
        for key,value in [('source_revision','old'),('tracked_tree_clean',False),('schema_version',0),('backend','native Windows SDK WARP command copies')]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        for source in ['changed',None]:
            reports=self.reports();reports[1]['artifacts'][SOURCE]=source
            with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_every_observed_byte_including_sentinels_and_each_result_is_required(self):
        for index in range(11):
            reports=self.reports();reports[1]['observation']['results'][index]+=1
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        for buffer in range(2):
            for byte in range(1024):
                reports=self.reports();text=reports[1]['observation']['buffers'][buffer]
                reports[1]['observation']['buffers'][buffer]=text[:byte*2]+'ff'+text[byte*2+2:]
                with self.assertRaises(ValueError):compare_reports(reports,'current')
