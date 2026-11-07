"""Synthetic factory-report checks, separate from actual Metal discovery."""
import json
from pathlib import Path
import unittest
from unittest.mock import patch
from tools.run_cpp_metal_factory import run,validate_observation

class FactoryObservationTests(unittest.TestCase):
    def valid(self):return {'scenario':'metal-dxgi-factory-v1','adapters':[{'name':'test GPU','registry_id':2**63,'virtual_video_budget':123}],'parent_lifetime':True,'unsupported_boundaries':True,'descriptor_bytes':568}
    def test_preserves_opaque_identity_and_unknown_zero_budget(self):
        value=self.valid();value['adapters'][0]['virtual_video_budget']=0;self.assertEqual(validate_observation(json.dumps(value)),value)
    def test_rejects_missing_adapters_failed_checks_and_wrong_schema(self):
        for key,value in [('scenario','other'),('adapters',[]),('parent_lifetime',False),('unsupported_boundaries',False),('descriptor_bytes',312)]:
            report=self.valid();report[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(report))
    def test_rejects_bad_descriptor_values_without_inventing_device_fields(self):
        for key,value in [('name',''),('registry_id',True),('registry_id',-1),('virtual_video_budget',2**64),('extra',0)]:
            report=self.valid();report['adapters'][0][key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(report))
    def test_refuses_non_metal_hosts_before_building(self):
        with patch('tools.run_cpp_metal_factory.platform.system',return_value='Linux'),patch('tools.run_cpp_metal_factory.subprocess.run') as command:
            with self.assertRaises(RuntimeError):run(Path('unused'))
            command.assert_not_called()
