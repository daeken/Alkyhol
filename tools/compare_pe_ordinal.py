#!/usr/bin/env python3
"""Compare actual Windows/Alkyhol execution of identical mixed ordinal DLLs."""
import argparse
import json
from pathlib import Path
import re
from build_pe_ordinal import SOURCES, IMPORTS
from run_pe_ordinal import REFERENCE, DEPENDENCY_REFERENCE, check_result

NATIVE, HOST = 'native Windows ordinal DLLs', 'Alkyhol ordinal DLLs'


def compare(reports, reference=None):
    if len(reports) != 2 or {r['backend'] for r in reports} != {NATIVE, HOST}:
        raise ValueError('need actual native Windows and Alkyhol ordinal execution reports')
    native = next(r for r in reports if r['backend'] == NATIVE)
    host = next(r for r in reports if r['backend'] == HOST)
    dependency = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
    for r in reports:
        if r['schema_version'] != 1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}', r['revision']):
            raise ValueError('unsupported schema, dirty checkout or missing revision')
        if set(r['binaries']) != {'alkyhol_exports.dll', 'alkyhol_ordinal.dll'} or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['binaries'].values()):
            raise ValueError('missing exact binary identities')
        if set(r['sources']) != set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['sources'].values()):
            raise ValueError('missing exact source identities')
        if r['binaries']['alkyhol_exports.dll'] != dependency['dll_sha256'] or any(r['sources'][s] != digest for s, digest in dependency['sources'].items()):
            raise ValueError('changed original frozen dependency')
        if r['imports'] != IMPORTS:
            raise ValueError('missing actual mixed ordinal thunk inventory')
    for field in ['revision', 'binaries', 'sources', 'imports']:
        if native[field] != host[field]:
            raise ValueError(f'ordinal evidence identity mismatch: {field}')
    check_result(native['normal'], windows=True)
    if native['rebased'] is not None:
        raise ValueError('invented native forced rebase')
    check_result(host['normal'])
    check_result(host['rebased'], rebase=True)
    if host['normal']['common'] != native['normal']['common'] or host['rebased']['common'] != native['normal']['common']:
        raise ValueError('actual ordinal execution differs')
    if reference is None and REFERENCE.exists():
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend'] != NATIVE or reference['schema_version'] != 1 or reference['tracked_tree_clean'] is not True:
            raise ValueError('invalid frozen native ordinal reference')
        for field in ['binaries', 'sources', 'imports', 'normal']:
            if native[field] != reference[field]:
                raise ValueError(f'changed frozen native ordinal evidence: {field}')
    return len(native['normal']['common'])


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports', type=Path, nargs='+')
    args = p.parse_args()
    count = compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol mixed ordinal DLL execution agree on {count} call/data/target observations')
