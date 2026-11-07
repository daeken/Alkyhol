"""Protocol units only. These synthetic reports do not execute Windows code."""
import copy
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from compare_uconfig_binary import compare
from fetch_uconfig_binary import PE_SHA256, SOURCE_SHA256, LICENSE_SHA256, verified
from run_uconfig_binary import guest_stderr


class ReleasedPEProtocolTests(unittest.TestCase):
    def reports(self):
        spec = json.loads((ROOT / 'pe/fixtures/u-config/cases.json').read_text(encoding='utf-8'))
        observations = {c['id']: {'status': c['status'], 'stdout_hex': c['stdout'].encode().hex(),
                                  'stderr_hex': c['stderr'].encode().hex()} for c in spec['cases']}
        native = {'schema_version': 1, 'backend': 'native Windows PE64 / kernel32',
                  'tracked_tree_clean': True, 'program': 'u-config', 'revision': 'a' * 40,
                  'pe_sha256': PE_SHA256, 'executed_pe_sha256': PE_SHA256,
                  'upstream_source_sha256': SOURCE_SHA256, 'upstream_license_sha256': LICENSE_SHA256,
                  'input_sha256': {'cases.json': 'b' * 64}, 'observations': observations,
                  'forced_rebase_observations': None}
        host = copy.deepcopy(native)
        host['backend'] = 'Alkyhol Win64 ABI / x86-64 Mach-O'
        host['forced_rebase_observations'] = copy.deepcopy(observations)
        return [native, host]

    def test_native_host_and_rebased_results_must_all_match(self):
        reports = self.reports()
        self.assertEqual(compare(reports, reference=reports[0]), 11)
        for field, value in [('pe_sha256', '0' * 64), ('executed_pe_sha256', '0' * 64),
                             ('upstream_source_sha256', '0' * 64), ('upstream_license_sha256', '0' * 64),
                             ('revision', '0' * 40), ('input_sha256', {}), ('observations', {}),
                             ('forced_rebase_observations', None), ('tracked_tree_clean', False)]:
            with self.subTest(field=field):
                reports = self.reports()
                reports[1][field] = value
                with self.assertRaises(ValueError):
                    compare(reports, reference=self.reports()[0])

    def test_joint_input_drift_and_missing_native_backend_fail(self):
        reports = self.reports()
        for r in reports:
            r['input_sha256'] = {'changed.pc': 'c' * 64}
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])
        reports = self.reports()
        reports[0]['backend'] = reports[1]['backend']
        with self.assertRaises(ValueError):
            compare(reports, reference=self.reports()[0])
        with self.assertRaises(ValueError):
            compare(self.reports()[:1], reference=self.reports()[0])

    def test_only_exact_host_entry_prefix_is_removed(self):
        prefix = b'PE64: 4 sections, 15 named imports, 11 DIR64 fixups\nentering Win64 code; relocated=true\n'
        self.assertEqual(guest_stderr(prefix + b'guest error\n', True), b'guest error\n')
        for stderr in [b'unsupported import\n', b'garbage\n' + prefix,
                       prefix.replace(b'true', b'false')]:
            with self.assertRaises(ValueError):
                guest_stderr(stderr, True)

    def test_unicode_case_has_the_same_explicit_utf8_meaning_on_every_host(self):
        spec = json.loads((ROOT / 'pe/fixtures/u-config/cases.json').read_text(encoding='utf-8'))
        case = next(c for c in spec['cases'] if c['id'] == 'quoted-unicode-variable')
        self.assertEqual(case['stdout'].encode('utf-8').hex(), 'cebbf09f8cb82071756f746564202276616c756522207461696c5c0a')
        self.assertIn('λ🌸', case['args'][0])

    def test_fetch_hash_failure_never_becomes_success(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / 'released.exe'
            path.write_bytes(b'not the pinned executable')
            with self.assertRaises(ValueError):
                verified(path, PE_SHA256)


if __name__ == '__main__':
    unittest.main()
