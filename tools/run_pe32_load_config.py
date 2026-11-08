#!/usr/bin/env python3
"""Record actual Windows legacy load-config / compiler GS cases without assuming cookie policy."""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import platform
import struct
import subprocess
import tempfile
from build_pe_fixture import ROOT
from build_pe32_load_config import SOURCES,CASES

NATIVE='native Windows PE32 legacy load configuration'
HOST='Alkyhol PE32 legacy load configuration / x86-64 Mach-O'
REFERENCE=ROOT/'pe/reference/pe32-load-config-windows-v1.json'


def verify(directory):
    manifest=json.loads((directory/'load-config-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES):
        raise ValueError('missing exact load-config source identity')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed load-config source')
    if len(manifest['cases'])!=len(CASES):raise ValueError('dropped load-config cases')
    items=[]
    for row,spec in zip(manifest['cases'],CASES):
        name,size,initial,absent,no_cookie,fault=spec
        expected=dict(case=name,config_size=size,initial_cookie=initial,absent=absent,no_cookie=no_cookie,fault=fault)
        if set(row)!=set(expected)|{'pe_sha256'} or any(type(row[k]) is not type(v) or row[k]!=v for k,v in expected.items()):
            raise ValueError('load-config parameters/order/types changed')
        data=(directory/(name+'.exe')).read_bytes()
        if hashlib.sha256(data).hexdigest()!=row['pe_sha256']:raise ValueError('changed load-config PE')
        items.append((row,data))
    return manifest,items


def capture(command,cwd):
    try:r=subprocess.run(command,cwd=cwd,capture_output=True,timeout=30)
    except OSError as error:
        if type(getattr(error,'winerror',None)) is not int:raise
        return dict(kind='launch_error',winerror=error.winerror,errno=error.errno)
    return dict(kind='process',status=r.returncode,stdout_hex=r.stdout.hex(),stderr_hex=r.stderr.hex())


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();manifest,items=verify(args.directory.resolve())
    if platform.system()!='Windows':raise RuntimeError('initial load-config collector requires Windows')
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    kernel.SetErrorMode.argtypes,kernel.SetErrorMode.restype=[ctypes.c_uint32],ctypes.c_uint32
    old=kernel.SetErrorMode(0x8003);observations=[]
    try:
        with tempfile.TemporaryDirectory(prefix='alkyhol-pe32-load-config-') as temp:
            for row,data in items:
                path=Path(temp)/(row['case']+'.exe');path.write_bytes(data)
                observations.append({'case':row['case'],**capture([str(path)],temp)})
    finally:kernel.SetErrorMode(old)
    baseline=observations[0]
    if baseline['kind']!='process' or baseline['status']!=0 or len(bytes.fromhex(baseline['stdout_hex']))!=64 or baseline['stderr_hex']:
        raise ValueError('unchanged no-config compiler-GS baseline failed')
    report=dict(schema_version=1,backend=NATIVE,sources=manifest['sources'],cases=manifest['cases'],
                normal=observations,rebased=None,
                revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                tracked_tree_clean=not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip())
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded native legacy load-config outcomes; cookie policy/intentional trap comparison is separate')


if __name__=='__main__':main()
