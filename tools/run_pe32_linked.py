#!/usr/bin/env python3
"""Capture real PE32 cross-DLL initialization, code/data imports and teardown."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
from build_pe_fixture import ROOT
from build_pe32_linked import SOURCES,FILES
from run_pe32_library import REFERENCE as LEAF_REFERENCE
from compare_pe32_library import record

NATIVE='native Windows i386 linked DLLs'
HOST='Alkyhol linked PE32 DLLs / shared thread'
REFERENCE=ROOT/'pe/reference/pe32-linked-windows-v1.json'
RESTORED=b'PE32 linked DLLs restored shared-thread selectors; forced_rebase='


def verify(directory):
    manifest=json.loads((directory/'pe32-linked-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['files'])!=set(FILES) or set(manifest['sources'])!=set(SOURCES):raise ValueError('missing exact linked-DLL identities')
    leaf=json.loads(LEAF_REFERENCE.read_text(encoding='utf-8'))
    if manifest['files']['accepted.dll']!=leaf['files']['accepted.dll']:raise ValueError('unchanged dependency replaced')
    for name,digest in leaf['sources'].items():
        if manifest['sources'].get(name)!=digest:raise ValueError('unchanged dependency source replaced')
    for name,digest in manifest['files'].items():
        if hashlib.sha256((directory/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed linked-DLL/driver bytes')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed linked-DLL source')
    if REFERENCE.exists():
        frozen=json.loads(REFERENCE.read_text(encoding='utf-8'))
        if any(manifest[k]!=frozen[k] for k in ['files','sources']):raise ValueError('changed frozen linked-DLL inputs')
    return manifest


def run(directory,output):
    directory=directory.resolve();manifest=verify(directory);windows=platform.system()=='Windows'
    if windows:
        import ctypes
        ctypes.windll.kernel32.SetErrorMode(0x8003)
        executable=directory/'driver.exe';backend=NATIVE
    elif platform.system()=='Darwin':
        target='x86_64-apple-darwin';backend=HOST
        cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
        rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
        subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline','--lib'],check=True)
        build=ROOT/'pe/target'/target/'debug';executable=directory/'probe-macos'
        subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_linked32.rs'),
                        '--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),'-L','dependency='+str(build/'deps'),'-L','dependency='+str(build.parent.parent/'debug/deps'),
                        '-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(executable)],check=True)
    else:raise RuntimeError('requires Windows or macOS/Rosetta')
    def observe(forced=False):
        args=[str(executable)]+([] if windows else [str(directory)]+(['--rebase'] if forced else []))
        result=subprocess.run(args,cwd=directory,capture_output=True,timeout=30)
        if result.returncode!=0 or len(result.stdout)!=324:raise ValueError(f'linked DLLs failed: {result.returncode}, {result.stdout.hex()}, {result.stderr!r}')
        import struct
        value={'status':result.returncode,'stdout_hex':result.stdout.hex(),'words':list(struct.unpack('<81I',result.stdout)),'stderr_hex':result.stderr.hex()}
        record(value,81,0,b'' if windows else RESTORED+str(forced).lower().encode()+b'\n')
        return value
    report={'schema_version':1,'backend':backend,'files':manifest['files'],'sources':manifest['sources'],
            'normal':observe(),'rebased':None if windows else observe(True),
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual PE32 linked DLL code/data/initialization: '+backend)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();run(a.directory,a.output)
