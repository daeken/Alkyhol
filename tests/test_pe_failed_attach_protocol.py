"""Synthetic observer negatives; these do not constitute native DLL evidence."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from compare_pe_failed_attach import compare, NATIVE, HOST, REFERENCE
from build_pe_failed_attach import SOURCES
from run_pe_failed_attach import check_execution, EXPECTED


class FailedAttachProtocolTests(unittest.TestCase):
    def execution(self, windows=False):
        key = 'windows_last_error' if windows else 'host_last_error'
        return subprocess.CompletedProcess([], 0, EXPECTED, json.dumps({
            'loaded': False, 'attach_rejected': True, key: 1114 if windows else 0x1234}).encode())

    def reports(self):
        native = {'schema_version': 1, 'backend': NATIVE, 'tracked_tree_clean': True,
                  'revision': 'a' * 40, 'dll_sha256': 'b' * 64,
                  'sources': {s: 'c' * 64 for s in SOURCES},
                  'common': {'loaded': False, 'trace_hex': EXPECTED.hex()},
                  'rebased_common': None, 'backend_only': {'windows_last_error': 1114}}
        host = copy.deepcopy(native)
        host.update(backend=HOST, rebased_common=copy.deepcopy(host['common']),
                    backend_only={'host_last_error': 0x1234})
        return [native, host]

    def test_exact_trace_and_independent_error_observations(self):
        for windows in (False, True):
            common, only = check_execution(self.execution(windows), windows)
            self.assertEqual(common, self.reports()[0]['common'])
            self.assertEqual(only, self.reports()[0 if windows else 1]['backend_only'])
        reports = self.reports()
        self.assertEqual(compare(reports, reference=reports[0]), 32)

    def test_crash_missing_duplicate_reordered_and_changed_trace_fail(self):
        bad_traces = [b'', EXPECTED[:-1], EXPECTED + EXPECTED[16:],
                      EXPECTED[16:] + EXPECTED[:16], bytes([0]) + EXPECTED[1:]]
        for trace in bad_traces:
            run = self.execution()
            run.stdout = trace
            with self.assertRaises(ValueError):
                check_execution(run, False)
        run = self.execution()
        run.returncode = -11
        with self.assertRaises(ValueError):
            check_execution(run, False)

    def test_claimed_success_wrong_error_and_merged_output_fail(self):
        for field, value in [('loaded', True), ('attach_rejected', False),
                             ('host_last_error', 0), ('host_last_error', True),
                             ('windows_last_error', 1114)]:
            run = self.execution()
            result = json.loads(run.stderr)
            result[field] = value
            run.stderr = json.dumps(result).encode()
            with self.assertRaises(ValueError):
                check_execution(run, False)
        run = self.execution()
        run.stderr += b'\nextra trace'
        with self.assertRaises(ValueError):
            check_execution(run, False)

    def test_dirty_missing_identity_trace_and_rebase_reports_fail(self):
        for field, value in [('tracked_tree_clean', False), ('revision', 'd' * 40),
                             ('dll_sha256', 'e' * 64), ('sources', {}), ('common', {}),
                             ('rebased_common', None), ('backend', NATIVE),
                             ('backend_only', {'windows_last_error': 1114})]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])
        with self.assertRaises(ValueError):
            compare(self.reports()[:1], reference=self.reports()[0])

    def test_frozen_reference_rejects_joint_identity_and_native_error_drift(self):
        for field, value in [('dll_sha256', 'd' * 64), ('sources', {s: 'e' * 64 for s in SOURCES})]:
            reports = self.reports()
            for r in reports:
                r[field] = value
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])
        reports = self.reports()
        reports[0]['backend_only']['windows_last_error'] = 0
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])

    def test_default_frozen_reference_cannot_be_bypassed_by_joint_drift(self):
        native = json.loads(REFERENCE.read_text(encoding='utf-8'))
        host = copy.deepcopy(native)
        host.update(backend=HOST, rebased_common=copy.deepcopy(host['common']),
                    backend_only={'host_last_error': 0x1234})
        self.assertEqual(compare([native, host]), 32)
        for r in (native, host):
            r['dll_sha256'] = 'e' * 64
        with self.assertRaises(ValueError):
            compare([native, host])


if __name__ == '__main__':
    unittest.main()
