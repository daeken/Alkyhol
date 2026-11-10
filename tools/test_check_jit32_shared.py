import unittest
from check_jit32_shared import decode
class SharedTransport(unittest.TestCase):
    def test_requires_real_success_exact_bytes_and_positive_counters(self):
        stderr=b'PE32 diagnostic mode: guarded external imports; every fault stops; no exception recovery\nJIT32 shared runtime: Stats { instructions: 9, services: 2, compiled: 3, invocations: 1 }\n'
        self.assertEqual(decode(0,b'ab',stderr,b'ab')['services'],2)
        for status,data,error in [(78,b'ab',stderr),(0,b'a',stderr),(0,b'ab',stderr+b'ignored'),(0,b'ab',stderr.replace(b'services: 2',b'services: 0'))]:
            with self.assertRaises(ValueError):decode(status,data,error,b'ab')
