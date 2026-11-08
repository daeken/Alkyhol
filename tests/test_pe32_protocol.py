"""Synthetic pe32 protocol negatives; native evidence is separate."""
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
from build_pe32_fixture import SOURCES
from run_pe32_probe import REFERENCE, EXPECTED_WORDS, NATIVE, HOST, decode
from compare_pe32_probe import compare


class Pe32ProtocolTests(unittest.TestCase):
    def execution(self, windows=False):
        return subprocess.CompletedProcess([], 0, struct.pack('<8I', *EXPECTED_WORDS), b'' if windows else
            b'PE32: 4 sections, 4 imports, 19 HIGHLOW fixups\nchecked i386 mapping; relocated=true\n')

    def reports(self):
        native = {'schema_version': 1, 'backend': NATIVE, 'tracked_tree_clean': True,
                  'revision': 'a'*40, 'pe_sha256': 'b'*64, 'executed_pe_sha256': 'b'*64,
                  'sources': {s: 'c'*64 for s in SOURCES},
                  'normal': decode(self.execution(True), windows=True), 'rebased': None}
        host = copy.deepcopy(native)
        host.update(backend=HOST, rebased=copy.deepcopy(host['normal']))
        return [native, host]

    def test_exact_real_frame_context_register_return_and_preservation_results_match(self):
        reports = self.reports()
        self.assertEqual(decode(self.execution(),rebase=True),reports[1]['normal'])
        self.assertEqual(compare(reports, reference=reports[0]), 8)
        self.assertEqual(EXPECTED_WORDS[1],4)
        self.assertEqual(EXPECTED_WORDS[2],10000)

    def test_crashes_truncation_extra_bytes_and_every_raw_observation_byte_fail(self):
        run = self.execution()
        run.returncode = -11
        with self.assertRaises(ValueError): decode(run)
        for stdout in [b'', run.stdout[:-1], run.stdout+b'\0']:
            run = self.execution(); run.stdout = stdout
            with self.assertRaises(ValueError): decode(run)
        for index in range(32):
            run = self.execution()
            changed = bytearray(run.stdout); changed[index] ^= 1; run.stdout = bytes(changed)
            with self.assertRaises(ValueError): decode(run)

    def test_diagnostics_and_missing_actual_forced_rebase_fail(self):
        for stderr in [b'', b'noise', self.execution().stderr.replace(b'true', b'false')]:
            run = self.execution(); run.stderr = stderr
            with self.assertRaises(ValueError): decode(run, rebase=True)
        run = self.execution(True); run.stderr = b'pretend native success'
        with self.assertRaises(ValueError): decode(run, windows=True)

    def test_identity_backend_payload_raw_words_and_integer_types_are_required(self):
        for field, value in [('schema_version', True), ('tracked_tree_clean', False), ('backend', NATIVE),
            ('revision', 'd'*40), ('pe_sha256', 'e'*64), ('executed_pe_sha256', 'e'*64),
            ('sources', {}), ('rebased', None)]:
            reports = self.reports(); reports[1][field] = value
            with self.assertRaises(ValueError): compare(reports, reference=self.reports()[0])
        for key in ['normal', 'rebased']:
            for field, value in [('status', False), ('stdout_hex', '00')]:
                reports = self.reports(); reports[1][key][field] = value
                with self.assertRaises(ValueError): compare(reports, reference=self.reports()[0])
            reports = self.reports(); reports[1][key]['words'][1] = True
            with self.assertRaises(ValueError): compare(reports, reference=self.reports()[0])

    def test_joint_input_drift_cannot_replace_frozen_native_evidence(self):
        reports = self.reports()
        for r in reports: r.update(pe_sha256='e'*64, executed_pe_sha256='e'*64)
        with self.assertRaises(ValueError): compare(reports, reference=self.reports()[0])



if __name__ == '__main__': unittest.main()
