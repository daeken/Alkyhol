import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('run_samples', Path(__file__).resolve().parents[1] / 'tools/run_samples.py')
samples = importlib.util.module_from_spec(spec)
spec.loader.exec_module(samples)


class SampleObservationTests(unittest.TestCase):
    def observe(self, text, **kwargs):
        with tempfile.TemporaryDirectory() as directory:
            return samples.run_observation([sys.executable, '-c', 'print(' + repr(text) + ')'], {'ok': True}, directory, **kwargs)

    def test_accept_exact_boolean_observation(self):
        self.assertEqual(self.observe('{"ok":true}'), {'ok': True})

    def test_reject_wrong_types_missing_extra_or_false_results(self):
        for value in [{'ok': 1}, {'ok': False}, {}, {'ok': True, 'extra': True}, [True], None]:
            with self.subTest(value=value), self.assertRaises(RuntimeError):
                self.observe(json.dumps(value))

    def test_reject_malformed_multiple_and_duplicate_json(self):
        for text in ['not JSON', '{"ok":true}\n{"ok":true}', '{"ok":true,"ok":true}']:
            with self.subTest(text=text), self.assertRaises(ValueError):
                self.observe(text)

    def test_nonzero_exit_cannot_pass_even_with_valid_observation(self):
        with tempfile.TemporaryDirectory() as directory, self.assertRaises(subprocess.CalledProcessError):
            samples.run_observation([sys.executable, '-c', 'print(\'{"ok":true}\'); raise SystemExit(1)'], {'ok': True}, directory)

    def test_timeout_cannot_pass(self):
        with tempfile.TemporaryDirectory() as directory, self.assertRaises(subprocess.TimeoutExpired):
            samples.run_observation([sys.executable, '-c', 'import time; time.sleep(30)'], {'ok': True}, directory, timeout=0.1)

    def test_all_sample_sources_exist_and_have_nonempty_expectations(self):
        self.assertEqual(set(samples.SAMPLES), {'kernel_contract', 'msdn_file_write', 'msdn_event_readers', 'msdn_semaphore_workers', 'msdn_thread_local_storage', 'msdn_creating_threads'})
        for source, expected in samples.SAMPLES.values():
            self.assertTrue((samples.ROOT / source).is_file())
            self.assertTrue(expected)
            self.assertTrue(all(type(v) is bool for v in expected.values()))

    def test_compile_commands_preserve_paths_with_spaces(self):
        for suffix in ['.c', '.cpp']:
            source, exe = Path('with spaces/example' + suffix), Path('with spaces/program')
            command = samples.compile_command(source, exe)
            self.assertIn(str(source), command)
            self.assertTrue(str(exe) in command or '/Fe:' + str(exe) in command)


if __name__ == '__main__':
    unittest.main()
