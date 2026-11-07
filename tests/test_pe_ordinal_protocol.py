"""Synthetic protocol negatives, distinct from actual native Windows ordinal evidence."""
import copy
import json
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from compare_pe_ordinal import compare, NATIVE, HOST
from build_pe_ordinal import SOURCES, IMPORTS
from run_pe_ordinal import DEPENDENCY_REFERENCE, EXPECTED, check_result


class OrdinalProtocolTests(unittest.TestCase):
    def reports(self):
        old = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
        native = {'schema_version': 1, 'backend': NATIVE, 'tracked_tree_clean': True,
                  'revision': 'a' * 40, 'binaries': {'alkyhol_exports.dll': old['dll_sha256'], 'alkyhol_ordinal.dll': 'b' * 64},
                  'sources': {s: old['sources'].get(s, 'c' * 64) for s in SOURCES}, 'imports': copy.deepcopy(IMPORTS),
                  'normal': {'common': copy.deepcopy(EXPECTED), 'relocated': None}, 'rebased': None}
        host = copy.deepcopy(native)
        host.update(backend=HOST, normal={'common': copy.deepcopy(EXPECTED), 'relocated': [False, True]},
                    rebased={'common': copy.deepcopy(EXPECTED), 'relocated': [True, True]})
        return [native, host]

    def test_actual_execution_fields_keep_native_and_forced_rebase_distinct(self):
        reports = self.reports()
        check_result(reports[0]['normal'], windows=True)
        check_result(reports[1]['rebased'], rebase=True)
        self.assertEqual(compare(reports, reference=reports[0]), 7)

    def test_wrong_data_calls_target_identities_and_rebase_flags_fail(self):
        for key in EXPECTED:
            value = copy.deepcopy(self.reports()[1]['normal'])
            value['common'][key] = 0
            with self.assertRaises(ValueError):
                check_result(value)
        for flags in [None, [], [True], [False, True], [1, True]]:
            value = copy.deepcopy(self.reports()[1]['normal'])
            value['relocated'] = flags
            with self.assertRaises(ValueError):
                check_result(value, rebase=True)

    def test_dirty_identity_missing_inventory_and_invented_native_rebase_fail(self):
        for field, value in [('tracked_tree_clean', False), ('revision', 'd' * 40), ('sources', {}),
                             ('binaries', {}), ('imports', []), ('backend', NATIVE),
                             ('rebased', {'common': EXPECTED, 'relocated': [False, True]})]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])
        reports = self.reports()
        reports[0]['rebased'] = reports[1]['rebased']
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])

    def test_frozen_reference_rejects_joint_consumer_and_dependency_drift(self):
        for name in ['alkyhol_exports.dll', 'alkyhol_ordinal.dll']:
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


if __name__ == '__main__':
    unittest.main()
