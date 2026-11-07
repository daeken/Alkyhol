import copy
import importlib.util
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('compare_pe_probe', ROOT / 'tools/compare_pe_probe.py')
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class PEComparisonTests(unittest.TestCase):
    def reports(self):
        native = {'backend': 'native Windows PE64 / kernel32', 'schema_version': 1,
                  'tracked_tree_clean': True, 'revision': 'a' * 40, 'pe_sha256': 'b' * 64, 'executed_pe_sha256': 'b' * 64,
                  'sources': {'sample.c': 'c' * 64}, 'observation': {'status': 0, 'words': [1, 2], 'file_hex': '00ff'},
                  'forced_rebase_observation': None}
        host = copy.deepcopy(native)
        host['backend'] = 'Alkyhol Win64 ABI / x86-64 Mach-O'
        host['forced_rebase_observation'] = copy.deepcopy(host['observation'])
        return [native, host]

    def compare(self, reports):
        return module.compare(reports, self.reports()[0])

    def test_matching_execution_is_accepted(self):
        self.assertEqual(self.compare(self.reports()), 'b' * 64)

    def test_payload_source_revision_and_results_cannot_drift(self):
        for field, value in [('pe_sha256', 'd' * 64), ('executed_pe_sha256', 'd' * 64), ('sources', {}), ('revision', 'e' * 40),
                             ('observation', {'status': 0, 'words': [9], 'file_hex': ''})]:
            with self.subTest(field=field):
                reports = self.reports()
                reports[1][field] = value
                with self.assertRaises(ValueError):
                    self.compare(reports)

    def test_both_executions_and_clean_tree_are_required(self):
        for backend, clean in [('synthetic interpreter', True), ('native Windows PE64 / kernel32', True),
                               ('Alkyhol Win64 ABI / x86-64 Mach-O', False)]:
            reports = self.reports()
            reports[1].update(backend=backend, tracked_tree_clean=clean)
            with self.assertRaises(ValueError):
                self.compare(reports)
        with self.assertRaises(ValueError):
            self.compare(self.reports()[:1])

    def test_relocation_result_must_be_a_real_matching_run(self):
        for value in [None, {'status': 0, 'words': [9], 'file_hex': '00ff'}]:
            reports = self.reports()
            reports[1]['forced_rebase_observation'] = value
            with self.assertRaises(ValueError):
                self.compare(reports)

    def test_joint_drift_cannot_replace_native_reference(self):
        for field, value in [('pe_sha256', 'e' * 64), ('sources', {'other.c': 'f' * 64}),
                             ('observation', {'status': 0, 'words': [99], 'file_hex': ''})]:
            reports = self.reports()
            for report in reports:
                report[field] = copy.deepcopy(value)
                if field == 'pe_sha256':
                    report['executed_pe_sha256'] = value
            reports[1]['forced_rebase_observation'] = copy.deepcopy(reports[1]['observation'])
            with self.assertRaises(ValueError):
                self.compare(reports)



if __name__ == '__main__':
    unittest.main()
