"""Synthetic report validation, separate from actual native/Metal GPU execution."""
import copy
import hashlib
import json
import unittest
from tools.run_cpp_command_copy import ROOT,SOURCE,compare_reports,validate_observation

class CommandCopyReportTests(unittest.TestCase):
    def reports(self):
        observation=json.loads((ROOT/'cpp/reference/command-copy-windows-v1.json').read_text())['observation']
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
                reports[1]['observation']['buffers'][buffer]=text[:byte*2]+format(int(text[byte*2:byte*2+2],16)^1,'02x')+text[byte*2+2:]
                with self.assertRaises(ValueError):compare_reports(reports,'current')

    def test_joint_backend_drift_cannot_replace_the_native_snapshot(self):
        reports=self.reports()
        for report in reports:report['observation']['results'][0]=0
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_frozen_native_source_and_observation_identity_are_preserved(self):
        report=json.loads((ROOT/'cpp/reference/command-copy-windows-v1.json').read_text())
        self.assertEqual(report['backend'],'native Windows SDK WARP command copies')
        self.assertIs(report['tracked_tree_clean'],True)
        self.assertEqual(report['source_revision'],'ffc1ec510d345ff8b067966f35b9c4970480fe5d')
        self.assertEqual(report['artifacts'][SOURCE],hashlib.sha256((ROOT/SOURCE).read_bytes()).hexdigest())
        self.assertEqual(hashlib.sha256(json.dumps(report['observation'],sort_keys=True,separators=(',',':')).encode()).hexdigest(),'419bf280bdeb59498bc05837f3a367ebe90ead351a8a799aa0962be6f067edcc')
