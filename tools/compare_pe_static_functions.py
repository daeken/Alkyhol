#!/usr/bin/env python3
"""Compare identical-PE manual frame restoration on native Windows and Alkyhol."""
import argparse
import json
from pathlib import Path
import re
import struct
from build_pe_static_functions import SOURCES
from run_pe_static_functions import REFERENCE, EXPECTED_WORDS, NATIVE, HOST


def compare(reports, reference=None):
    if len(reports) != 2 or {r['backend'] for r in reports} != {NATIVE, HOST}:
        raise ValueError('need independent native Windows and Alkyhol static-functions reports')
    native = next(r for r in reports if r['backend'] == NATIVE)
    host = next(r for r in reports if r['backend'] == HOST)
    expected = {'status': 0, 'words': EXPECTED_WORDS, 'stdout_hex': struct.pack('<20I', *EXPECTED_WORDS).hex()}
    for r in reports:
        if type(r['schema_version']) is not int or r['schema_version'] != 1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}', r['revision']):
            raise ValueError('unsupported schema, dirty checkout or missing revision')
        if not re.fullmatch('[0-9a-f]{64}', r['pe_sha256']) or r['executed_pe_sha256'] != r['pe_sha256']:
            raise ValueError('executed payload is not the identical PE')
        if set(r['sources']) != set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['sources'].values()):
            raise ValueError('missing exact static-functions source identity')
        if r['normal'] != expected or type(r['normal']['status']) is not int or any(type(v) is not int for v in r['normal']['words']):
            raise ValueError('actual static-functions/memory observations differ')
    for field in ['revision', 'pe_sha256', 'sources', 'normal']:
        if native[field] != host[field]: raise ValueError(f'static-functions identity mismatch: {field}')
    if native['rebased'] is not None or host['rebased'] != expected or type(host['rebased']['status']) is not int or any(type(v) is not int for v in host['rebased']['words']):
        raise ValueError('missing exact forced-rebase code execution')
    if reference is None: reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend'] != NATIVE or type(reference['schema_version']) is not int or reference['schema_version'] != 1 or reference['tracked_tree_clean'] is not True:
            raise ValueError('invalid frozen native static-functions reference')
        for field in ['pe_sha256', 'sources', 'normal']:
            if native[field] != reference[field]: raise ValueError(f'changed frozen native static-functions evidence: {field}')
    return len(EXPECTED_WORDS)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports', type=Path, nargs='+')
    args = p.parse_args()
    count = compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol identical-PE static functions agree on all {count} words')
