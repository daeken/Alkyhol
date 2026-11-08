#!/usr/bin/env python3
"""Record native Windows/Alkyhol outcomes for identical owned certificate-directory variants.

Evidence capture only: comparison is separate; crashes never imply successful execution.
"""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import tempfile
from build_pe_fixture import ROOT
from build_pe32_certificates import BASE_REFERENCE,SOURCES,variants
from run_pe32_probe import EXPECTED_WORDS
import struct

NATIVE='native Windows PE32 image certificates'
HOST='Alkyhol PE32 image certificates / x86-64 Mach-O'
REFERENCE=ROOT/'pe/reference/pe32-certificates-windows-v1.json'


def verify(directory):
    manifest=json.loads((directory/'certificates-fixture.json').read_text(encoding='utf-8'))
    base=(directory/'base.exe').read_bytes()
    frozen=json.loads(BASE_REFERENCE.read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or manifest['base_pe_sha256']!=frozen['pe_sha256'] or hashlib.sha256(base).hexdigest()!=frozen['pe_sha256']:
        raise ValueError('changed frozen owned base input')
    if set(manifest['sources'])!=set(SOURCES):raise ValueError('missing exact certificates source identities')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed certificates source')
    for name,digest in frozen['sources'].items():
        if manifest['sources'][name]!=digest:raise ValueError('changed frozen original source')
    offset,items=variants(base)
    if manifest['directory_offset']!=offset or manifest['cases']!=[row for row,_ in items]:raise ValueError('certificates case identity/order changed')
    for row,data in items:
        if (directory/(row['case']+'.exe')).read_bytes()!=data:raise ValueError('more than the declared header bytes changed')
    return manifest,items


def capture(command,cwd):
    try:run=subprocess.run(command,cwd=cwd,capture_output=True,timeout=30)
    except OSError as error:
        # No child ran. Preserve the actual native CreateProcess failure codes.
        if type(getattr(error,'winerror',None)) is not int:raise
        return {'kind':'launch_error','winerror':error.winerror,'errno':error.errno}
    return {'kind':'process','status':run.returncode,'stdout_hex':run.stdout.hex(),'stderr_hex':run.stderr.hex()}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args()
    system=platform.system()
    if system!='Windows':raise RuntimeError('initial certificate collector requires Windows')
    manifest,items=verify(args.directory.resolve())
    if system=='Windows':
        backend=NATIVE
        kernel=ctypes.WinDLL('kernel32',use_last_error=True)
        kernel.SetErrorMode.argtypes,kernel.SetErrorMode.restype=[ctypes.c_uint32],ctypes.c_uint32
        old=kernel.SetErrorMode(0x8003)
        observations=[]
        try:
            with tempfile.TemporaryDirectory(prefix='alkyhol-pe32-certificates-') as temp:
                for row,data in items:
                    path=Path(temp)/(row['case']+'.exe');path.write_bytes(data)
                    observations.append({'case':row['case'],**capture([str(path)],temp)})
        finally:kernel.SetErrorMode(old)
        baseline={'case':'absent','kind':'process','status':0,'stdout_hex':struct.pack('<8I',*EXPECTED_WORDS).hex(),'stderr_hex':''}
        if observations[0]!=baseline:raise ValueError('unchanged native baseline failed')
        rebased=None
    report={'schema_version':1,'backend':backend,'base_pe_sha256':manifest['base_pe_sha256'],
            'sources':manifest['sources'],'cases':manifest['cases'],'normal':observations,'rebased':rebased,
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded eight actual loader outcomes; parity requires the separate comparator: '+backend)


if __name__=='__main__':main()
