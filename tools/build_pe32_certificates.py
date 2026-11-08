#!/usr/bin/env python3
"""Derive owned non-enforced certificate-directory probes; never accept game input."""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from build_pe_fixture import ROOT
from build_pe32_fixture import SOURCES as BASE_SOURCES

SOURCES=[*BASE_SOURCES,'tools/build_pe32_certificates.py']
CASES=['absent','empty_certificate','malformed_certificate','offset_past_eof',
       'length_past_eof','header_bytes','unaligned_range','overflowing_range']
BASE_REFERENCE=ROOT/'pe/reference/pe32-windows-v1.json'


def variants(raw):
    try:
        if raw[:2]!=b'MZ':raise ValueError('missing DOS signature')
        pe=struct.unpack_from('<I',raw,0x3c)[0];opt=pe+24
        if raw[pe:pe+4]!=b'PE\0\0' or struct.unpack_from('<H',raw,pe+4)[0]!=0x14c or struct.unpack_from('<H',raw,opt)[0]!=0x10b:
            raise ValueError('expected PE32 i386 input')
        if struct.unpack_from('<H',raw,opt+70)[0]&0x80:raise ValueError('code-integrity enforcement is outside this probe')
        if struct.unpack_from('<I',raw,opt+92)[0]<5:raise ValueError('missing certificate-directory slot')
        offset=opt+96+4*8
        if struct.unpack_from('<II',raw,offset)!=(0,0):raise ValueError('base certificate directory is not absent')
    except struct.error as error:raise ValueError('truncated PE32 headers') from error
    if len(raw)<16 or len(raw)%8:raise ValueError('owned base must have an eight-byte aligned EOF')
    definitions=[(0,0,b''),(len(raw),8,struct.pack('<IHH',8,0x200,2)),
                 (len(raw),8,b'notacert'),(len(raw)+4096,16,b''),
                 (len(raw)-8,16,b''),(8,16,b''),(len(raw)-7,7,b''),(0xfffffff8,16,b'')]
    items=[]
    for case,(first,length,append) in zip(CASES,definitions):
        data=bytearray(raw+append);struct.pack_into('<II',data,offset,first,length)
        items.append(({'case':case,'certificate_offset':first,'certificate_size':length,
                       'file_size':len(data),'pe_sha256':hashlib.sha256(data).hexdigest()},bytes(data)))
    return offset,items


def build(base,output):
    raw=base.read_bytes();reference=json.loads(BASE_REFERENCE.read_text(encoding='utf-8'))
    if hashlib.sha256(raw).hexdigest()!=reference['pe_sha256']:raise ValueError('only the unchanged frozen owned PE32 is accepted')
    for name in BASE_SOURCES:
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=reference['sources'][name]:raise ValueError('changed original PE32 source')
    offset,items=variants(raw);output.mkdir(parents=True,exist_ok=True)
    (output/'base.exe').write_bytes(raw)
    for row,data in items:(output/(row['case']+'.exe')).write_bytes(data)
    manifest={'schema_version':1,'base_pe_sha256':reference['pe_sha256'],'directory_offset':offset,
              'sources':{name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in SOURCES},
              'cases':[row for row,_ in items]}
    (output/'certificates-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built eight owned certificate-directory variants; no trust verification is implied')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--base',type=Path,default=ROOT/'local/pe/pe32/pe32.exe')
    p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-certificates');args=p.parse_args();build(args.base,args.output)
