#!/usr/bin/env python3
"""Build original no-CRT PE32 DLL lifecycle observers with open Clang/LLD."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT,rust_tool

SOURCES=['pe/fixtures/pe32-library/'+n for n in ['library.c','driver.c','exports.def','kernel32.def']]
API_BYTES={'GetLastError':0,'SetLastError':4,'GetStdHandle':4,'WriteFile':20,
           'LoadLibraryA':4,'FreeLibrary':4,'GetProcAddress':8}
FILES=['accepted.dll','rejected.dll','driver.exe']


def build(output):
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang is required')
    lld=rust_tool('rust-lld');lib=output/'kernel32.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(ROOT/SOURCES[3]),'/out:'+str(lib)],check=True)
    common=[lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86',
            '/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no',str(lib),
            *[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in API_BYTES.items()]]
    for name,reject in [('accepted',0),('rejected',1),('driver',None)]:
        obj=output/(name+'.obj');source=SOURCES[1] if reject is None else SOURCES[0]
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O2','-ffreestanding','-fno-builtin',
                        '-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables',
                        '-Wall','-Wextra','-Werror',*([f'-DREJECT={reject}'] if reject is not None else []),
                        '-c',str(ROOT/source),'-o',str(obj)],check=True)
        subprocess.run([*common,str(obj),*(['/entry:entry'] if reject is None else
                       ['/dll','/entry:DllMain@12','/def:'+str(ROOT/SOURCES[2])]),
                       '/out:'+str(output/(name+('.exe' if reject is None else '.dll')))],check=True)
    manifest={'schema_version':1,
              'files':{n:hashlib.sha256((output/n).read_bytes()).hexdigest() for n in FILES},
              'sources':{n:hashlib.sha256((ROOT/n).read_bytes()).hexdigest() for n in SOURCES}}
    (output/'pe32-library-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built owned PE32 accepted/rejected DLLs and native i386 observer')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-library');build(p.parse_args().output)
