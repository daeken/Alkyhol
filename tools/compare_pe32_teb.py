#!/usr/bin/env python3
"""Compare actual unchanged PE32 FS/TEB self/stack/last-error reads and writes."""
import argparse
import json
from pathlib import Path
import re
import struct
from build_pe32_teb import SOURCES
from run_pe32_teb import REFERENCE,EXPECTED_WORDS,HOST_DIAGNOSTIC,RESTORED,NATIVE,HOST


def validate(report,native):
    if type(report['schema_version']) is not int or report['schema_version']!=1 or report['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',report['revision']):
        raise ValueError('unsupported schema, dirty checkout or invalid revision')
    if not re.fullmatch('[0-9a-f]{64}',report['pe_sha256']) or report['executed_pe_sha256']!=report['pe_sha256']:
        raise ValueError('executed PE32 TEB bytes differ')
    if set(report['sources'])!=set(SOURCES) or not all(re.fullmatch('[0-9a-f]{64}',v) for v in report['sources'].values()):
        raise ValueError('missing exact TEB source identity')
    for key in ['normal']+([] if native else ['rebased','repeated']):
        row=report[key];expected=EXPECTED_WORDS*(2 if key=='repeated' else 1)
        if set(row)!={'status','stdout_hex','words','stderr_hex'} or type(row['status']) is not int or row['status']!=0 or row['words']!=expected or any(type(v) is not int for v in row['words']) or row['stdout_hex']!=struct.pack('<'+str(len(expected))+'I',*expected).hex():
            raise ValueError('actual TEB output/status differs')
        diagnostic=bytes.fromhex(row['stderr_hex'])
        if native:
            if diagnostic:raise ValueError('unexpected native TEB diagnostic')
        else:
            match=re.fullmatch(HOST_DIAGNOSTIC+re.escape(RESTORED)*(2 if key=='repeated' else 1),diagnostic)
            if not match or (key=='rebased' and match[1]!=b'true'):raise ValueError('missing host TEB/forced-relocation evidence')
    if native and (report.get('rebased') is not None or report.get('repeated') is not None):raise ValueError('invented native rebase')


def compare(reports,reference=None):
    if len(reports)!=2 or {r['backend'] for r in reports}!={NATIVE,HOST}:raise ValueError('need native Windows and Alkyhol TEB reports')
    native=next(r for r in reports if r['backend']==NATIVE);host=next(r for r in reports if r['backend']==HOST)
    validate(native,True);validate(host,False)
    for field in ['revision','pe_sha256','executed_pe_sha256','sources']:
        if native[field]!=host[field]:raise ValueError('TEB identity mismatch: '+field)
    if reference is None:reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    if reference['backend']!=NATIVE:raise ValueError('invalid frozen native backend')
    validate(reference,True)
    for field in ['pe_sha256','executed_pe_sha256','sources','normal']:
        if native[field]!=reference[field]:raise ValueError('changed frozen native TEB evidence: '+field)
    return len(EXPECTED_WORDS)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+');args=p.parse_args()
    count=compare([json.loads(path.read_text(encoding='utf-8')) for path in args.reports])
    print(f'Identical PE32 FS/TEB observations agree on all {count} DWORDs; host repeat and segment restoration pass')
