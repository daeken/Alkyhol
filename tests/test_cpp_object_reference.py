"""Synthetic transport checks; only the native observer supplies API behavior."""
import json
from pathlib import Path
from unittest.mock import patch
import unittest
from tools.run_cpp_object_reference import CASES,EVENTS,parse_observation,run

class ObjectObservationTests(unittest.TestCase):
    def fixture(self):
        return {'scenario':'d3d12-object-private-data-v1','records':[{'case':name,'hr':None if name in EVENTS else 0x887a0002,'size':0,'data':'a500ff','refs':1,'adds':0,'releases':0,'destroyed':0} for name in CASES]}
    def test_preserves_failure_codes_and_exact_bytes_without_manufacturing_success(self):
        report=self.fixture();self.assertEqual(parse_observation(json.dumps(report)),report)
    def test_rejects_missing_duplicate_reordered_and_unknown_cases(self):
        for mutation in [lambda r:r.pop(),lambda r:r.append(r[0]),lambda r:r.reverse(),lambda r:r[0].update(case='unknown')]:
            report=self.fixture();mutation(report['records'])
            with self.assertRaises(ValueError):parse_observation(json.dumps(report))
    def test_rejects_malformed_fields_and_bytes(self):
        for key,value in [('hr',-1),('size',True),('refs',2**32),('data','a'),('data','FF'),('data','00'*65),('extra',0)]:
            report=self.fixture();report['records'][0][key]=value
            with self.assertRaises(ValueError):parse_observation(json.dumps(report))
        report=self.fixture();report['records'][-1]['hr']=0
        with self.assertRaises(ValueError):parse_observation(json.dumps(report))
        report=self.fixture();report['scenario']='different'
        with self.assertRaises(ValueError):parse_observation(json.dumps(report))
    def test_refuses_non_native_execution_before_building(self):
        with patch('tools.run_cpp_object_reference.os.name','posix'),patch('tools.run_cpp_object_reference.subprocess.run') as command:
            with self.assertRaises(RuntimeError):run(Path('unused'))
            command.assert_not_called()
