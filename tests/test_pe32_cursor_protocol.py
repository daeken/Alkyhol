"""Keep cursor ABI assertions distinct from per-host desktop coordinates."""
import struct
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'tools'))
import observe_pe32_cursor as cursor


class CursorProtocol(unittest.TestCase):
    def packet(self, x=-30, y=2048):
        return struct.pack('<8I2iI', 0x43503332, 4, 8, 1, 1,
                           0x715e7a11, 0x715e7a11, 0xface1234, x, y, 11)

    def test_signed_coordinates_are_records_not_cross_host_constants(self):
        first = cursor.decode(self.packet())
        other = cursor.decode(self.packet(-2147483648, 2147483647))
        self.assertEqual((first['x'], first['y']), (-30, 2048))
        self.assertEqual(other['x'], -2147483648)
        self.assertEqual(other['y'], 2147483647)
        self.assertEqual(first['abi_words'], other['abi_words'])

    def test_every_contract_field_and_transport_extent_is_required(self):
        data = self.packet()
        for index in [0, 1, 2, 3, 4, 5, 6, 7, 10]:
            changed = bytearray(data)
            changed[index * 4] ^= 1
            with self.subTest(index=index), self.assertRaises(ValueError):
                cursor.decode(changed)
        for changed in [b'', data[:-1], data + b'\0']:
            with self.assertRaises(ValueError):
                cursor.decode(changed)


if __name__ == '__main__':
    unittest.main()
