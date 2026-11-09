"""Transport/identity failures must not pass the real JIT comparator."""
import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('check_jit32',Path(__file__).with_name('check_jit32.py'))
probe=importlib.util.module_from_spec(spec)
spec.loader.exec_module(probe)


class JitReportTests(unittest.TestCase):
    def test_exact_success_and_reject_status_bytes_or_diagnostics(self):
        expected=bytes(range(32))
        summary=b'JIT32 ARM64: instructions=100089 services=20005 blocks=99 stop=Returned(0)\n'
        report=probe.decode_jit(0,expected,summary,expected)
        self.assertEqual(report['instructions'],100089)
        for status,stdout,stderr in [
            (1,expected,summary),(False,expected,summary),(0,expected[:-1],summary),
            (0,expected+b'\0',summary),(0,expected,b''),(0,expected,summary+b'extra\n'),
            (0,expected,summary.replace(b'20005',b'20004')),
            (0,expected,summary.replace(b'100089',b'9')),
            (0,expected,summary.replace(b'blocks=99',b'blocks=0')),
            (0,expected,summary.replace(b'Returned(0)',b'Budget { pc: 0 }'))]:
            with self.subTest(status=status,stdout=stdout,stderr=stderr):
                with self.assertRaises(ValueError):
                    probe.decode_jit(status,stdout,stderr,expected)


if __name__=='__main__':unittest.main()
