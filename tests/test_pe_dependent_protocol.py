"""Synthetic evidence-protocol negatives, not actual Windows execution."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from compare_pe_dependent import compare, NATIVE, HOST
from build_pe_dependent import SOURCES
from run_pe_dependent import check_execution, REFERENCE, DEPENDENCY_REFERENCE, EXPECTED_CALLS, EXPECTED_TRACE


class DependentProtocolTests(unittest.TestCase):
    def reports(self):
        old = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
        native = {'schema_version': 1, 'backend': NATIVE, 'tracked_tree_clean': True,
                  'revision': 'a' * 40, 'binaries': {'alkyhol_exports.dll': old['dll_sha256'], 'alkyhol_dependent.dll': 'b' * 64},
                  'sources': {s: old['sources'].get(s, 'c' * 64) for s in SOURCES},
                  'common': {'calls': copy.deepcopy(EXPECTED_CALLS), 'trace_hex': EXPECTED_TRACE.hex()},
                  'relocated': None, 'rebased_common': None, 'rebased_relocated': None}
        host = copy.deepcopy(native)
        host.update(backend=HOST, relocated=[False, True], rebased_relocated=[True, True], rebased_common=copy.deepcopy(host['common']))
        return [native, host]

    def execution(self, windows=False):
        return subprocess.CompletedProcess([], 0, EXPECTED_TRACE,
            json.dumps({'calls': EXPECTED_CALLS, 'relocated': None if windows else [True, True]}).encode())

    def test_exact_calls_trace_and_relocation_observations(self):
        for windows in [False, True]:
            common, flags = check_execution(self.execution(windows), windows, rebase=not windows)
            self.assertEqual(common, self.reports()[0]['common'])
            self.assertEqual(flags, None if windows else [True, True])
        reports = self.reports()
        self.assertEqual(compare(reports, reference=reports[0]), 48)

    def test_crashed_reordered_truncated_duplicate_and_changed_trace_fail(self):
        for trace in [b'', EXPECTED_TRACE[:-1], EXPECTED_TRACE + EXPECTED_TRACE[:24],
                      EXPECTED_TRACE[24:] + EXPECTED_TRACE[:24], b'X' + EXPECTED_TRACE[1:]]:
            run = self.execution()
            run.stdout = trace
            with self.assertRaises(ValueError):
                check_execution(run, False)
        run = self.execution()
        run.returncode = -11
        with self.assertRaises(ValueError):
            check_execution(run, False)

    def test_wrong_calls_addresses_and_missing_rebase_cannot_pass(self):
        for key in EXPECTED_CALLS:
            run = self.execution()
            value = json.loads(run.stderr)
            value['calls'][key] = 0
            run.stderr = json.dumps(value).encode()
            with self.assertRaises(ValueError):
                check_execution(run, False)
        for flags in [None, [True], [False, True], [1, True]]:
            run = self.execution()
            value = json.loads(run.stderr)
            value['relocated'] = flags
            run.stderr = json.dumps(value).encode()
            with self.assertRaises(ValueError):
                check_execution(run, False, rebase=True)

    def test_identity_backend_trace_and_missing_rebase_reports_fail(self):
        for field, value in [('tracked_tree_clean', False), ('revision', 'd' * 40), ('sources', {}),
                             ('binaries', {}), ('common', {}), ('backend', NATIVE),
                             ('rebased_common', None), ('rebased_relocated', [False, True])]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])

    def test_frozen_dependency_and_joint_consumer_drift_cannot_pass(self):
        for name in ['alkyhol_exports.dll', 'alkyhol_dependent.dll']:
            reports = self.reports()
            for r in reports:
                r['binaries'][name] = 'e' * 64
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])
        for name in SOURCES:
            reports = self.reports()
            for r in reports:
                r['sources'][name] = 'e' * 64
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])

    def test_checked_in_native_reference_is_required_by_default(self):
        native = json.loads(REFERENCE.read_text(encoding='utf-8'))
        host = copy.deepcopy(native)
        host.update(backend=HOST, relocated=[False, True], rebased_relocated=[True, True], rebased_common=copy.deepcopy(host['common']))
        self.assertEqual(compare([native, host]), 48)
        for r in [native, host]:
            r['binaries']['alkyhol_dependent.dll'] = 'e' * 64
        with self.assertRaises(ValueError):
            compare([native, host])


if __name__ == '__main__':
    unittest.main()
