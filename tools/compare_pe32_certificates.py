#!/usr/bin/env python3
"""Compare finite PE32 non-enforced certificate-directory loading, keeping raw errors separate."""
import argparse
import json
from pathlib import Path
import re
from build_pe32_certificates import CASES,SOURCES
from run_pe32_certificates import NATIVE,HOST,REFERENCE

from compare_pe32_geometry import success


def validate(report,native):
    if type(report['schema_version']) is not int or report['schema_version']!=1 or report['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',report['revision']):
        raise ValueError('unsupported schema, dirty tree or invalid revision')
    if not re.fullmatch('[0-9a-f]{64}',report['base_pe_sha256']):raise ValueError('missing frozen base identity')
    if set(report['sources'])!=set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}',v) for v in report['sources'].values()):raise ValueError('missing exact source identity')
    if [row['case'] for row in report['cases']]!=CASES:raise ValueError('changed input cases/order')
    for row in report['cases']:
        if set(row)!={'case','certificate_offset','certificate_size','file_size','pe_sha256'} or any(type(row[key]) is not int or not 0<=row[key]<=0xffffffff for key in ['certificate_offset','certificate_size','file_size']) or not re.fullmatch('[0-9a-f]{64}',row['pe_sha256']):raise ValueError('invalid input certificates identity')
    result=[]
    for key,forced in [('normal',False)]+([] if native else [('rebased',True)]):
        rows=report[key]
        if not isinstance(rows,list) or [row['case'] for row in rows]!=CASES:raise ValueError('dropped/changed observations')
        result.append([success(row,native,forced) for row in rows])
    if not result[0][0]:raise ValueError('unchanged baseline did not execute')
    if native:
        if report.get('rebased') is not None:raise ValueError('invented native forced rebase')
    elif result[0]!=result[1]:raise ValueError('normal/forced-rebase acceptance differs')
    return result[0]


def compare(reports,reference=None):
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:raise ValueError('need independent Windows/Alkyhol certificates reports')
    native=next(r for r in reports if r['backend']==NATIVE);host=next(r for r in reports if r['backend']==HOST)
    accepted=validate(native,True)
    if accepted!=validate(host,False):raise ValueError('native/Alkyhol certificates acceptance differs')
    for field in ['revision','base_pe_sha256','sources','cases']:
        if native[field]!=host[field]:raise ValueError('certificates identity differs: '+field)
    if reference is None:reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend']!=NATIVE:raise ValueError('invalid frozen backend')
        validate(reference,True)
        for field in ['base_pe_sha256','sources','cases','normal']:
            if native[field]!=reference[field]:raise ValueError('changed frozen native certificates evidence: '+field)
    return sum(accepted),len(accepted)-sum(accepted)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+');args=p.parse_args()
    accepted,rejected=compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Identical owned PE32 certificate loading agrees on {accepted} executions and {rejected} launch/preflight rejections; raw OS errors remain separate')
