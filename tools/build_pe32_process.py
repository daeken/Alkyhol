#!/usr/bin/env python3
"""Build a PE32 EXE graph around two unchanged frozen owned DLLs."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT,rust_tool
from build_pe32_linked import build as build_linked,SOURCES as LINKED_SOURCES
from run_pe32_linked import verify as verify_linked
from pe_imports import PE

SOURCES=LINKED_SOURCES+['pe/fixtures/pe32-process/'+n for n in ['program.c','terminate.c','failed.c','kernel32.def','terminate.def']]
FILES=['accepted.dll','consumer.dll','terminate.dll','program.exe','failed.dll']
API_BYTES={'GetModuleHandleA':4,'GetModuleHandleW':4,'GetProcAddress':8,'GetStdHandle':4,'WriteFile':20,'SetLastError':4,'GetLastError':0,'ExitProcess':4,'Finish':4}

def build(output):
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    legacy=output/'legacy';build_linked(legacy);verify_linked(legacy)
    for name in FILES[:2]:shutil.copyfile(legacy/name,output/name)
    clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang is required')
    lld=rust_tool('rust-lld');kernel=output/'kernel32.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(ROOT/'pe/fixtures/pe32-process/kernel32.def'),'/out:'+str(kernel)],check=True)
    common=[lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no',str(kernel),
            *[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in API_BYTES.items()]]
    for name in ['terminate','program','failed']:
        obj=output/(name+'.obj')
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O2','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(ROOT/'pe/fixtures/pe32-process'/(name+'.c')),'-o',str(obj)],check=True)
        extra=['/dll','/noentry','/def:'+str(ROOT/'pe/fixtures/pe32-process/terminate.def')] if name=='terminate' else ['/dll','/entry:DllMain@12',str(legacy/'accepted.lib')] if name=='failed' else ['/entry:entry',str(legacy/'accepted.lib'),str(legacy/'consumer.lib'),str(output/'terminate.lib')]
        subprocess.run([*common,str(obj),*extra,'/out:'+str(output/(name+('.exe' if name=='program' else '.dll')))],check=True)
    imports=PE((output/'program.exe').read_bytes()).imports()
    if {v['module'].lower() for v in imports}!={'accepted.dll','consumer.dll','terminate.dll','kernel32.dll'}:raise ValueError('missing actual process imports')
    manifest={'schema_version':1,'files':{n:hashlib.sha256((output/n).read_bytes()).hexdigest() for n in FILES},'sources':{n:hashlib.sha256((ROOT/n).read_bytes()).hexdigest() for n in SOURCES}}
    (output/'pe32-process-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built real PE32 static EXE/DLL graph around unchanged frozen dependencies')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-process');build(p.parse_args().output)
