#!/usr/bin/env python3
"""Execute the identical PE64 fixture on Windows or via Alkyhol/Rosetta."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
EXPECTED_WORDS = [0x41424931, 0xcafebabe, 0x9876270f, 9999, 258, 0x45564e54,
                  0, 1, 16, 8, 0x31415926, 0xdeadbeef, 0xfeedface, 12, 6]
EXPECTED_FILE = struct.pack('<III', 0x12345678, 0x90abcdef, 0x31415926)


def observe(executable, rebase=False):
    with tempfile.TemporaryDirectory(prefix='alkyhol-pe-observation-') as directory:
        args = [str(executable.resolve())] + (['--rebase'] if rebase else [])
        result = subprocess.run(args, cwd=directory, capture_output=True, timeout=30)
        if result.returncode != 0:
            raise RuntimeError(f'PE execution failed ({result.returncode}): {result.stderr.decode(errors="replace")}')
        if rebase and b'relocated=true' not in result.stderr:
            raise RuntimeError('forced relocation was not observed')
        data = (Path(directory) / 'pe-observation.bin').read_bytes()
        if len(data) != 4 * len(EXPECTED_WORDS):
            raise RuntimeError('invalid PE observation length')
        words = list(struct.unpack('<' + 'I' * len(EXPECTED_WORDS), data))
        if words != EXPECTED_WORDS:
            raise RuntimeError(f'PE predicates differ: {words}')
        file_bytes = (Path(directory) / 'pe-λ-🌸.bin').read_bytes()
        if file_bytes != EXPECTED_FILE:
            raise RuntimeError('Unicode-named PE file readback differs')
        return {'status': result.returncode, 'words': words, 'file_hex': file_bytes.hex()}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    pe = args.fixture.resolve()
    manifest = json.loads((pe.parent / 'fixture.json').read_text())
    if hashlib.sha256(pe.read_bytes()).hexdigest() != manifest['pe_sha256']:
        raise RuntimeError('PE payload changed since build manifest')
    for source, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT / source).read_bytes()).hexdigest() != digest:
            raise RuntimeError(f'fixture source mismatch: {source}')
    if platform.system() == 'Windows':
        executable = pe
        backend = 'native Windows PE64 / kernel32'
        executed_digest = hashlib.sha256(executable.read_bytes()).hexdigest()
    elif platform.system() == 'Darwin':
        from translate_pe import translate, embedded_payload
        executable = translate(pe, ROOT / 'local/pe/contract-macho')
        backend = 'Alkyhol Win64 ABI / x86-64 Mach-O'
        executed_digest = hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
    else:
        raise RuntimeError('execution requires Windows or macOS')
    if executed_digest != manifest['pe_sha256']:
        raise RuntimeError('executed payload differs from transferred PE fixture')
    normal = observe(executable)
    rebased = observe(executable, True) if platform.system() == 'Darwin' else None
    if rebased is not None and normal != rebased:
        raise RuntimeError('rebased PE observations differ')
    report = {'schema_version': 1, 'backend': backend, 'host_machine': platform.machine(),
              'pe_sha256': manifest['pe_sha256'], 'executed_pe_sha256': executed_digest, 'sources': manifest['sources'],
              'observation': normal, 'forced_rebase_observation': rebased,
              'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=ROOT, text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(f'Passed actual PE64 execution: {backend}, {len(EXPECTED_WORDS)} words + Unicode file bytes')


if __name__ == '__main__':
    main()
