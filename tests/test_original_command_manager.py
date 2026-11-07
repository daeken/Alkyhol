"""Synthetic provenance/observation gates; physical execution is separate."""
import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from tools.run_original_command_manager import ORIGINALS,PIN,current_objects,run,validate_observation

class OriginalQueueTests(unittest.TestCase):
    def test_requires_exact_original_object_and_source_bytes_and_current_headers(self):
        with tempfile.TemporaryDirectory() as scratch:
            upstream=Path(scratch)/'upstream';build=Path(scratch)/'build';build.mkdir()
            report={'complete':True,'upstream_revision':PIN,'headers_sha256':{'header':'digest'},'units':[]}
            for index,source in enumerate(ORIGINALS):
                path=upstream/source;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(b'synthetic source')
                obj=build/f'{index}.o';obj.write_bytes(b'synthetic object')
                report['units'].append({'source':source,'object':obj.name,'exit_code':0,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'object_sha256':hashlib.sha256(obj.read_bytes()).hexdigest()})
            self.assertEqual(current_objects(upstream,build,report,{'header':'digest'}),[build/'0.o',build/'1.o'])
            for key,value in [('complete',False),('upstream_revision','different'),('headers_sha256',{})]:
                invalid=copy.deepcopy(report);invalid[key]=value
                with self.assertRaises(ValueError):current_objects(upstream,build,invalid,{'header':'digest'})
            for rows in [[],report['units']+[report['units'][0]]]:
                invalid=copy.deepcopy(report);invalid['units']=rows
                with self.assertRaises(ValueError):current_objects(upstream,build,invalid,{'header':'digest'})
            for path in [upstream/ORIGINALS[0],build/'0.o']:
                saved=path.read_bytes();path.write_bytes(b'changed')
                with self.assertRaises(ValueError):current_objects(upstream,build,report,{'header':'digest'})
                path.write_bytes(saved)
    def test_original_queue_sequence_requires_high_bit_values_and_all_shutdown_checks(self):
        expected={'scenario':'original-command-manager-v1','next_values':[9,(2<<56)|9,(3<<56)|9],
                  'queues':3,'increments_per_queue':8,'idle':True,'shutdown':True}
        self.assertEqual(validate_observation(json.dumps(expected)),expected)
        for key,value in [('scenario','other'),('next_values',[9,9,9]),('queues',2),('increments_per_queue',7),('idle',1),('shutdown',False)]:
            invalid=copy.deepcopy(expected);invalid[key]=value
            with self.assertRaises(ValueError):validate_observation(json.dumps(invalid))
        with self.assertRaises(ValueError):validate_observation(json.dumps({**expected,'extra':True}))
    def test_unsupported_host_fails_before_dependency_or_build_actions(self):
        with patch('tools.run_original_command_manager.platform.system',return_value='Linux'),patch('tools.run_original_command_manager.ensure') as intake:
            with self.assertRaises(RuntimeError):run(Path('unused'),Path('unused'))
            intake.assert_not_called()
