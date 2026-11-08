#!/usr/bin/env python3
"""Capture unchanged PE32 hardware SEH and explicit host-only profile rejections."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT
from build_pe32_seh import SOURCES,CASES
from compare_pe32_library import record

NATIVE='native Windows PE32 hardware SEH'
HOST='Alkyhol bounded PE32 UD2 SEH / Rosetta'
REFERENCE=ROOT/'pe/reference/pe32-seh-windows-v1.json'
RESTORED=b'PE32 bounded UD2 SEH returned; restored host signals, alternate stack and selectors; forced_rebase='
REJECTIONS={
 'imports':b'Alkyhol PE32 unsupported: imports from an exception handler\n',
 'context':b'Alkyhol PE32 unsupported: exception handler changed unsupported context or registration state\n',
 'nested':b'Alkyhol PE32 unsupported: nested hardware exception\n',
 'unregistered':b'Alkyhol PE32 unsupported: invalid SEH chain or unregistered SafeSEH handler\n'}


def verify(directory):
    m=json.loads((directory/'pe32-seh-fixture.json').read_text(encoding='utf-8'))
    if type(m['schema_version']) is not int or m['schema_version']!=1 or set(m['sources'])!=set(SOURCES):raise ValueError('missing exact SEH source identity')
    if [(v['case'],v['mode']) for v in m['cases']]!=CASES or any(type(v['mode']) is not int for v in m['cases']):raise ValueError('changed SEH case roster')
    for v in m['cases']:
        if hashlib.sha256((directory/(v['case']+'.exe')).read_bytes()).hexdigest()!=v['pe_sha256']:raise ValueError('changed SEH executable bytes')
    for name,digest in m['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed SEH source bytes')
    if REFERENCE.exists():
        reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
        if any(m[k]!=reference[k] for k in ['sources','cases']):raise ValueError('changed frozen SEH inputs')
    return m


def run(directory,output):
    directory=directory.resolve();manifest=verify(directory);windows=platform.system()=='Windows'
    if windows:
        import ctypes
        ctypes.windll.kernel32.SetErrorMode(0x8003)
        backend=NATIVE;probe=None
    elif platform.system()=='Darwin':
        backend=HOST;target='x86_64-apple-darwin'
        cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
        rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
        subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline','--lib'],check=True)
        subprocess.run(['clang','-arch','x86_64','-Wall','-Wextra','-Werror','-c',str(ROOT/SOURCES[-1]),'-o',str(directory/'darwin-layout.o')],check=True)
        build=ROOT/'pe/target'/target/'debug';probe=directory/'probe-macos'
        subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_seh32.rs'),
                        '--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),'-L','dependency='+str(build/'deps'),
                        '-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(probe)],check=True)
    else:raise RuntimeError('requires Windows or macOS/Rosetta')
    def observe(case='pe32-seh',forced=False):
        fixture=directory/(case+'.exe')
        command=[str(fixture)] if windows else [str(probe),str(fixture)]+(['--rebase'] if forced else [])
        r=subprocess.run(command,cwd=directory,capture_output=True,timeout=30)
        if case!='pe32-seh':
            if r.returncode!=78 or r.stdout or r.stderr!=REJECTIONS[case]:raise ValueError(f'profile rejection failed: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
            return {'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
        if r.returncode!=0 or len(r.stdout)!=128:raise ValueError(f'actual SEH failed: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        value={'status':r.returncode,'stdout_hex':r.stdout.hex(),'words':list(struct.unpack('<32I',r.stdout)),'stderr_hex':r.stderr.hex()}
        record(value,32,0,b'' if windows else RESTORED+str(forced).lower().encode()+b'\n')
        return value
    report={'schema_version':1,'backend':backend,'sources':manifest['sources'],'cases':manifest['cases'],
            'executed_cases':['pe32-seh'] if windows else [n for n,_ in CASES],
            'normal':observe(),'rebased':None if windows else observe(forced=True),
            'rejected':None if windows else {n:observe(n) for n in REJECTIONS},
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual PE32 hardware SEH: '+backend)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();run(a.directory,a.output)
