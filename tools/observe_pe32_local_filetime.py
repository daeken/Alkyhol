#!/usr/bin/env python3
"""Native-only UTC-qualified FileTimeToLocalFileTime evidence for DAO startup."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetTimeZoneInformation',4),('FileTimeToLocalFileTime',8)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['local-filetime.c','local-filetime.def','time-layout.cpp']},'files':{'local-filetime.exe':digest(d/'local-filetime.exe')}}
def verify(d):
    v=json.loads((d/'local-filetime-inputs.json').read_text())
    if v!=inputs(d):raise ValueError('local FILETIME source/PE identities changed')
    return v
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');obj=d/'local-filetime.obj';lib=d/'local-filetime.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'local-filetime.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'local-filetime.c'),'-o',str(obj)],check=True)
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/'local-filetime.exe')],check=True)
    (d/'local-filetime-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned local-FILETIME observer; native UTC qualification and execution pending')
def decode(data):
    if len(data)!=28+9*44:raise ValueError('incomplete native local-FILETIME observations')
    head=list(struct.unpack('<7I',data[:28]))
    if head[:2]!=[0x544c3332,4] or head[2]>2 or head[3:6]!=[0,0,0] or head[6]!=0x715e7a11:raise ValueError('native clock is not qualified for the bounded UTC profile')
    rows=[list(struct.unpack('<11I',data[28+i*44:28+(i+1)*44])) for i in range(9)]
    if any(r[0]!=i or r[1]!=int(i>=7) or r[8:10]!=[0x715e7a11]*2 for i,r in enumerate(rows)):raise ValueError('native local-FILETIME order/layout/guards differ')
    return {'timezone':head,'records':rows}
def native(d):
    identity=verify(d);r=subprocess.run([str((d/'local-filetime.exe').resolve())],capture_output=True,timeout=20)
    report={'backend':'native Windows PE32 UTC-qualified local FILETIME observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/'native-local-filetime.json';path.write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or r.stderr:raise ValueError('native local-FILETIME observer failed; no conversion support claimed')
    report['decoded']=decode(r.stdout);path.write_text(json.dumps(report,indent=2)+'\n');print('Captured actual UTC timezone and FILETIME result/error/alias/boundary observations; host implementation separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
