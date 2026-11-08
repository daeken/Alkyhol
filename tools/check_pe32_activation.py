#!/usr/bin/env python3
"""Resource inspection and native activation observations for DAO's CRT blocker.

Only original owned resources/programs enter CI. Activation APIs execute only on
native Windows; the host currently implements resource metadata inspection.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT, rust_tool

SOURCE = ROOT / 'pe/fixtures/pe32-startup'
SOURCES = ['activation.c', 'activation.def', 'activation-empty.manifest',
           'activation-file.manifest', 'activation-layout.cpp']
SYMBOLS = [('GetStdHandle', 4), ('WriteFile', 20), ('GetLastError', 0),
           ('SetLastError', 4), ('GetModuleHandleW', 4), ('FindResourceExW', 16),
           ('LoadResource', 8), ('LockResource', 4), ('SizeofResource', 8),
           ('CreateActCtxW', 4), ('ActivateActCtx', 8), ('DeactivateActCtx', 8),
           ('ReleaseActCtx', 4), ('GetCurrentActCtx', 4),
           ('FindActCtxSectionStringW', 20)]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def resources():
    return [(24, 1, 1033, (SOURCE / 'activation-empty.manifest').read_bytes()),
            (24, 2, 1033, (SOURCE / 'activation-file.manifest').read_bytes()),
            (10, 'Λ🌸', 1041, b'\x00owned\xffresource\x00')]


def resource_record(kind, name, language, payload):
    def key(value):
        return struct.pack('<HH', 0xffff, value) if isinstance(value, int) else (value+'\0').encode('utf-16-le')
    header = key(kind)+key(name)
    header += b'\0' * (-len(header) % 4)
    header += struct.pack('<IHHII', 0, 0x30, language, 0, 0)
    record = struct.pack('<II', len(payload), len(header)+8)+header+payload
    return record+b'\0'*(-len(record) % 4)


def expected_resources():
    return b''.join(struct.pack('<I', len(data))+data for _, _, _, data in resources())


def frozen():
    raw = (ROOT/'pe/reference/pe32-activation-windows-v1.json').read_bytes()
    if hashlib.sha256(raw).hexdigest() != '8f965ed831084348e098cbbe73613c456c197249b4de9ee7746f8ed813e65bb1':
        raise ValueError('changed frozen native activation report')
    value = json.loads(raw)
    if value['backend'] != 'native Windows i386 activation-context observer' or value['status'] or value['stderr_hex']:
        raise ValueError('invalid frozen native activation evidence')
    return value


def compare_records(records):
    if records != frozen()['records']:
        raise ValueError('native activation observations differ from the frozen identical-PE capture')


def verify(directory):
    value = json.loads((directory/'activation-inputs.json').read_text())
    expected = {(SOURCE/n).relative_to(ROOT).as_posix(): digest(SOURCE/n) for n in SOURCES}
    if value['sources'] != expected or value['files'] != {'activation.exe': digest(directory/'activation.exe')}:
        raise ValueError('activation source/identical-PE identity changed')
    if value['resource_prefix_sha256'] != hashlib.sha256(expected_resources()).hexdigest():
        raise ValueError('activation resource contract changed')
    if value != frozen()['inputs']:
        raise ValueError('changed frozen native activation source/PE inputs')
    return value


def host(directory):
    directory.mkdir(parents=True, exist_ok=True)
    clang = shutil.which('clang')
    if not clang:
        raise RuntimeError('open Clang is required')
    lld = rust_tool('rust-lld')
    res = directory/'activation.res'
    res.write_bytes(resource_record(0, 0, 0, b'') +
                    b''.join(resource_record(*r) for r in resources()))
    lib = directory/'activation.lib'
    subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x86',
                    '/def:'+str(SOURCE/'activation.def'), '/out:'+str(lib)], check=True)
    obj = directory/'activation.obj'
    subprocess.run([clang, '--target=i686-pc-windows-msvc', '-O1', '-ffreestanding',
                    '-fno-builtin', '-fno-stack-protector', '-fno-unwind-tables',
                    '-fno-asynchronous-unwind-tables', '-Wall', '-Wextra', '-Werror',
                    '-c', str(SOURCE/'activation.c'), '-o', str(obj)], check=True)
    exe = directory/'activation.exe'
    subprocess.run([lld, '-flavor', 'link', '/nodefaultlib', '/subsystem:console',
                    '/machine:x86', '/dynamicbase', '/fixed:no', '/timestamp:0',
                    '/safeseh:no', '/manifest:no', '/entry:entry', str(obj), str(res), str(lib),
                    *[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n, size in SYMBOLS],
                    '/out:'+str(exe)], check=True)
    manifest = {'sources': {(SOURCE/n).relative_to(ROOT).as_posix(): digest(SOURCE/n) for n in SOURCES},
                'files': {'activation.exe': digest(exe)},
                'resource_prefix_sha256': hashlib.sha256(expected_resources()).hexdigest()}
    (directory/'activation-inputs.json').write_text(json.dumps(manifest, indent=2)+'\n')
    cargo = shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    rustc = shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
    target = 'x86_64-apple-darwin'
    subprocess.run([cargo, 'build', '--manifest-path', str(ROOT/'pe/Cargo.toml'),
                    '--target', target, '--locked', '--offline'], check=True)
    build = ROOT/'pe/target'/target/'debug'
    probe = directory/'resource-macos'
    subprocess.run([rustc, '--edition=2024', '--target', target, str(ROOT/'pe/probe_resources.rs'),
                    '--extern', 'alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),
                    '-L', 'dependency='+str(build/'deps'), '-o', str(probe)], check=True)
    result = subprocess.run([str(probe), str(exe)], capture_output=True, timeout=15)
    if result.returncode or result.stdout != expected_resources() or result.stderr:
        raise ValueError('resource parser disagrees with original linked payloads: '+repr(result.stderr))
    (directory/'activation-host.json').write_text(json.dumps({
        'backend': 'physical or hosted Mac read-only resource inspection; no PE execution',
        'inputs': verify(directory), 'resource_prefix_hex': result.stdout.hex()}, indent=2)+'\n')
    print('Exact owned manifest/Unicode-name/binary resources inspected; activation requires native observation')


def decode_native(data):
    prefix = expected_resources()
    if not data.startswith(prefix):
        raise ValueError('native Windows resource bytes differ from checked host inspection')
    remaining = data[len(prefix):]
    # Header, two empty-default queries, create/activate, current context,
    # five active queries, deactivate, and one post-deactivation query.
    lengths = [4, 20, 20, 4, 3, 20, 20, 20, 20, 20, 3, 20]
    if len(remaining) != sum(lengths)*4:
        raise ValueError('native activation record width changed')
    records = []
    for length in lengths:
        records.append(list(struct.unpack('<'+'I'*length, remaining[:length*4])))
        remaining = remaining[length*4:]
    if records[0] != [0x41433332, 32, 64, 4]:
        raise ValueError('native activation ABI header differs')
    if records[3][0:2] != [1, 1] or records[3][3] != 1 or records[4] != [2, 1, 1] or records[-2][0:2] != [3, 1]:
        raise ValueError('native activation create/current/deactivate lifecycle failed')
    # Observations, not guessed expected API outputs. The sentinel checks are
    # memory-safety assertions for our owned observer, not a native oracle.
    for record in [records[i] for i in [1, 2, 5, 6, 7, 8, 9, 11]]:
        if record[17] != 0x715e7a11:
            raise ValueError('native query overwrote keyed-data guard')
    return records


def native(directory):
    inputs = verify(directory)
    result = subprocess.run([str((directory/'activation.exe').resolve())], capture_output=True, timeout=20)
    report = {'backend': 'native Windows i386 activation-context observer', 'inputs': inputs,
              'status': result.returncode, 'stdout_hex': result.stdout.hex(), 'stderr_hex': result.stderr.hex()}
    # Preserve failed observations too. They cannot count as successful evidence.
    report_path = directory/'native-activation.json'
    report_path.write_text(json.dumps(report, indent=2)+'\n')
    if result.returncode or result.stderr:
        raise ValueError('native activation observer failed: '+repr(report))
    report['records'] = decode_native(result.stdout)
    report_path.write_text(json.dumps(report, indent=2)+'\n')
    compare_records(report['records'])
    print('Native resource bytes agree; captured empty/active/deactivated contexts and 40/64-byte query layouts')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT/'local/pe/startup-microtests')
    parser.add_argument('--native', action='store_true')
    args = parser.parse_args()
    (native if args.native else host)(args.output.resolve())
