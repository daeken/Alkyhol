#!/usr/bin/env python3
"""Compare identical released PE execution and exact status/stdout/stderr bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import re
from run_make2compdb_binary import CORPUS,REFERENCE,NATIVE,HOST,PROGRAM,expected
from fetch_make2compdb_binary import PE_SHA256,SOURCE_SHA256,LICENSE_SHA256


def compare(reports,reference=None):
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:raise ValueError('need independent native Windows and Alkyhol records')
    native=next(r for r in reports if r['backend']==NATIVE);host=next(r for r in reports if r['backend']==HOST)
    _,wanted=expected();inputs={'pe/fixtures/make2compdb/cases.json':hashlib.sha256(CORPUS.read_bytes()).hexdigest()}
    for r in reports:
        if type(r['schema_version']) is not int or r['schema_version']!=1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',r['revision']):raise ValueError('invalid schema, revision or dirty tree')
        if r['program']!=PROGRAM or r['pe_sha256']!=PE_SHA256 or r['executed_pe_sha256']!=PE_SHA256 or r['upstream_source_sha256']!=SOURCE_SHA256 or r['upstream_license_sha256']!=LICENSE_SHA256:raise ValueError('changed release identity')
        if r['input_sha256']!=inputs or r['observations']!=wanted or any(type(x['status']) is not int for x in r['observations'].values()):raise ValueError('changed actual corpus bytes or results')
    if native['revision']!=host['revision']:raise ValueError('different executed revisions')
    if native['forced_rebase_observations'] is not None or host['forced_rebase_observations']!=wanted or any(type(x['status']) is not int for x in host['forced_rebase_observations'].values()):raise ValueError('missing exact forced-rebase execution')
    if reference is None and REFERENCE.exists():reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend']!=NATIVE or reference['tracked_tree_clean'] is not True:raise ValueError('invalid frozen native record')
        for key in ['program','pe_sha256','executed_pe_sha256','upstream_source_sha256','upstream_license_sha256','input_sha256','observations']:
            if reference[key]!=native[key]:raise ValueError('changed frozen native evidence: '+key)
    return len(wanted)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+');args=p.parse_args()
    print('Native Windows and Alkyhol agree on '+str(compare([json.loads(p.read_text(encoding='utf-8')) for p in args.reports]))+' unchanged released make2compdb cases')
