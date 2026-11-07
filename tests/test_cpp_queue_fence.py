"""Synthetic synchronization report gates, separate from actual API evidence."""
import copy
import hashlib
import json
import unittest
from tools.run_cpp_queue_fence import ROOT,SOURCE,compare_reports,validate_observation

class QueueFenceReportTests(unittest.TestCase):
    def reports(self):
        observation=json.loads((ROOT/'cpp/reference/queue-fence-windows-v1.json').read_text())['observation']
        native={'schema_version':1,'backend':'native Windows SDK WARP queue/fence',
                'source_revision':'revision','tracked_tree_clean':True,
                'artifacts':{SOURCE:'source'},'observation':observation}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM queue/fence'
        return [native,host]
    def test_accepts_complete_independent_reports_without_inventing_native_descriptor(self):
        reports=self.reports();compare_reports(reports,'revision')
        observation=reports[0]['observation'];observation['desc'][3]=0
        self.assertEqual(validate_observation(json.dumps(observation)),observation)
    def test_rejects_joint_drift_from_frozen_native_reference(self):
        reports=self.reports()
        for report in reports:report['observation']['desc'][3]=0
        with self.assertRaises(ValueError):compare_reports(reports,'revision')
    def test_frozen_native_evidence_preserves_actual_execution_and_source(self):
        report=json.loads((ROOT/'cpp/reference/queue-fence-windows-v1.json').read_text())
        self.assertEqual(report['backend'],'native Windows SDK WARP queue/fence')
        self.assertIs(report['tracked_tree_clean'],True)
        self.assertEqual(report['source_revision'],'52df42e386bdacfa25c7dd971d2950ed93a947f8')
        self.assertEqual(report['artifacts'][SOURCE],hashlib.sha256((ROOT/SOURCE).read_bytes()).hexdigest())
        self.assertEqual(report['observation']['desc'],[3,0,0,1])
        self.assertEqual(report['observation']['values'],[5,7,8,3,9,12,21,2])
        self.assertEqual(report['observation']['waits'],[0,258,258,0,0,258,0,0,0,0])
    def test_rejects_incomplete_invalid_and_failed_observations(self):
        for key,value in [('scenario','unknown'),('desc',[0]),('values',[0]*7),('waits',[False]*10),('retained_submission',False),('parent_lifetime',1)]:
            observation=self.reports()[0]['observation'];observation[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
        for key in ['desc','values','waits']:
            observation=self.reports()[0]['observation'];observation[key][0]=-1
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
    def test_requires_same_clean_revision_source_and_independent_backend(self):
        for key,value in [('source_revision','old'),('tracked_tree_clean',False),('schema_version',0),('backend','native Windows SDK WARP queue/fence')]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'revision')
        reports=self.reports();reports[1]['artifacts'][SOURCE]='changed'
        with self.assertRaises(ValueError):compare_reports(reports,'revision')
        reports=self.reports();reports[1]['artifacts'].clear()
        with self.assertRaises(ValueError):compare_reports(reports,'revision')
    def test_rejects_each_returned_word_difference(self):
        for field in ['desc','values','waits']:
            for i in range(len(self.reports()[0]['observation'][field])):
                reports=self.reports();reports[1]['observation'][field][i]+=1
                with self.assertRaises(ValueError):compare_reports(reports,'revision')
