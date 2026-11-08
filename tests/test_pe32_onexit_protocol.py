"""Transport integrity for the owned exit-table witness; not game evidence."""
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import observe_pe32_onexit as onexit


class OnexitProtocol(unittest.TestCase):
    def packet(self):
        error = 0xface1234
        locks = [1, error]
        table = [1, 1, 0, 1, 1, error, 1, 2, 1, 1, 1, 1, error,
                 1, 3, 1, 1, 1, 1, 1, 10, 20, 10, 121]
        preallocated = [1, 1, 2, 1, 1, 1, 1, 21, error]
        startup = [1, 68, 68, 1, 1, 1, 1, 1, 1, error]
        paths = [error, 1, 1, 1, 0]
        words = [0x4f583332, 4] + locks + table + preallocated + startup + paths + [53]
        return struct.pack('<53I', *words)

    def test_every_output_word_is_required_including_callback_order_and_error_state(self):
        data = self.packet()
        self.assertEqual(len(onexit.decode(data)), 53)
        for index in range(53):
            changed = bytearray(data)
            changed[index * 4] ^= 1
            with self.subTest(index=index), self.assertRaises(ValueError):
                onexit.decode(changed)

    def test_incomplete_extra_or_wrong_width_transport_is_rejected(self):
        data = self.packet()
        for malformed in [b'', data[:-1], data + b'\0', data[:8] + data[12:]]:
            with self.assertRaises(ValueError):
                onexit.decode(malformed)


if __name__ == '__main__':
    unittest.main()
