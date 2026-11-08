#!/usr/bin/env python3
"""Compare actual unchanged PE32 self-module/export lookup, calls and data access."""
import argparse
import json
from pathlib import Path
import re
import struct
from build_pe32_exports import SOURCES
from run_pe32_exports import REFERENCE,EXPECTED_WORDS,HOST_DIAGNOSTIC,NATIVE,HOST


def validate(report,native):
    if type(report['schema_version']) is not int or report['schema_version']!=1 or report['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',report['revision']):
        raise ValueError('unsupported schema, dirty checkout or invalid revision')
    if not re.fullmatch('[0-9a-f]{64}',report['pe_sha256']) or report['executed_pe_sha256']!=report['pe_sha256']:
        raise ValueError('executed PE32 self-export bytes differ')
    if set(report['sources'])!=set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}',v) for v in report['sources'].values()):
        raise ValueError('missing exact self-export source identity')
    for key in ['normal']+([] if native else ['rebased']):
        row=report[key]
        if set(row)!={'status','stdout_hex','words','stderr_hex'} or type(row['status']) is not int or row['status']!=0 or row['words']!=EXPECTED_WORDS or any(type(v) is not int for v in row['words']) or row['stdout_hex']!=struct.pack('<26I',*EXPECTED_WORDS).hex():
            raise ValueError('actual self-export output/status differs')
        diagnostic=bytes.fromhex(row['stderr_hex'])
        if native:
            if diagnostic:raise ValueError('unexpected native self-export diagnostic')
        else:
            match=re.fullmatch(HOST_DIAGNOSTIC,diagnostic)
            if not match or (key=='rebased' and match[1]!=b'true'):raise ValueError('missing host self-export/forced-relocation evidence')
    if native and report.get('rebased') is not None:raise ValueError('invented native rebase')


def compare(reports,reference=None):
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:raise ValueError('need native Windows and Alkyhol self-export reports')
    native=next(r for r in reports if r['backend']==NATIVE);host=next(r for r in reports if r['backend']==HOST)
    validate(native,True);validate(host,False)
    for field in ['revision','pe_sha256','executed_pe_sha256','sources']:
        if native[field]!=host[field]:raise ValueError('self-export identity mismatch: '+field)
    if reference is None:reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference['backend']!=NATIVE:raise ValueError('invalid frozen native backend')
    validate(reference,True)
    for field in ['pe_sha256','executed_pe_sha256','sources','normal']:
        if native[field]!=reference[field]:raise ValueError('changed frozen native self-export evidence: '+field)
    return len(EXPECTED_WORDS)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+');args=p.parse_args()
    count=compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Identical PE32 self-export calls/data/last-error observations agree on all {count} DWORDs')
