#!/usr/bin/env python3
"""Build two owned DLLs observing static metadata through normal/failed lifecycle."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool
SOURCES = ['pe/fixtures/registered-dll/library.c', 'pe/fixtures/registered-dll/kernel32.def',
           'pe/fixtures/registered-dll/frames.S', 'pe/fixtures/registered-dll/exports.def']


def build(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    clang = shutil.which('clang')
    if not clang:
        raise RuntimeError('open Clang is required')
    lld = rust_tool('rust-lld')
    lib,assembly=output/'kernel32.lib',output/'frames.obj'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x64','/def:'+str(ROOT/SOURCES[1]),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=x86_64-pc-windows-msvc','-c',str(ROOT/SOURCES[2]),'-o',str(assembly)],check=True)
    binaries={}
    for case,reject in [('accepted',0),('rejected',1)]:
        obj,dll=output/(case+'.obj'),output/(case+'.dll')
        subprocess.run([clang,'--target=x86_64-pc-windows-msvc','-O2','-ffreestanding','-fno-builtin',
                        '-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables',
                        '-Wall','-Wextra','-Werror','-DFAIL_ATTACH='+str(reject),'-c',str(ROOT/SOURCES[0]),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/dll','/nodefaultlib','/entry:DllMain','/subsystem:console',
                        '/machine:x64','/dynamicbase','/fixed:no','/timestamp:0','/def:'+str(ROOT/SOURCES[3]),
                        '/out:'+str(dll),str(obj),str(assembly),str(lib)],check=True)
        binaries[case]=hashlib.sha256(dll.read_bytes()).hexdigest()
    manifest={'schema_version':1,'binaries':binaries,
              'sources':{s:hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
    (output/'registered-dll-fixture.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Built two Windows static-table lifecycle DLLs: '+str(output))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, default=ROOT / 'local/pe/registered-dll')
    build(p.parse_args().output)
