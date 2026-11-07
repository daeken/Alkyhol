"""Synthetic observation gates; native/Metal executions supply actual GPU data."""
import copy
import hashlib
import json
import unittest
from tools.run_cpp_byte_copies import ROOT,RESIDUE_BYTES,SOURCE,compare_reports,validate_observation
class ByteCopyReportTests(unittest.TestCase):
    def reports(self):
        observation=json.loads((ROOT/'cpp/reference/byte-copies-windows-v1.json').read_text())['observation']
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
                reports[1]['observation'][name]=value[:byte*2]+format(int(value[byte*2:byte*2+2],16)^1,'02x')+value[byte*2+2:]
                with self.assertRaises(ValueError):compare_reports(reports,'current')
        for window in range(3):
            for byte in range([32,64,32][window]):
                reports=self.reports();value=reports[1]['observation']['large_windows'][window]
                reports[1]['observation']['large_windows'][window]=value[:byte*2]+format(int(value[byte*2:byte*2+2],16)^1,'02x')+value[byte*2+2:]
                with self.assertRaises(ValueError):compare_reports(reports,'current')
        reports=self.reports();reports[1]['observation']['large_checksum']+=1
        with self.assertRaises(ValueError):compare_reports(reports,'current')

    def test_live_backends_cannot_jointly_replace_frozen_native_results(self):
        reports=self.reports()
        for report in reports:report['observation']['large_checksum']=0
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_frozen_native_execution_source_and_observation_hash_are_preserved(self):
        report=json.loads((ROOT/'cpp/reference/byte-copies-windows-v1.json').read_text())
        self.assertEqual(report['backend'],'native Windows SDK WARP byte copies')
        self.assertIs(report['tracked_tree_clean'],True)
        self.assertEqual(report['source_revision'],'57e6321e63f67da84ebb26ae0b71df32033f0d9e')
        self.assertEqual(report['artifacts'][SOURCE],hashlib.sha256((ROOT/SOURCE).read_bytes()).hexdigest())
        self.assertEqual(hashlib.sha256(json.dumps(report['observation'],sort_keys=True,separators=(',',':')).encode()).hexdigest(),'c1016c4a93300ff0b85f43975a2e447fc9b8466b8455487d373ce4413f65868a')
