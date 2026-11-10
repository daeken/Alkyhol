#!/usr/bin/env python3
"""Targeted host clock/ABI assertions for the original DAO GetLocalTime stop.

Only the explicitly configured UTC guest profile is supported. Dynamic clock
values are bracketed on the executing host, never compared across machines.
"""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT, rust_tool

SOURCE = ROOT / 'pe/fixtures/pe32-startup'
SYMBOLS = [('GetStdHandle', 4), ('WriteFile', 20), ('SetLastError', 4),
           ('GetLastError', 0), ('GetSystemTimeAsFileTime', 4), ('GetLocalTime', 4)]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(directory):
    directory.mkdir(parents=True, exist_ok=True)
    clang = shutil.which('clang')
    if not clang:
        raise RuntimeError('open Clang is required')
    lld = rust_tool('rust-lld')
    library = directory / 'kernel32.lib'
    subprocess.run([lld, '-flavor', 'link', '/lib', '/machine:x86',
                    '/def:' + str(SOURCE / 'current-local-time.def'),
                    '/out:' + str(library)], check=True)
    files = []
    for mode in range(4):
        obj = directory / f'case{mode}.obj'
        exe = directory / f'case{mode}.exe'
        subprocess.run([clang, '--target=i686-pc-windows-msvc', '-O1',
                        '-ffreestanding', '-fno-builtin', '-fno-stack-protector',
                        '-fno-unwind-tables', '-fno-asynchronous-unwind-tables',
                        '-Wall', '-Wextra', '-Werror', f'-DMODE={mode}', '-c',
                        str(SOURCE / 'current-local-time.c'), '-o', str(obj)], check=True)
        subprocess.run([lld, '-flavor', 'link', '/nodefaultlib', '/subsystem:console',
                        '/machine:x86', '/dynamicbase', '/fixed:no', '/timestamp:0',
                        '/safeseh:no', '/manifest:no', '/entry:entry', str(obj),
                        str(library), *[f'/alternatename:__imp__{n}@{s}=__imp__{n}'
                                       for n, s in SYMBOLS], '/out:' + str(exe)], check=True)
        files.append(exe)
    return files


def decode(data):
    if len(data) != 8 + 24 * 40 or data[:8] != struct.pack('<2I', 0x4c543332, 0):
        raise ValueError('incomplete GetLocalTime observation')
    rows = []
    epoch = datetime.datetime(1601, 1, 1, tzinfo=datetime.timezone.utc)
    for offset in range(8, len(data), 40):
        before, after, guard1, *rest = struct.unpack_from('<QQH8HHI', data, offset)
        fields, guard2, error = rest[:8], rest[8], rest[9]
        if (guard1, guard2, error) != (0x1a2b, 0x3c4d, 0xaabbeeff):
            raise ValueError('output guards or last-error state changed')
        year, month, weekday, day, hour, minute, second, millis = fields
        instant = datetime.datetime(year, month, day, hour, minute, second,
                                    millis * 1000, tzinfo=datetime.timezone.utc)
        delta = instant - epoch
        ticks = ((delta.days * 86400 + delta.seconds) * 1000 + millis) * 10000
        if weekday != (instant.weekday() + 1) % 7 or not before // 10000 <= ticks // 10000 <= after // 10000:
            raise ValueError('UTC calendar is outside surrounding host clock calls')
        rows.append(dict(before=before, after=after, systemtime=fields))
    return rows


def check(directory, host):
    files = build(directory)
    if host is None:
        from observe_pe32_activation_startup import runner
        host = runner(directory)
    host = host.resolve()
    report = dict(backend='executing-host UTC clock and PE32 ABI assertions',
                  host_sha256=digest(host), sources={n: digest(SOURCE / n) for n in
                  ['current-local-time.c', 'current-local-time.def']},
                  files={p.name: digest(p) for p in files}, observations=[])
    for rebase in [False, True]:
        for mode, utc in [(0, True), (0, False), (1, True), (2, True), (3, True)]:
            command = [str(host), str(files[mode]), '--strict', '--file-origins']
            command += ['--rebase'] if rebase else []
            command += ['--timezone', 'UTC'] if utc else []
            result = subprocess.run(command, capture_output=True, timeout=30)
            row = dict(mode=mode, utc=utc, rebased=rebase, status=result.returncode,
                       stdout_hex=result.stdout.hex(), stderr=result.stderr.decode(errors='replace'))
            report['observations'].append(row)
            (directory / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
            if mode == 0 and utc:
                if result.returncode:
                    raise ValueError('supported clock call did not complete')
                row['rows'] = decode(result.stdout)
            else:
                reason = ('local-time APIs require an explicitly configured supported guest timezone'
                          if not utc else 'SYSTEMTIME output outside checked writable guest storage')
                if (result.returncode != 78 or reason not in row['stderr']
                        or result.stdout != struct.pack('<2I', 0x4c543332, mode)):
                    raise ValueError('unsupported profile/pointer was not explicitly rejected')
    (directory / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    print('Passed 48 host-bracketed UTC clock samples and eight explicit profile/pointer failures')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'local/pe/current-local-time')
    parser.add_argument('--host', type=Path)
    args = parser.parse_args()
    check(args.output.resolve(), args.host)
