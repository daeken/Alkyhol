#!/usr/bin/env python3
"""Package unchanged PE64 bytes in an x86-64 Mach-O linked to Alkyhol's binary ABI.

This is a checked loader/wrapper milestone, not static instruction rewriting.
Execution is trusted native code. No proprietary conversion tools are used.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TARGET = 'x86_64-apple-darwin'


def translate(source, output):
    if platform.system() != 'Darwin':
        raise RuntimeError('Mach-O packaging currently requires macOS and Xcode command line tools')
    source, output = source.resolve(), output.resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    cargo = shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
    rustc = shutil.which('rustc') or str(Path.home() / '.cargo/bin/rustc')
    subprocess.run([cargo, 'build', '--manifest-path', str(ROOT / 'pe/Cargo.toml'),
                    '--target', TARGET, '--locked', '--offline'], cwd=ROOT, check=True)
    build = ROOT / 'pe/target' / TARGET / 'debug'
    # Separate linker arguments preserve spaces/commas in the input path. Copy
    # to a staging file so the embedded bytes/hash describe one stable snapshot.
    with tempfile.TemporaryDirectory(prefix='alkyhol-pe-') as temp:
        payload = Path(temp) / 'payload.exe'
        shutil.copyfile(source, payload)
        digest = hashlib.sha256(payload.read_bytes()).hexdigest()
        args = [rustc, '--edition=2024', '--target', TARGET, str(ROOT / 'pe/launcher.rs'),
                '--extern', 'alkyhol_pe=' + str(build / 'libalkyhol_pe.rlib'),
                '-L', 'dependency=' + str(build / 'deps'), '-o', str(output)]
        # cc is the linker driver: every ld argument gets its own -Xlinker pair.
        for arg in ['-sectcreate', '__ALKYHOL', '__pefile', str(payload), '-segprot', '__ALKYHOL', 'r', 'r']:
            args += ['-C', 'link-arg=-Xlinker', '-C', 'link-arg=' + arg]
        subprocess.run(args, cwd=ROOT, check=True)
    report = {'schema_version': 1, 'kind': 'unchanged PE64 in x86-64 Mach-O / Alkyhol loader',
              'pe_sha256': digest, 'macho_sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
              'target': TARGET, 'executed': False}
    output.with_name(output.name + '.json').write_text(json.dumps(report, indent=2) + '\n')
    print(f'Packaged {output}; PE preflight/execution has not run')
    return output


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('input', type=Path)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    translate(args.input, args.output)


if __name__ == '__main__':
    main()
