"""Scalar/validator and synthetic subprocess faults, not GPU evidence."""
import copy
import json
import subprocess
import sys
import tempfile
import unittest
from tools.shader_compute_contract import expected_observation, validate_observation, compare_reports
from tools.run_shader_compute import execute_probe


class ComputeContractTests(unittest.TestCase):
    def test_fixed_words_cover_layout_wraparound_barrier_reversal_and_tail(self):
        observed = expected_observation()
        self.assertEqual(len(observed['output_words']), 96)
        self.assertEqual(observed['output_words'][:4], [0x1234566f, 0x1234567f, 0x41b00000, 0xfffffff7])
        self.assertEqual(observed['output_words'][64:68], [0x12345678, 0x1234564f, 0x42ac0000, 0])
        self.assertEqual(observed['output_words'][76:], [0xdeadbeef] * 20)
        self.assertEqual(validate_observation(json.dumps(observed)), observed)

    def test_each_observed_word_and_sentinel_is_required(self):
        for i in range(96):
            observed = expected_observation()
            observed['output_words'][i] ^= 1
            with self.subTest(word=i), self.assertRaises(ValueError):
                validate_observation(json.dumps(observed))

    def test_bad_schema_types_duplicates_and_truncation_cannot_pass(self):
        baseline = expected_observation()
        variants = [None, [], {**baseline, 'extra': 0}, {**baseline, 'count': 19.0},
                    {**baseline, 'scenario': 'other'}, {**baseline, 'output_words': baseline['output_words'][:-1]}]
        for key in baseline:
            missing = baseline.copy(); del missing[key]; variants.append(missing)
        for replacement in [True, float(baseline['output_words'][0]), -1, 2**32]:
            bad = copy.deepcopy(baseline); bad['output_words'][0] = replacement; variants.append(bad)
        for observed in variants:
            with self.subTest(observed=type(observed)), self.assertRaises(ValueError):
                validate_observation(json.dumps(observed))
        valid = json.dumps(baseline)
        for text in ['oops', valid + valid, valid[:-1] + ',"count":19}']:
            with self.assertRaises(ValueError): validate_observation(text)

    def reports(self):
        return [{'schema_version': 1, 'backend': backend, 'source_revision': 'revision',
                 'tracked_tree_clean': True, 'shader_dxil_sha256': 'a' * 64, 'observation': expected_observation()}
                for backend in ['translated Metal', 'native D3D12 WARP']]

    def test_comparison_requires_same_shader_clean_revision_and_exact_words(self):
        self.assertIsNone(compare_reports(self.reports(), 'revision'))
        for key, value in [('source_revision', 'old'), ('tracked_tree_clean', False),
                           ('tracked_tree_clean', 1), ('shader_dxil_sha256', 'b' * 64),
                           ('shader_dxil_sha256', 'not a hash'), ('backend', 'translated Metal'),
                           ('schema_version', 2), ('schema_version', True)]:
            reports = self.reports(); reports[1][key] = value
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                compare_reports(reports, 'revision')
        reports = self.reports(); reports[1]['observation']['output_words'][-1] ^= 1
        with self.assertRaises(ValueError): compare_reports(reports, 'revision')
        for reports in [[], self.reports()[:1], self.reports() * 2]:
            with self.assertRaises(ValueError): compare_reports(reports, 'revision')


class ComputeProcessTests(unittest.TestCase):
    def command(self, adapter, extra=''):
        return [sys.executable, '-c', 'import sys; print(' + repr(json.dumps(expected_observation()))
                + '); sys.stderr.write(' + repr(adapter) + '); ' + extra]

    def test_exact_words_and_single_executed_backend_identity_are_required(self):
        for backend, adapter in [('translated Metal', 'adapter: translated Metal backend: Metal\n'),
                                 ('native D3D12 WARP', 'adapter: native D3D12 WARP synthetic fixture\n')]:
            with tempfile.TemporaryDirectory() as directory:
                observed, identity, _ = execute_probe(self.command(adapter), directory, backend)
            self.assertEqual(observed, expected_observation())
            self.assertEqual(identity, adapter.strip())
        for adapter, backend in [('', 'translated Metal'), ('adapter: translated Metal backend: Vulkan\n', 'translated Metal'),
                                 ('adapter: native D3D12 WARP x\n', 'translated Metal'),
                                 ('adapter: translated Metal backend: Metal\n' * 2, 'translated Metal'),
                                 ('adapter: translated Metal backend: Metal\n', 'unknown')]:
            with tempfile.TemporaryDirectory() as directory, self.assertRaises((ValueError, RuntimeError)):
                execute_probe(self.command(adapter), directory, backend)

    def test_nonzero_exit_and_timeout_cannot_pass_with_valid_output(self):
        adapter = 'adapter: translated Metal backend: Metal\n'
        for extra, timeout, error in [('raise SystemExit(1)', 30, subprocess.CalledProcessError),
                                      ('sys.stdout.flush(); import time; time.sleep(30)', .1, subprocess.TimeoutExpired)]:
            with tempfile.TemporaryDirectory() as directory, self.assertRaises(error):
                execute_probe(self.command(adapter, extra), directory, 'translated Metal', timeout)
