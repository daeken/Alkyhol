#!/usr/bin/env python3
"""Build a public no-CRT AMD64 DLL once; the identical DLL goes to native Windows."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool

SOURCES = ['pe/fixtures/exports/library.c', 'pe/fixtures/exports/library.def']


def build(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    clang = shutil.which('clang')
    if not clang:
        raise RuntimeError('open Clang is required')
    obj = output / 'library.obj'
    dll = output / 'alkyhol_exports.dll'
    subprocess.run([clang, '--target=x86_64-pc-windows-msvc', '-O2', '-ffreestanding',
                    '-fno-builtin', '-fno-stack-protector', '-fno-unwind-tables',
                    '-fno-asynchronous-unwind-tables', '-Wall', '-Wextra', '-Werror',
                    '-c', str(ROOT / SOURCES[0]), '-o', str(obj)], check=True)
    subprocess.run([rust_tool('rust-lld'), '-flavor', 'link', '/dll', '/nodefaultlib',
                    '/entry:DllMain', '/subsystem:console', '/machine:x64', '/dynamicbase',
                    '/fixed:no', '/timestamp:0', '/def:' + str(ROOT / SOURCES[1]),
                    '/out:' + str(dll), str(obj)], check=True)
    manifest = {'schema_version': 1, 'dll_sha256': hashlib.sha256(dll.read_bytes()).hexdigest(),
                'sources': {s: hashlib.sha256((ROOT / s).read_bytes()).hexdigest() for s in SOURCES}}
    (output / 'exports-fixture.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print(f'Built owned PE64 export fixture: {dll}')
    return dll


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, default=ROOT / 'local/pe/export-fixture')
    build(p.parse_args().output)
