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


def run(manifest,items,output,native_backend=NATIVE,host_backend=HOST,reference_path=REFERENCE):
    reference=json.loads(reference_path.read_text(encoding='utf-8'))
    for field in ['sources','cases']:
        if manifest[field]!=reference[field]:raise ValueError('changed frozen load-config inputs')
    windows=platform.system()=='Windows'
    if not windows and platform.system()!='Darwin':raise RuntimeError('requires Windows or macOS')
    observations=[];rebased=None if windows else []
    old=None;kernel=None
    if windows:
        kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        kernel.SetErrorMode.argtypes,kernel.SetErrorMode.restype=[ctypes.c_uint32],ctypes.c_uint32
        old=kernel.SetErrorMode(0x8003)
    try:
        with tempfile.TemporaryDirectory(prefix='alkyhol-pe32-load-config-') as temp:
            for row,data in items:
                path=Path(temp)/(row['case']+'.exe');path.write_bytes(data)
                executable=path
                if not windows:
                    from translate_pe32 import translate,embedded_payload
                    executable=translate(path,Path(temp)/(row['case']+'-macos'))
                    if hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()!=row['pe_sha256']:
                        raise ValueError('executed load-config payload differs')
                flags=[] if windows else ['--loader-info','--teb-info']
                observations.append({'case':row['case'],**capture([str(executable),*flags],temp)})
                if not windows:
                    rebased.append({'case':row['case'],**capture([str(executable),*flags,'--rebase'],temp)})
    finally:
        if windows:kernel.SetErrorMode(old)
    report=dict(schema_version=1,backend=native_backend if windows else host_backend,
                sources=manifest['sources'],cases=manifest['cases'],normal=observations,rebased=rebased,
                revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                tracked_tree_clean=not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip())
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual legacy load-config/compiler-GS outcomes: '+report['backend'])


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();manifest,items=verify(args.directory.resolve());run(manifest,items,args.output)


if __name__=='__main__':main()
