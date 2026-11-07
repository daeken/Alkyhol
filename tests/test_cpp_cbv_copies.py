"""Synthetic report validation only; actual GPU execution is a separate CI step."""
import copy
import hashlib
import json
import unittest
from unittest.mock import patch
from tools.run_cpp_cbv_copies import ROOT,INPUTS,compare_reports,validate_observation
from tools import build_cpp_cbv_probe

class CbvReportTests(unittest.TestCase):
    def reports(self):
        # Measured payload in synthetic report envelopes; this does not run a GPU.
        observation=json.loads((ROOT/'cpp/reference/cbv-copies-windows-v1.json').read_text())['observation']
        native={'schema_version':1,'backend':'native Windows SDK WARP CBV copies','source_revision':'current','tracked_tree_clean':True,'artifacts':{name:'bytes' for name in INPUTS},'observation':observation}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM CBV copies'
        return [native,host]
    def test_requires_complete_unsigned_words_and_explicit_execution_assertions(self):
        validate_observation(json.dumps(self.reports()[0]['observation']))
        for key,value in [('scenario','other'),('cases',[]),('snapshot_independent',False),('gpu_consumed',1)]:
            observation=self.reports()[0]['observation'];observation[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
        for value in [[],[0]*23,[0]*25,[True]*24,[-1]*24,[2**32]*24]:
            observation=self.reports()[0]['observation'];observation['cases'][0]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
    def test_independent_clean_same_revision_and_fixture_closure_required(self):
        compare_reports(self.reports(),'current')
        for key,value in [('schema_version',2),('source_revision','old'),('tracked_tree_clean',False),('backend','native Windows SDK WARP CBV copies')]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        for name in INPUTS:
            for missing in [False,True]:
                reports=self.reports()
                if missing:del reports[1]['artifacts'][name]
                else:reports[1]['artifacts'][name]='changed'
                with self.assertRaises(ValueError):compare_reports(reports,'current')
        with self.assertRaises(ValueError):compare_reports(self.reports()[:1],'current')
    def test_every_data_and_guard_word_is_an_exact_comparison_field(self):
        for row in range(3):
            for word in range(24):
                reports=self.reports();reports[1]['observation']['cases'][row][word]^=1
                with self.assertRaises(ValueError):compare_reports(reports,'current')

    def test_joint_backend_drift_cannot_replace_frozen_native_contents(self):
        reports=self.reports()
        for report in reports:report['observation']['cases'][0][0]^=1
        with self.assertRaises(ValueError):compare_reports(reports,'current')
    def test_frozen_native_execution_source_shader_and_complete_output_identity(self):
        report=json.loads((ROOT/'cpp/reference/cbv-copies-windows-v1.json').read_text())
        self.assertEqual(report['backend'],'native Windows SDK WARP CBV copies')
        self.assertIs(report['tracked_tree_clean'],True)
        self.assertEqual(report['source_revision'],'b10a7dfa5043dcf74248a9613b8c59342c2bb8a6')
        for name in INPUTS:self.assertEqual(report['artifacts'][name],hashlib.sha256((ROOT/name).read_bytes()).hexdigest())
        canonical=json.dumps(report['observation'],sort_keys=True,separators=(',',':')).encode()
        self.assertEqual(hashlib.sha256(canonical).hexdigest(),'309c6fedf53fb0e3bf1915edda937b7b35d32bbb5ff87c400ebdf6cc169aa769')

class CbvShaderFixtureTests(unittest.TestCase):
    def test_regeneration_uses_open_dxc_and_checks_exact_generated_bytes(self):
        import tempfile
        from pathlib import Path
        with tempfile.TemporaryDirectory() as scratch:
            root=Path(scratch);(root/'cpp/tests').mkdir(parents=True)
            source=root/'cpp/tests/cbv_probe.hlsl';source.write_text('test')
            target=root/'cpp/tests/cbv_probe_dxil.h'
            def compile_probe(command,**kwargs):
                self.assertEqual(command[0],str(Path('/open/dxc')))
                self.assertEqual(command[1:7],['-T','cs_6_0','-E','main','-Ges','-WX'])
                self.assertEqual(command[-1],str(source))
                self.assertTrue(kwargs['check'])
                Path(command[command.index('-Fo')+1]).write_bytes(b'DXBC\x00\xff')
            with patch.object(build_cpp_cbv_probe,'ROOT',root),patch.object(build_cpp_cbv_probe.subprocess,'run',side_effect=compile_probe):
                build_cpp_cbv_probe.build(Path('/open/dxc'))
                self.assertIn('0x44,0x58,0x42,0x43,0x00,0xff,',target.read_text())
                build_cpp_cbv_probe.build(Path('/open/dxc'),check=True)
                target.write_text(target.read_text()+'\n')
                with self.assertRaises(ValueError):build_cpp_cbv_probe.build(Path('/open/dxc'),check=True)
