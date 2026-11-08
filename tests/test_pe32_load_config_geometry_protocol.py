"""Synthetic exact-patch/snapshot tests, not native loading or GS evidence."""
import copy
import hashlib
import json
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest import mock
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'tools'))
from build_pe32_load_config_geometry import SOURCES,variants
from run_pe32_load_config_geometry import verify


def bases():
    result={}
    for size in [64,72]:
        b=bytearray(0x400);b[:2]=b'MZ';struct.pack_into('<I',b,0x3c,0x80)
        b[0x80:0x84]=b'PE\0\0';struct.pack_into('<H',b,0x84,0x14c)
        struct.pack_into('<H',b,0x98,0x10b);struct.pack_into('<I',b,0x98+96+10*8+4,size)
        b[-8:]=b'CANARY!!';result[size]=bytes(b)
    return result


class LoadConfigGeometryTests(unittest.TestCase):
    def test_only_directory_length_changes_and_bases_remain_untouched(self):
        original=bases();before=copy.deepcopy(original)
        for row,data in variants(original):
            expected=bytearray(original[row['declared_size']])
            struct.pack_into('<I',expected,0x98+96+10*8+4,row['directory_size'])
            self.assertEqual(data,expected)
        self.assertEqual(original,before)
        for at,value in [(0,0),(0x84,0x8664),(0x98,0x20b)]:
            bad=copy.deepcopy(original);b=bytearray(bad[64]);struct.pack_into('<H',b,at,value);bad[64]=bytes(b)
            with self.assertRaises(ValueError):variants(bad)

    def test_frozen_base_source_and_exact_child_bytes_are_required(self):
        with tempfile.TemporaryDirectory() as temp:
            directory=Path(temp);original=bases();items=variants(original)
            sources={s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}
            reference={'sources':{k:v for k,v in sources.items() if k.startswith('pe/')},
                       'cases':[dict(case=f'legacy{size}_default',pe_sha256=hashlib.sha256(data).hexdigest()) for size,data in original.items()]}
            path=directory/'native.json';path.write_text(json.dumps(reference))
            for size,data in original.items():(directory/f'base{size}.exe').write_bytes(data)
            for row,data in items:(directory/(row['case']+'.exe')).write_bytes(data)
            manifest=dict(schema_version=1,sources=sources,cases=[row for row,_ in items]);mp=directory/'load-config-geometry-fixture.json';mp.write_text(json.dumps(manifest))
            with mock.patch('run_pe32_load_config_geometry.BASE_REFERENCE',path):
                self.assertEqual(len(verify(directory)[1]),4)
                bad=bytearray(items[2][1]);bad[-1]^=1;(directory/'dir64_size72.exe').write_bytes(bad)
                with self.assertRaises(ValueError):verify(directory)
                manifest['cases'][2]['pe_sha256']=hashlib.sha256(bad).hexdigest();mp.write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):verify(directory)
                (directory/'base64.exe').write_bytes(b'changed')
                with self.assertRaises(ValueError):verify(directory)


if __name__=='__main__':unittest.main()
