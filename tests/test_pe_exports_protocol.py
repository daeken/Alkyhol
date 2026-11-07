"""Synthetic report protocol tests; no native DLL execution is performed here."""
import copy
from pathlib import Path
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from compare_pe_exports import compare
from run_pe_exports import SPEC


class ExportProtocolTests(unittest.TestCase):
    def reports(self):
        entries = [{'ordinal': o, 'names': names, 'target': {'forwarder': f} if f else {'rva': 4096 if o in (17,18) else 8192 + o}} for o,names,f in SPEC]
        native = {'schema_version': 1, 'backend': 'native Windows LoadLibrary/GetProcAddress',
                  'tracked_tree_clean': True, 'revision': 'a' * 40, 'dll_sha256': 'b' * 64,
                  'sources': {'library.c': 'c' * 64},
                  'common': {'dll_name': 'alkyhol_exports.dll', 'ordinal_base': 17, 'entries': entries},
                  'native_only': {'initialized_data': 0x31415926, 'bss_after_attach': 0x13579bdf,
                                  'direct_call': (7 * 33) ^ 11 ^ 0x31415926 ^ 0x13579bdf,
                                  'ordinal_call': 0x12345681, 'hole19': 'NULL'}}
        host = copy.deepcopy(native)
        host.update(backend='Rust metadata only (no DLL entry)', native_only=None)
        return [native, host]

    def test_matching_metadata_keeps_native_execution_separate(self):
        reports = self.reports()
        self.assertEqual(compare(reports, reference=reports[0]), 6)
        reports[1]['native_only'] = copy.deepcopy(reports[0]['native_only'])
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])

    def test_identity_result_and_joint_drift_cannot_pass(self):
        for field, value in [('revision', 'z'*40), ('dll_sha256', 'z'*64), ('sources', {}),
                             ('common', {}), ('tracked_tree_clean', False)]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises(ValueError):
                compare(reports, reference=self.reports()[0])
        reports = self.reports()
        for r in reports:
            r['dll_sha256'] = 'd' * 64
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])
        reports = self.reports()
        reports[0]['native_only']['bss_after_attach'] = 0
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])


if __name__ == '__main__':
    unittest.main()
