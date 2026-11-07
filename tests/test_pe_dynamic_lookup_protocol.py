"""Synthetic guest lookup protocol negatives; native execution is separate."""
import copy
import json
from pathlib import Path
import struct
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'tools'))
from build_pe_dynamic_lookup import SOURCES, IMPORTS, DEPENDENCY_REFERENCE
from run_pe_dynamic_lookup import REFERENCE, EXPECTED_WORDS, NATIVE, HOST, decode
from compare_pe_dynamic_lookup import compare


class DynamicLookupProtocolTests(unittest.TestCase):
    def execution(self, windows=False):
        return subprocess.CompletedProcess([], 0, struct.pack('<21I', *EXPECTED_WORDS),
            b'' if windows else b'{"entry_status":0,"relocated":[true,true]}\n')

    def reports(self):
        dependency = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
        native = {'schema_version': 1, 'backend': NATIVE, 'tracked_tree_clean': True,
                  'revision': 'a'*40, 'binaries': {'dynamic-lookup.exe': 'b'*64,
                  'alkyhol_exports.dll': dependency['dll_sha256']},
                  'sources': {s: 'c'*64 for s in SOURCES}, 'imports': IMPORTS,
                  'normal': decode(self.execution(True), windows=True), 'rebased': None}
        native['sources'].update(dependency['sources'])
        host = copy.deepcopy(native)
        host.update(backend=HOST, normal=decode(self.execution()), rebased=decode(self.execution(), rebase=True))
        return [native, host]

    def test_exact_native_and_host_module_symbol_data_and_failure_results_match(self):
        reports = self.reports()
        self.assertEqual(compare(reports, reference=reports[0]), 21)
        self.assertEqual(reports[0]['normal']['words'], reports[1]['rebased']['words'])

    def test_crash_truncation_extra_bytes_and_every_actual_result_byte_fail(self):
        run = self.execution()
        run.returncode = -11
        with self.assertRaises(ValueError): decode(run)
        for stdout in [b'', run.stdout[:-1], run.stdout+b'\0']:
            run = self.execution()
            run.stdout = stdout
            with self.assertRaises(ValueError): decode(run)
        for i in range(84):
            run = self.execution()
            changed = bytearray(run.stdout)
            changed[i] ^= 1
            run.stdout = bytes(changed)
            with self.assertRaises(ValueError): decode(run)

    def test_missing_entry_status_diagnostics_or_actual_rebases_fail(self):
        for stderr in [b'', b'noise', b'{}', b'{"entry_status":false,"relocated":[true,true]}',
            b'{"entry_status":0,"relocated":[true]}', b'{"entry_status":0,"relocated":[1,1]}',
            b'{"entry_status":0,"relocated":[true,false]}']:
            run = self.execution()
            run.stderr = stderr
            with self.assertRaises(ValueError): decode(run, rebase=True)
        run = self.execution(True)
        run.stderr = b'pretend native success'
        with self.assertRaises(ValueError): decode(run, windows=True)

    def test_identity_imports_backend_raw_results_and_rebases_are_required(self):
        for field, value in [('schema_version', 2), ('schema_version', True), ('tracked_tree_clean', False),
            ('backend', NATIVE), ('revision', 'd'*40), ('binaries', {}),
            ('sources', {}), ('imports', []), ('rebased', None)]:
            reports = self.reports()
            reports[1][field] = value
            with self.assertRaises((ValueError, TypeError)): compare(reports, reference=self.reports()[0])
        for field, value in [('words', [0]*21), ('stdout_hex', '00'), ('relocated', [True, False])]:
            reports = self.reports()
            reports[1]['rebased'][field] = value
            with self.assertRaises(ValueError): compare(reports, reference=self.reports()[0])

    def test_joint_input_drift_cannot_replace_either_native_binary_contract(self):
        for binary in ['dynamic-lookup.exe', 'alkyhol_exports.dll']:
            reports = self.reports()
            for r in reports: r['binaries'][binary] = 'e'*64
            with self.assertRaises(ValueError): compare(reports, reference=self.reports()[0])

    def test_default_frozen_native_record_and_exact_input_are_required(self):
        native = json.loads(REFERENCE.read_text(encoding='utf-8'))
        host = copy.deepcopy(native)
        host.update(backend=HOST, normal=decode(self.execution()), rebased=decode(self.execution(), rebase=True))
        self.assertEqual(compare([native, host]), 21)
        with tempfile.TemporaryDirectory() as directory:
            with mock.patch('compare_pe_dynamic_lookup.REFERENCE', Path(directory)/'missing.json'):
                with self.assertRaises(FileNotFoundError): compare([native, host])
        for r in [native, host]: r['binaries']['dynamic-lookup.exe'] = 'e'*64
        with self.assertRaises(ValueError): compare([native, host])


if __name__ == '__main__': unittest.main()
