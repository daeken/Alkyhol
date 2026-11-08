#!/usr/bin/env python3
"""Compare actual bounded SEH observations without treating crashes as recovery."""
import argparse
import json
from pathlib import Path
import re
from run_pe32_seh import NATIVE,HOST,REFERENCE,RESTORED,REJECTIONS,SOURCES,CASES
from compare_pe32_library import record


def compare(windows,host,reference):
    for value,backend,executed in [(windows,NATIVE,['pe32-seh']),(host,HOST,[n for n,_ in CASES])]:
        if type(value.get('schema_version')) is not int or value['schema_version']!=1 or value.get('backend')!=backend:raise ValueError('wrong SEH backend/schema')
        if value.get('tracked_tree_clean') is not True or type(value.get('revision')) is not str or not re.fullmatch('[0-9a-f]{40}',value['revision']):raise ValueError('missing clean SEH revision')
        if value.get('executed_cases')!=executed:raise ValueError('missing actual executed SEH case identities')
        if type(value.get('sources')) is not dict or set(value['sources'])!=set(SOURCES) or any(type(d) is not str or not re.fullmatch('[0-9a-f]{64}',d) for d in value['sources'].values()):raise ValueError('missing exact SEH source identities')
        cases=value.get('cases')
        if type(cases) is not list or [(v.get('case'),v.get('mode')) for v in cases]!=CASES or any(type(v.get('mode')) is not int or type(v.get('pe_sha256')) is not str or not re.fullmatch('[0-9a-f]{64}',v['pe_sha256']) for v in cases):raise ValueError('missing exact SEH PE identities')
        if value['sources']!=windows['sources'] or value['cases']!=windows['cases']:raise ValueError('SEH inputs differ')
    if windows['revision']!=host['revision'] or windows.get('rebased') is not None or windows.get('rejected') is not None:raise ValueError('different revision or fabricated native negative results')
    native=record(windows.get('normal'),32,0,b'')
    if reference is not None and (reference.get('backend')!=NATIVE or reference.get('tracked_tree_clean') is not True or any(reference.get(k)!=windows[k] for k in ['sources','cases','normal','executed_cases'])):raise ValueError('frozen native SEH evidence changed')
    for key,forced in [('normal',False),('rebased',True)]:
        if record(host.get(key),32,0,RESTORED+str(forced).lower().encode()+b'\n')!=native:raise ValueError('actual native/Alkyhol SEH differs')
    rejected=host.get('rejected')
    if type(rejected) is not dict or set(rejected)!=set(REJECTIONS):raise ValueError('missing explicit profile rejections')
    for name,diagnostic in REJECTIONS.items():
        value=rejected[name]
        if type(value.get('status')) is not int or value['status']!=78 or value.get('stdout_hex')!='' or value.get('stderr_hex')!=diagnostic.hex():raise ValueError('crash/changed rejection is not an explicit unsupported result')
    return 32


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('windows',type=Path);p.add_argument('host',type=Path);p.add_argument('--initial-capture',action='store_true');a=p.parse_args()
    if a.initial_capture and REFERENCE.exists():raise ValueError('cannot bypass frozen SEH reference')
    frozen=None if a.initial_capture else json.loads(REFERENCE.read_text(encoding='utf-8'))
    print('Identical PE32 SafeSEH inputs agree on',compare(json.loads(a.windows.read_text(encoding='utf-8')),json.loads(a.host.read_text(encoding='utf-8')),frozen),'DWORDs of real search, integer/PC/flags continuation and FS state; four host-only restrictions reject explicitly')
