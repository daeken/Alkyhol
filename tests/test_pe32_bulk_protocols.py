"""Owned wire-protocol rejection tests; no synthetic execution equivalence claim."""
import sys,struct,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_runtime_family as family
import observe_pe32_profile_roundtrip as profile
class BulkProtocols(unittest.TestCase):
    def test_native_profiles_preserve_raw_bytes_and_literal_inputs(self):
        r=profile.frozen()
        for i,row in enumerate(r['reports']):
            data=bytes.fromhex(row['stdout_hex']);profile.decode(data,i)
            for at in [0,4,8,12,20,28,32,36,164]:
                bad=bytearray(data);bad[at]^=1
                with self.subTest(step=i,at=at),self.assertRaises(ValueError):profile.decode(bad,i)
            self.assertEqual(profile.digest(profile.ROOT/f'pe/reference/pe32-profile-roundtrip-{i}-v1.exe'),r['inputs']['files'][profile.FILES[i]])
        for name,sha in r['inputs']['sources'].items():self.assertEqual(profile.digest(profile.ROOT/name),sha)
    def test_profile_transport_is_complete(self):
        data=bytes.fromhex(profile.frozen()['reports'][0]['stdout_hex'])
        for value in [b'',data[:-1],data+b'\0']:
            with self.assertRaises(ValueError):profile.decode(value,0)
    def test_family_transport_rejects_missing_and_extra_bytes(self):
        for data in [b'',b'x',struct.pack('<4I',0x46414d32,4,1,4)]:
            with self.assertRaises(ValueError):family.decode(data)
if __name__=='__main__':unittest.main()
