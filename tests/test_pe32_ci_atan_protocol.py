"""Frozen native atan state and transport checks; not game equivalence."""
import struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_ci_atan as atan

class AtanProtocol(unittest.TestCase):
    def test_native_reference_preserves_the_precision_flag_and_full_operand(self):
        report=atan.frozen();rows={(r['input'],r['requested']):r for r in report['records']}
        self.assertEqual(atan.digest(atan.ROOT/'pe/reference/pe32-ci-atan-v1.exe'),report['inputs']['files']['ci-atan.exe'])
        self.assertEqual(rows[2,0x27f]['sw_after'],0x3220)
        self.assertEqual(rows[8,0x37f]['sw_after'],0x3022)
        self.assertNotEqual(rows[2,0x27f]['result80'],rows[2,0x37f]['result80'])

    def packet(self):
        report=atan.frozen();raw=bytes.fromhex(report['stdout_hex']);out=bytearray(struct.pack('<6I',0x53513332,4,60,68,16,1))
        for i in range(16):
            source=i if i<=9 or i==12 else 2;at=24+(8*17+source)*68
            b=bytearray(raw[at:at+68]);struct.pack_into('<I',b,4,i);out.extend(b)
        return out

    def test_every_known_field_is_required_and_unverified_rows_are_separate(self):
        packet=self.packet();rows=atan.decode_domain(packet);atan.compare_known(rows)
        for field in ['result80','below80','cw_after','sw_after','sw_pop','mx_after','errno','last_error']:
            changed=[dict(r) for r in rows];changed[8][field]='00'*10 if field.endswith('80') else changed[8][field]^1
            with self.subTest(field=field),self.assertRaises(ValueError):atan.compare_known(changed)
        for data in [packet[:-1],packet+b'\0',b'']:
            with self.assertRaises(ValueError):atan.decode_domain(data)

if __name__=='__main__':unittest.main()
