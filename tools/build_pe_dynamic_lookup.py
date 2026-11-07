#!/usr/bin/env python3
"""Build an owned Windows EXE using dynamic exports of the frozen DLL."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool
from build_pe_export_fixture import build as build_dependency, SOURCES as DEPENDENCY_SOURCES
from pe_imports import PE

SOURCES = ['pe/fixtures/dynamic-lookup/contract.c', 'pe/fixtures/dynamic-lookup/kernel32.def',
           *DEPENDENCY_SOURCES]
DEPENDENCY_REFERENCE = ROOT/'pe/reference/exports-windows-v1.json'
BINARIES = {'dynamic-lookup.exe', 'alkyhol_exports.dll'}
IMPORTS = [
    {'module': 'KERNEL32.dll', 'symbols': [{'name': n} for n in
        ['ExitProcess', 'GetLastError', 'GetModuleHandleW', 'GetProcAddress', 'GetStdHandle', 'SetLastError', 'WriteFile']]},
    {'module': 'alkyhol_exports.dll', 'symbols': [{'name': 'Direct'}]}]


def build(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    dependency = build_dependency(output)
    frozen = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
    if hashlib.sha256(dependency.read_bytes()).hexdigest() != frozen['dll_sha256']:
        raise ValueError('dynamic lookup must retain the exact frozen dependency DLL')
    clang = shutil.which('clang')
    if not clang:
        raise RuntimeError('open Clang is required')
    obj, exe, library = output/'dynamic-lookup.obj', output/'dynamic-lookup.exe', output/'kernel32.lib'
    subprocess.run([clang, '--target=x86_64-pc-windows-msvc', '-O2', '-ffreestanding',
                    '-fno-builtin', '-fno-stack-protector', '-fno-unwind-tables',
                    '-fno-asynchronous-unwind-tables', '-Wall', '-Wextra', '-Werror',
                    '-c', str(ROOT/SOURCES[0]), '-o', str(obj)], check=True)
    lld = rust_tool('rust-lld')
    subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x64',
                    '/def:'+str(ROOT/SOURCES[1]), '/out:'+str(library)], check=True)
    subprocess.run([lld, '-flavor', 'link', '/nodefaultlib', '/entry:entry',
                    '/subsystem:console', '/machine:x64', '/dynamicbase', '/fixed:no',
                    '/timestamp:0', '/out:'+str(exe), str(obj), str(library),
                    str(output/'alkyhol_exports.lib')], check=True)
    if PE(exe.read_bytes()).imports() != IMPORTS:
        raise ValueError('LLVM did not produce the intended dynamic lookup import thunks')
    manifest = {'schema_version': 1, 'imports': IMPORTS,
                'binaries': {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in [exe, dependency]},
                'sources': {s: hashlib.sha256((ROOT/s).read_bytes()).hexdigest() for s in SOURCES}}
    (output/'dynamic-lookup-fixture.json').write_text(json.dumps(manifest, indent=2)+'\n', encoding='utf-8')
    print('Built actual guest module/export lookup client: '+str(exe))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, default=ROOT/'local/pe/dynamic-lookup')
    build(p.parse_args().output)
