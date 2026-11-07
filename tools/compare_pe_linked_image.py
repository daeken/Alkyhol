#!/usr/bin/env python3
"""Compare independent execution of identical linked Windows EXE/DLL snapshots."""
import argparse
import json
from pathlib import Path
import re
import struct
from build_pe_linked_image import SOURCES, BINARIES, IMPORTS, DEPENDENCY_REFERENCE
from run_pe_linked_image import REFERENCE, EXPECTED_WORDS, NATIVE, HOST


def check_observation(value, windows=False, rebase=False):
    if set(value) != {'status', 'words', 'stdout_hex', 'relocated'} or type(value['status']) is not int or value['status'] != 0:
        raise ValueError('missing exact linked EXE observations')
    if value['words'] != EXPECTED_WORDS or any(type(v) is not int for v in value['words']) or value['stdout_hex'] != struct.pack('<14I', *EXPECTED_WORDS).hex():
        raise ValueError('actual EXE-to-DLL results/retained bytes differ')
    flags = value['relocated']
    if windows:
        if flags is not None:
            raise ValueError('invented native forced rebase')
    elif not isinstance(flags, list) or len(flags) != 2 or any(type(v) is not bool for v in flags) or (rebase and flags != [True, True]):
        raise ValueError('missing actual EXE and DLL relocation predicates')


def compare(reports, reference=None):
    if len(reports) != 2 or {r['backend'] for r in reports} != {NATIVE, HOST}:
        raise ValueError('need independent native Windows and Alkyhol linked EXE reports')
    native = next(r for r in reports if r['backend'] == NATIVE)
    host = next(r for r in reports if r['backend'] == HOST)
    dependency = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
    for r in reports:
        if type(r['schema_version']) is not int or r['schema_version'] != 1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}', r['revision']):
            raise ValueError('unsupported schema, dirty checkout or missing revision')
        if set(r['binaries']) != BINARIES or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['binaries'].values()):
            raise ValueError('missing exact EXE/DLL binary identities')
        if set(r['sources']) != set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}', v) for v in r['sources'].values()):
            raise ValueError('missing exact EXE/DLL source identities')
        if r['binaries']['alkyhol_exports.dll'] != dependency['dll_sha256'] or any(r['sources'][s] != v for s, v in dependency['sources'].items()):
            raise ValueError('changed original frozen dependency')
        if r['imports'] != IMPORTS:
            raise ValueError('missing actual mixed EXE import thunks')
    for field in ['revision', 'binaries', 'sources', 'imports']:
        if native[field] != host[field]:
            raise ValueError(f'linked EXE identity mismatch: {field}')
    check_observation(native['normal'], windows=True)
    if native['rebased'] is not None:
        raise ValueError('invented native forced rebase')
    check_observation(host['normal'])
    check_observation(host['rebased'], rebase=True)
    for field in ['status', 'words', 'stdout_hex']:
        if host['normal'][field] != native['normal'][field] or host['rebased'][field] != native['normal'][field]:
            raise ValueError('actual linked EXE observations differ')
    if reference is None:
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference['backend'] != NATIVE or reference['schema_version'] != 1 or reference['tracked_tree_clean'] is not True:
        raise ValueError('invalid frozen native linked EXE reference')
    for field in ['binaries', 'sources', 'imports', 'normal']:
        if native[field] != reference[field]:
            raise ValueError(f'changed frozen native linked EXE evidence: {field}')
    return len(EXPECTED_WORDS)


if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports', type=Path, nargs='+')
    args = p.parse_args()
    count = compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol identical linked EXE/DLL snapshots agree on all {count} words')
