#!/usr/bin/env python3
"""Targeted file-backed CopyFileW checks for DAO's selected-save load path."""
import argparse
import hashlib
import json
import os
import platform
from pathlib import Path
import shutil
import stat
import struct
import subprocess
import zipfile
from build_pe_fixture import ROOT, rust_tool

SOURCE = ROOT / 'pe/fixtures/pe32-startup'
SYMBOLS = [('GetStdHandle',4), ('WriteFile',20), ('GetLastError',0), ('SetLastError',4),
           ('GetEnvironmentVariableA',12), ('SetCurrentDirectoryW',4), ('CopyFileW',12),
           ('CreateFileW',28), ('CloseHandle',4)]


def frozen():
    raw = (ROOT/'pe/reference/pe32-copy-file-windows-v1.json').read_bytes()
    if hashlib.sha256(raw).hexdigest()!='56d3dfea175b9de0c67e95e60fc13889d67f5cd6663d2f0d9f7c60c4ddf0be4d':
        raise ValueError('changed frozen native copy observation')
    return json.loads(raw)


def fixture(directory):
    reference = frozen()
    if any(digest(SOURCE/n)!=h for n,h in reference['inputs']['sources'].items()):
        raise ValueError('changed frozen copy source')
    archive = ROOT/'pe/reference/pe32-copy-file-v1.zip'
    if digest(archive)!='d59a20904d87e6f61050213e75af2416cefa01f1ae4b9bfd6f44114cc03f5dd1':
        raise ValueError('changed frozen owned PE archive')
    directory.mkdir(parents=True,exist_ok=True)
    with zipfile.ZipFile(archive) as z:
        if set(z.namelist())!=set(reference['inputs']['files']):
            raise ValueError('changed owned copy PE roster')
        for name,identity in reference['inputs']['files'].items():
            data=z.read(name)
            if Path(name).name!=name or hashlib.sha256(data).hexdigest()!=identity:
                raise ValueError('changed owned copy PE bytes/name')
            (directory/name).write_bytes(data)
    (directory/'inputs.json').write_text(json.dumps(reference['inputs'],indent=2)+'\n')


def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def build(directory):
    directory.mkdir(parents=True, exist_ok=True)
    clang, lld = shutil.which('clang'), rust_tool('rust-lld')
    if not clang:
        raise RuntimeError('open Clang is required')
    library = directory / 'kernel32.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86',
                    '/def:'+str(SOURCE/'copy-file.def'),'/out:'+str(library)],check=True)
    for mode in range(11):
        obj, exe = directory/f'case{mode}.obj', directory/f'case{mode}.exe'
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding',
                        '-fno-builtin','-fno-stack-protector','-fno-unwind-tables',
                        '-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',
                        f'-DMODE={mode}','-c',str(SOURCE/'copy-file.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console',
                        '/machine:x86','/dynamicbase','/fixed:no','/timestamp:0',
                        '/safeseh:no','/manifest:no','/entry:entry',str(obj),str(library),
                        *[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],
                        '/out:'+str(exe)],check=True)
    identity = dict(sources={n:digest(SOURCE/n) for n in ['copy-file.c','copy-file.def']},
                    files={f'case{i}.exe':digest(directory/f'case{i}.exe') for i in range(11)})
    (directory/'inputs.json').write_text(json.dumps(identity,indent=2)+'\n')


def check(directory, host, native):
    if native:
        if os.name != 'nt':
            raise RuntimeError('native mode requires actual Windows')
        identity = json.loads((directory/'inputs.json').read_text())
        if any(digest(directory/n)!=h for n,h in identity['files'].items()):
            raise ValueError('changed identical-PE input')
        if any(digest(SOURCE/n)!=h for n,h in identity['sources'].items()):
            raise ValueError('changed owned source identity')
    else:
        fixture(directory)
        identity = json.loads((directory/'inputs.json').read_text())
        if host is None:
            from observe_pe32_activation_startup import runner
            host = runner(directory)
        host = host.resolve()
    report = dict(backend='native Windows' if native else 'Alkyhol executing-host PE32 file assertions',
                  platform=platform.platform(), machine=platform.machine(),
                  host_sha256=None if native else digest(host), inputs=identity, observations=[])
    source_bytes = bytes(range(256))*259+b'\x00copy tail\xff'
    prior = b'previous target longer than source' * 3000
    for rebase in ([False] if native else [False,True]):
        for mode in range(11):
            if native and mode in [6,7,9]:
                continue  # Explicit Alkyhol profile exclusions, not Windows semantics.
            root = directory/f'profile-{mode}-{int(rebase)}'
            root.mkdir(exist_ok=True)
            source, target, outside = root/'source.bin', root/'target.bin', directory/'outside.bin'
            if target.exists():
                target.chmod(stat.S_IRUSR|stat.S_IWUSR); target.unlink()
            if source.exists():
                source.chmod(stat.S_IRUSR|stat.S_IWUSR)
            source.write_bytes(source_bytes)
            os.utime(source,ns=(978307200123456700,978307201765432100))
            if mode in [1,2,4]:
                target.write_bytes(prior)
            if mode in [4,5]:
                (target if mode==4 else source).chmod(stat.S_IRUSR|stat.S_IRGRP|stat.S_IROTH)
            outside.write_bytes(b'outside write capability')
            path = str(root.resolve()) if native else 'Z:'+str(root.resolve()).replace('/','\\')
            command = [str((directory/f'case{mode}.exe').resolve())] if native else [
                str(host),str((directory/f'case{mode}.exe').resolve()),'--strict','--file-origins',
                '--documents-directory',str(root.resolve()),*(['--rebase'] if rebase else [])]
            p = subprocess.run(command,env=dict(os.environ,ALKYHOL_COPY_ROOT=path),capture_output=True,timeout=30)
            row = dict(mode=mode,rebased=rebase,status=p.returncode,stdout_hex=p.stdout.hex(),
                       stderr=p.stderr.decode(errors='replace'))
            report['observations'].append(row)
            (directory/'report.json').write_text(json.dumps(report,indent=2)+'\n')
            if mode in [6,7,9]:
                reason = {6:'same-file destination',7:'path outside checked UTF-16 storage',
                          9:'file writes are restricted to the explicit Documents backing'}[mode]
                if p.returncode!=78 or p.stdout!=struct.pack('<2I',0x43503332,mode) or reason not in row['stderr']:
                    raise ValueError('missing explicit unsupported copy case')
            else:
                if p.returncode or len(p.stdout)!=16 or struct.unpack_from('<2I',p.stdout)!=(0x43503332,mode):
                    raise ValueError('incomplete actual copy result')
                result,error=struct.unpack_from('<2I',p.stdout,8)
                expected_error={2:80,3:2,4:5,8:32}.get(mode)
                if (not result if expected_error is None else result!=0 or error!=expected_error):
                    raise ValueError(f'copy result/error differs in mode {mode}: {result}, {error}')
                native_row=next(r for r in frozen()['observations'] if r['mode']==mode)
                expected=struct.unpack('<4I',bytes.fromhex(native_row['stdout_hex']))
                # GetLastError is not defined by the API on successful copies.
                if result!=expected[2] or (not result and error!=expected[3]):
                    raise ValueError('copy result/defined error differs from frozen Windows')
                if result:
                    if target.read_bytes()!=source_bytes:
                        raise ValueError('copy did not retain every byte or truncate old tail')
                    if target.stat().st_mtime_ns!=source.stat().st_mtime_ns:
                        raise ValueError('copy modification time differs')
                    if mode==5 and target.stat().st_mode&stat.S_IWUSR:
                        raise ValueError('copied readonly attribute differs')
            if source.read_bytes()!=source_bytes or outside.read_bytes()!=b'outside write capability':
                raise ValueError('copy changed source/outside file')
            if bool(source.stat().st_mode&stat.S_IWUSR) != (mode!=5):
                raise ValueError('copy changed source readonly state')
            if mode in [2,4] and target.read_bytes()!=prior:
                raise ValueError('failed copy changed existing destination')
            if mode in [3,6,7,8,9] and target.exists():
                raise ValueError('declined copy created a destination')
            row.update(source_sha256=digest(source), target_sha256=digest(target) if target.exists() else None)
            row.update(source_size=source.stat().st_size,source_mtime_ns=source.stat().st_mtime_ns,
                       source_readonly=not bool(source.stat().st_mode&stat.S_IWUSR),
                       target_size=target.stat().st_size if target.exists() else None,
                       target_mtime_ns=target.stat().st_mtime_ns if target.exists() else None,
                       target_readonly=not bool(target.stat().st_mode&stat.S_IWUSR) if target.exists() else None)
    if any(digest(directory/n)!=h for n,h in identity['files'].items()):
        raise ValueError('PE input changed during execution')
    (directory/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'Passed {len(report["observations"])} real-file CopyFileW cases; no original game input used')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=ROOT/'local/pe/copy-file')
    parser.add_argument('--host',type=Path)
    parser.add_argument('--native',action='store_true')
    parser.add_argument('--build',action='store_true',help='build new observer candidates; does not replace frozen evidence')
    args=parser.parse_args()
    if args.build:
        if args.native or args.host:
            parser.error('--build is separate from execution modes')
        build(args.output.resolve())
    else:
        check(args.output.resolve(),args.host,args.native)
