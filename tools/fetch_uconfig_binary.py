#!/usr/bin/env python3
"""Fetch the unchanged public u-config PE64 shipped in official w64devkit 2.11.0.

Archive, binary, matching public source and upstream license are SHA-256 pinned.
No build, patch, binary rewrite or installed Windows runtime is involved.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
URL = 'https://github.com/skeeto/w64devkit/releases/download/v2.11.0/w64devkit-x64-2.11.0.7z.exe'
ARCHIVE_SHA256 = '54c2bb9962e8a30c8270e34c5efc3d5fee579f53087fa7d435b95fbba2af7b2b'
PE_SHA256 = '390f6c01b87cd8c71d3b5aa6cabd1cb63539ba6204972d920347f2376ef4debc'
SOURCE_SHA256 = 'b1d5362c0c84753bc3a22831866cb92ada5b467da6fda406a33162e35e201210'
LICENSE_SHA256 = '7e12e5df4bae12cb21581ba157ced20e1986a0508dd10d0e8a4ab9a4cf94e85c'
LICENSE_URL = 'https://raw.githubusercontent.com/skeeto/u-config/v0.35.0/UNLICENSE'


def verified(path, expected):
    actual = hashlib.sha256(path.read_bytes()).hexdigest()
    if actual != expected:
        raise ValueError(f'{path.name}: expected {expected}, got {actual}')


def fetch(output):
    if platform.system() != 'Darwin':
        raise RuntimeError('fetch/extract uses macOS libarchive tar; native CI receives these exact extracted files')
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    cache = ROOT / 'local/pe/off-the-shelf/w64devkit-x64-2.11.0.7z.exe'
    cache.parent.mkdir(parents=True, exist_ok=True)
    if not cache.exists():
        with urllib.request.urlopen(URL, timeout=90) as response:
            data = response.read()
        cache.write_bytes(data)
    verified(cache, ARCHIVE_SHA256)
    for member, name, digest in [('w64devkit/bin/pkg-config.exe', 'pkg-config.exe', PE_SHA256),
                                 ('w64devkit/src/pkg-config.c', 'pkg-config.c', SOURCE_SHA256)]:
        path = output / name
        if not path.exists():
            path.write_bytes(subprocess.check_output(['/usr/bin/tar', '-xOf', str(cache), member]))
        verified(path, digest)
    license_path = output / 'UNLICENSE'
    if not license_path.exists():
        license_path.write_bytes(urllib.request.urlopen(LICENSE_URL, timeout=30).read())
    verified(license_path, LICENSE_SHA256)
    manifest = {'schema_version': 1, 'program': 'u-config', 'version': '0.35.0',
                'distribution': 'w64devkit 2.11.0 x64', 'archive_url': URL,
                'archive_sha256': ARCHIVE_SHA256, 'pe_member': 'w64devkit/bin/pkg-config.exe',
                'pe_sha256': PE_SHA256, 'source_member': 'w64devkit/src/pkg-config.c',
                'source_sha256': SOURCE_SHA256, 'license_url': LICENSE_URL,
                'license_sha256': hashlib.sha256(license_path.read_bytes()).hexdigest(),
                'rebuilt': False, 'patched': False}
    (output / 'provenance.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Verified unchanged released Windows u-config 0.35.0: {output / "pkg-config.exe"}')
    return output / 'pkg-config.exe'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'local/pe/u-config')
    args = parser.parse_args()
    fetch(args.output)


if __name__ == '__main__':
    main()
