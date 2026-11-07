#!/usr/bin/env python3
"""Build an owned Windows EXE importing the unchanged frozen export DLL."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool
from build_pe_export_fixture import build as build_dependency, SOURCES as DEPENDENCY_SOURCES
from pe_imports import PE

SOURCES = ['pe/fixtures/linked-image/contract.c', 'pe/fixtures/linked-image/kernel32.def',
           'pe/fixtures/linked-image/imports.def', *DEPENDENCY_SOURCES]
DEPENDENCY_REFERENCE = ROOT/'pe/reference/exports-windows-v1.json'
BINARIES = {'linked-image.exe', 'alkyhol_exports.dll'}
IMPORTS = [
    {'module': 'KERNEL32.dll', 'symbols': [{'name': n} for n in
        ['ExitProcess', 'GetStdHandle', 'SetLastError', 'WriteFile']]},
    {'module': 'alkyhol_exports.dll', 'symbols': [{'name': 'Alias'}, {'name': 'Bss'},
        {'name': 'Data'}, {'ordinal': 21}, {'name': 'Direct'}, {'name': 'Forwarded'}, {'ordinal': 23}]}]


def build(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    dependency = build_dependency(output)
    frozen = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
    if hashlib.sha256(dependency.read_bytes()).hexdigest() != frozen['dll_sha256']:
        raise ValueError('linked EXE must retain the exact frozen dependency DLL')
    clang = shutil.which('clang')
    if not clang:
        raise RuntimeError('open Clang is required')
    obj, exe = output/'linked-image.obj', output/'linked-image.exe'
    subprocess.run([clang, '--target=x86_64-pc-windows-msvc', '-O2', '-ffreestanding',
                    '-fno-builtin', '-fno-stack-protector', '-fno-unwind-tables',
                    '-fno-asynchronous-unwind-tables', '-Wall', '-Wextra', '-Werror',
                    '-c', str(ROOT/SOURCES[0]), '-o', str(obj)], check=True)
    lld = rust_tool('rust-lld')
    libraries = []
    for source, name in [(SOURCES[1], 'kernel32.lib'), (SOURCES[2], 'imports.lib')]:
        library = output/name
        subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x64',
                        '/def:'+str(ROOT/source), '/out:'+str(library)], check=True)
        libraries.append(str(library))
    subprocess.run([lld, '-flavor', 'link', '/nodefaultlib', '/entry:entry',
                    '/subsystem:console', '/machine:x64', '/dynamicbase', '/fixed:no',
                    '/timestamp:0', '/out:'+str(exe), str(obj), *libraries], check=True)
    if PE(exe.read_bytes()).imports() != IMPORTS:
        raise ValueError('LLVM did not produce the intended named/ordinal EXE import thunks')
    manifest = {'schema_version': 1, 'imports': IMPORTS,
                'binaries': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [exe, dependency]},
                'sources': {s: hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
    (output/'linked-image-fixture.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8')
    print('Built actual EXE-to-DLL import client: '+str(exe))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, default=ROOT/'local/pe/linked-image')
    build(p.parse_args().output)
