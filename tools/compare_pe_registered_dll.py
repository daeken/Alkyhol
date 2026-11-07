#!/usr/bin/env python3
"""Compare identical DLLs' actual lifecycle traces and exported calls on both OSes."""
import argparse
import json
from pathlib import Path
import re
from build_pe_registered_dll import SOURCES
from run_pe_registered_dll import REFERENCE,NATIVE,HOST,EXPECTED


def validate_values(values):
    if values!=EXPECTED:raise ValueError('actual DLL traces/exports/attach statuses differ')
    for value in values:
        if type(value['loaded']) is not bool:raise ValueError('missing actual typed DLL load result')
        if value['case']=='accepted' and (type(value['probe']) is not int or value['alias'] is not True):
            raise ValueError('missing typed exported result/alias identity')


def compare(reports,reference=None):
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:
        raise ValueError('need independent Windows/Alkyhol static DLL lifecycle reports')
    native=next(r for r in reports if r['backend']==NATIVE)
    host=next(r for r in reports if r['backend']==HOST)
    for r in reports:
        if type(r['schema_version']) is not int or r['schema_version']!=1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',r['revision']):
            raise ValueError('unsupported schema, dirty checkout or missing revision')
        if set(r['binaries'])!={'accepted','rejected'} or not all(re.fullmatch('[0-9a-f]{64}',v) for v in r['binaries'].values()):
            raise ValueError('missing identical DLL identities')
        if set(r['sources'])!=set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}',v) for v in r['sources'].values()):
            raise ValueError('missing exact DLL source identities')
        validate_values(r['normal'])
    for field in ['revision','binaries','sources','normal']:
        if native[field]!=host[field]:raise ValueError('DLL lifecycle mismatch: '+field)
    if native['rebased'] is not None:raise ValueError('invented native forced rebase')
    validate_values(host['rebased'])
    if reference is None and REFERENCE.exists():reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend']!=NATIVE or reference['tracked_tree_clean'] is not True or type(reference['schema_version']) is not int or reference['schema_version']!=1:
            raise ValueError('invalid frozen native lifecycle record')
        for field in ['binaries','sources','normal']:
            if native[field]!=reference[field]:raise ValueError('changed frozen DLL lifecycle evidence: '+field)
    return 40


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+')
    args=p.parse_args()
    count=compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Native Windows and Alkyhol identical DLL lifecycle agree on all {count} trace words and exported call/status results')
