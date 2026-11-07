#!/usr/bin/env python3
"""Execute the frozen owned PE64 DLL through Alkyhol, including real DllMain."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
from build_pe_fixture import ROOT

REFERENCE = ROOT / 'pe/reference/exports-windows-v1.json'
TARGET = 'x86_64-apple-darwin'


def check_result(result, reference):
    if result['common'] != reference['common'] or result['execution'] != reference['native_only']:
        raise ValueError('actual DLL lookups/initialization/calls differ from frozen Windows observations')
    if result['forwarded_last_error'] != 0x1234:
        raise ValueError('actual forwarded Win64 API did not preserve the test sentinel')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--dll', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    if platform.system() != 'Darwin':
        raise RuntimeError('Alkyhol DLL runtime requires macOS/Rosetta')
    dll = args.dll.resolve()
    manifest = json.loads((dll.parent / 'exports-fixture.json').read_text(encoding='utf-8'))
    reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    actual_hash = hashlib.sha256(dll.read_bytes()).hexdigest()
    if actual_hash != manifest['dll_sha256'] or actual_hash != reference['dll_sha256'] or manifest['sources'] != reference['sources']:
        raise ValueError('DLL is not the unchanged owned fixture measured on native Windows')
    for name, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest:
            raise ValueError('DLL source identity changed')
    cargo = shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
    subprocess.run([cargo, 'build', '--manifest-path', str(ROOT / 'pe/Cargo.toml'), '--target', TARGET,
                    '--locked', '--offline', '--bin', 'probe_library'], check=True)
    executable = ROOT / 'pe/target' / TARGET / 'debug/probe_library'
    results = []
    for flags in [[], ['--rebase']]:
        result = subprocess.run([str(executable), str(dll), *flags], capture_output=True, check=True, timeout=30)
        value = json.loads(result.stdout.decode('utf-8'))
        check_result(value, reference)
        if flags and value['relocated'] is not True:
            raise ValueError('forced DLL rebasing was not observed')
        results.append(value)
    if any(results[0][key] != results[1][key] for key in ['common', 'execution', 'forwarded_last_error']):
        raise ValueError('rebased DLL behavior differs')
    report = {'schema_version': 1, 'backend': 'Alkyhol isolated Win64 DLL',
              'dll_sha256': actual_hash, 'sources': manifest['sources'],
              'normal': results[0], 'rebased': results[1],
              'revision': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Passed actual PE64 DLL attach, symbols, data, calls and detach on Alkyhol (normal + forced rebase)')


if __name__ == '__main__':
    main()
