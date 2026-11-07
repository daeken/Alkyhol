#!/usr/bin/env python3
"""Compare independent native and Alkyhol execution of the identical serializer PE."""
import argparse
import json
from pathlib import Path
import re
from build_pe_root_signatures import SOURCES
from run_pe_root_signatures import REFERENCE, NATIVE, HOST, expected_bytes


def compare(reports, reference=None):
    if len(reports) != 2 or {r['backend'] for r in reports} != {NATIVE, HOST}:
        raise ValueError('need actual native Windows and Alkyhol serializer PE reports')
    native = next(r for r in reports if r['backend'] == NATIVE)
    host = next(r for r in reports if r['backend'] == HOST)
    expected_host = {'status': 0, 'stdout_hex': expected_bytes().hex()}
    expected_native = {'status': 0, 'stdout_hex': expected_bytes(True).hex()}
    for r in reports:
        if r['schema_version'] != 1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}', r['revision']):
            raise ValueError('unsupported schema, dirty checkout or missing revision')
        if not re.fullmatch('[0-9a-f]{64}', r['pe_sha256']) or r['executed_pe_sha256'] != r['pe_sha256']:
            raise ValueError('executed serializer payload is not the identical PE')
        if set(r['sources']) != set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['sources'].values()):
            raise ValueError('missing exact serializer source identity')
        if r['observation'] != (expected_native if r['backend'] == NATIVE else expected_host):
            raise ValueError('actual serializer bytes/COM/error predicates differ')
    for field in ['revision', 'pe_sha256', 'sources']:
        if native[field] != host[field]:
            raise ValueError(f'serializer PE evidence mismatch: {field}')
    if native['forced_rebase_observation'] is not None or host['forced_rebase_observation'] != expected_host:
        raise ValueError('missing exact forced-rebase serializer execution')
    if reference is None and REFERENCE.exists():
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend'] != NATIVE or reference['tracked_tree_clean'] is not True:
            raise ValueError('invalid frozen native serializer reference')
        for field in ['pe_sha256', 'sources', 'observation']:
            if native[field] != reference[field]:
                raise ValueError(f'changed frozen native serializer evidence: {field}')
    common = len(expected_bytes())-9*4
    if bytes.fromhex(native['observation']['stdout_hex'])[:common] != bytes.fromhex(host['observation']['stdout_hex'])[:common]:
        raise ValueError('supported six-layout observations differ')
    return common


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports', type=Path, nargs='+')
    args = p.parse_args()
    count = compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol identical-PE serializers agree on {count} common bytes; '
          'the explicit 65-DWORD bound differs and is checked separately')
