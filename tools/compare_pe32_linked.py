#!/usr/bin/env python3
"""Compare complete identical-DLL native and shared-thread PE32 observations."""
import argparse
import json
from pathlib import Path
import re
from run_pe32_linked import NATIVE,HOST,REFERENCE,RESTORED,SOURCES,FILES,LEAF_REFERENCE
from compare_pe32_library import record


def compare(windows,host,reference):
    leaf=json.loads(LEAF_REFERENCE.read_text(encoding='utf-8'))
    for value,backend in [(windows,NATIVE),(host,HOST)]:
        if type(value.get('schema_version')) is not int or value['schema_version']!=1 or value.get('backend')!=backend:raise ValueError('wrong linked-DLL backend/schema')
        if value.get('tracked_tree_clean') is not True or not isinstance(value.get('revision'),str) or not re.fullmatch('[0-9a-f]{40}',value['revision']):raise ValueError('missing clean linked-DLL revision')
        for key,names in [('files',FILES),('sources',SOURCES)]:
            if type(value.get(key)) is not dict or set(value[key])!=set(names) or any(type(d) is not str or not re.fullmatch('[0-9a-f]{64}',d) for d in value[key].values()):raise ValueError('missing exact linked-DLL identities')
            if value[key]!=windows.get(key):raise ValueError('linked-DLL identities differ')
        if value['files']['accepted.dll']!=leaf['files']['accepted.dll'] or any(value['sources'][k]!=v for k,v in leaf['sources'].items()):raise ValueError('frozen dependency changed')
    if windows['revision']!=host['revision'] or windows.get('rebased') is not None:raise ValueError('different revision or invalid native report')
    native=record(windows.get('normal'),81,0,b'')
    if type(reference) is not dict or reference.get('backend')!=NATIVE or reference.get('tracked_tree_clean') is not True or any(reference.get(k)!=windows[k] for k in ['files','sources','normal']):raise ValueError('missing or changed frozen linked-DLL observation')
    for key,forced in [('normal',False),('rebased',True)]:
        if record(host.get(key),81,0,RESTORED+str(forced).lower().encode()+b'\n')!=native:raise ValueError('native/Alkyhol linked-DLL execution differs')
    return 81


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('windows',type=Path);p.add_argument('host',type=Path)
    a=p.parse_args()
    reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    print('Identical PE32 linked DLLs agree on',compare(json.loads(a.windows.read_text(encoding='utf-8')),json.loads(a.host.read_text(encoding='utf-8')),reference),'DWORDs, with shared TEB, named/ordinal code, writable data and dependency lifetime through detach')
