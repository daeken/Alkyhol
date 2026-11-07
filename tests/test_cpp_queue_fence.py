"""Synthetic synchronization report gates, separate from actual API evidence."""
import copy
import json
import unittest
from tools.run_cpp_queue_fence import SOURCE,compare_reports,validate_observation

class QueueFenceReportTests(unittest.TestCase):
    def reports(self):
        observation={'scenario':'queue-fence-v1','desc':[3,0,0,0],
                     'values':[5,7,8,3,9,12,21,2],
                     'waits':[0,258,258,0,0,258,0,0,0,0],
                     'retained_submission':True,'parent_lifetime':True}
        native={'schema_version':1,'backend':'native Windows SDK WARP queue/fence',
                'source_revision':'revision','tracked_tree_clean':True,
                'artifacts':{SOURCE:'source'},'observation':observation}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM queue/fence'
        return [native,host]
    def test_accepts_complete_independent_reports_without_inventing_native_descriptor(self):
        reports=self.reports();compare_reports(reports,'revision')
        for report in reports:report['observation']['desc'][3]=1
        compare_reports(reports,'revision')
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
