#!/usr/bin/env python3
"""Require complete identical-binary static-process observations on Windows/Mac."""
import argparse
import json
from pathlib import Path
import re
import struct
from run_pe32_process import NATIVE,HOST,REFERENCE,RESTORED,REJECTIONS,SOURCES,FILES
from compare_pe32_library import record

def compare(windows,host,reference):
    for value,backend in [(windows,NATIVE),(host,HOST)]:
        if type(value.get('schema_version')) is not int or value['schema_version']!=1 or value.get('backend')!=backend:raise ValueError('wrong process backend/schema')
        if value.get('tracked_tree_clean') is not True or type(value.get('revision')) is not str or not re.fullmatch('[0-9a-f]{40}',value['revision']):raise ValueError('missing clean process revision')
        for key,names in [('sources',SOURCES),('files',FILES)]:
            if type(value.get(key)) is not dict or set(value[key])!=set(names) or any(type(d) is not str or not re.fullmatch('[0-9a-f]{64}',d) for d in value[key].values()):raise ValueError('missing exact process identities')
            if value[key]!=windows.get(key):raise ValueError('different process inputs')
    if windows['revision']!=host['revision'] or windows.get('rebased') is not None or windows.get('rejected') is not None:raise ValueError('different process revisions or fabricated native restriction tests')
    native=record(windows.get('normal'),105,7,b'')
    if type(reference) is not dict or reference.get('backend')!=NATIVE or reference.get('tracked_tree_clean') is not True or any(reference.get(k)!=windows[k] for k in ['files','sources','normal']):raise ValueError('missing or changed frozen process evidence')
    for key,forced in [('normal',False),('rebased',True)]:
        if record(host.get(key),105,7,RESTORED+str(forced).lower().encode()+b'\n')!=native:raise ValueError('actual native/Alkyhol static startup, calls or teardown differ')
    rejected=host.get('rejected')
    if type(rejected) is not dict or set(rejected)!=set(REJECTIONS):raise ValueError('missing exact process restrictions')
    for mode,diagnostic in REJECTIONS.items():
        value=rejected[mode]
        if type(value.get('status')) is not int or value['status']!=78 or value.get('stderr_hex')!=diagnostic.hex():raise ValueError('crash or changed process rejection')
        if mode!='failed':
            if value.get('stdout_hex')!='':raise ValueError('guest initialization ran before complete process preflight')
        else:
            # Frozen dependency, one genuine failing DLL, immediate failed detach,
            # then the acquired dependency rolls back. This is a host restriction
            # assertion, not a claim about Windows failed static startup behavior.
            words=[0x4c323349,0,1,0,1,4,1,1,0x12345678,0,0x31415926,1,
                   0x43323349,3,4,1,0x31415926,0x11223344,0x55667788,0x99aabbcc,
                   0x4641494c,1,1,0x31415926,4,1,
                   0x43323349,3,3,2,0x31415926,0x11223344,0x55667788,0x99aabbcc,
                   0x4641494c,0,0,0x31415926,3,1,
                   0x4c323349,0,0,1,1,4,1,1,0x12345678,2,0x31415926,1]
            if value.get('stdout_hex')!=struct.pack('<52I',*words).hex():raise ValueError('failed initialization rollback sequence differs')
    return 105

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('windows',type=Path);p.add_argument('host',type=Path);p.add_argument('--initial-native-capture',action='store_true');a=p.parse_args()
    windows=json.loads(a.windows.read_text(encoding='utf-8'));host=json.loads(a.host.read_text(encoding='utf-8'))
    reference=windows if a.initial_native_capture else json.loads(REFERENCE.read_text(encoding='utf-8'))
    print('Identical PE32 EXE/DLL process inputs agree on',compare(windows,host,reference),'DWORDs of real static startup, binary calls and process termination')
