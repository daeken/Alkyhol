#!/usr/bin/env python3
"""Build an original no-CRT i386 compiler static TLS observer."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool

SOURCES=['pe/fixtures/pe32-tls/contract.c','pe/fixtures/pe32-tls/kernel32.def']
API_BYTES={'GetLastError':0,'SetLastError':4,'GetStdHandle':4,'WriteFile':20}


def build(output):
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang is required')
    obj,exe,lib=output/'contract.obj',output/'pe32-tls.exe',output/'kernel32.lib'
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O2','-ffreestanding',
                    '-fno-builtin','-fno-stack-protector','-fno-unwind-tables',
                    '-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',
                    '-c',str(ROOT/SOURCES[0]),'-o',str(obj)],check=True)
    lld=rust_tool('rust-lld')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(ROOT/SOURCES[1]),'/out:'+str(lib)],check=True)
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/entry:entry','/subsystem:console',
                    '/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no',
                    '/include:__tls_used','/out:'+str(exe),str(obj),str(lib),
                    *[f'/alternatename:__imp__{name}@{size}=__imp__{name}' for name,size in API_BYTES.items()]],check=True)
    manifest={'schema_version':1,'pe_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),
              'sources':{s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
    (output/'pe32-tls-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built owned PE32 compiler TLS observer: '+str(exe))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/pe32-tls')
    build(p.parse_args().output)
