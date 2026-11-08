"""Owned x87 capture transport checks, not native math or game equivalence."""
import struct
import sys
import unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_ci_sqrt as sqrt

class SqrtProtocol(unittest.TestCase):
    def test_frozen_native_precision_double_rounding_and_domain_errors_are_retained(self):
        report=sqrt.frozen()
        self.assertEqual(sqrt.digest(sqrt.ROOT/'pe/reference/pe32-ci-sqrt-v1.exe'),report['inputs']['files']['ci-sqrt.exe'])
        rows={(r['input'],r['requested']):r for r in report['records']}
        self.assertNotEqual(rows[9,0x27f]['result80'],rows[9,0x37f]['result80'])
        self.assertEqual(rows[12,0x27f]['errno'],33)
        self.assertEqual(rows[3,0x27f]['below80'],'00000000000000c0ff3f')

    def test_bounded_records_require_every_known_native_field(self):
        raw=bytes.fromhex(sqrt.frozen()['stdout_hex'])
        packet=bytearray(struct.pack('<6I',0x53513332,4,60,68,12,2))
        for group in [4,8]:
            for i in range(12):
                original=min(i,9);at=24+(group*17+original)*68
                b=bytearray(raw[at:at+68]);struct.pack_into('<I',b,4,i);packet.extend(b)
        records=sqrt.decode_domain(packet);sqrt.compare_known(records)
        for field in ['result80','below80','sw_after','mx_after','last_error','errno']:
            changed=[dict(r) for r in records]
            changed[3][field]='00'*10 if field.endswith('80') else changed[3][field]^1
            with self.subTest(field=field),self.assertRaises(ValueError):sqrt.compare_known(changed)
        for data in [packet[:-1],packet+b'\0']:
            with self.assertRaises(ValueError):sqrt.decode_domain(data)

    def packet(self):
        result=bytearray(struct.pack('<6I',0x53513332,4,60,68,17,12))
        for row in range(204):
            b=bytearray([0xa5]*68)
            for at in [0,60,64]:struct.pack_into('<I',b,at,0x715e7a11)
            control=0x7f|[0,0x200,0x300][row//68]|(((row//17)%4)<<10)
            struct.pack_into('<2I',b,4,row%17,control)
            b[24:34]=bytes(range(10))
            result.extend(b)
        return result

    def test_all_numeric_contexts_and_raw_80bit_results_survive_transport(self):
        rows=sqrt.decode(self.packet())
        self.assertEqual(len(rows),204)
        self.assertEqual(rows[0]['result80'],bytes(range(10)).hex())
        self.assertEqual(rows[-1]['input'],16)
        self.assertEqual(rows[-1]['requested'],0xf7f)

    def test_truncation_wrong_layout_order_control_or_guards_cannot_pass(self):
        data=self.packet()
        for value in [data[:-1],data+b'\0',b'']:
            with self.assertRaises(ValueError):sqrt.decode(value)
        for at in [0,4,8,12,16,20,24,28,32,46,84,88,len(data)-4]:
            changed=data.copy();changed[at]^=1
            with self.subTest(offset=at),self.assertRaises(ValueError):sqrt.decode(changed)

if __name__=='__main__':unittest.main()
