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
import struct
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TARGET = 'x86_64-apple-darwin'



def embedded_payload(data):
    """Extract exactly one read-only PE section from a little-endian Mach-O64."""
    def take(offset, size):
        if offset < 0 or size < 0 or offset + size > len(data):
            raise ValueError('truncated Mach-O range')
        return data[offset:offset + size]
    def u32(offset):
        return struct.unpack('<I', take(offset, 4))[0]
    if u32(0) != 0xfeedfacf or u32(4) != 0x1000007:
        raise ValueError('expected AMD64 Mach-O64')
    take(0, 32)
    count, total = u32(16), u32(20)
    take(32, total)
    cursor, found = 32, []
    for _ in range(count):
        if cursor + 8 > 32 + total:
            raise ValueError('truncated Mach-O load command')
        kind, size = u32(cursor), u32(cursor + 4)
        if size < 8 or cursor + size > 32 + total:
            raise ValueError('invalid Mach-O load command')
        if kind == 0x19:
            if size < 72:
                raise ValueError('truncated Mach-O segment')
            sections = u32(cursor + 64)
            if 72 + sections * 80 > size:
                raise ValueError('truncated Mach-O section table')
            for i in range(sections):
                section = cursor + 72 + i * 80
                if take(section, 16).rstrip(b'\0') == b'__pefile' and take(section + 16, 16).rstrip(b'\0') == b'__ALKYHOL':
                    if take(cursor + 8, 16).rstrip(b'\0') != b'__ALKYHOL' or u32(cursor + 56) != 1 or u32(cursor + 60) != 1:
                        raise ValueError('PE payload segment is not read-only')
                    length = struct.unpack('<Q', take(section + 40, 8))[0]
                    found.append(take(u32(section + 48), length))
        cursor += size
    if cursor != 32 + total or len(found) != 1:
        raise ValueError('expected exactly one embedded PE payload')
    return found[0]


def translate(source, output):
    return _package(source, output, ROOT/'pe/launcher.rs',
                    'unchanged PE64 in x86-64 Mach-O / Alkyhol loader')


def _package(source, output, launcher, kind, extra_link_args=()):
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
        args = [rustc, '--edition=2024', '--target', TARGET, str(launcher),
                '--extern', 'alkyhol_pe=' + str(build / 'libalkyhol_pe.rlib'),
                '-L', 'dependency=' + str(build / 'deps'), '-L', 'dependency=' + str(build.parent.parent / 'debug/deps'), '-o', str(output)]
        # cc is the linker driver: every ld argument gets its own -Xlinker pair.
        for arg in [*extra_link_args, '-sectcreate', '__ALKYHOL', '__pefile', str(payload), '-segprot', '__ALKYHOL', 'r', 'r']:
            args += ['-C', 'link-arg=-Xlinker', '-C', 'link-arg=' + arg]
        subprocess.run(args, cwd=ROOT, check=True)
    embedded_digest = hashlib.sha256(embedded_payload(output.read_bytes())).hexdigest()
    if embedded_digest != digest:
        raise RuntimeError('linked Mach-O payload differs from original PE snapshot')
    report = {'schema_version': 1, 'kind': kind,
              'pe_sha256': digest, 'embedded_pe_sha256': embedded_digest, 'macho_sha256': hashlib.sha256(output.read_bytes()).hexdigest(),
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
