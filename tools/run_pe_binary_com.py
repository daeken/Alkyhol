#!/usr/bin/env python3
"""Run the identical PE virtual-COM caller on Windows or Alkyhol/Rosetta."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import struct
import subprocess
import tempfile
from build_pe_fixture import ROOT
from build_pe_binary_com import SOURCES

REFERENCE=ROOT/'pe/reference/binary-com-windows-v1.json'


def digest(size):
    value=0x12345678
    for i in range(size): value=((value*33)&0xffffffff)^((i*13+7)&255)
    return value


EXPECTED_WORDS=[0x42434f4d,0,37,digest(37),0,0,0,1,1,37,0x80004002,1,5,4,3,2,1,digest(37),0,0,16384,digest(16384),0]


def decode(run,windows=False,rebase=False):
    if run.returncode!=0 or len(run.stdout)!=4*len(EXPECTED_WORDS):
        raise ValueError(f'virtual COM PE failed/truncated: {run.returncode}, {run.stdout.hex()}, {run.stderr!r}')
    words=list(struct.unpack('<'+'I'*len(EXPECTED_WORDS),run.stdout))
    if words!=EXPECTED_WORDS: raise ValueError(f'actual virtual COM observations differ: {words}')
    if windows:
        if run.stderr: raise ValueError('unexpected native COM diagnostic')
    else:
        match=re.fullmatch(rb'PE64: 4 sections, 4 named imports, 1 DIR64 fixups\nentering Win64 code; relocated=(true|false)\n',run.stderr)
        if not match or (rebase and match[1]!=b'true'):
            raise ValueError('missing exact loader/forced rebase observation')
    return {'status':run.returncode,'words':words,'stdout_hex':run.stdout.hex()}


def observe(executable,windows=False,rebase=False):
    command=[str(executable.resolve())]+([] if windows else ['--loader-info'])+(['--rebase'] if rebase else [])
    with tempfile.TemporaryDirectory(prefix='alkyhol-binary-com-') as directory:
        return decode(subprocess.run(command,cwd=directory,capture_output=True,timeout=30),windows,rebase)


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--fixture',type=Path,required=True)
    p.add_argument('--output',type=Path,required=True)
    args=p.parse_args()
    pe=args.fixture.resolve()
    manifest=json.loads((pe.parent/'binary-com-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES):
        raise ValueError('missing exact binary COM source manifest')
    if hashlib.sha256(pe.read_bytes()).hexdigest()!=manifest['pe_sha256']:
        raise ValueError('binary COM PE identity changed')
    for name,value in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=value:
            raise ValueError('binary COM source identity changed')
    reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['pe_sha256','sources']:
        if manifest[field]!=reference[field]: raise ValueError('changed frozen binary COM fixture')
    windows=platform.system()=='Windows'
    if windows:
        executable=pe
        executed=hashlib.sha256(pe.read_bytes()).hexdigest()
        backend='native Windows PE64 / d3dcompiler_47 COM'
    elif platform.system()=='Darwin':
        from translate_pe import translate,embedded_payload
        executable=translate(pe,ROOT/'local/pe/binary-com/binary-com-macos')
        executed=hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
        backend='Alkyhol Win64 COM / x86-64 Mach-O'
    else: raise RuntimeError('requires Windows or macOS/Rosetta')
    if executed!=manifest['pe_sha256']: raise ValueError('executed COM PE bytes differ')
    normal=observe(executable,windows)
    rebased=None if windows else observe(executable,rebase=True)
    if rebased is not None and rebased!=normal: raise ValueError('rebased COM observations differ')
    report={'schema_version':1,'backend':backend,'pe_sha256':manifest['pe_sha256'],'executed_pe_sha256':executed,
            'sources':manifest['sources'],'observation':normal,'forced_rebase_observation':rebased,
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Passed actual PE virtual COM calls: '+backend)


if __name__=='__main__': main()
