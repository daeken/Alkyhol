#!/usr/bin/env python3
"""Measure ID3D12Object metadata on an independent native Windows SDK WARP fence.

No local compatibility implementation or predetermined successful return codes
are substituted. The observation is a reference for upcoming COM runtime work.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
CASES=('bytes.missing bytes.set bytes.size bytes.small bytes.exact bytes.large bytes.replace '
       'bytes.replaced bytes.remove bytes.removed bytes.remove_missing bytes.zero_nonnull '
       'bytes.zero_size bytes.zero_read interface.set interface.size interface.small '
       'interface.get interface.release_copy interface.replace_same interface.replace_bytes '
       'interface.bytes interface.restore interface.clear interface.cleared interface.clear_missing '
       'interface.retained_until_destroy interface.release_caller name.set name.size name.read '
       'name.empty name.empty_size object.destroyed').split()

EVENTS={'interface.release_copy','interface.release_caller','object.destroyed'}

def parse_observation(text):
    observation=json.loads(text)
    if not isinstance(observation,dict) or set(observation)!= {'scenario','records'} or observation['scenario']!='d3d12-object-private-data-v1':
        raise ValueError('unknown object observation scenario')
    records=observation['records']
    if not isinstance(records,list) or not all(isinstance(row,dict) for row in records) or [row.get('case') for row in records]!=CASES:
        raise ValueError('missing, duplicate or reordered object observations')
    for row in records:
        if set(row)!= {'case','hr','size','data','refs','adds','releases','destroyed'}:
            raise ValueError('unknown object observation fields')
        if row['case'] in EVENTS:
            if row['hr'] is not None:raise ValueError('diagnostic event cannot manufacture an HRESULT')
        elif type(row['hr']) is not int or not 0<=row['hr']<=0xffffffff:
            raise ValueError('API call requires an actual HRESULT')
        for key in ['size','refs','adds','releases','destroyed']:
            if type(row[key]) is not int or not 0<=row[key]<=0xffffffff:
                raise ValueError('invalid unsigned observation field')
        raw=row['data']
        if not isinstance(raw,str) or len(raw)%2 or len(raw)>128 or any(c not in '0123456789abcdef' for c in raw):
            raise ValueError('invalid exact observation bytes')
    return observation


def run(output):
    if os.name!='nt':raise RuntimeError('native Windows SDK reference required; no emulated observation is accepted')
    source=ROOT/'cpp/tests/object_private_data.cpp'
    inputs=[source,Path(__file__).resolve()]
    before={p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    with tempfile.TemporaryDirectory(prefix='alkyhol-object-reference-') as scratch:
        executable=Path(scratch)/'object_private_data.exe'
        command=['cl','/nologo','/W4','/WX','/EHsc','/std:c++17',str(source),'/Fe:'+str(executable),'/link','d3d12.lib','dxgi.lib','dxguid.lib']
        compiler=subprocess.run(['cl'],capture_output=True,text=True,timeout=30)
        subprocess.run(command,cwd=scratch,check=True,timeout=120)
        digest=hashlib.sha256(executable.read_bytes()).hexdigest()
        result=subprocess.run([str(executable)],capture_output=True,text=True,check=True,cwd=scratch,timeout=30)
        observation=parse_observation(result.stdout)
        adapters=[line for line in result.stderr.splitlines() if line.startswith('adapter: D3D12 WARP ')]
        if len(adapters)!=1:raise RuntimeError('missing or ambiguous actual WARP adapter identity')
        if before!={p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs} or digest!=hashlib.sha256(executable.read_bytes()).hexdigest():
            raise RuntimeError('reference inputs changed during observation')
        report={'schema_version':1,'backend':'native Windows SDK D3D12 WARP object metadata',
                'source_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                'tracked_tree_clean':subprocess.run(['git','diff','--quiet','HEAD'],cwd=ROOT).returncode==0,
                'compiler':compiler.stdout+compiler.stderr,'windows_sdk':os.environ.get('WindowsSDKVersion'),
                'host':platform.platform(),'machine':platform.machine(),'build_command':command,
                'adapter':adapters[0],'artifacts':before,'executable_sha256':digest,'observation':observation}
    output.write_text(json.dumps(report,indent=2)+'\n');return report

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();run(args.output)
