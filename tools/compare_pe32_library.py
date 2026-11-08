#!/usr/bin/env python3
"""Require identical DLL inputs and complete native Windows/Alkyhol observations."""
import argparse
import json
from pathlib import Path
import re
import struct
from run_pe32_library import NATIVE,HOST,REFERENCE,RESTORED,MISMATCH,SOURCES,FILES


def record(value,count,status,diagnostic):
    if type(value) is not dict or type(value.get('status')) is not int or value['status']!=status:
        raise ValueError('missing/failed DLL process record')
    words=value.get('words')
    if type(words) is not list or len(words)!=count or any(type(w) is not int or not 0<=w<=0xffffffff for w in words):
        raise ValueError('missing raw DWORD records')
    if value.get('stdout_hex')!=struct.pack('<'+'I'*count,*words).hex() or value.get('stderr_hex')!=diagnostic.hex():
        raise ValueError('raw DLL output/diagnostic differs from declared record')
    return words


def compare(windows,host,reference):
    for value,backend in [(windows,NATIVE),(host,HOST)]:
        if type(value.get('schema_version')) is not int or value['schema_version']!=1 or value.get('backend')!=backend:
            raise ValueError('wrong PE32 DLL backend/schema')
        if value.get('tracked_tree_clean') is not True or not re.fullmatch('[0-9a-f]{40}',value.get('revision','')):
            raise ValueError('missing clean DLL revision')
        for key,names in [('sources',SOURCES),('files',FILES)]:
            if type(value.get(key)) is not dict or set(value[key])!=set(names) or any(not isinstance(d,str) or not re.fullmatch('[0-9a-f]{64}',d) for d in value[key].values()):
                raise ValueError('missing exact DLL identities')
            if value[key]!=windows.get(key):raise ValueError('DLL identities differ')
    if windows['revision']!=host['revision']:raise ValueError('different execution revisions')
    if windows.get('rebased') is not None or windows.get('mismatch_rejected') is not None:
        raise ValueError('unexpected native-only host diagnostics')
    native=record(windows.get('normal'),80,0,b'')
    if reference is not None:
        if reference.get('backend')!=NATIVE or reference.get('tracked_tree_clean') is not True or any(reference.get(k)!=windows[k] for k in ['sources','files','normal']):
            raise ValueError('frozen native DLL observation changed')
    for key,forced in [('normal',False),('rebased',True)]:
        if record(host.get(key),80,0,RESTORED+str(forced).lower().encode()+b'\n')!=native:
            raise ValueError('actual native/Alkyhol DLL lifecycle or exports differ')
    # The intentional wrong calling convention must fail after a real cdecl
    # call and still detach once. It is a host ABI-negative, not a native crash.
    bad=record(host.get('mismatch_rejected'),32,78,MISMATCH)
    expected=native[:12]+native[20:28]+native[36:48]
    expected[12+3]=1 # Only the cdecl export ran, rather than the successful trio.
    expected[20+9]=1 # DETACH observes that single genuine call.
    if bad!=expected:raise ValueError('wrong-convention execution or cleanup changed')
    return 80


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('windows',type=Path);p.add_argument('host',type=Path)
    p.add_argument('--initial-capture',action='store_true',help='first native observation only; forbidden after freezing')
    a=p.parse_args()
    if a.initial_capture and REFERENCE.exists():raise ValueError('cannot bypass the frozen DLL reference')
    reference=None if a.initial_capture else json.loads(REFERENCE.read_text(encoding='utf-8'))
    print('Identical PE32 DLLs agree on',compare(json.loads(a.windows.read_text(encoding='utf-8')),json.loads(a.host.read_text(encoding='utf-8')),reference),'DWORDs, including failed attach/detach; host rejects a real stack-cleanup mismatch')
