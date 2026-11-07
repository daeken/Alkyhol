#!/usr/bin/env python3
"""Compare actual native Windows and Alkyhol runs of the pinned released PE."""
import argparse
import json
from pathlib import Path
from fetch_uconfig_binary import PE_SHA256, SOURCE_SHA256, LICENSE_SHA256

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / 'pe/reference/u-config-windows-v1.json'


def compare(reports, reference=None):
    if len(reports) != 2:
        raise ValueError('need two actual execution reports')
    by_backend = {r['backend']: r for r in reports}
    if set(by_backend) != {'native Windows PE64 / kernel32', 'Alkyhol Win64 ABI / x86-64 Mach-O'}:
        raise ValueError('need native Windows and Alkyhol reports')
    native = by_backend['native Windows PE64 / kernel32']
    host = by_backend['Alkyhol Win64 ABI / x86-64 Mach-O']
    for r in reports:
        if r['schema_version'] != 1 or r['tracked_tree_clean'] is not True:
            raise ValueError('unsupported schema or dirty checkout')
        if r['pe_sha256'] != PE_SHA256 or r['executed_pe_sha256'] != PE_SHA256:
            raise ValueError('upstream executable bytes changed')
        if r['upstream_source_sha256'] != SOURCE_SHA256 or r['upstream_license_sha256'] != LICENSE_SHA256:
            raise ValueError('upstream source/license identity changed')
    for key in ['program', 'revision', 'input_sha256', 'observations']:
        if native[key] != host[key]:
            raise ValueError(f'native Windows/Alkyhol mismatch: {key}')
    spec = json.loads((ROOT / 'pe/fixtures/u-config/cases.json').read_text())
    expected = {c['id']: {'status': c['status'], 'stdout_hex': c['stdout'].encode().hex(),
                           'stderr_hex': c['stderr'].encode().hex()} for c in spec['cases']}
    if native['observations'] != expected:
        raise ValueError('missing or changed case observations')
    if native['forced_rebase_observations'] is not None or host['forced_rebase_observations'] != host['observations']:
        raise ValueError('forced rebasing was not executed or differs')
    if reference is None and REFERENCE.exists():
        reference = json.loads(REFERENCE.read_text())
    if reference is not None:
        if reference['backend'] != 'native Windows PE64 / kernel32' or reference['tracked_tree_clean'] is not True:
            raise ValueError('invalid frozen native reference')
        for key in ['pe_sha256', 'upstream_source_sha256', 'upstream_license_sha256', 'input_sha256', 'observations']:
            if native[key] != reference[key]:
                raise ValueError(f'changed frozen native evidence: {key}')
    return len(expected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reports', type=Path, nargs='+')
    args = parser.parse_args()
    count = compare([json.loads(path.read_text()) for path in args.reports])
    print(f'Native Windows and Alkyhol agree on {count} cases from unchanged released u-config {PE_SHA256}')


if __name__ == '__main__':
    main()
