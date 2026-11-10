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
import zipfile
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


def frozen(directory):
    record = json.loads((ROOT/'pe/reference/pe32-number-format-windows-v1.json').read_text())
    archive = ROOT/'pe/reference/pe32-number-format-v1.zip'
    if digest(archive) != record['fixture_sha256']:
        raise ValueError('frozen number-format fixture changed')
    directory.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(archive) as z:
        if set(z.namelist()) != {'number-format.exe', 'inputs.json', 'native-observations.bin'}:
            raise ValueError('unexpected frozen fixture members')
        for name in z.namelist():
            data = z.read(name)
            target = directory/name
            if target.exists() and target.read_bytes() != data:
                raise ValueError('output contains a different candidate; use a fresh directory: '+name)
            target.write_bytes(data)
    if digest(directory/'number-format.exe') != record['identity']['files']['number-format.exe']:
        raise ValueError('frozen Windows PE identity mismatch')
    reference = directory/'native-observations.bin'
    if digest(reference) != record['observations'][0]['stdout_sha256']:
        raise ValueError('frozen Windows observation identity mismatch')
    return reference


def guards(directory, host):
    source = SOURCE/'number-format-guard.c'
    lld = rust_tool('rust-lld')
    directory.mkdir(exist_ok=True)
    library = directory/'kernel32.lib'
    subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x86',
                    '/def:'+str(SOURCE/'number-format.def'), '/out:'+str(library)], check=True)
    reasons = ['locale outside explicit en-US profile', 'defaults and user locale overrides',
               'structure outside checked readable guest storage', 'output outside checked writable guest storage',
               'unaligned string pointer outside measured profile', 'output outside checked writable guest storage',
               'overlapping number-format input/output storage', 'grouping outside bounded profile']
    rows = []
    for mode, reason in enumerate(reasons):
        obj, exe = directory/f'guard{mode}.obj', directory/f'guard{mode}.exe'
        subprocess.run([shutil.which('clang') or 'clang', '--target=i686-pc-windows-msvc',
                        '-O1', '-ffreestanding', '-fno-builtin', '-fno-stack-protector',
                        '-fno-unwind-tables', '-fno-asynchronous-unwind-tables',
                        '-Wall', '-Wextra', '-Werror', f'-DMODE={mode}', '-c',str(source),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86',
                        '/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',
                        str(obj),str(library),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],
                        '/out:'+str(exe)],check=True)
        for rebased in [False, True]:
            result=subprocess.run([str(host.resolve()),str(exe),'--strict','--file-origins',
                                   *(['--rebase'] if rebased else [])],capture_output=True,timeout=30)
            row=dict(mode=mode,rebased=rebased,status=result.returncode,pe_sha256=digest(exe),
                     stderr=result.stderr.decode(errors='replace'),stdout_hex=result.stdout.hex())
            rows.append(row)
            (directory/'report.json').write_text(json.dumps(dict(source_sha256=digest(source),observations=rows),indent=2)+'\n')
            if result.returncode!=78 or result.stdout!=struct.pack('<2I',0x4e464731,mode) or reason not in row['stderr']:
                raise ValueError(f'number-format exclusion {mode} did not fail explicitly')
    return len(rows)


def main(args):
    directory = args.output.resolve()
    exe = directory / 'number-format.exe'
    if args.build:
        if args.native:
            raise RuntimeError('native observation requires the identical supplied PE')
        build(directory)
        print('Built owned number-format PE; no execution claimed')
        return
    reference_path = args.reference
    if not args.native and reference_path is None:
        reference_path = frozen(directory)
    if not exe.exists():
        raise RuntimeError('execution requires the supplied identical PE; --build creates candidates')
    identity = json.loads((directory / 'inputs.json').read_text())
    if digest(exe) != identity['files'][exe.name] or any(digest(SOURCE/n) != h for n,h in identity['sources'].items()):
        raise ValueError('PE/source identity changed')
    host = args.host
    if not args.native and host is None:
        from observe_pe32_activation_startup import runner
        host = runner(directory)
    report = dict(identity=identity, backend='native Windows' if args.native else 'Alkyhol PE32',
                  platform=platform.platform(), machine=platform.machine(), observations=[])
    if not args.native:
        report['host_sha256'] = digest(host.resolve())
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
    reference = decode(reference_path.read_bytes()) if reference_path else None
    if reference_path:
        report['reference_sha256'] = digest(reference_path)
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
    if not args.native:
        report['explicit_profile_storage_exclusions'] = guards(directory/'guards',host)
        (directory/'report.json').write_text(json.dumps(report,indent=2)+'\n')
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
