"""Synthetic observation gates; native/Metal executions supply actual GPU data."""
import copy
import json
import unittest
from tools.run_cpp_byte_copies import RESIDUE_BYTES,SOURCE,compare_reports,validate_observation
class ByteCopyReportTests(unittest.TestCase):
    def reports(self):
        observation={'scenario':'byte-copies-v1','cases':272,'residue_buffer':'a5'*RESIDUE_BYTES,
                     'overlay_buffer':'a5'*4099,'large_bytes_checked':16*1024*1024+97,
                     'large_checksum':123,'large_windows':['a5'*32,'a5'*64,'a5'*32],'deferred':True}
        native={'schema_version':1,'backend':'native Windows SDK WARP byte copies','source_revision':'current',
                'tracked_tree_clean':True,'artifacts':{SOURCE:'source'},'observation':observation}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM byte copies';return [native,host]
    def test_preserves_observed_bytes_without_inventing_native_results(self):
        reports=self.reports();compare_reports(reports,'current');observation=reports[0]['observation']
        observation['large_checksum']=9
        self.assertEqual(validate_observation(json.dumps(observation)),observation)
    def test_rejects_wrong_shape_truncation_types_or_missing_work(self):
        for name,value in [('scenario','other'),('cases',271),('large_bytes_checked',0),('large_checksum',True),('residue_buffer','00'),('overlay_buffer','zz'*4099),('large_windows',[]),('deferred',False)]:
            observation=self.reports()[0]['observation'];observation[name]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
    def test_requires_independent_clean_same_revision_and_source(self):
        for name,value in [('schema_version',0),('source_revision','old'),('tracked_tree_clean',False),('backend','native Windows SDK WARP byte copies')]:
            reports=self.reports();reports[1][name]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        reports=self.reports();reports[1]['artifacts'][SOURCE]='changed'
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_byte_changes_in_each_buffer_and_boundary_and_checksum_mismatch_fail(self):
        for name in ['residue_buffer','overlay_buffer']:
            for fraction in [0,1,2,3,7]:
                reports=self.reports();value=reports[1]['observation'][name];byte=(len(value)//2-1)*fraction//7
                reports[1]['observation'][name]=value[:byte*2]+'00'+value[byte*2+2:]
                with self.assertRaises(ValueError):compare_reports(reports,'current')
        for window in range(3):
            for byte in range([32,64,32][window]):
                reports=self.reports();value=reports[1]['observation']['large_windows'][window]
                reports[1]['observation']['large_windows'][window]=value[:byte*2]+'00'+value[byte*2+2:]
                with self.assertRaises(ValueError):compare_reports(reports,'current')
        reports=self.reports();reports[1]['observation']['large_checksum']+=1
        with self.assertRaises(ValueError):compare_reports(reports,'current')
