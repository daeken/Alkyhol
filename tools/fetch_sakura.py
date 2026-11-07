#!/usr/bin/env python3
"""Fetch only pinned, hash-verified public Microsoft sample assets."""
import hashlib
import json
from pathlib import Path
import urllib.request

ROOT = Path(__file__).resolve().parents[1]


def fetch(destination=None):
    manifest = json.loads((ROOT / 'sakura/upstream.json').read_text())
    destination = Path(destination) if destination else ROOT / 'local/sakura/assets'
    destination.mkdir(parents=True, exist_ok=True)
    for name, expected in manifest['assets'].items():
        path = destination / name
        data = path.read_bytes() if path.exists() else b''
        if hashlib.sha256(data).hexdigest() != expected['sha256']:
            url = (f"https://raw.githubusercontent.com/microsoft/DirectX-Graphics-Samples/"
                   f"{manifest['revision']}/{manifest['directory']}/{name}")
            with urllib.request.urlopen(url, timeout=90) as response:
                data = response.read(expected['bytes'] + 1)
            if len(data) != expected['bytes'] or hashlib.sha256(data).hexdigest() != expected['sha256']:
                raise ValueError(f'upstream asset hash mismatch: {name}')
            temporary = path.with_suffix(path.suffix + '.tmp')
            temporary.write_bytes(data)
            temporary.replace(path)
        print(f'{name}: verified {len(data)} bytes')
    return destination


if __name__ == '__main__':
    print(fetch())
