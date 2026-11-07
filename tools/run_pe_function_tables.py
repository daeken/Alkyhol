#!/usr/bin/env python3
"""Run the identical PE registering, looking up and removing dynamic AMD64 function tables on Windows/Alkyhol."""
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
from build_pe_function_tables import SOURCES

REFERENCE = ROOT/'pe/reference/function-tables-windows-v1.json'
NATIVE, HOST = 'native Windows PE64 function tables', 'Alkyhol PE64 function tables / Rosetta'
def result(immediate): return ((immediate^0x55aa0102)+0x10203040)&0xffffffff
EXPECTED_WORDS = [0x4654424c,1,1,4,1,1,1,1,1,1,64,74,512,result(0x11223344),
                  1,1,1,1,1,1,result(0xfedcba98),1,1,result(0x10203040),
                  1,1,1,1,1,0,1,1,1,1,1,1,1]


def decode(run, windows=False, rebase=False):
    if run.returncode != 0 or run.stdout != struct.pack('<37I', *EXPECTED_WORDS):
        raise ValueError(f'function tables PE failed or actual results differ: {run.returncode}, '
                         f'{run.stdout.hex()}, {run.stderr!r}')
    if windows:
        if run.stderr: raise ValueError('unexpected native function-table diagnostic')
    else:
        match = re.fullmatch(rb'PE64: 4 sections, 10 named imports, 1 DIR64 fixups\n'
                             rb'entering Win64 code; relocated=(true|false)\n', run.stderr)
        if not match or (rebase and match[1] != b'true'):
            raise ValueError('missing exact loader/forced rebase evidence')
    return {'status': 0, 'words': EXPECTED_WORDS.copy(), 'stdout_hex': run.stdout.hex()}


def observe(executable, windows=False, rebase=False):
    command = [str(executable.resolve())]+([] if windows else ['--loader-info'])
    if rebase: command.append('--rebase')
    with tempfile.TemporaryDirectory(prefix='alkyhol-function-tables-') as directory:
        return decode(subprocess.run(command, cwd=directory, capture_output=True, timeout=30), windows, rebase)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--fixture', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    pe = args.fixture.resolve()
    manifest = json.loads((pe.parent/'function-tables-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version'] != 1 or set(manifest['sources']) != set(SOURCES):
        raise ValueError('missing exact function-tables source manifest')
    if hashlib.sha256(pe.read_bytes()).hexdigest() != manifest['pe_sha256']:
        raise ValueError('function-tables PE identity changed')
    for name, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest() != digest:
            raise ValueError('function-tables source identity changed')
    reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['pe_sha256', 'sources']:
        if manifest[field] != reference[field]: raise ValueError('changed frozen function-tables fixture')
    windows = platform.system() == 'Windows'
    if windows:
        executable, executed, backend = pe, hashlib.sha256(pe.read_bytes()).hexdigest(), NATIVE
    elif platform.system() == 'Darwin':
        from translate_pe import translate, embedded_payload
        executable = translate(pe, ROOT/'local/pe/function-tables/function-tables-macos')
        executed = hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
        backend = HOST
    else: raise RuntimeError('requires Windows or macOS/Rosetta')
    if executed != manifest['pe_sha256']: raise ValueError('executed PE payload differs')
    normal = observe(executable, windows)
    rebased = None if windows else observe(executable, rebase=True)
    if rebased is not None and rebased != normal: raise ValueError('rebased function tables results differ')
    report = {'schema_version': 1, 'backend': backend, 'pe_sha256': manifest['pe_sha256'],
              'executed_pe_sha256': executed, 'sources': manifest['sources'], 'normal': normal, 'rebased': rebased,
              'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(
                  ['git', 'status', '--porcelain', '--untracked-files=no'], cwd=ROOT, text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print('Passed actual guest function-table registration/lookup/removal: '+backend)


if __name__ == '__main__': main()
