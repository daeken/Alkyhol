#!/usr/bin/env python3
"""Build one no-CRT PE64 with real ID3DBlob virtual calls, using open LLVM."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool

SOURCES = ['pe/fixtures/binary-com/contract.c','pe/fixtures/binary-com/kernel32.def','pe/fixtures/binary-com/d3dcompiler.def']


def build(output):
    output=output.resolve()
    output.mkdir(parents=True,exist_ok=True)
    clang=shutil.which('clang')
    if not clang: raise RuntimeError('open Clang is required')
    obj,exe=output/'binary-com.obj',output/'binary-com.exe'
    subprocess.run([clang,'--target=x86_64-pc-windows-msvc','-O2','-ffreestanding','-fno-builtin',
                    '-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables',
                    '-Wall','-Wextra','-Werror','-c',str(ROOT/SOURCES[0]),'-o',str(obj)],check=True)
    lld=rust_tool('rust-lld')
    libs=[]
    for source,name in [(SOURCES[1],'kernel32.lib'),(SOURCES[2],'d3dcompiler.lib')]:
        lib=output/name
        subprocess.run([lld,'-flavor','link','/lib','/machine:x64','/def:'+str(ROOT/source),'/out:'+str(lib)],check=True)
        libs.append(str(lib))
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/entry:entry','/subsystem:console',
                    '/machine:x64','/dynamicbase','/fixed:no','/timestamp:0','/out:'+str(exe),str(obj),*libs],check=True)
    manifest={'schema_version':1,'pe_sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),
              'sources':{s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
    (output/'binary-com-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built actual Win64 virtual COM caller: '+str(exe))


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output',type=Path,default=ROOT/'local/pe/binary-com')
    build(p.parse_args().output)
