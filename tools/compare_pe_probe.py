#!/usr/bin/env python3
"""Compare real executions of one identical Windows executable, never rebuilds."""
import argparse
import json
from pathlib import Path


REFERENCE = Path(__file__).resolve().parents[1] / 'pe/reference/windows-v1.json'


def compare(reports, reference=None):
    if len(reports) != 2:
        raise ValueError('need exactly two execution reports')
    by_backend = {r['backend']: r for r in reports}
    if set(by_backend) != {'native Windows PE64 / kernel32', 'Alkyhol Win64 ABI / x86-64 Mach-O'}:
        raise ValueError('need actual native Windows and Alkyhol binary execution')
    native, host = (by_backend[k] for k in ['native Windows PE64 / kernel32', 'Alkyhol Win64 ABI / x86-64 Mach-O'])
    if not all(r['schema_version'] == 1 and r['tracked_tree_clean'] is True for r in reports):
        raise ValueError('unclean or unsupported report schema')
    for r in reports:
        if r['executed_pe_sha256'] != r['pe_sha256']:
            raise ValueError('executed payload differs from original PE')
    for field in ['revision', 'pe_sha256', 'sources', 'observation']:
        if native[field] != host[field]:
            raise ValueError(f'PE execution mismatch: {field}')
    if native['forced_rebase_observation'] is not None or host['forced_rebase_observation'] != host['observation']:
        raise ValueError('missing independent forced-rebase execution')
    if native['observation']['status'] != 0:
        raise ValueError('failed guest execution')
    if reference is None:
        reference = json.loads(REFERENCE.read_text())
    if reference['backend'] != 'native Windows PE64 / kernel32' or reference['tracked_tree_clean'] is not True:
        raise ValueError('invalid frozen native Windows reference')
    for field in ['pe_sha256', 'sources', 'observation']:
        if native[field] != reference[field]:
            raise ValueError(f'PE result differs from frozen native Windows reference: {field}')
    return native['pe_sha256']


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports', type=Path, nargs='+')
    args = p.parse_args()
    digest = compare([json.loads(path.read_text()) for path in args.reports])
    print(f'Native Windows and Alkyhol agree on identical PE64 {digest}, including forced-rebase execution')


if __name__ == '__main__':
    main()
