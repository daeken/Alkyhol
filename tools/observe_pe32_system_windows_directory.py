#!/usr/bin/env python3
"""Identical-PE evidence for DAO's GetSystemWindowsDirectoryW frontier.

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
from observe_pe32_activation_startup import runner

def frozen():
    path=ROOT/'pe/reference/pe32-system-windows-directory-windows-v1.json'
    if digest(path)!='eac73c7dbf686427f111d279655157bf15c36b0a3998ba33f8cc257e06b42ae7':raise ValueError('changed frozen native system Windows directory report')
    value=json.loads(path.read_bytes())
    if value['status'] or value['stderr_hex'] or value['backend']!='native Windows PE32 GetSystemWindowsDirectoryW observer':raise ValueError('invalid frozen system Windows directory observation')
    decode(bytes.fromhex(value['stdout_hex']),value['system_windows_directory'])
    return value

def inputs(directory):
    return {'contract':'owned GetSystemWindowsDirectoryW mode0 v1','sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['system-directory.c','system-windows-directory.def']},'files':{'system-windows-directory.exe':digest(directory/'system-windows-directory.exe')}}

def verify(directory):
    value=json.loads((directory/'system-windows-directory-inputs.json').read_text())
    if value!=inputs(directory):raise ValueError('system-Windows-directory source/PE identities changed')
    if value!=frozen()['inputs']:raise ValueError('changed frozen source/identical-PE directory inputs')
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

def host(directory):
    identity=verify(directory);probe=runner(directory);root=directory/'system Windows profile λ🌸/Windows';system=root/'System32';system.mkdir(parents=True,exist_ok=True)
    reports=[]
    def run(label,exe,profile,reason=None,boot=False):
        command=[str(probe),str(exe),'--strict','--file-origins','--system-directory',str(system)]
        if profile is not None:command+=['--system-windows-directory',str(profile)]
        result=subprocess.run(command,capture_output=True,timeout=20)
        report={'case':label,'backend':'Rosetta PE32 explicit system Windows directory','inputs':identity,'pe_sha256':digest(exe),'status':result.returncode,'stdout_hex':result.stdout.hex(),'stderr_hex':result.stderr.hex()}
        reports.append(report);(directory/'system-windows-directory-host.json').write_text(json.dumps(reports,indent=2)+'\n')
        if reason is None:
            if result.returncode:raise ValueError('host system Windows directory query failed')
            report['system_windows_directory']=decode(result.stdout,'Z:'+str(root.resolve()).replace('/','\\'))
            if result.stdout[:72]!=bytes.fromhex(frozen()['stdout_hex'])[:72]:raise ValueError('directory contract differs from native Windows')
        elif result.returncode!=78 or result.stdout!=(bytes.fromhex('32334453') if boot else b'') or reason not in result.stderr:
            raise ValueError('system Windows directory restriction failed for its actual reason: '+label)
        (directory/'system-windows-directory-host.json').write_text(json.dumps(reports,indent=2)+'\n')
    exe=directory/'system-windows-directory.exe';run('positive-independent-roles',exe,root)
    clang=shutil.which('clang');lld=rust_tool('rust-lld')
    for mode in [1,2]:
        obj=directory/f'system-windows-restrict{mode}.obj';negative=directory/f'system-windows-restrict{mode}.exe'
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-DGetSystemDirectoryW=GetSystemWindowsDirectoryW','-c',str(SOURCE/'system-directory.c'),'-o',str(obj)],check=True)
        symbols=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetSystemWindowsDirectoryW',8)]
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(directory/'system-windows-directory.lib'),*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n,size in symbols],'/out:'+str(negative)],check=True)
        run('unconfigured-role' if mode==1 else 'read-only-output',negative,None if mode==1 else root,b'system Windows directory requires an explicitly configured' if mode==1 else b'system Windows directory output outside checked writable',True)
    run('file-as-directory',exe,exe,b'system Windows directory must identify an existing host directory')
    run('missing-directory',exe,directory/'absent-system-Windows-directory',b'system Windows directory:')
    print('All 18 normalized words agree with native Windows; independent root/System32 roles and four restrictions checked')

def native(directory):
    identity=verify(directory);path=directory/'native-system-windows-directory.json'
    r=subprocess.run([str((directory/'system-windows-directory.exe').resolve())],capture_output=True,timeout=20)
    report={'backend':'native Windows PE32 GetSystemWindowsDirectoryW observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
    path.write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or r.stderr:raise ValueError('native system-Windows-directory observation failed')
    value=decode(r.stdout);root=str(Path(os.environ['SystemRoot']).resolve())
    if not Path(value).is_dir() or value.casefold()!=root.casefold():raise ValueError('native system Windows directory is not its actual OS root')
    report.update(system_windows_directory=value,windows_root=root);path.write_text(json.dumps(report,indent=2)+'\n')
    if r.stdout[:72]!=bytes.fromhex(frozen()['stdout_hex'])[:72]:raise ValueError('native directory contract differs from the frozen record')
    print('Actual native system Windows root and all 18 normalized copy/size/error/guard words agree')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');mode=p.add_mutually_exclusive_group();mode.add_argument('--native',action='store_true');mode.add_argument('--host',action='store_true')
    a=p.parse_args();(native if a.native else lambda d:host(d) if a.host else build(d))(a.output.resolve())
