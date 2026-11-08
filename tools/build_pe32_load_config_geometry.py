#!/usr/bin/env python3
"""Vary only directory lengths in frozen owned legacy load-config executables."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from build_pe_fixture import ROOT
from build_pe32_load_config import SOURCES as ORIGINAL_SOURCES
from run_pe32_load_config import verify as verify_original

SOURCES=[*ORIGINAL_SOURCES,'tools/build_pe32_load_config_geometry.py']
BASE_REFERENCE=ROOT/'pe/reference/pe32-load-config-windows-v1.json'
CASES=[('dir64_size64',64,64),('dir72_size72',72,72),('dir64_size72',64,72),('dir72_size64',72,64)]


def variants(bases):
    items=[]
    for name,directory_size,declared_size in CASES:
        raw=bases[declared_size];offset=struct.unpack_from('<I',raw,0x3c)[0]+24
        if raw[:2]!=b'MZ' or raw[offset-24:offset-20]!=b'PE\0\0' or struct.unpack_from('<H',raw,offset-20)[0]!=0x14c or struct.unpack_from('<H',raw,offset)[0]!=0x10b:
            raise ValueError('geometry base must be PE32')
        slot=offset+96+10*8+4
        if struct.unpack_from('<I',raw,slot)[0]!=declared_size:raise ValueError('changed canonical directory size')
        changed=bytearray(raw);struct.pack_into('<I',changed,slot,directory_size);data=bytes(changed)
        items.append((dict(case=name,directory_size=directory_size,declared_size=declared_size,
                           pe_sha256=hashlib.sha256(data).hexdigest()),data))
    return items


def build(base_directory,output):
    manifest,original=verify_original(base_directory.resolve())
    reference=json.loads(BASE_REFERENCE.read_text(encoding='utf-8'))
    for key in ['sources','cases']:
        if manifest[key]!=reference[key]:raise ValueError('changed frozen canonical load-config inputs')
    bases={size:next(data for row,data in original if row['case']==f'legacy{size}_default') for size in [64,72]}
    items=variants(bases);output.mkdir(parents=True,exist_ok=True)
    for size,data in bases.items():(output/f'base{size}.exe').write_bytes(data)
    for row,data in items:(output/(row['case']+'.exe')).write_bytes(data)
    result=dict(schema_version=1,sources={s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES},cases=[row for row,_ in items])
    (output/'load-config-geometry-fixture.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
    print('Built four owned load-config length pairs; only four header bytes vary')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--base-directory',type=Path,default=ROOT/'local/pe/pe32-load-config');p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-load-config-geometry');a=p.parse_args();build(a.base_directory,a.output)
