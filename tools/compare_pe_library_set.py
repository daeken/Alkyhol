#!/usr/bin/env python3
"""Compare real shared DLL bundle execution, without claiming a global loader."""
import argparse
import json
from pathlib import Path
import re
from build_pe_library_set import ORDER, expected_identity
from run_pe_library_set import REFERENCE, EXPECTED, EXPECTED_TRACE

NATIVE, HOST = 'native Windows shared DLL bundle', 'Alkyhol closed DLL bundle'


def compare(reports, reference=None):
    if len(reports) != 2 or {r['backend'] for r in reports} != {NATIVE, HOST}:
        raise ValueError('need actual native Windows and Alkyhol shared DLL reports')
    native = next(r for r in reports if r['backend'] == NATIVE)
    host = next(r for r in reports if r['backend'] == HOST)
    identity = expected_identity()
    for r in reports:
        if r['schema_version'] != 1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}', r['revision']):
            raise ValueError('unsupported schema, dirty checkout or missing revision')
        if r['provided_modules'] != ORDER or any(r[field] != value for field, value in identity.items()):
            raise ValueError('changed frozen three-DLL/source identity or provided order')
        if r['common'] != {'results': EXPECTED, 'trace_hex': EXPECTED_TRACE.hex()} or r['common']['results']['shared_leaf_targets'] is not True:
            raise ValueError('shared DLL calls/target identities/lifecycle differ')
        for body, keys in [('dependent', ['through_matches_dependency','error_matches_dependency']),
                           ('ordinal', ['through_matches_ordinal','data_matches_ordinal','direct_matches_ordinal'])]:
            if any(r['common']['results'][body][key] is not True for key in keys):
                raise ValueError('missing actual target equality')
    if native['revision'] != host['revision']:
        raise ValueError('DLL bundle revision mismatch')
    if any(native[k] is not None for k in ['relocated','rebased_common','rebased_relocated']):
        raise ValueError('invented native forced rebase observations')
    if host['rebased_common'] != host['common'] or host['rebased_relocated'] != [True]*3:
        raise ValueError('missing actual forced rebase of all three DLLs')
    for flags in [host['relocated'], host['rebased_relocated']]:
        if not isinstance(flags,list) or len(flags) != 3 or any(type(v) is not bool for v in flags):
            raise ValueError('invalid actual relocation observations')
    if reference is None and REFERENCE.exists():
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend'] != NATIVE or reference['schema_version'] != 1 or reference['tracked_tree_clean'] is not True:
            raise ValueError('invalid frozen native bundle record')
        for field in ['binaries','sources','provided_modules','common']:
            if native[field] != reference[field]:
                raise ValueError(f'changed frozen native bundle evidence: {field}')
    return len(EXPECTED_TRACE)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports',type=Path,nargs='+')
    args = p.parse_args()
    count = compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol shared DLL execution agree, including {count} actual lifecycle bytes')
