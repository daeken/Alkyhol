#!/usr/bin/env python3
"""Compare actual Windows/Alkyhol DLL-to-DLL execution, distinct from graph discovery."""
import argparse
import json
from pathlib import Path
import re
from build_pe_dependent import SOURCES
from run_pe_dependent import REFERENCE, DEPENDENCY_REFERENCE, EXPECTED_CALLS, EXPECTED_TRACE

NATIVE = 'native Windows dependent DLL LoadLibraryEx'
HOST = 'Alkyhol explicit dependent DLLs'


def compare(reports, reference=None):
    if len(reports) != 2 or {r['backend'] for r in reports} != {NATIVE, HOST}:
        raise ValueError('need actual native Windows and Alkyhol dependent DLL reports')
    native = next(r for r in reports if r['backend'] == NATIVE)
    host = next(r for r in reports if r['backend'] == HOST)
    dependency = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
    for r in reports:
        if r['schema_version'] != 1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}', r['revision']):
            raise ValueError('unsupported schema, dirty checkout or missing revision')
        if set(r['binaries']) != {'alkyhol_exports.dll', 'alkyhol_dependent.dll'} or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['binaries'].values()):
            raise ValueError('missing exact dependency/consumer binary identities')
        if r['binaries']['alkyhol_exports.dll'] != dependency['dll_sha256']:
            raise ValueError('changed original frozen DLL')
        if set(r['sources']) != set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['sources'].values()):
            raise ValueError('missing exact source identities')
        if any(r['sources'][s] != digest for s, digest in dependency['sources'].items()):
            raise ValueError('changed original frozen DLL sources')
        if r['common'] != {'calls': EXPECTED_CALLS, 'trace_hex': EXPECTED_TRACE.hex()}:
            raise ValueError('cross-DLL calls or lifecycle trace changed')
        if any(r['common']['calls'][key] is not True for key in ['through_matches_dependency', 'error_matches_dependency']):
            raise ValueError('missing actual resolved target identity')
    for field in ['revision', 'binaries', 'sources', 'common']:
        if native[field] != host[field]:
            raise ValueError(f'dependent DLL evidence mismatch: {field}')
    if native['relocated'] is not None or native['rebased_common'] is not None or native['rebased_relocated'] is not None:
        raise ValueError('invented native forced rebase observations')
    if host['rebased_common'] != host['common'] or host['rebased_relocated'] != [True, True]:
        raise ValueError('missing forced rebase of both DLLs')
    for flags in [host['relocated'], host['rebased_relocated']]:
        if not isinstance(flags, list) or len(flags) != 2 or any(type(v) is not bool for v in flags):
            raise ValueError('invalid actual relocation flags')
    if reference is None:
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference['backend'] != NATIVE or reference['schema_version'] != 1 or reference['tracked_tree_clean'] is not True:
        raise ValueError('invalid frozen native dependent DLL reference')
    for field in ['binaries', 'sources', 'common']:
        if native[field] != reference[field]:
            raise ValueError(f'changed frozen native dependent DLL evidence: {field}')
    return len(EXPECTED_TRACE)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports', type=Path, nargs='+')
    args = p.parse_args()
    count = compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol agree on actual cross-DLL calls/forwarders and {count} lifecycle trace bytes')
