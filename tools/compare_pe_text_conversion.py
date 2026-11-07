#!/usr/bin/env python3
"""Compare all direct-call UTF conversion observation bytes on two backends."""
import argparse
import json
from pathlib import Path
import re
from run_pe_text_conversion import REFERENCE,NATIVE,HOST,WORD_COUNT,validate
from build_pe_text_conversion import SOURCES


def compare(reports,reference=None):
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:raise ValueError('need native Windows and Alkyhol observations')
    native=next(r for r in reports if r['backend']==NATIVE);host=next(r for r in reports if r['backend']==HOST)
    for r in reports:
        if type(r['schema_version']) is not int or r['schema_version']!=1 or r['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',r['revision']):raise ValueError('invalid schema/revision or dirty tree')
        if not re.fullmatch('[0-9a-f]{64}',r['pe_sha256']) or r['pe_sha256']!=r['executed_pe_sha256'] or set(r['sources'])!=set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}',v) for v in r['sources'].values()):raise ValueError('invalid input/executed identity')
        validate(r['normal'])
    for key in ['revision','pe_sha256','sources']:
        if native[key]!=host[key]:raise ValueError('different input identity: '+key)
    if native['normal']!=host['normal']:
        differences=[(i,a,b) for i,(a,b) in enumerate(zip(native['normal']['words'],host['normal']['words'])) if a!=b]
        raise ValueError('native/Alkyhol UTF observations differ: '+repr(differences[:12]))
    if native['rebased'] is not None or validate(host['rebased'])!=native['normal']:raise ValueError('missing exact forced-rebase observations')
    if reference is None and REFERENCE.exists():reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend']!=NATIVE or reference['tracked_tree_clean'] is not True:raise ValueError('invalid frozen native record')
        for key in ['pe_sha256','sources','normal']:
            if native[key]!=reference[key]:raise ValueError('changed frozen native text-conversion evidence: '+key)
    return WORD_COUNT


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+');args=p.parse_args()
    print('Native Windows and Alkyhol agree on all '+str(compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports]))+' UTF observation words')
