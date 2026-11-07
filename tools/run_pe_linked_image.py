#!/usr/bin/env python3
"""Observe identical Windows EXE/DLL bytes with native or explicit Alkyhol linking."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import struct
import subprocess
import tempfile
from build_pe_fixture import ROOT
from build_pe_linked_image import SOURCES, BINARIES, IMPORTS, DEPENDENCY_REFERENCE
from pe_imports import PE

REFERENCE = ROOT/'pe/reference/linked-image-windows-v1.json'
NATIVE, HOST = 'native Windows linked EXE/DLL', 'Alkyhol linked EXE/DLL'
DATA, BSS = 0x31415926, 0x13579bdf


def direct(a, b, data=DATA, bss=BSS):
    return ((a*33)^b^data^bss)&0xffffffff


EXPECTED_WORDS = [0x4c455845, DATA, BSS, direct(7, 11), direct(7, 11), 9+0x12345678,
                  DATA, 1, 0x3456, direct(7, 11, bss=BSS^0x55aa55aa), 0x10203040,
                  direct(3, 5, data=0x10203040, bss=BSS^0x55aa55aa), direct(7, 11), BSS]


def decode(run, windows=False, rebase=False):
    expected = struct.pack('<14I', *EXPECTED_WORDS)
    if run.returncode != 0 or run.stdout != expected:
        raise ValueError(f'linked EXE failed or actual data differ: {run.returncode}, '
                         f'{run.stdout.hex()}, {run.stderr!r}')
    flags = None
    if windows:
        if run.stderr:
            raise ValueError('unexpected native EXE diagnostic')
    else:
        info = json.loads(run.stderr.decode('utf-8'))
        if set(info) != {'entry_status', 'relocated'} or type(info['entry_status']) is not int or info['entry_status'] != 0:
            raise ValueError('missing actual EXE entry return')
        flags = info['relocated']
        if not isinstance(flags, list) or len(flags) != 2 or any(type(v) is not bool for v in flags) or (rebase and flags != [True, True]):
            raise ValueError('missing actual EXE/DLL rebase observations')
    return {'status': 0, 'words': EXPECTED_WORDS.copy(), 'stdout_hex': run.stdout.hex(), 'relocated': flags}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--directory', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    directory = args.directory.resolve()
    manifest = json.loads((directory/'linked-image-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version'] != 1 or set(manifest['sources']) != set(SOURCES) or set(manifest['binaries']) != BINARIES:
        raise ValueError('missing exact linked EXE/DLL manifest')
    if manifest['imports'] != IMPORTS or PE((directory/'linked-image.exe').read_bytes()).imports() != IMPORTS:
        raise ValueError('missing actual mixed EXE import thunks')
    dependency = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
    if manifest['binaries']['alkyhol_exports.dll'] != dependency['dll_sha256'] or any(manifest['sources'][s] != v for s, v in dependency['sources'].items()):
        raise ValueError('changed original frozen dependency')
    for name, digest in manifest['binaries'].items():
        if hashlib.sha256((directory/name).read_bytes()).hexdigest() != digest:
            raise ValueError('linked EXE/DLL binary identity changed')
    for name, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest() != digest:
            raise ValueError('linked EXE/DLL source identity changed')
    reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['binaries', 'sources', 'imports']:
        if manifest[field] != reference[field]:
            raise ValueError('changed frozen linked EXE/DLL fixture')
    windows = platform.system() == 'Windows'
    if windows:
        backend, executable = NATIVE, None
    elif platform.system() == 'Darwin':
        cargo = shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
        subprocess.run([cargo, 'build', '--manifest-path', str(ROOT/'pe/Cargo.toml'),
                        '--target', 'x86_64-apple-darwin', '--locked', '--offline', '--bin', 'probe_linked_image'], check=True)
        backend, executable = HOST, ROOT/'pe/target/x86_64-apple-darwin/debug/probe_linked_image'
    else:
        raise RuntimeError('requires Windows or macOS/Rosetta')
    # Run stable copies of the exact checked binary snapshots in an isolated
    # directory. The Windows OS discovers the actual DLL beside the actual EXE.
    with tempfile.TemporaryDirectory(prefix='alkyhol-linked-exe-') as temp:
        snapshot = Path(temp)
        for name, digest in manifest['binaries'].items():
            target = snapshot/name
            shutil.copyfile(directory/name, target)
            if hashlib.sha256(target.read_bytes()).hexdigest() != digest:
                raise ValueError('execution snapshot differs from checked binary')
        command = [str(snapshot/'linked-image.exe')] if windows else [str(executable), str(snapshot)]
        normal = decode(subprocess.run(command, cwd=snapshot, capture_output=True, timeout=30), windows)
        rebased = None if windows else decode(subprocess.run(command+['--rebase'], cwd=snapshot, capture_output=True, timeout=30), rebase=True)
    report = {'schema_version': 1, 'backend': backend, 'binaries': manifest['binaries'],
              'sources': manifest['sources'], 'imports': IMPORTS, 'normal': normal, 'rebased': rebased,
              'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(
                  ['git', 'status', '--porcelain', '--untracked-files=no'], cwd=ROOT, text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print('Passed unchanged EXE-to-DLL code/data/ordinal/forwarded imports: '+backend)


if __name__ == '__main__':
    main()
