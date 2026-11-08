#!/usr/bin/env python3
"""Run an identical PE32 binary on Windows and the experimental Rosetta i386 path."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import struct
import subprocess
import tempfile
from build_pe_fixture import ROOT
from build_pe32_fixture import SOURCES

REFERENCE = ROOT/'pe/reference/pe32-windows-v1.json'
NATIVE, HOST = 'native Windows PE32 i386', 'Alkyhol PE32 i386 / x86-64 Mach-O'
EXPECTED_WORDS = [0x49333836,4,10000,9999^0x12345678,1,1,0xa1b2c3d4,0x88776655]


def decode(run, windows=False, rebase=False):
    if run.returncode != 0 or run.stdout != struct.pack('<8I', *EXPECTED_WORDS):
        raise ValueError(f'PE32 PE failed or actual results differ: {run.returncode}, '
                         f'{run.stdout.hex()}, {run.stderr!r}')
    if windows:
        if run.stderr: raise ValueError('unexpected native pe32 diagnostic')
    else:
        match = re.fullmatch(rb'PE32: 4 sections, 4 imports, 19 HIGHLOW fixups\n'
                             rb'checked i386 mapping; relocated=(true|false)\n', run.stderr)
        if not match or (rebase and match[1] != b'true'):
            raise ValueError('missing exact loader/forced rebase evidence')
    return {'status': 0, 'words': EXPECTED_WORDS.copy(), 'stdout_hex': run.stdout.hex()}


def observe(executable, windows=False, rebase=False):
    command = [str(executable.resolve())]+([] if windows else ['--loader-info'])
    if rebase: command.append('--rebase')
    with tempfile.TemporaryDirectory(prefix='alkyhol-pe32-') as directory:
        return decode(subprocess.run(command, cwd=directory, capture_output=True, timeout=30), windows, rebase)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--fixture', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    pe = args.fixture.resolve()
    manifest = json.loads((pe.parent/'pe32-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version'] != 1 or set(manifest['sources']) != set(SOURCES):
        raise ValueError('missing exact pe32 source manifest')
    if hashlib.sha256(pe.read_bytes()).hexdigest() != manifest['pe_sha256']:
        raise ValueError('pe32 PE identity changed')
    for name, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest() != digest:
            raise ValueError('pe32 source identity changed')
    reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['pe_sha256', 'sources']:
        if manifest[field] != reference[field]: raise ValueError('changed frozen pe32 fixture')
    windows = platform.system() == 'Windows'
    if windows:
        executable, executed, backend = pe, hashlib.sha256(pe.read_bytes()).hexdigest(), NATIVE
    elif platform.system() == 'Darwin':
        from translate_pe32 import translate, embedded_payload
        executable = translate(pe, ROOT/'local/pe/pe32/pe32-macos')
        executed = hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
        backend = HOST
    else: raise RuntimeError('requires Windows or macOS/Rosetta')
    if executed != manifest['pe_sha256']: raise ValueError('executed PE payload differs')
    normal = observe(executable, windows)
    rebased = None if windows else observe(executable, rebase=True)
    if rebased is not None and rebased != normal: raise ValueError('rebased PE32 results differ')
    report = {'schema_version': 1, 'backend': backend, 'pe_sha256': manifest['pe_sha256'],
              'executed_pe_sha256': executed, 'sources': manifest['sources'], 'normal': normal, 'rebased': rebased,
              'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(
                  ['git', 'status', '--porcelain', '--untracked-files=no'], cwd=ROOT, text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print('Passed actual unchanged PE32/i386 execution and stdcall API bridge: '+backend)


if __name__ == '__main__': main()
