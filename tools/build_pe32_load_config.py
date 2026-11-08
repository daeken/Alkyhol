#!/usr/bin/env python3
"""Build owned PE32 legacy-load-config cases with real compiler GS epilogues."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT,rust_tool

SOURCES=['pe/fixtures/pe32-load-config/contract.c','pe/fixtures/pe32-load-config/kernel32.def',
         'pe/fixtures/pe32-load-config/sdk-layout.cpp']
CASES=[('absent',0,0xbb40e64e,True,False,False),
       ('no_cookie',72,0xbb40e64e,False,True,False),
       ('legacy64_default',64,0xbb40e64e,False,False,False),
       ('legacy72_default',72,0xbb40e64e,False,False,False),
       ('legacy64_custom',64,0x1234abcd,False,False,False),
       ('legacy72_zero',72,0,False,False,False),
       ('legacy72_small',72,0x1234,False,False,False),
       ('corrupt_frame',72,0xbb40e64e,False,False,True)]
API_BYTES={'GetLastError':0,'SetLastError':4,'GetStdHandle':4,'WriteFile':20}


def build(output):
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang is required')
    lld=rust_tool('rust-lld');lib=output/'kernel32.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(ROOT/SOURCES[1]),'/out:'+str(lib)],check=True)
    rows=[]
    for name,size,initial,absent,no_cookie,fault in CASES:
        obj,exe=output/(name+'.obj'),output/(name+'.exe')
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O2','-ffreestanding','-fno-builtin',
                        '-fstack-protector-strong','-fno-unwind-tables','-fno-asynchronous-unwind-tables',
                        '-Wall','-Wextra','-Werror',
                        f'-DCONFIG_SIZE={size}',f'-DCOOKIE_INITIAL={initial}u',f'-DABSENT={int(absent)}',
                        f'-DNO_COOKIE={int(no_cookie)}',f'-DFAULT={int(fault)}',
                        '-c',str(ROOT/SOURCES[0]),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/entry:entry','/subsystem:console','/machine:x86',
                        '/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no',
                        *([] if absent else ['/include:__load_config_used']),'/out:'+str(exe),str(obj),str(lib),
                        *[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in API_BYTES.items()]],check=True)
        rows.append({'case':name,'config_size':size,'initial_cookie':initial,'absent':absent,
                     'no_cookie':no_cookie,'fault':fault,'pe_sha256':hashlib.sha256(exe.read_bytes()).hexdigest()})
    manifest={'schema_version':1,'sources':{s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES},'cases':rows}
    (output/'load-config-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built eight owned PE32 load-config/compiler-GS cases')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-load-config');build(p.parse_args().output)
