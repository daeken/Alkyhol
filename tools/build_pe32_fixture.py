#!/usr/bin/env python3
"""Build an original no-CRT PE32 i386 binary/ABI observer."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool

SOURCES = ['pe/fixtures/pe32/contract.c', 'pe/fixtures/pe32/kernel32.def',
           'pe/fixtures/pe32/canary.S']


def build(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    clang = shutil.which('clang')
    if not clang: raise RuntimeError('open Clang is required')
    obj, exe, library = output/'pe32.obj', output/'pe32.exe', output/'kernel32.lib'
    subprocess.run([clang, '--target=i686-pc-windows-msvc', '-O2', '-ffreestanding',
                    '-fno-builtin', '-fno-stack-protector', '-fno-unwind-tables',
                    '-fno-asynchronous-unwind-tables', '-Wall', '-Wextra', '-Werror',
                    '-c', str(ROOT/SOURCES[0]), '-o', str(obj)], check=True)
    assembly = output/'canary.obj'
    subprocess.run([clang, '--target=i686-pc-windows-msvc', '-c', str(ROOT/SOURCES[2]), '-o', str(assembly)], check=True)
    lld = rust_tool('rust-lld')
    subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x86',
                    '/def:'+str(ROOT/SOURCES[1]), '/out:'+str(library)], check=True)
    subprocess.run([lld, '-flavor', 'link', '/nodefaultlib', '/entry:entry',
                    '/subsystem:console', '/machine:x86', '/dynamicbase', '/fixed:no',
                    '/timestamp:0', '/safeseh:no', '/out:'+str(exe), str(obj), str(assembly), str(library),
                    '/alternatename:__imp__GetLastError@0=__imp__GetLastError',
                    '/alternatename:__imp__SetLastError@4=__imp__SetLastError',
                    '/alternatename:__imp__GetStdHandle@4=__imp__GetStdHandle',
                    '/alternatename:__imp__WriteFile@20=__imp__WriteFile'], check=True)
    manifest = {'schema_version': 1, 'pe_sha256': hashlib.sha256(exe.read_bytes()).hexdigest(),
                'sources': {s: hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
    (output/'pe32-fixture.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8')
    print('Built actual Windows PE32 observer: '+str(exe))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, default=ROOT/'local/pe/pe32')
    build(p.parse_args().output)
