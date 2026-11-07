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
                  'tracked_tree_clean': True, 'revision': 'a' * 40, 'pe_sha256': 'b' * 64,
                  'sources': {'sample.c': 'c' * 64}, 'observation': {'status': 0, 'words': [1, 2], 'file_hex': '00ff'},
                  'forced_rebase_observation': None}
        host = copy.deepcopy(native)
        host['backend'] = 'Alkyhol Win64 ABI / x86-64 Mach-O'
        host['forced_rebase_observation'] = copy.deepcopy(host['observation'])
        return [native, host]

    def test_matching_execution_is_accepted(self):
        self.assertEqual(module.compare(self.reports()), 'b' * 64)

    def test_payload_source_revision_and_results_cannot_drift(self):
        for field, value in [('pe_sha256', 'd' * 64), ('sources', {}), ('revision', 'e' * 40),
                             ('observation', {'status': 0, 'words': [9], 'file_hex': ''})]:
            with self.subTest(field=field):
                reports = self.reports()
                reports[1][field] = value
                with self.assertRaises(ValueError):
                    module.compare(reports)

    def test_both_executions_and_clean_tree_are_required(self):
        for backend, clean in [('synthetic interpreter', True), ('native Windows PE64 / kernel32', True),
                               ('Alkyhol Win64 ABI / x86-64 Mach-O', False)]:
            reports = self.reports()
            reports[1].update(backend=backend, tracked_tree_clean=clean)
            with self.assertRaises(ValueError):
                module.compare(reports)
        with self.assertRaises(ValueError):
            module.compare(self.reports()[:1])

    def test_relocation_result_must_be_a_real_matching_run(self):
        for value in [None, {'status': 0, 'words': [9], 'file_hex': '00ff'}]:
            reports = self.reports()
            reports[1]['forced_rebase_observation'] = value
            with self.assertRaises(ValueError):
                module.compare(reports)


if __name__ == '__main__':
    unittest.main()
