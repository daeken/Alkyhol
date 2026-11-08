#!/usr/bin/env python3
"""Build a real PE32 consumer for the unchanged frozen owned DLL."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT,rust_tool
from build_pe32_library import build as build_leaf,SOURCES as LEAF_SOURCES,API_BYTES
from run_pe32_library import verify as verify_leaf,REFERENCE as LEAF_REFERENCE
from pe_imports import PE

SOURCES=LEAF_SOURCES+['pe/fixtures/pe32-linked/'+n for n in ['consumer.c','driver.c','imports.def']]
FILES=['accepted.dll','consumer.dll','driver.exe']


def build(output):
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    leaf=output/'leaf';build_leaf(leaf);verify_leaf(leaf)
    reference=json.loads(LEAF_REFERENCE.read_text(encoding='utf-8'))
    data=(leaf/'accepted.dll').read_bytes()
    assert hashlib.sha256(data).hexdigest()==reference['files']['accepted.dll']
    # Verify actual biased ordinals before using a NONAME import library.
    pe=PE(data);directory=pe.offset(pe.directories[0][0],40)
    base,count,names=pe.unpack('<III',directory+16);names_rva,ordinals_rva=pe.unpack('<II',directory+32)
    actual={pe.string(pe.unpack('<I',pe.offset(names_rva+i*4,4))[0]):base+pe.unpack('<H',pe.offset(ordinals_rva+i*2,2))[0] for i in range(names)}
    if actual!={'Data':1,'RunCdecl':2,'RunStdcall':3} or count!=3:raise ValueError('frozen DLL ordinals changed')
    (output/'accepted.dll').write_bytes(data)
    clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang is required')
    lld=rust_tool('rust-lld');lib=output/'accepted.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(ROOT/SOURCES[-1]),'/out:'+str(lib)],check=True)
    for name in ['consumer','driver']:
        obj=output/(name+'.obj');source=ROOT/'pe/fixtures/pe32-linked'/(name+'.c')
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O2','-ffreestanding','-fno-builtin',
                        '-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables',
                        '-Wall','-Wextra','-Werror','-c',str(source),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86',
                        '/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no',str(obj),str(leaf/'kernel32.lib'),
                        *([str(lib),'/dll','/entry:DllMain@12','/alternatename:__imp__RunStdcall@20=__imp__RunStdcall'] if name=='consumer' else ['/entry:entry']),
                        *[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in API_BYTES.items()],
                        '/out:'+str(output/(name+('.dll' if name=='consumer' else '.exe')))],check=True)
    imports=PE((output/'consumer.dll').read_bytes()).imports()
    deps=[i for i in imports if i['module'].lower()=='accepted.dll']
    if len(deps)!=1 or not any(s.get('ordinal')==2 for s in deps[0]['symbols']):raise ValueError('missing genuine ordinal code import')
    manifest={'schema_version':1,'files':{n:hashlib.sha256((output/n).read_bytes()).hexdigest() for n in FILES},
              'sources':{n:hashlib.sha256((ROOT/n).read_bytes()).hexdigest() for n in SOURCES}}
    (output/'pe32-linked-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built original PE32 consumer with named/ordinal code and data imports from frozen DLL')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-linked');build(p.parse_args().output)
