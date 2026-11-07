#!/usr/bin/env python3
"""Build an owned PE client manually unwinding actual AMD64 assembly frames."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool

SOURCES = ['pe/fixtures/unwind-saves/contract.c', 'pe/fixtures/unwind-saves/kernel32.def',
           'pe/fixtures/unwind-saves/frames.S', 'pe/fixtures/unwind-saves/sdk-layout.cpp']


def build(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    clang = shutil.which('clang')
    if not clang: raise RuntimeError('open Clang is required')
    obj, exe, library = output/'unwind-saves.obj', output/'unwind-saves.exe', output/'kernel32.lib'
    subprocess.run([clang, '--target=x86_64-pc-windows-msvc', '-O2', '-ffreestanding',
                    '-fno-builtin', '-fno-stack-protector', '-fno-unwind-tables',
                    '-fno-asynchronous-unwind-tables', '-Wall', '-Wextra', '-Werror',
                    '-c', str(ROOT/SOURCES[0]), '-o', str(obj)], check=True)
    assembly = output/'frames.obj'
    subprocess.run([clang, '--target=x86_64-pc-windows-msvc', '-c', str(ROOT/SOURCES[2]), '-o', str(assembly)], check=True)
    lld = rust_tool('rust-lld')
    subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x64',
                    '/def:'+str(ROOT/SOURCES[1]), '/out:'+str(library)], check=True)
    subprocess.run([lld, '-flavor', 'link', '/nodefaultlib', '/entry:entry',
                    '/subsystem:console', '/machine:x64', '/dynamicbase', '/fixed:no',
                    '/stack:4194304', '/timestamp:0', '/out:'+str(exe), str(obj), str(assembly), str(library)], check=True)
    manifest = {'schema_version': 1, 'pe_sha256': hashlib.sha256(exe.read_bytes()).hexdigest(),
                'sources': {s: hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
    (output/'unwind-saves-fixture.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8')
    print('Built actual Windows manual frame-unwind client: '+str(exe))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, default=ROOT/'local/pe/unwind-saves')
    build(p.parse_args().output)
