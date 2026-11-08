"""Synthetic transport rejection checks, not native Windows or game evidence."""
import struct,sys,unittest
from pathlib import Path
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'tools'))
import observe_pe32_private_profile_missing as profile
class MissingProfileProtocol(unittest.TestCase):
    def report(self):
        data=bytearray(struct.pack('<4I',0x494e4932,4,10,92))
        for i,(text,size) in enumerate(zip(profile.DEFAULTS,profile.SIZES)):
            units=list(struct.unpack('<'+'H'*(len(text.encode('utf-16-le'))//2),text.encode('utf-16-le')))
            n=len(units);units+=[0]+[0x7171]*(31-n)
            data+=struct.pack('<6I32HI',i,size,n,2,1,0x715e7a11,*units,0x715e7a11)
        return data
    def test_defaults_include_unicode_null_and_truncation_boundaries(self):
        rows=profile.decode(self.report())
        self.assertEqual([r['text'] for r in rows],profile.DEFAULTS)
        self.assertEqual(rows[4]['result'],4)
        self.assertEqual(rows[6]['result'],0)
    def test_all_results_errors_guards_stack_and_output_are_required(self):
        data=self.report()
        for i in range(10):
            for field in [0,4,8,12,16,20,24,88]:
                changed=bytearray(data);changed[16+i*92+field]^=1
                with self.subTest(case=i,field=field),self.assertRaises(ValueError):profile.decode(changed)
        for changed in [data[:-1],data+b'\0',b'']:
            with self.assertRaises(ValueError):profile.decode(changed)
        # Capacity is the actual caller-owned writable bound, even for size 1.
        changed=bytearray(data);changed[16+6*92+26]=0
        with self.assertRaises(ValueError):profile.decode(changed)
if __name__=='__main__':unittest.main()
