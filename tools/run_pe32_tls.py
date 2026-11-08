#!/usr/bin/env python3
"""Capture compiler-generated PE32 static TLS initialization on native Windows."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import struct
import subprocess
import tempfile
from build_pe_fixture import ROOT
from build_pe32_tls import SOURCES

NATIVE='native Windows PE32 static TLS'
HOST='Alkyhol PE32 static TLS / x86-64 Mach-O'
REFERENCE=ROOT/'pe/reference/pe32-tls-windows-v1.json'


def verify(pe):
    manifest=json.loads((pe.parent/'pe32-tls-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES):
        raise ValueError('missing exact PE32 TLS source identity')
    if hashlib.sha256(pe.read_bytes()).hexdigest()!=manifest['pe_sha256']:raise ValueError('PE32 TLS executable changed')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('PE32 TLS source changed')
    return manifest


def decode(run):
    if run.returncode!=0 or len(run.stdout)!=96 or run.stderr:
        raise ValueError(f'PE32 TLS observer failed: {run.returncode}, {run.stdout.hex()}, {run.stderr!r}')
    return {'status':run.returncode,'stdout_hex':run.stdout.hex(),
            'words':list(struct.unpack('<24I',run.stdout)),'stderr_hex':run.stderr.hex()}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();pe=args.fixture.resolve();manifest=verify(pe)
    if platform.system()!='Windows':raise RuntimeError('initial TLS collector requires native Windows')
    with tempfile.TemporaryDirectory(prefix='alkyhol-pe32-tls-') as temp:
        normal=decode(subprocess.run([str(pe)],cwd=temp,capture_output=True,timeout=30))
    report={'schema_version':1,'backend':NATIVE,'pe_sha256':manifest['pe_sha256'],
            'executed_pe_sha256':hashlib.sha256(pe.read_bytes()).hexdigest(),'sources':manifest['sources'],
            'normal':normal,'rebased':None,
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual native PE32 compiler TLS initialization; host comparison is separate')


if __name__=='__main__':main()
