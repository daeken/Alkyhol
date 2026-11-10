"""Do not accept changed outputs or incomplete startup as native comparison success."""
import importlib.util
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('check_jit32_startup',Path(__file__).with_name('check_jit32_startup.py'))
probe=importlib.util.module_from_spec(spec)
# The companion tool uses the same tools-directory imports as its CLI.
import sys
sys.path.insert(0,str(Path(__file__).parent))
spec.loader.exec_module(probe)


class StartupTransport(unittest.TestCase):
    def test_exact_startup_execution_transport_only(self):
        expected=b'finite result'
        stderr=b'JIT32 startup: phase=Ready modules=1 stop=Returned(0)\nJIT32 ARM64: instructions=135 services=4 blocks=135 stop=Returned(0)\n'
        self.assertEqual(probe.decode(0,expected,stderr,expected)['instructions'],135)
        for status,output,diagnostic in [(False,expected,stderr),(1,expected,stderr),(0,b'',stderr),(0,expected,stderr+b'extra\n'),(0,expected,stderr.replace(b'Ready',b'Failed')),(0,expected,stderr.replace(b'services=4',b'services=0')),(0,expected,stderr.replace(b'Returned(0)',b'Budget'))]:
            with self.subTest(status=status,output=output,diagnostic=diagnostic):
                with self.assertRaises(ValueError):probe.decode(status,output,diagnostic,expected)
