#!/usr/bin/env python3
"""Build a mixed named/ordinal consumer against the unchanged frozen DLL."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool
from build_pe_export_fixture import build as build_dependency
from pe_imports import PE

SOURCES = ['pe/fixtures/ordinal/library.c', 'pe/fixtures/ordinal/library.def', 'pe/fixtures/ordinal/imports.def',
           'pe/fixtures/exports/library.c', 'pe/fixtures/exports/library.def']
IMPORTS = [{'module': 'alkyhol_exports.dll', 'symbols': [{'ordinal': 21}, {'name': 'Direct'}, {'ordinal': 23}]}]


def build(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    dependency = build_dependency(output)
    reference = json.loads((ROOT / 'pe/reference/exports-windows-v1.json').read_text(encoding='utf-8'))
    if hashlib.sha256(dependency.read_bytes()).hexdigest() != reference['dll_sha256']:
        raise ValueError('ordinal probe requires the unchanged frozen dependency DLL')
    clang = shutil.which('clang')
    if not clang:
        raise RuntimeError('open Clang is required')
    obj, lib, dll = (output / n for n in ['ordinal.obj', 'ordinal-imports.lib', 'alkyhol_ordinal.dll'])
    subprocess.run([clang, '--target=x86_64-pc-windows-msvc', '-O2', '-ffreestanding', '-fno-builtin',
                    '-fno-stack-protector', '-fno-unwind-tables', '-fno-asynchronous-unwind-tables',
                    '-Wall', '-Wextra', '-Werror', '-c', str(ROOT / SOURCES[0]), '-o', str(obj)], check=True)
    lld = rust_tool('rust-lld')
    subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x64', '/def:' + str(ROOT / SOURCES[2]), '/out:' + str(lib)], check=True)
    subprocess.run([lld, '-flavor', 'link', '/dll', '/nodefaultlib', '/entry:DllMain', '/subsystem:console',
                    '/machine:x64', '/dynamicbase', '/fixed:no', '/timestamp:0', '/out:' + str(dll),
                    '/def:' + str(ROOT / SOURCES[1]), str(obj), str(lib)], check=True)
    if PE(dll.read_bytes()).imports() != IMPORTS:
        raise ValueError('LLVM did not produce the intended mixed code/data ordinal thunks')
    manifest = {'schema_version': 1, 'imports': IMPORTS,
                'binaries': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [dependency, dll]},
                'sources': {s: hashlib.sha256((ROOT / s).read_bytes()).hexdigest() for s in SOURCES}}
    (output / 'ordinal-fixture.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print('Built actual mixed ordinal PE64 consumer: ' + str(dll))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, default=ROOT / 'local/pe/ordinal')
    build(p.parse_args().output)
