#!/usr/bin/env python3
"""Compare actual native Windows and Alkyhol executions of the frozen owned DLL."""
import argparse
import json
from pathlib import Path
from run_pe_library import REFERENCE, check_result


def compare(reports, reference=None):
    if len(reports) != 2:
        raise ValueError('need two actual DLL execution reports')
    by_backend = {r['backend']: r for r in reports}
    if set(by_backend) != {'native Windows LoadLibrary/GetProcAddress', 'Alkyhol isolated Win64 DLL'}:
        raise ValueError('metadata-only inspection cannot claim DLL runtime parity')
    native = by_backend['native Windows LoadLibrary/GetProcAddress']
    host = by_backend['Alkyhol isolated Win64 DLL']
    if not all(r['schema_version'] == 1 and r['tracked_tree_clean'] is True for r in reports):
        raise ValueError('unsupported schema or dirty checkout')
    for key in ['revision', 'dll_sha256', 'sources']:
        if native[key] != host[key]:
            raise ValueError(f'DLL execution identity mismatch: {key}')
    if reference is None:
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference['backend'] != 'native Windows LoadLibrary/GetProcAddress' or reference['tracked_tree_clean'] is not True:
        raise ValueError('invalid frozen native DLL execution reference')
    for key in ['dll_sha256', 'sources', 'common', 'native_only']:
        if native[key] != reference[key]:
            raise ValueError(f'changed frozen native DLL evidence: {key}')
    for result in [host['normal'], host['rebased']]:
        check_result(result, native)
    if host['rebased']['relocated'] is not True:
        raise ValueError('missing actual forced DLL rebase')
    return len(native['common']['entries'])


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports', nargs='+', type=Path)
    args = p.parse_args()
    print(f'Actual native Windows and Alkyhol DLL execution agree on {compare([json.loads(p.read_text(encoding="utf-8")) for p in args.reports])} exports and frozen initialization/call observations')
