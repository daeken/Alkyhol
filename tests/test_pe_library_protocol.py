"""Synthetic protocol negatives; actual DLL execution belongs to the PE workflow."""
import copy
from pathlib import Path
import sys
import unittest
ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from compare_pe_library import compare
import test_pe_exports_protocol as export_protocol


class LibraryProtocolTests(unittest.TestCase):
    def reports(self):
        native = export_protocol.ExportProtocolTests().reports()[0]
        result = {'common': copy.deepcopy(native['common']), 'execution': copy.deepcopy(native['native_only']),
                  'relocated': False, 'forwarded_last_error': 0x1234}
        host = {k: copy.deepcopy(native[k]) for k in ['schema_version', 'tracked_tree_clean', 'revision', 'dll_sha256', 'sources']}
        host.update(backend='Alkyhol isolated Win64 DLL', normal=result, rebased=copy.deepcopy(result))
        host['rebased']['relocated'] = True
        return [native, host]

    def test_actual_runtime_observations_and_rebase_must_match_native(self):
        reports = self.reports()
        self.assertEqual(compare(reports, reference=reports[0]), 6)
        for field, value in [('execution', {}), ('common', {}), ('forwarded_last_error', 0)]:
            reports = self.reports()
            reports[1]['normal'][field] = value
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])
        reports = self.reports()
        reports[1]['rebased']['relocated'] = False
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])

    def test_metadata_dirty_identity_and_joint_drift_do_not_pass_as_runtime(self):
        for field, value in [('backend', 'Rust metadata only (no DLL entry)'), ('tracked_tree_clean', False),
                             ('revision', 'z' * 40), ('dll_sha256', 'z' * 64), ('sources', {})]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])
        reports = self.reports()
        for r in reports:
            r['dll_sha256'] = 'd' * 64
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])


if __name__ == '__main__':
    unittest.main()
