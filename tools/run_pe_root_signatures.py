#!/usr/bin/env python3
"""Observe identical PE D3D12 serializers and returned COM blobs on Windows/Mac."""
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
from build_pe_root_signatures import SOURCES

REFERENCE = ROOT/'pe/reference/root-signatures-windows-v1.json'
LAYOUT_REFERENCE = ROOT/'cpp/reference/root-signatures-windows-v1.json'
NATIVE = 'native Windows PE64 / d3d12 serializers'
HOST = 'Alkyhol Win64 serializers / x86-64 Mach-O'


def expected_bytes(windows=False):
    # Independently frozen SDK byte evidence predates this PE client. This does
    # not reuse source-ABI device/metadata assertions as binary execution evidence.
    layouts = json.loads(LAYOUT_REFERENCE.read_text(encoding='utf-8'))['observation']['layouts']
    if len(layouts) != 6 or sum(map(len, layouts)) != 980:
        raise ValueError('missing frozen six-layout SDK byte evidence')
    output = struct.pack('<II', 0x52545045, 6)
    for layout in layouts:
        output += struct.pack('<6I', 0, 0, len(layout), 1, 1, 1)
        output += bytes(layout)
        output += struct.pack('<II', 0, 0)
    # Native run 42404657986 measured S_OK/non-null success/no diagnostic for
    # 65 constants. The bounded Rust serializer deliberately rejects that input.
    # Preserve this difference explicitly; only the preceding six layouts agree.
    tail = ([0, 0, 0, 92, 65, 0, 0xffffffff, 0xffffffff, 0xffffffff] if windows else
            [0x80070057, 1, 1, 0xffffffff, 0xffffffff, 0xffffffff, 1, 1, 0])
    return output+struct.pack('<9I', *tail)


def decode(run, windows=False, rebase=False):
    if run.returncode != 0 or run.stdout != expected_bytes(windows):
        raise ValueError(f'PE serializer failed or raw observations differ: {run.returncode}, '
                         f'{run.stdout.hex()}, {run.stderr!r}')
    if windows:
        if run.stderr:
            raise ValueError('unexpected native serializer diagnostic')
    else:
        match = re.fullmatch(rb'PE64: 4 sections, 5 named imports, 1 DIR64 fixups\n'
                             rb'entering Win64 code; relocated=(true|false)\n', run.stderr)
        if not match or (rebase and match[1] != b'true'):
            raise ValueError('missing exact loader/forced rebase observation')
    return {'status': 0, 'stdout_hex': run.stdout.hex()}


def observe(executable, windows=False, rebase=False):
    command = [str(executable.resolve())]+([] if windows else ['--loader-info'])
    if rebase:
        command.append('--rebase')
    with tempfile.TemporaryDirectory(prefix='alkyhol-pe-roots-') as directory:
        return decode(subprocess.run(command, cwd=directory, capture_output=True, timeout=30),
                      windows, rebase)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--fixture', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    pe = args.fixture.resolve()
    manifest = json.loads((pe.parent/'root-signatures-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version'] != 1 or set(manifest['sources']) != set(SOURCES):
        raise ValueError('missing exact serializer source manifest')
    if hashlib.sha256(pe.read_bytes()).hexdigest() != manifest['pe_sha256']:
        raise ValueError('serializer PE identity changed')
    for name, value in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest() != value:
            raise ValueError('serializer source identity changed')
    # The first native observation establishes the new PE reference. The older
    # source-ABI SDK bytes above are already mandatory on that first execution.
    if REFERENCE.exists():
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
        for field in ['pe_sha256', 'sources']:
            if manifest[field] != reference[field]:
                raise ValueError('changed frozen serializer PE fixture')
    windows = platform.system() == 'Windows'
    if windows:
        executable, executed, backend = pe, hashlib.sha256(pe.read_bytes()).hexdigest(), NATIVE
    elif platform.system() == 'Darwin':
        from translate_pe import translate, embedded_payload
        executable = translate(pe, ROOT/'local/pe/root-signatures/root-signatures-macos')
        executed = hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
        backend = HOST
    else:
        raise RuntimeError('requires Windows or macOS/Rosetta')
    if executed != manifest['pe_sha256']:
        raise ValueError('executed serializer PE bytes differ')
    normal = observe(executable, windows)
    rebased = None if windows else observe(executable, rebase=True)
    if rebased is not None and rebased != normal:
        raise ValueError('rebased serializer observations differ')
    report = {'schema_version': 1, 'backend': backend, 'pe_sha256': manifest['pe_sha256'],
              'executed_pe_sha256': executed, 'sources': manifest['sources'],
              'observation': normal, 'forced_rebase_observation': rebased,
              'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(
                  ['git', 'status', '--porcelain', '--untracked-files=no'], cwd=ROOT, text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2)+'\n', encoding='utf-8')
    print('Passed unchanged PE serializer/virtual blob calls: '+backend)


if __name__ == '__main__':
    main()
