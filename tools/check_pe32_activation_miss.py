#!/usr/bin/env python3
"""Exercise the actual activation lookup bridge and fail-closed context states."""
import argparse
import json
from pathlib import Path
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT, rust_tool
from check_pe32_activation import SOURCE, digest, resource_record

SOURCES = ['activation-miss.c', 'activation-miss.def', 'activation-dependency.c',
           'activation-dependency.def', 'activation-empty.manifest', 'activation-file.manifest']
FILES = [f'activation-miss{i}.exe' for i in range(6)]+['owned-context.dll']
SYMBOLS = [('GetStdHandle', 4), ('WriteFile', 20), ('GetLastError', 0), ('SetLastError', 4),
           ('GetModuleHandleW', 4), ('GetProcAddress', 8), ('FindActCtxSectionStringW', 20)]
BOOT = struct.pack('<I', 0x414d3332)


def expected():
    return BOOT+b''.join(struct.pack('<8I', size, flags, 0, 14007, 1, 1, 4, 0x715e7a11)
                         for flags in [0, 1] for size in [40, 64])


def identities(directory):
    return {'sources': {(SOURCE/n).relative_to(ROOT).as_posix(): digest(SOURCE/n) for n in SOURCES},
            'files': {n: digest(directory/n) for n in FILES}}


def verify(directory):
    value = json.loads((directory/'activation-miss-inputs.json').read_text())
    if value != identities(directory):
        raise ValueError('changed activation bridge source/PE/DLL inputs')
    return value


def host(directory):
    directory.mkdir(parents=True, exist_ok=True)
    clang = shutil.which('clang')
    if not clang: raise RuntimeError('open Clang is required')
    lld = rust_tool('rust-lld')
    compile_args = [clang, '--target=i686-pc-windows-msvc', '-O1', '-ffreestanding', '-fno-builtin',
                    '-fno-stack-protector', '-fno-unwind-tables', '-fno-asynchronous-unwind-tables',
                    '-Wall', '-Wextra', '-Werror']
    link = [lld, '-flavor', 'link', '/nodefaultlib', '/subsystem:console', '/machine:x86',
            '/dynamicbase', '/fixed:no', '/timestamp:0', '/safeseh:no', '/manifest:no']
    for kind in ['activation-miss', 'activation-dependency']:
        subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x86',
                        '/def:'+str(SOURCE/(kind+'.def')), '/out:'+str(directory/(kind+'.lib'))], check=True)
    null = resource_record(0, 0, 0, b'')
    empty = (SOURCE/'activation-empty.manifest').read_bytes()
    nonempty = (SOURCE/'activation-file.manifest').read_bytes()
    for name, resource_id, data in [('empty', 1, empty), ('nonempty', 1, nonempty), ('dll', 2, nonempty)]:
        (directory/(name+'.res')).write_bytes(null+resource_record(24, resource_id, 1033, data))
    obj = directory/'activation-dependency.obj'
    subprocess.run([*compile_args, '-c', str(SOURCE/'activation-dependency.c'), '-o', str(obj)], check=True)
    subprocess.run([*link, '/dll', '/noentry', str(obj), str(directory/'dll.res'),
                    '/export:OwnedDependency=_OwnedDependency@0', '/out:'+str(directory/'owned-context.dll')], check=True)
    for mode in range(6):
        obj = directory/f'activation-miss{mode}.obj'
        subprocess.run([*compile_args, f'-DMODE={mode}', '-c', str(SOURCE/'activation-miss.c'), '-o', str(obj)], check=True)
        subprocess.run([*link, '/entry:entry', str(obj),
                        *([str(directory/('nonempty.res' if mode == 1 else 'empty.res'))] if mode != 5 else []),
                        str(directory/'activation-miss.lib'),
                        *([str(directory/'activation-dependency.lib'), '/alternatename:__imp__OwnedDependency@0=__imp__OwnedDependency'] if mode == 4 else []),
                        *[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n, size in SYMBOLS],
                        '/out:'+str(directory/f'activation-miss{mode}.exe')], check=True)
    (directory/'activation-miss-inputs.json').write_text(json.dumps(identities(directory), indent=2)+'\n')
    cargo = shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    rustc = shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
    target = 'x86_64-apple-darwin'
    subprocess.run([cargo, 'build', '--manifest-path', str(ROOT/'pe/Cargo.toml'), '--target', target, '--locked', '--offline'], check=True)
    build = ROOT/'pe/target'/target/'debug'
    probe = directory/'activation-miss-macos'
    subprocess.run([rustc, '--edition=2024', '--target', target, str(ROOT/'pe/probe_startup32.rs'),
                    '--extern', 'alkyhol_pe='+str(build/'libalkyhol_pe.rlib'), '-L', 'dependency='+str(build/'deps'),
                    '-C', 'link-arg=-Wl,-pagezero_size,0x1000', '-o', str(probe)], check=True)
    records = []
    for mode in range(6):
        dlls = ['--dll', 'owned-context.dll='+str(directory/'owned-context.dll')] if mode == 4 else []
        result = subprocess.run([str(probe), str(directory/f'activation-miss{mode}.exe'), *dlls], capture_output=True, timeout=15)
        if result.returncode != (78 if mode else 0) or result.stdout != (BOOT if mode else expected()):
            raise ValueError(f'activation bridge case {mode} failed: {result.returncode}, {result.stdout.hex()}, {result.stderr!r}')
        if mode and b'unsupported' not in result.stderr:
            raise ValueError('unsupported activation context/input did not fail explicitly')
        records.append({'case': mode, 'status': result.returncode, 'stdout_hex': result.stdout.hex(), 'stderr_hex': result.stderr.hex()})
    (directory/'activation-miss-host.json').write_text(json.dumps({'backend': 'Rosetta PE32 activation bridge',
        'inputs': verify(directory), 'observations': records}, indent=2)+'\n')
    print('Empty-context activation lookups pass; nonempty EXE/DLL and unsupported outputs/flags stop explicitly')


def native(directory):
    inputs = verify(directory)
    result = subprocess.run([str((directory/'activation-miss0.exe').resolve())], capture_output=True, timeout=15)
    report = {'backend': 'native Windows PE32 activation bridge microtest', 'inputs': inputs,
              'status': result.returncode, 'stdout_hex': result.stdout.hex(), 'stderr_hex': result.stderr.hex()}
    (directory/'native-activation-miss.json').write_text(json.dumps(report, indent=2)+'\n')
    if result.returncode or result.stdout != expected() or result.stderr:
        raise ValueError('native empty-context activation results differ')
    print('All 33 DWORDs agree with actual host lookup/ABI/output preservation; host restrictions are separate')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT/'local/pe/startup-microtests')
    parser.add_argument('--native', action='store_true')
    args = parser.parse_args()
    (native if args.native else host)(args.output.resolve())
