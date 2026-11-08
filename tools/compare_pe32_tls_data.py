#!/usr/bin/env python3
"""Compare actual unchanged PE32 compiler static TLS data initialization and mutation."""
import argparse
import json
from pathlib import Path
import re
import struct
from build_pe32_tls_data import SOURCES
from run_pe32_tls_data import REFERENCE,EXPECTED_WORDS,HOST_DIAGNOSTIC,RESTORED,REPEAT_ERROR,NATIVE,HOST


def validate(report,native):
    if type(report['schema_version']) is not int or report['schema_version']!=1 or report['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',report['revision']):
        raise ValueError('unsupported schema, dirty checkout or invalid revision')
    if not re.fullmatch('[0-9a-f]{64}',report['pe_sha256']) or report['executed_pe_sha256']!=report['pe_sha256']:
        raise ValueError('executed PE32 TLS data bytes differ')
    if set(report['sources'])!=set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}',v) for v in report['sources'].values()):
        raise ValueError('missing exact TLS data source identity')
    for key in ['normal']+([] if native else ['rebased','repeat_rejected']):
        row=report[key]
        if set(row)!={'status','stdout_hex','words','stderr_hex'} or type(row['status']) is not int or row['status']!=(78 if key=='repeat_rejected' else 0) or row['words']!=EXPECTED_WORDS or any(type(v) is not int for v in row['words']) or row['stdout_hex']!=struct.pack('<24I',*EXPECTED_WORDS).hex():
            raise ValueError('actual TLS data output/status differs')
        diagnostic=bytes.fromhex(row['stderr_hex'])
        if native:
            if diagnostic:raise ValueError('unexpected native TLS data diagnostic')
        else:
            match=re.fullmatch(HOST_DIAGNOSTIC+re.escape(RESTORED)+(re.escape(REPEAT_ERROR) if key=='repeat_rejected' else b''),diagnostic)
            if not match or (key=='rebased' and match[1]!=b'true'):raise ValueError('missing host TLS data/forced-relocation evidence')
    if native and (report.get('rebased') is not None or report.get('repeat_rejected') is not None):raise ValueError('invented native rebase')


def compare(reports,reference=None):
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:raise ValueError('need native Windows and Alkyhol TLS data reports')
    native=next(r for r in reports if r['backend']==NATIVE);host=next(r for r in reports if r['backend']==HOST)
    validate(native,True);validate(host,False)
    for field in ['revision','pe_sha256','executed_pe_sha256','sources']:
        if native[field]!=host[field]:raise ValueError('TLS data identity mismatch: '+field)
    if reference is None:reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference['backend']!=NATIVE:raise ValueError('invalid frozen native backend')
    validate(reference,True)
    for field in ['pe_sha256','executed_pe_sha256','sources','normal']:
        if native[field]!=reference[field]:raise ValueError('changed frozen native TLS data evidence: '+field)
    return len(EXPECTED_WORDS)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+');args=p.parse_args()
    count=compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Identical PE32 compiler TLS data agrees on all {count} DWORDs; host repeat is explicitly rejected')
