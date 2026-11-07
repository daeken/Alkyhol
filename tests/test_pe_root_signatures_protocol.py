"""Synthetic serializer protocol negatives; actual native evidence is separate."""
import copy
import subprocess
import sys
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'tools'))
from build_pe_root_signatures import SOURCES
from run_pe_root_signatures import NATIVE, HOST, decode, expected_bytes
from compare_pe_root_signatures import compare


class RootSignaturePeProtocolTests(unittest.TestCase):
    def execution(self, windows=False):
        return subprocess.CompletedProcess([], 0, expected_bytes(windows), b'' if windows else
            b'PE64: 4 sections, 5 named imports, 1 DIR64 fixups\nentering Win64 code; relocated=true\n')

    def reports(self):
        native = {'schema_version': 1, 'backend': NATIVE, 'tracked_tree_clean': True,
                  'revision': 'a'*40, 'pe_sha256': 'b'*64, 'executed_pe_sha256': 'b'*64,
                  'sources': {s: 'c'*64 for s in SOURCES},
                  'observation': decode(self.execution(True), windows=True),
                  'forced_rebase_observation': None}
        host = copy.deepcopy(native)
        host.update(backend=HOST, observation=decode(self.execution()))
        host['forced_rebase_observation'] = copy.deepcopy(host['observation'])
        return [native, host]

    def test_exact_source_sdk_layout_bytes_and_binary_observations_match(self):
        self.assertEqual(len(expected_bytes()), 1216)
        for windows in [True, False]:
            self.assertEqual(decode(self.execution(windows), windows, not windows),
                             self.reports()[0 if windows else 1]['observation'])
        reports = self.reports()
        self.assertEqual(compare(reports, reference=reports[0]), 1180)
        self.assertEqual(expected_bytes()[:-36], expected_bytes(True)[:-36])
        self.assertNotEqual(expected_bytes()[-36:], expected_bytes(True)[-36:])

    def test_crashes_truncation_extra_bytes_and_each_observed_byte_fail(self):
        run = self.execution()
        run.returncode = -11
        with self.assertRaises(ValueError):
            decode(run)
        for stdout in [b'', run.stdout[:-1], run.stdout+b'\0']:
            run = self.execution()
            run.stdout = stdout
            with self.assertRaises(ValueError):
                decode(run)
        for windows in [True, False]:
            for index in range(len(expected_bytes())):
                run = self.execution(windows)
                changed = bytearray(run.stdout)
                changed[index] ^= 1
                run.stdout = bytes(changed)
                with self.assertRaises(ValueError):
                    decode(run, windows)

    def test_unexpected_diagnostic_or_missing_actual_rebase_fails(self):
        for stderr in [b'', b'noise', self.execution().stderr.replace(b'true', b'false')]:
            run = self.execution()
            run.stderr = stderr
            with self.assertRaises(ValueError):
                decode(run, rebase=True)
        run = self.execution(True)
        run.stderr = b'pretend success'
        with self.assertRaises(ValueError):
            decode(run, windows=True)

    def test_known_native_acceptance_and_host_bound_must_remain_visible(self):
        for target, source in [(0, 1), (1, 0)]:
            reports = self.reports()
            reports[target]['observation'] = copy.deepcopy(reports[source]['observation'])
            if target == 1:
                reports[1]['forced_rebase_observation'] = copy.deepcopy(reports[1]['observation'])
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])

    def test_identity_raw_results_backends_and_joint_frozen_drift_fail(self):
        for field, value in [('tracked_tree_clean', False), ('schema_version', 2),
            ('backend', NATIVE), ('revision', 'd'*40), ('pe_sha256', 'e'*64),
            ('executed_pe_sha256', 'e'*64), ('sources', {}),
            ('forced_rebase_observation', None), ('observation', {'status': 0, 'stdout_hex': '00'})]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])
        reports = self.reports()
        for r in reports:
            r.update(pe_sha256='e'*64, executed_pe_sha256='e'*64)
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])


if __name__ == '__main__':
    unittest.main()
