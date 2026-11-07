#!/usr/bin/env python3
"""Compare real failed DLL initialization traces, keeping API-specific errors separate."""
import argparse
import json
from pathlib import Path
import re
from build_pe_failed_attach import SOURCES
from run_pe_failed_attach import EXPECTED, REFERENCE

NATIVE = 'native Windows DLL attach rejection'
HOST = 'Alkyhol DLL attach rejection'


def compare(reports, reference=None):
    if len(reports) != 2 or {r['backend'] for r in reports} != {NATIVE, HOST}:
        raise ValueError('need actual native Windows and Alkyhol failed-attach reports')
    native = next(r for r in reports if r['backend'] == NATIVE)
    host = next(r for r in reports if r['backend'] == HOST)
    for r in reports:
        if r['schema_version'] != 1 or r['tracked_tree_clean'] is not True:
            raise ValueError('unsupported schema or dirty checkout')
        if not re.fullmatch('[0-9a-f]{40}', r['revision']) or not re.fullmatch('[0-9a-f]{64}', r['dll_sha256']):
            raise ValueError('missing executable/revision identity')
        if set(r['sources']) != set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['sources'].values()):
            raise ValueError('missing exact source identity')
        if set(r['common']) != {'loaded', 'trace_hex'} or r['common']['loaded'] is not False or r['common']['trace_hex'] != EXPECTED.hex():
            raise ValueError('missing actual ordered ATTACH/DETACH trace')
    for field in ['revision', 'dll_sha256', 'sources', 'common']:
        if native[field] != host[field]:
            raise ValueError(f'failed-attach execution mismatch: {field}')
    if native['rebased_common'] is not None or host['rebased_common'] != host['common'] or host['rebased_common']['loaded'] is not False:
        raise ValueError('missing host forced-rebase execution or invented native rebase')
    if host['backend_only'] != {'host_last_error': 0x1234}:
        raise ValueError('host must retain actual guest IAT last-error sentinel')
    only = native['backend_only']
    if set(only) != {'windows_last_error'} or type(only['windows_last_error']) is not int or not 0 <= only['windows_last_error'] <= 0xffffffff:
        raise ValueError('missing actual Windows loader error observation')
    if reference is None:
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference['backend'] != NATIVE or reference['tracked_tree_clean'] is not True or reference['schema_version'] != 1:
        raise ValueError('invalid frozen native attach-rejection reference')
    for field in ['dll_sha256', 'sources', 'common', 'backend_only']:
        if native[field] != reference[field]:
            raise ValueError(f'changed frozen failed-attach evidence: {field}')
    return len(EXPECTED)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports', type=Path, nargs='+')
    args = p.parse_args()
    count = compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol failed DLL initialization agree on {count} ordered trace bytes; loader errors remain backend-specific')
