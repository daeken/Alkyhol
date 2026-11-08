#!/usr/bin/env python3
"""Derive owned SizeOfImage observers from the unchanged frozen PE32 fixture.

Only that fixture's four SizeOfImage bytes may differ. No game input is accepted.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
from build_pe_fixture import ROOT
from build_pe32_fixture import SOURCES as BASE_SOURCES

SOURCES=[*BASE_SOURCES,'tools/build_pe32_geometry.py']
CASES=['aligned','last_byte','virtual_end','initialized_end','one_byte_final_page',
       'final_page_start','before_final_page','zero']
BASE_REFERENCE=ROOT/'pe/reference/pe32-windows-v1.json'


def variants(raw):
    """Return bounded header variants; input is additionally identity-checked by build."""
    try:
        if raw[:2]!=b'MZ':raise ValueError('missing DOS signature')
        pe=struct.unpack_from('<I',raw,0x3c)[0];opt=pe+24
        if raw[pe:pe+4]!=b'PE\0\0' or struct.unpack_from('<H',raw,pe+4)[0]!=0x14c or struct.unpack_from('<H',raw,opt)[0]!=0x10b:
            raise ValueError('expected i386 PE32 input')
        count=struct.unpack_from('<H',raw,pe+6)[0];size_opt=struct.unpack_from('<H',raw,pe+20)[0]
        alignment=struct.unpack_from('<I',raw,opt+32)[0]
        original=struct.unpack_from('<I',raw,opt+56)[0]
        if alignment!=4096 or original%alignment or not 1<=count<=96 or size_opt<224:
            raise ValueError('unexpected owned fixture geometry')
        sections=[struct.unpack_from('<III',raw,opt+size_opt+i*40+8) for i in range(count)]
    except struct.error as error:raise ValueError('truncated PE32 headers') from error
    virtual=max(rva+length for length,rva,_ in sections)
    initialized=max(rva+length for _,rva,length in sections)
    final_page=max(rva for _,rva,_ in sections)
    values=[original,original-1,virtual,initialized,final_page+1,final_page,final_page-1,0]
    if len(set(values))!=len(values) or max(virtual,initialized)>original or final_page==0:
        raise ValueError('fixture does not distinguish the requested geometries')
    result=[]
    for case,value in zip(CASES,values):
        changed=bytearray(raw);struct.pack_into('<I',changed,opt+56,value)
        result.append(({'case':case,'declared_size':value,'pe_sha256':hashlib.sha256(changed).hexdigest()},bytes(changed)))
    return opt+56,result


def build(base,output):
    raw=base.read_bytes();reference=json.loads(BASE_REFERENCE.read_text(encoding='utf-8'))
    if hashlib.sha256(raw).hexdigest()!=reference['pe_sha256']:raise ValueError('only the frozen owned PE32 is accepted')
    for name in BASE_SOURCES:
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=reference['sources'][name]:raise ValueError('changed original PE32 source')
    offset,items=variants(raw)
    output.mkdir(parents=True,exist_ok=True)
    (output/'base.exe').write_bytes(raw)
    for row,data in items:(output/(row['case']+'.exe')).write_bytes(data)
    manifest={'schema_version':1,'base_pe_sha256':reference['pe_sha256'],'size_offset':offset,
              'sources':{name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in SOURCES},
              'cases':[row for row,_ in items]}
    (output/'geometry-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built eight owned PE32 SizeOfImage variants; original input preserved')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--base',type=Path,default=ROOT/'local/pe/pe32/pe32.exe')
    p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-geometry')
    args=p.parse_args();build(args.base,args.output)
