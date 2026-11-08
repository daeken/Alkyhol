#!/usr/bin/env python3
"""Execute identical owned PE32 DLLs through Windows and Alkyhol loaders."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import struct
import subprocess
from build_pe32_library import SOURCES,FILES
from build_pe_fixture import ROOT

NATIVE='native Windows i386 dynamic DLL'
HOST='Alkyhol isolated PE32 DLL / x86-64 Mach-O'
REFERENCE=ROOT/'pe/reference/pe32-library-windows-v1.json'
RESTORED=b'PE32 DLL calls restored selectors and cleared FS leases; forced_rebase='
MISMATCH=b'Alkyhol PE32 unsupported: PE32 Stdcall call returned with an unbalanced guest stack\n'


def verify(directory):
    manifest=json.loads((directory/'pe32-library-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['files'])!=set(FILES) or set(manifest['sources'])!=set(SOURCES):
        raise ValueError('missing exact PE32 DLL identities')
    for name,digest in manifest['files'].items():
        if hashlib.sha256((directory/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed PE32 DLL/driver bytes')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed PE32 DLL source')
    if REFERENCE.exists():
        reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
        if any(manifest[k]!=reference[k] for k in ['files','sources']):raise ValueError('changed frozen PE32 DLL inputs')
    return manifest


def decode(result,windows=False,forced=False,mismatch=False):
    expected_status=78 if mismatch else 0
    expected_bytes=128 if mismatch else 320
    diagnostic=b'' if windows else (MISMATCH if mismatch else RESTORED+str(forced).lower().encode()+b'\n')
    if result.returncode!=expected_status or len(result.stdout)!=expected_bytes or result.stderr!=diagnostic:
        raise ValueError(f'PE32 DLL execution failed: {result.returncode}, {result.stdout.hex()}, {result.stderr!r}')
    return {'status':result.returncode,'stdout_hex':result.stdout.hex(),
            'words':list(struct.unpack('<'+'I'*(expected_bytes//4),result.stdout)),
            'stderr_hex':result.stderr.hex()}


def run(directory,output):
    directory=directory.resolve();manifest=verify(directory);windows=platform.system()=='Windows'
    if windows:
        executable=directory/'driver.exe';backend=NATIVE
        import ctypes
        ctypes.windll.kernel32.SetErrorMode(0x8003)
    elif platform.system()=='Darwin':
        backend=HOST;target='x86_64-apple-darwin'
        cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
        rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
        subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline','--lib'],check=True)
        build=ROOT/'pe/target'/target/'debug';executable=directory/'probe-macos'
        subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_library32.rs'),
                        '--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),'-L','dependency='+str(build/'deps'),
                        '-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(executable)],check=True)
    else:raise RuntimeError('requires native Windows or macOS/Rosetta')
    def observe(forced=False,mismatch=False):
        command=[str(executable)]+([] if windows else [str(directory)]+(['--rebase'] if forced else [])+(['--mismatch'] if mismatch else []))
        result=subprocess.run(command,cwd=directory,capture_output=True,timeout=30)
        return decode(result,windows,forced,mismatch)
    report={'schema_version':1,'backend':backend,'files':manifest['files'],'sources':manifest['sources'],
            'normal':observe(),'rebased':None if windows else observe(True),
            'mismatch_rejected':None if windows else observe(mismatch=True),
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual PE32 DLL attach/export/detach execution: '+backend)


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();run(a.directory,a.output)
