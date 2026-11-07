"""Synthetic report protocol tests; GPU execution is a separate runtime unit."""
import copy
import hashlib
import json
import re
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from tools.run_cpp_compute_pipelines import ROOT,INPUTS,DXIL_SHA256,compare_reports,validate_observation
from tools import build_cpp_compute_probe
class ComputePipelineReportTests(unittest.TestCase):
    def reports(self):
        observation={'scenario':'compute-pso-v1','created':2,'capability_checked':True,'canonical_identity':True,'pageable':True,'metadata':True,'parent_lifetime':True}
        native={'schema_version':1,'backend':'native Windows SDK WARP compute pipelines','source_revision':'current','tracked_tree_clean':True,'shader_dxil_sha256':DXIL_SHA256,'artifacts':{name:'bytes' for name in INPUTS},'observation':observation}
        host=copy.deepcopy(native);host['backend']='Rust restricted Metal COM compute pipelines';return [native,host]
    def test_requires_every_observed_common_construction_and_com_assertion(self):
        validate_observation(json.dumps(self.reports()[0]['observation']))
        for key,value in [('scenario','other'),('created',True),('created',1),('capability_checked',False),('canonical_identity',1),('pageable',False),('metadata',False),('parent_lifetime',False)]:
            observation=self.reports()[0]['observation'];observation[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(observation))
    def test_independent_clean_same_revision_and_exact_fixture_identity_required(self):
        compare_reports(self.reports(),'current')
        for key,value in [('schema_version',0),('backend','native Windows SDK WARP compute pipelines'),('source_revision','old'),('tracked_tree_clean',False),('shader_dxil_sha256','0'*64)]:
            reports=self.reports();reports[1][key]=value
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        for name in INPUTS:
            reports=self.reports();reports[1]['artifacts'][name]='changed'
            with self.assertRaises(ValueError):compare_reports(reports,'current')
        with self.assertRaises(ValueError):compare_reports(self.reports()[:1],'current')
    def test_committed_header_matches_binary_and_established_native_compute_container(self):
        binary=(ROOT/'cpp/tests/compute_probe.dxil').read_bytes()
        self.assertEqual(hashlib.sha256(binary).hexdigest(),DXIL_SHA256)
        header=(ROOT/'cpp/tests/compute_probe_dxil.h').read_text()
        self.assertEqual(bytes(int(v,16) for v in re.findall(r'0x([0-9a-f]{2})',header)),binary)
    def test_frozen_native_execution_and_complete_shared_source_fixture_identity(self):
        report=json.loads((ROOT/'cpp/reference/compute-pipelines-windows-v1.json').read_text())
        self.assertEqual(report['backend'],'native Windows SDK WARP compute pipelines')
        self.assertIs(report['tracked_tree_clean'],True)
        self.assertEqual(report['source_revision'],'ded5bbda89b924eac05ea2740a840c1c2f4403be')
        self.assertEqual(report['shader_dxil_sha256'],DXIL_SHA256)
        for name in INPUTS:self.assertEqual(report['artifacts'][name],hashlib.sha256((ROOT/name).read_bytes()).hexdigest())
        canonical=json.dumps(report['observation'],sort_keys=True,separators=(',',':')).encode()
        self.assertEqual(hashlib.sha256(canonical).hexdigest(),'b8eefd69ed8e0166548fa8765e1b38467a4a52fee408ffc9f6c943351e8c6348')
    def test_generator_calls_open_dxc_and_checks_both_outputs_exactly(self):
        with tempfile.TemporaryDirectory() as scratch:
            root=Path(scratch);(root/'cpp/tests').mkdir(parents=True);(root/'shader/tests/fixtures').mkdir(parents=True)
            source=root/'shader/tests/fixtures/compute.hlsl';source.write_text('test')
            def compile_shader(command,**kwargs):
                self.assertEqual(command[:7],[str(Path('/open/dxc')),'-T','cs_6_0','-E','CSMain','-Ges','-WX']);self.assertEqual(command[-1],str(source));self.assertTrue(kwargs['check'])
                Path(command[command.index('-Fo')+1]).write_bytes(b'DXBC\x00\xff')
            with patch.object(build_cpp_compute_probe,'ROOT',root),patch.object(build_cpp_compute_probe.subprocess,'run',side_effect=compile_shader):
                build_cpp_compute_probe.build(Path('/open/dxc'));build_cpp_compute_probe.build(Path('/open/dxc'),check=True)
                for name in ['compute_probe.dxil','compute_probe_dxil.h']:
                    path=root/'cpp/tests'/name;saved=path.read_bytes();path.write_bytes(saved+b'x')
                    with self.assertRaises(ValueError):build_cpp_compute_probe.build(Path('/open/dxc'),check=True)
                    path.write_bytes(saved)
