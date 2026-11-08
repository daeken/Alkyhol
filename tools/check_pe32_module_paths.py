#!/usr/bin/env python3
"""Test the module-origin/path blocker actually reached by DAO's original CRT."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT, rust_tool
from check_pe32_activation import SOURCE, digest

SOURCES = ['module-path.c', 'module-path.def', 'module-path-dll.c', 'module-path-dll.def']
FILES = [f'module-path{i}.exe' for i in range(5)]+['module-path.dll']
SYMBOLS = [('GetStdHandle', 4), ('WriteFile', 20), ('SetLastError', 4), ('GetLastError', 0),
           ('GetModuleHandleW', 4), ('GetModuleFileNameW', 12), ('GetLongPathNameW', 12)]
BOOT = struct.pack('<I', 0x4d503332)


def expected_words():
    seed = 0xface1234
    row = [1,0,1,1,1, 1,0,1, 1,122,1,1, 5,122,1,1, 1,122,1, 0,122,1]
    row += [1,seed,1]*4
    words = [0x4d503332,4,1]+row*3
    assert len(words) == 105
    return words


def decode(data, expected_paths):
    if len(expected_paths) != 3: raise ValueError('all three module path observations are required')
    if len(data) < 420 or list(struct.unpack('<105I', data[:420])) != expected_words():
        raise ValueError('module path ABI/truncation/last-error/guard observations differ')
    remaining = data[420:]
    paths = []
    for expected in expected_paths:
        if len(remaining) < 4: raise ValueError('missing module path record')
        length = struct.unpack_from('<I', remaining)[0]
        remaining = remaining[4:]
        if not 0 < length < 2048 or len(remaining) < (length+1)*2:
            raise ValueError('invalid module path width')
        text = remaining[:(length+1)*2].decode('utf-16-le')
        remaining = remaining[(length+1)*2:]
        if text != expected+'\0':
            raise ValueError(f'module path does not identify its independently known source: {text!r}, {expected!r}')
        paths.append(text[:-1])
    if remaining: raise ValueError('unexpected module path output tail')
    return paths


def identities(directory):
    return {'sources': {(SOURCE/n).relative_to(ROOT).as_posix(): digest(SOURCE/n) for n in SOURCES},
            'files': {n: digest(directory/n) for n in FILES}}


def frozen():
    raw = (ROOT/'pe/reference/pe32-module-path-windows-v1.json').read_bytes()
    if hashlib.sha256(raw).hexdigest() != '9a4ac6ec8aa07c071e3b43739cc9faab01a2eb246081463b132c24f61395d26e':
        raise ValueError('changed frozen native module/path report')
    value = json.loads(raw)
    if value['backend'] != 'native Windows PE32 module/path microtest' or value['status'] or value['stderr_hex']:
        raise ValueError('invalid frozen native module/path evidence')
    decode(bytes.fromhex(value['stdout_hex']),value['paths'])
    return value


def verify(directory):
    value = json.loads((directory/'module-path-inputs.json').read_text())
    if value != identities(directory): raise ValueError('module path source/PE/DLL identities changed')
    if value != frozen()['inputs']: raise ValueError('changed frozen native module/path source/PE/DLL identities')
    return value


def setup_run(directory):
    target = directory/'module paths λ🌸'
    target.mkdir(exist_ok=True)
    for name in FILES: shutil.copyfile(directory/name, target/name)
    return target


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
    for kind in ['module-path', 'module-path-dll']:
        subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x86', '/def:'+str(SOURCE/(kind+'.def')),
                        '/out:'+str(directory/(kind+'.lib'))], check=True)
    common = [str(directory/'module-path.lib'), *[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n, size in SYMBOLS]]
    obj = directory/'module-path-dll.obj'
    subprocess.run([*compile_args, '-c', str(SOURCE/'module-path-dll.c'), '-o', str(obj)], check=True)
    subprocess.run([*link, '/dll', '/entry:DllMain@12', str(obj), *common,
                    '/export:SourceModule=_SourceModule@0', '/implib:'+str(directory/'module-path-generated.lib'),
                    '/out:'+str(directory/'module-path.dll')], check=True)
    for mode in range(5):
        obj = directory/f'module-path{mode}.obj'
        subprocess.run([*compile_args, f'-DMODE={mode}', '-c', str(SOURCE/'module-path.c'), '-o', str(obj)], check=True)
        subprocess.run([*link, '/entry:entry', str(obj), *common, str(directory/'module-path-dll.lib'),
                        '/alternatename:__imp__SourceModule@0=__imp__SourceModule',
                        '/out:'+str(directory/f'module-path{mode}.exe')], check=True)
    (directory/'module-path-inputs.json').write_text(json.dumps(identities(directory), indent=2)+'\n')
    cargo = shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    rustc = shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
    target = 'x86_64-apple-darwin'
    subprocess.run([cargo, 'build', '--manifest-path', str(ROOT/'pe/Cargo.toml'), '--target', target, '--locked', '--offline'], check=True)
    build = ROOT/'pe/target'/target/'debug'
    probe = directory/'module-path-macos'
    subprocess.run([rustc, '--edition=2024', '--target', target, str(ROOT/'pe/probe_startup32.rs'),
                    '--extern', 'alkyhol_pe='+str(build/'libalkyhol_pe.rlib'), '-L', 'dependency='+str(build/'deps'),
                    '-C', 'link-arg=-Wl,-pagezero_size,0x1000', '-o', str(probe)], check=True)
    run = setup_run(directory)
    observations = []
    for mode, rebase in [(0,False),(0,True),(1,False),(2,False),(3,False),(4,False)]:
        exe = run/f'module-path{mode}.exe'; dll = run/'module-path.dll'
        command = [str(probe), str(exe), '--dll', 'module-path.dll='+str(dll)]
        if mode != 4: command.append('--file-origins')
        if mode == 0: command.append('--strict')
        if rebase: command.append('--rebase')
        result = subprocess.run(command, capture_output=True, timeout=15)
        row = {'case': mode, 'rebase': rebase, 'status': result.returncode,
               'stdout_hex': result.stdout.hex(), 'stderr_hex': result.stderr.hex()}
        if mode == 0:
            if result.returncode: raise ValueError('host file-backed module path execution failed: '+repr(row))
            expected_paths = ['Z:'+str(p.resolve()).replace('/', '\\') for p in [exe,exe,dll]]
            row['paths'] = decode(result.stdout, expected_paths)
        else:
            reasons = {1:b'module filename output outside checked writable',
                       2:b'module filename requires a known file-backed module origin',
                       3:b'long-path lookup requires an exact registered module source',
                       4:b'module filename requires a known file-backed module origin'}
            if result.returncode != 78 or result.stdout != (b'' if mode == 4 else BOOT) or reasons[mode] not in result.stderr:
                raise ValueError('module path restriction stopped for the wrong reason: '+repr(row))
        observations.append(row)
    (directory/'module-path-host.json').write_text(json.dumps({'backend':'Rosetta PE32 file-backed module/path profile',
        'inputs':verify(directory),'observations':observations}, indent=2)+'\n')
    print('File-backed EXE/DLL paths, initializer lookup, UTF-16, truncation, sizing and rebase pass; four restrictions stop')


def native(directory):
    inputs = verify(directory)
    run = setup_run(directory);exe = run/'module-path0.exe';dll = run/'module-path.dll'
    result = subprocess.run([str(exe)], capture_output=True, timeout=15)
    report = {'backend':'native Windows PE32 module/path microtest','inputs':inputs,
              'status':result.returncode,'stdout_hex':result.stdout.hex(),'stderr_hex':result.stderr.hex()}
    path = directory/'native-module-path.json';path.write_text(json.dumps(report,indent=2)+'\n')
    if result.returncode or result.stderr: raise ValueError('native module path execution failed')
    report['paths'] = decode(result.stdout, [str(p.resolve()) for p in [exe,exe,dll]])
    path.write_text(json.dumps(report,indent=2)+'\n')
    print('All 105 normalized words agree; each complete path independently matches its actual source on this host')


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests')
    parser.add_argument('--native',action='store_true');args=parser.parse_args()
    (native if args.native else host)(args.output.resolve())
