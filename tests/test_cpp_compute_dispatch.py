"""Synthetic dispatch report gates; actual GPU readbacks come from C++ CI."""
import copy
import hashlib
import json
import unittest
from tools.run_cpp_compute_dispatch import ROOT,INPUTS,DXIL_SHA256,compare_reports,validate_observation
class ComputeDispatchReportTests(unittest.TestCase):
    def reports(self):
        # Frozen measured payload in synthetic envelopes; no GPU runs here.
        observation=json.loads((ROOT/'cpp/reference/compute-dispatch-windows-v1.json').read_text())['observation']
        native={'schema_version':1,'backend':'native Windows SDK WARP compute dispatch','source_revision':'current','tracked_tree_clean':True,'shader_dxil_sha256':DXIL_SHA256,'artifacts':{name:'bytes' for name in INPUTS},'observation':observation}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM compute dispatch';return [native,host]
    def test_complete_unsigned_readbacks_and_all_state_assertions_are_required(self):
        validate_observation(json.dumps(self.reports()[0]['observation']))
        for key,value in [('scenario','other'),('cases',[]),('queues',True),('queues',1),('root_offsets',[0]*4),('reset',False),('zero_dispatch',1),('binding_snapshots',False),('same_root_preserved',False)]:
            observation=self.reports()[0]['observation'];observation[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
        for value in [[0]*103,[0]*105,[True]*104,[-1]*104,[2**32]*104]:
            observation=self.reports()[0]['observation'];observation['cases'][0]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
    def test_independent_clean_same_revision_shader_and_all_fixture_inputs_required(self):
        compare_reports(self.reports(),'current')
        for key,value in [('schema_version',0),('backend','native Windows SDK WARP compute dispatch'),('source_revision','old'),('tracked_tree_clean',False),('shader_dxil_sha256','0'*64)]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        for name in INPUTS:
            reports=self.reports();reports[1]['artifacts'][name]='changed'
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        with self.assertRaises(ValueError):compare_reports(self.reports()[:1],'current')
    def test_every_dispatch_data_prefix_tail_and_suffix_word_is_compared_exactly(self):
        for row in range(6):
            for word in range(104):
                reports=self.reports();reports[1]['observation']['cases'][row][word]^=1
                with self.assertRaises(ValueError):compare_reports(reports,'current')

    def test_joint_backend_drift_cannot_replace_the_native_readbacks(self):
        reports=self.reports()
        for report in reports:report['observation']['cases'][0][4]^=1
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_frozen_native_source_fixture_and_complete_observation_identity(self):
        report=json.loads((ROOT/'cpp/reference/compute-dispatch-windows-v1.json').read_text())
        self.assertEqual(report['backend'],'native Windows SDK WARP compute dispatch')
        self.assertIs(report['tracked_tree_clean'],True)
        self.assertEqual(report['source_revision'],'8e4c6b258441220f0b65980e89729e8615f03f15')
        self.assertEqual(report['shader_dxil_sha256'],DXIL_SHA256)
        for name in INPUTS:self.assertEqual(report['artifacts'][name],hashlib.sha256((ROOT/name).read_bytes()).hexdigest())
        canonical=json.dumps(report['observation'],sort_keys=True,separators=(',',':')).encode()
        self.assertEqual(hashlib.sha256(canonical).hexdigest(),'263fe289e48e36cfe4745892f83e6b51f513fcf74a3fa79528ccb7a89ecbfaa9')
