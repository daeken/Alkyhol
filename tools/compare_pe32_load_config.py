#!/usr/bin/env python3
"""Compare finite legacy load-config properties and controlled compiler-GS failure."""
import argparse
import json
from pathlib import Path
import re
from build_pe32_load_config import SOURCES
from run_pe32_load_config import NATIVE,HOST,REFERENCE
from build_pe32_load_config_geometry import SOURCES as GEOMETRY_SOURCES
from run_pe32_load_config_geometry import NATIVE as GNATIVE,HOST as GHOST,REFERENCE as GREFERENCE

RESTORED=b'host segment selectors restored; owned FS descriptor cleared\n'


def validate(report,native,reference):
    geometry=reference['backend']==GNATIVE
    if reference['backend'] not in [NATIVE,GNATIVE] or report['backend']!=((GNATIVE if native else GHOST) if geometry else (NATIVE if native else HOST)):
        raise ValueError('wrong load-config evidence backend')
    if type(report['schema_version']) is not int or report['schema_version']!=1 or report['tracked_tree_clean'] is not True or not re.fullmatch('[0-9a-f]{40}',report['revision']):
        raise ValueError('invalid schema, revision or dirty tree')
    sources=GEOMETRY_SOURCES if geometry else SOURCES
    if set(report['sources'])!=set(sources) or report['sources']!=reference['sources'] or not all(re.fullmatch('[0-9a-f]{64}',v) for v in report['sources'].values()):
        raise ValueError('changed frozen source identities')
    if len(report['cases'])!=len(reference['cases']):raise ValueError('dropped input cases')
    for row,expected in zip(report['cases'],reference['cases']):
        if set(row)!=set(expected) or any(type(row[k]) is not type(v) or row[k]!=v for k,v in expected.items()):raise ValueError('changed case parameters/types/payload')
        if not re.fullmatch('[0-9a-f]{64}',row['pe_sha256']):raise ValueError('invalid payload identity')
    for key in ['normal']+([] if native else ['rebased']):
        rows=report[key]
        if not isinstance(rows,list) or len(rows)!=len(reference['normal']):raise ValueError('missing actual observations')
        for row,expected in zip(rows,reference['normal']):
            if set(row)!={'case','kind','status','stdout_hex','stderr_hex'} or row['case']!=expected['case'] or row['kind']!='process' or type(row['status']) is not int:
                raise ValueError('invalid actual process observation')
            fault=expected['case']=='corrupt_frame' and not geometry
            if fault:
                if expected['status']!=0xc000001d or expected['stdout_hex']!='4933474604000000':raise ValueError('invalid frozen deliberate-trap record')
                status=0xc000001d if native else -4
            else:
                if expected['status']!=0 or len(bytes.fromhex(expected['stdout_hex']))!=64:raise ValueError('invalid frozen successful record')
                status=0
            if row['status']!=status or row['stdout_hex']!=expected['stdout_hex']:raise ValueError('actual compiler-GS/property output differs')
            if native:
                if row['stderr_hex']!=expected['stderr_hex'] or expected['stderr_hex']:raise ValueError('unexpected native diagnostic')
            else:
                imports,fixups=(2,10) if fault else (4,29 if row['case']=='absent' else 37 if row['case']=='no_cookie' else 38)
                prefix=f'PE32: 4 sections, {imports} imports, {fixups} HIGHLOW fixups\nchecked i386 mapping; relocated='.encode()
                match=re.fullmatch(re.escape(prefix)+rb'(true|false)\n'+(b'' if fault else re.escape(RESTORED)),bytes.fromhex(row['stderr_hex']))
                if not match or (key=='rebased' and match[1]!=b'true'):raise ValueError('missing loader/relocation/cleanup evidence')
    if native and report.get('rebased') is not None:raise ValueError('invented native forced relocation')


def compare(reports,reference=None):
    backends={r['backend'] for r in reports}
    if len(reports)!=2 or backends not in [{NATIVE,HOST},{GNATIVE,GHOST}]:raise ValueError('need independent matching native/host reports')
    geometry=GNATIVE in backends
    if reference is None:reference=json.loads((GREFERENCE if geometry else REFERENCE).read_text(encoding='utf-8'))
    validate(reference,True,reference)
    native=next(r for r in reports if r['backend'] in [NATIVE,GNATIVE]);host=next(r for r in reports if r['backend'] in [HOST,GHOST])
    validate(native,True,reference);validate(host,False,reference)
    if native['revision']!=host['revision']:raise ValueError('clean code revisions differ')
    positives=sum(row['status']==0 for row in reference['normal'])
    return positives,0 if geometry else 1


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('reports',type=Path,nargs='+');a=p.parse_args()
    successful,traps=compare([json.loads(path.read_text(encoding='utf-8')) for path in a.reports])
    print(f'Identical PE32 load-config inputs agree on {successful} successful property/GS records and {traps} deliberate compiler-check trap; raw OS trap statuses remain separate')
