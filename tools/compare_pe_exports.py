#!/usr/bin/env python3
"""Compare Rust-only metadata to actual native Windows export lookups."""
import argparse
import json
from pathlib import Path
from run_pe_exports import ROOT, SPEC

REFERENCE = ROOT / 'pe/reference/exports-windows-v1.json'


def compare(reports, reference=None):
    if len(reports) != 2:
        raise ValueError('need exactly two reports')
    by_backend = {r['backend']: r for r in reports}
    if set(by_backend) != {'native Windows LoadLibrary/GetProcAddress', 'Rust metadata only (no DLL entry)'}:
        raise ValueError('need actual Windows and explicitly metadata-only Rust evidence')
    native = by_backend['native Windows LoadLibrary/GetProcAddress']
    host = by_backend['Rust metadata only (no DLL entry)']
    if not all(r['schema_version'] == 1 and r['tracked_tree_clean'] is True for r in reports):
        raise ValueError('unsupported schema or dirty checkout')
    for field in ['revision', 'dll_sha256', 'sources', 'common']:
        if native[field] != host[field]:
            raise ValueError(f'Windows export/metadata mismatch: {field}')
    expected = [(o, names, forwarder) for o, names, forwarder in SPEC]
    actual = [(e['ordinal'], e['names'], e['target'].get('forwarder')) for e in host['common']['entries']]
    if actual != expected or host['native_only'] is not None:
        raise ValueError('missing export contract or falsely claimed host DLL execution')
    if host['common']['entries'][0]['target'] != host['common']['entries'][1]['target']:
        raise ValueError('alias target differs')
    only = native['native_only']
    if only['initialized_data'] != 0x31415926 or only['bss_after_attach'] != 0x13579bdf or only['direct_call'] != ((7 * 33) ^ 11 ^ 0x31415926 ^ 0x13579bdf) or only['ordinal_call'] != 0x12345681:
        raise ValueError('missing actual native DLL initialization/call observations')
    if reference is None and REFERENCE.exists():
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend'] != native['backend'] or reference['tracked_tree_clean'] is not True:
            raise ValueError('invalid native export reference')
        for field in ['dll_sha256', 'sources', 'common', 'native_only']:
            if native[field] != reference[field]:
                raise ValueError(f'frozen native export evidence drift: {field}')
    return len(actual)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reports', type=Path, nargs='+')
    args = parser.parse_args()
    print(f'Native GetProcAddress and Rust metadata agree on {compare([json.loads(p.read_text(encoding="utf-8")) for p in args.reports])} export entries; DLL execution is Windows-only')
