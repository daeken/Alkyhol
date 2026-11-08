#!/usr/bin/env python3
"""Native x87 arctangent observations for the actually reached DAO blocker."""
import argparse,json,shutil,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_ci_sqrt import decode
SOURCES=['ci-atan.c','ci-atan.S','ci-atan.def','ci-sqrt.c','ci-sqrt.S','crt-kernel32.def']
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{'ci-atan.exe':digest(d/'ci-atan.exe')}}
def verify(d):
    r=json.loads((d/'ci-atan-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('arctangent source/PE identity changed')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[];objects=[]
    if not clang:raise RuntimeError('open Clang is required')
    for name in ['ci-atan','crt-kernel32']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    for suffix in ['c','S']:
        obj=d/('ci-atan-'+suffix+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/('ci-atan.'+suffix)),'-o',str(obj)],check=True);objects.append(str(obj))
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('SetLastError',4),('GetLastError',0)]]
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',*objects,*libs,*aliases,'/out:'+str(d/'ci-atan.exe')],check=True)
    (d/'ci-atan-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built direct x87 atan witness; native behavior remains unmeasured')
def native(d):
    identity=verify(d);r=subprocess.run([str((d/'ci-atan.exe').resolve())],capture_output=True,timeout=20)
    report={'backend':'native Windows direct x87 _CIatan observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};p=d/'native-ci-atan.json';p.write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or r.stderr:raise ValueError('native arctangent observer failed; no parity claimed')
    report['records']=decode(r.stdout);p.write_text(json.dumps(report,indent=2)+'\n');print('Recorded actual 204 atan results, control/status, stack witness and error state; host implementation remains separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
