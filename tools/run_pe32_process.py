#!/usr/bin/env python3
"""Capture actual static PE32 EXE/DLL startup, binary calls and process teardown."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT
from build_pe32_process import SOURCES,FILES
from run_pe32_linked import REFERENCE as LINKED_REFERENCE
from compare_pe32_library import record

NATIVE='native Windows i386 static EXE/DLL process'
HOST='Alkyhol closed PE32 EXE/DLL process / Rosetta'
REFERENCE=ROOT/'pe/reference/pe32-process-windows-v1.json'
RESTORED=b'PE32 process statically attached once, terminated and restored host selectors; forced_rebase='
REJECTIONS={
 'missing':b'Alkyhol PE32 unsupported: missing DLL from PE32 process: terminate.dll\n',
 'duplicate':b'Alkyhol PE32 unsupported: duplicate PE32 process DLL basename\n',
 'unused':b'Alkyhol PE32 unsupported: unreferenced DLL in explicit PE32 process\n',
 'symbol':b'Alkyhol PE32 unsupported: missing PE32 process export Name("Run")\n',
 'failed':b'Alkyhol PE32 unsupported: PE32 static DLL PROCESS_ATTACH returned FALSE\n'}

def verify(directory):
    manifest=json.loads((directory/'pe32-process-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['files'])!=set(FILES) or set(manifest['sources'])!=set(SOURCES):raise ValueError('missing process input identities')
    previous=json.loads(LINKED_REFERENCE.read_text(encoding='utf-8'))
    for name in ['accepted.dll','consumer.dll']:
        if manifest['files'][name]!=previous['files'][name]:raise ValueError('frozen process dependency changed')
    for name,digest in previous['sources'].items():
        if manifest['sources'].get(name)!=digest:raise ValueError('frozen process dependency source changed')
    for name,digest in manifest['files'].items():
        if hashlib.sha256((directory/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed process PE bytes')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed process source bytes')
    if REFERENCE.exists():
        reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
        if any(manifest[k]!=reference[k] for k in ['sources','files']):raise ValueError('changed frozen process inputs')
    return manifest

def run(directory,output):
    directory=directory.resolve();manifest=verify(directory);windows=platform.system()=='Windows'
    if windows:
        import ctypes
        ctypes.windll.kernel32.SetErrorMode(0x8003)
        executable=directory/'program.exe';backend=NATIVE
    elif platform.system()=='Darwin':
        target='x86_64-apple-darwin';backend=HOST
        cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
        rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
        subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline','--lib'],check=True)
        build=ROOT/'pe/target'/target/'debug';executable=directory/'probe-macos'
        subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_process32.rs'),'--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),'-L','dependency='+str(build/'deps'),'-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(executable)],check=True)
    else:raise RuntimeError('requires Windows or macOS/Rosetta')
    def observe(forced=False,mode=None):
        command=[str(executable)] if windows else [str(executable),str(directory)]+(['--'+mode] if mode else ['--rebase'] if forced else [])
        r=subprocess.run(command,cwd=directory,capture_output=True,timeout=30)
        if mode:
            if r.returncode!=78 or r.stderr!=REJECTIONS[mode] or (mode!='failed' and r.stdout):raise ValueError(f'invalid process rejection {mode}: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
            if mode=='failed' and len(r.stdout)!=208:raise ValueError('missing real failed initializer rollback sequence')
            return {'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
        if r.returncode!=7 or len(r.stdout)!=420:raise ValueError(f'actual process execution failed: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        value={'status':r.returncode,'stdout_hex':r.stdout.hex(),'words':list(struct.unpack('<105I',r.stdout)),'stderr_hex':r.stderr.hex()}
        record(value,105,7,b'' if windows else RESTORED+str(forced).lower().encode()+b'\n')
        return value
    report={'schema_version':1,'backend':backend,'files':manifest['files'],'sources':manifest['sources'],'normal':observe(),'rebased':None if windows else observe(True),
            'rejected':None if windows else {mode:observe(mode=mode) for mode in REJECTIONS},
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual PE32 static process startup/calls/termination: '+backend)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();run(a.directory,a.output)
