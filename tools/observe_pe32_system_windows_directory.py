#!/usr/bin/env python3
"""Native-only observer for DAO's measured GetSystemWindowsDirectoryW frontier.

The existing owned C sizing/guard probe is compiled with only its API name
selected differently. Its frozen GetSystemDirectoryW PEs/source remain unchanged.
"""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from check_pe32_system_directory import decode

def inputs(directory):
    return {'contract':'owned GetSystemWindowsDirectoryW mode0 v1','sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['system-directory.c','system-windows-directory.def']},'files':{'system-windows-directory.exe':digest(directory/'system-windows-directory.exe')}}

def verify(directory):
    value=json.loads((directory/'system-windows-directory-inputs.json').read_text())
    if value!=inputs(directory):raise ValueError('system-Windows-directory source/PE identities changed')
    return value

def build(directory):
    clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang is required')
    directory.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');lib=directory/'system-windows-directory.lib';obj=directory/'system-windows-directory.obj'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'system-windows-directory.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-DMODE=0','-DGetSystemDirectoryW=GetSystemWindowsDirectoryW','-c',str(SOURCE/'system-directory.c'),'-o',str(obj)],check=True)
    symbols=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetSystemWindowsDirectoryW',8)]
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n,size in symbols],'/out:'+str(directory/'system-windows-directory.exe')],check=True)
    (directory/'system-windows-directory-inputs.json').write_text(json.dumps(inputs(directory),indent=2)+'\n');verify(directory)
    print('Built owned API-name variant for the measured next DAO call; no execution claimed')

def native(directory):
    identity=verify(directory);path=directory/'native-system-windows-directory.json'
    r=subprocess.run([str((directory/'system-windows-directory.exe').resolve())],capture_output=True,timeout=20)
    report={'backend':'native Windows PE32 GetSystemWindowsDirectoryW observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
    path.write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or r.stderr:raise ValueError('native system-Windows-directory observation failed')
    value=decode(r.stdout);root=str(Path(os.environ['SystemRoot']).resolve())
    if not Path(value).is_dir() or value.casefold()!=root.casefold():raise ValueError('native system Windows directory is not its actual OS root')
    report.update(system_windows_directory=value,windows_root=root);path.write_text(json.dumps(report,indent=2)+'\n')
    print('Recorded actual native system Windows root, copy/sizing, guards and last error; host support remains separate')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true')
    a=p.parse_args();(native if a.native else build)(a.output.resolve())
