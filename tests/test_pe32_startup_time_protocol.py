"""Frozen UTC evidence and synthetic clock transport checks, not game proof."""
import copy
import struct
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import observe_pe32_local_filetime as utc
import observe_pe32_startup_clock as clock


class LocalFiletimeProtocol(unittest.TestCase):
    def test_literal_native_pe_identity_and_every_conversion_word(self):
        report = utc.frozen()
        self.assertEqual(utc.digest(utc.ROOT / 'pe/reference/pe32-local-filetime-v1.exe'),
                         report['inputs']['files']['local-filetime.exe'])
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            utc.fixture(path)
            self.assertEqual(utc.verify(path), report['inputs'])
        utc.compare(report['decoded'])
        self.assertEqual(len(bytes.fromhex(report['stdout_hex'])), 106 * 4)
        # Native UTC accepts high-bit FILETIMEs and both exact-alias cases.
        self.assertEqual([row[4] for row in report['decoded']['records']], [1] * 9)

    def test_guards_timezone_order_and_truncation_are_required(self):
        raw = bytes.fromhex(utc.frozen()['stdout_hex'])
        for offset in [0, 4, 12, 16, 20, 24, 28, 28 + 8 * 4, 28 + 9 * 4]:
            data = bytearray(raw)
            data[offset] ^= 1
            with self.subTest(offset=offset), self.assertRaises(ValueError):
                utc.decode(data)
        for data in [b'', raw[:-1], raw + b'\0']:
            with self.assertRaises(ValueError):
                utc.decode(data)

    def test_changed_result_error_source_or_alias_cannot_match_native(self):
        report = utc.frozen()
        for row in [0, 5, 6, 7, 8]:
            for field in range(2, 11):
                value = copy.deepcopy(report['decoded'])
                value['records'][row][field] ^= 1
                with self.subTest(row=row, field=field), self.assertRaises(ValueError):
                    utc.compare(value)


class StartupClockProtocol(unittest.TestCase):
    def packet(self, boot=1000):
        error, guard = 0xface1234, 0x715e7a11
        words = [0x434c3332, 4, 1234, error, 1234, 7, error, 7,
                 1, error, 1_000_000_000, 0, guard, boot, error,
                 1, error, 10_000_000, 0, guard, (boot + 32) & 0xffffffff,
                 1, error, 42_000_000, 0, guard]
        for old, new in [(0, 0xffffffff), (0xffffffff, 0xffffffff),
                         (0xffffffff, 0x80000000), (0x80000000, 0xffffffff)]:
            words += [old, error, new, guard, guard]
        return words + [47]

    def decode(self, words, pid=1234, before=990, after=1050):
        return clock.decode(struct.pack('<47I', *words), pid, before, after, 0.060)

    def test_dynamic_pid_clock_brackets_rate_and_atomic_bits(self):
        self.assertEqual(self.decode(self.packet())['tick_delta_ms'], 32)
        # A boot DWORD can wrap during a single observation.
        self.assertEqual(self.decode(self.packet(0xfffffff0), before=0xffffffe0,
                                     after=0x100000030)['tick_delta_ms'], 32)

    def test_constants_wrong_pid_guards_errors_and_atomic_results_fail(self):
        with self.assertRaises(ValueError):
            self.decode(self.packet(), pid=1235)
        for index in [0, 1, 2, 3, 4, 5, 7, 8, 9, 11, 12, 14, 15, 16, 19,
                      21, 22, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35,
                      36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46]:
            words = self.packet()
            words[index] ^= 1
            with self.subTest(index=index), self.assertRaises(ValueError):
                self.decode(words)
        for index, value in [(13, 0), (20, 1000), (23, 10_000_000),
                             (23, 1_000_000_000), (10, 1)]:
            words = self.packet()
            words[index] = value
            with self.subTest(index=index, value=value), self.assertRaises(ValueError):
                self.decode(words)
        raw = struct.pack('<47I', *self.packet())
        for data in [b'', raw[:-1], raw + b'\0\0\0\0']:
            with self.assertRaises(ValueError):
                clock.decode(data, 1234, 990, 1050, 0.06)


if __name__ == '__main__':
    unittest.main()
