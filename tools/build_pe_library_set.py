#!/usr/bin/env python3
"""Assemble three unchanged, already frozen DLLs into a shared-dependency fixture."""
import argparse
import hashlib
import json
from pathlib import Path
from build_pe_fixture import ROOT
from build_pe_dependent import build as build_dependent
from build_pe_ordinal import build as build_ordinal

ORDER = ['alkyhol_dependent.dll', 'alkyhol_ordinal.dll', 'alkyhol_exports.dll']


def expected_identity():
    result = {'binaries': {}, 'sources': {}}
    for name in ['dependent-windows-v1.json', 'ordinal-windows-v1.json']:
        reference = json.loads((ROOT / 'pe/reference' / name).read_text(encoding='utf-8'))
        for field in result:
            for key, value in reference[field].items():
                if key in result[field] and result[field][key] != value:
                    raise ValueError('old native references disagree on shared identity')
                result[field][key] = value
    return result


def build(output):
    output = output.resolve()
    build_dependent(output)
    build_ordinal(output)
    identity = expected_identity()
    for name, digest in identity['binaries'].items():
        if hashlib.sha256((output / name).read_bytes()).hexdigest() != digest:
            raise ValueError('bundle must preserve every previously frozen DLL')
    for name, digest in identity['sources'].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest:
            raise ValueError('bundle must preserve every previously frozen source')
    manifest = {'schema_version': 1, **identity, 'provided_modules': ORDER}
    (output / 'library-set-fixture.json').write_text(json.dumps(manifest, indent=2) + '\n', encoding='utf-8')
    print('Assembled unchanged shared DLL bundle: ' + str(output))


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--output', type=Path, default=ROOT / 'local/pe/library-set')
    build(p.parse_args().output)
