#!/usr/bin/env python3
"""Execute the same owned GetNumberFormatW PE on Windows and Mac backends."""
import argparse
import hashlib
import json
import platform
from pathlib import Path
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT, rust_tool

SOURCE = ROOT / 'pe/fixtures/pe32-startup'
SYMBOLS = [('GetStdHandle', 4), ('WriteFile', 20), ('SetLastError', 4),
           ('GetLastError', 0), ('GetNumberFormatW', 24)]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(directory):
    directory.mkdir(parents=True, exist_ok=True)
    lld = rust_tool('rust-lld')
    library, obj, exe = [directory / ('number-format.' + ext) for ext in ['lib', 'obj', 'exe']]
    subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x86',
                    '/def:' + str(SOURCE / 'number-format.def'), '/out:' + str(library)], check=True)
    subprocess.run([shutil.which('clang') or 'clang', '--target=i686-pc-windows-msvc',
                    '-O1', '-ffreestanding', '-fno-builtin', '-fno-stack-protector',
                    '-fno-unwind-tables', '-fno-asynchronous-unwind-tables',
                    '-Wall', '-Wextra', '-Werror', '-c', str(SOURCE / 'number-format.c'),
                    '-o', str(obj)], check=True)
    subprocess.run([lld, '-flavor', 'link', '/nodefaultlib', '/subsystem:console',
                    '/machine:x86', '/dynamicbase', '/fixed:no', '/timestamp:0',
                    '/safeseh:no', '/manifest:no', '/entry:entry', str(obj), str(library),
                    *[f'/alternatename:__imp__{name}@{size}=__imp__{name}' for name, size in SYMBOLS],
                    '/out:' + str(exe)], check=True)
    identity = dict(files={exe.name: digest(exe)}, sources={name: digest(SOURCE / name)
                    for name in ['number-format.c', 'number-format.def']})
    (directory / 'inputs.json').write_text(json.dumps(identity, indent=2) + '\n')
    return exe


def decode(data):
    if len(data) < 16 or data[:8] != struct.pack('<2I', 0x4e463332, 0):
        raise ValueError('missing number-format observation header')
    magic, count = struct.unpack_from('<2I', data, len(data)-8)
    if magic != 0x454e4432 or len(data) != 16 + count * 272:
        raise ValueError('incomplete number-format observation')
    rows = []
    for i in range(count):
        values = struct.unpack_from('<3I130H', data, 8 + i*272)
        serial, result, error, before, *rest = values
        output, after = rest[:128], rest[128]
        if serial != i or before != 0x1234 or after != 0x5678:
            raise ValueError(f'row {i}: identity/output guards changed')
        rows.append(dict(count=result, error=error, output=output))
    return rows


def compare(actual, reference):
    if len(actual) != len(reference):
        raise ValueError('different number of native observations')
    differences = []
    for i, (a, b) in enumerate(zip(actual, reference)):
        # A successful call does not define GetLastError. Every output code
        # unit, including the untouched tail and failure buffer, is compared.
        if a['count'] != b['count'] or a['output'] != b['output'] or (not b['count'] and a['error'] != b['error']):
            differences.append(dict(row=i, actual=a, native=b))
    return differences


def main(args):
    directory = args.output.resolve()
    exe = directory / 'number-format.exe'
    if args.build or not exe.exists():
        if args.native:
            raise RuntimeError('native observation requires the identical supplied PE')
        exe = build(directory)
    if args.build:
        print('Built owned number-format PE; no execution claimed')
        return
    identity = json.loads((directory / 'inputs.json').read_text())
    if digest(exe) != identity['files'][exe.name] or any(digest(SOURCE/n) != h for n,h in identity['sources'].items()):
        raise ValueError('PE/source identity changed')
    host = args.host
    if not args.native and host is None:
        from observe_pe32_activation_startup import runner
        host = runner(directory)
    report = dict(identity=identity, backend='native Windows' if args.native else 'Alkyhol PE32',
                  platform=platform.platform(), machine=platform.machine(), observations=[])
    if args.native:
        import ctypes
        if platform.system() != 'Windows':
            raise RuntimeError('native observation requires Windows')
        kernel = ctypes.WinDLL('kernel32', use_last_error=True)
        kernel.GetCurrentProcess.restype = ctypes.c_void_p
        kernel.IsWow64Process2.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ushort), ctypes.POINTER(ctypes.c_ushort)]
        process, native = ctypes.c_ushort(), ctypes.c_ushort()
        if not kernel.IsWow64Process2(kernel.GetCurrentProcess(), ctypes.byref(process), ctypes.byref(native)) or native.value != 0x8664:
            raise RuntimeError('native AMD64 Windows observation required')
        report.update(observer_process_machine=process.value, native_machine=native.value)
    reference = decode(args.reference.read_bytes()) if args.reference else None
    for rebased in ([False] if args.native else [False, True]):
        command = [str(exe)] if args.native else [str(host.resolve()), str(exe), '--strict', '--file-origins']
        if rebased:
            command.append('--rebase')
        result = subprocess.run(command, capture_output=True, timeout=180)
        name = ('native' if args.native else 'rebased' if rebased else 'normal') + '-observations.bin'
        (directory/name).write_bytes(result.stdout)
        row = dict(rebased=rebased, status=result.returncode, stdout_sha256=hashlib.sha256(result.stdout).hexdigest(),
                   stderr=result.stderr.decode(errors='replace'), data=name)
        report['observations'].append(row)
        (directory/'report.json').write_text(json.dumps(report, indent=2)+'\n')
        if result.returncode:
            raise ValueError('number-format PE did not complete')
        rows = decode(result.stdout)
        row['rows'] = len(rows)
        if reference is not None:
            row['differences'] = compare(rows, reference)
            (directory/'report.json').write_text(json.dumps(report, indent=2)+'\n')
            if row['differences']:
                raise ValueError(f"{len(row['differences'])} strict native number-format differences")
    (directory/'report.json').write_text(json.dumps(report, indent=2)+'\n')
    print('Recorded', len(rows), 'complete number-format rows;',
          'native comparison passed' if reference is not None else 'no native equivalence claimed')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT/'local/pe/number-format')
    parser.add_argument('--host', type=Path)
    parser.add_argument('--build', action='store_true')
    parser.add_argument('--native', action='store_true')
    parser.add_argument('--reference', type=Path)
    main(parser.parse_args())
