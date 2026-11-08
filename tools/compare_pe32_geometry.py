#!/usr/bin/env python3
"""Compare finite native PE32 geometry acceptance, keeping raw errors separate."""
import argparse
import json
from pathlib import Path
import re
import struct
from build_pe32_geometry import CASES,SOURCES
from run_pe32_geometry import NATIVE,HOST,REFERENCE
from run_pe32_probe import EXPECTED_WORDS

PAYLOAD=struct.pack('<8I',*EXPECTED_WORDS).hex()


def success(row,native,forced=False):
    if row['kind']=='launch_error':
        if not native or set(row)!={'case','kind','winerror','errno'} or type(row['winerror']) is not int or row['winerror']!=193 or type(row['errno']) is not int or row['errno']!=8:
            raise ValueError('not a measured invalid-image CreateProcess rejection')
        return False
    if set(row)!={'case','kind','status','stdout_hex','stderr_hex'} or row['kind']!='process' or type(row['status']) is not int:
        raise ValueError('missing raw process result')
    if not all(isinstance(row[key],str) and re.fullmatch('(?:[0-9a-f]{2})*',row[key]) for key in ['stdout_hex','stderr_hex']):
        raise ValueError('malformed raw output')
    if row['status']==0:
        if row['stdout_hex']!=PAYLOAD:raise ValueError('successful process dropped or changed probe bytes')
        if native:
            if row['stderr_hex']!='':raise ValueError('unexpected native diagnostic')
        else:
            match=re.fullmatch(rb'PE32: 4 sections, 4 imports, 19 HIGHLOW fixups\nchecked i386 mapping; relocated=(true|false)\n',bytes.fromhex(row['stderr_hex']))
            if not match or forced and match[1]!=b'true':raise ValueError('missing exact loader/forced-rebase evidence')
        return True
    # Executed observer failures/crashes never become compatible rejections.
    if native or row['status']!=78 or row['stdout_hex']!='' or not bytes.fromhex(row['stderr_hex']).startswith(b'Alkyhol PE32 unsupported: '):
        raise ValueError('crash, observer failure or unsupported failure shape')
    return False


def validate(report,native):
    if type(report['schema_version']) is not int or report['schema_version']!=1 or report['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',report['revision']):
        raise ValueError('unsupported schema, dirty tree or invalid revision')
    if not re.fullmatch('[0-9a-f]{64}',report['base_pe_sha256']):raise ValueError('missing frozen base identity')
    if set(report['sources'])!=set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}',v) for v in report['sources'].values()):raise ValueError('missing exact source identity')
    if [row['case'] for row in report['cases']]!=CASES:raise ValueError('changed input cases/order')
    for row in report['cases']:
        if set(row)!={'case','declared_size','pe_sha256'} or type(row['declared_size']) is not int or not 0<=row['declared_size']<=0xffffffff or not re.fullmatch('[0-9a-f]{64}',row['pe_sha256']):raise ValueError('invalid input geometry identity')
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
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:raise ValueError('need independent Windows/Alkyhol geometry reports')
    native=next(r for r in reports if r['backend']==NATIVE);host=next(r for r in reports if r['backend']==HOST)
    accepted=validate(native,True)
    if accepted!=validate(host,False):raise ValueError('native/Alkyhol geometry acceptance differs')
    for field in ['revision','base_pe_sha256','sources','cases']:
        if native[field]!=host[field]:raise ValueError('geometry identity differs: '+field)
    if reference is None:reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference is not None:
        if reference['backend']!=NATIVE:raise ValueError('invalid frozen backend')
        validate(reference,True)
        for field in ['base_pe_sha256','sources','cases','normal']:
            if native[field]!=reference[field]:raise ValueError('changed frozen native geometry evidence: '+field)
    return sum(accepted),len(accepted)-sum(accepted)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+');args=p.parse_args()
    accepted,rejected=compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Identical owned PE32 geometry agrees on {accepted} executions and {rejected} launch/preflight rejections; raw OS errors remain separate')
