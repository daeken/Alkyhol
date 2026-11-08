#!/usr/bin/env python3
"""Build an original PE32 with real SafeSEH handlers and UD2 continuation."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT,rust_tool

SOURCES=['pe/fixtures/pe32-seh/'+n for n in ['contract.c','frames.S','types.h','kernel32.def','sdk-layout.cpp','darwin-layout.c']]
CASES=[('pe32-seh',0),('imports',1),('context',2),('nested',3),('unregistered',4)]
def build(output):
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang required')
    lld=rust_tool('rust-lld');lib=output/'kernel32.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(ROOT/SOURCES[3]),'/out:'+str(lib)],check=True)
    cases=[]
    for name,mode in CASES:
      objects=[]
      for source in SOURCES[:2]:
        obj=output/(name+'-'+Path(source).stem+'.obj');objects.append(str(obj))
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O2','-ffreestanding','-fno-builtin','-fno-stack-protector',f'-DNEGATIVE={mode}',
                        '-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',
                        '-c',str(ROOT/source),'-o',str(obj)],check=True)
      exe=output/(name+'.exe')
      subprocess.run([lld,'-flavor','link','/nodefaultlib','/machine:x86','/entry:entry','/subsystem:console',
                    '/safeseh','/dynamicbase','/fixed:no','/timestamp:0','/include:__load_config_used',
                    '/out:'+str(exe),*objects,str(lib),
                    '/alternatename:__imp__GetStdHandle@4=__imp__GetStdHandle',
                    '/alternatename:__imp__GetLastError@0=__imp__GetLastError',
                    '/alternatename:__imp__SetLastError@4=__imp__SetLastError',
                    '/alternatename:__imp__WriteFile@20=__imp__WriteFile'],check=True)
      cases.append({'case':name,'mode':mode,'pe_sha256':hashlib.sha256(exe.read_bytes()).hexdigest()})
    manifest={'schema_version':1,'cases':cases,
              'sources':{s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
    (output/'pe32-seh-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built original PE32 SafeSEH UD2 search/continuation observer')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-seh');build(p.parse_args().output)
