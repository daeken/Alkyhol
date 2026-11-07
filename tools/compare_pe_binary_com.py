#!/usr/bin/env python3
"""Compare real unchanged-PE virtual COM observations with native Windows."""
import argparse
import json
from pathlib import Path
import re
import struct
from build_pe_binary_com import SOURCES
from run_pe_binary_com import REFERENCE,EXPECTED_WORDS

NATIVE='native Windows PE64 / d3dcompiler_47 COM'
HOST='Alkyhol Win64 COM / x86-64 Mach-O'


def compare(reports,reference=None):
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:
        raise ValueError('need actual native Windows and Alkyhol binary COM reports')
    native=next(r for r in reports if r['backend']==NATIVE)
    host=next(r for r in reports if r['backend']==HOST)
    expected={'status':0,'words':EXPECTED_WORDS,'stdout_hex':struct.pack('<'+'I'*len(EXPECTED_WORDS),*EXPECTED_WORDS).hex()}
    for r in reports:
        if r['schema_version']!=1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',r['revision']):
            raise ValueError('unsupported schema, dirty checkout or missing revision')
        if not re.fullmatch('[0-9a-f]{64}',r['pe_sha256']) or r['executed_pe_sha256']!=r['pe_sha256']:
            raise ValueError('executed COM payload is not the identical PE')
        if set(r['sources'])!=set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}',v) for v in r['sources'].values()):
            raise ValueError('missing exact COM source identity')
        if r['observation']!=expected: raise ValueError('actual COM observations/retained bytes differ')
    for field in ['revision','pe_sha256','sources','observation']:
        if native[field]!=host[field]: raise ValueError(f'binary COM evidence mismatch: {field}')
    if native['forced_rebase_observation'] is not None or host['forced_rebase_observation']!=expected:
        raise ValueError('missing host/invented native forced rebase observations')
    if reference is None and REFERENCE.exists(): reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['schema_version']!=1 or reference['backend']!=NATIVE or reference['tracked_tree_clean'] is not True:
            raise ValueError('invalid frozen native COM reference')
        for field in ['pe_sha256','sources','observation']:
            if native[field]!=reference[field]: raise ValueError(f'changed frozen native COM evidence: {field}')
    return len(EXPECTED_WORDS)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('reports',type=Path,nargs='+')
    args=p.parse_args()
    count=compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol identical-PE virtual COM calls agree on {count} output words')
