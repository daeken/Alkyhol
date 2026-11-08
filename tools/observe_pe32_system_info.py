#!/usr/bin/env python3
"""Native SYSTEM_INFO and actual CPUID observations for the reached DAO blocker."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
SOURCES=['system-info.c','system-info.def','system-info-layout.cpp']
FILES=['system-info.exe','system-info-large.exe','system-info-bad.exe']
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    value=json.loads((d/'system-info-inputs.json').read_text())
    if value!=inputs(d):raise ValueError('system information source/PE identities changed')
    return value

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');obj=d/'system-info.obj';lib=d/'system-info.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'system-info.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'system-info.c'),'-o',str(obj)],check=True)
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetSystemInfo',4)]]
    common=[lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(lib),*aliases]
    for name,flag in zip(FILES[:2],['/largeaddressaware:no','/largeaddressaware']):subprocess.run([*common,flag,str(obj),'/out:'+str(d/name)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-DMODE=1','-c',str(SOURCE/'system-info.c'),'-o',str(d/'system-info-bad.obj')],check=True)
    subprocess.run([*common,str(d/'system-info-bad.obj'),'/out:'+str(d/FILES[2])],check=True)
    (d/'system-info-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');print('Built owned native SYSTEM_INFO/CPUID observer; runtime support remains separate')

def decode(data):
    if len(data)!=100:raise ValueError('incomplete system information transport')
    words=list(struct.unpack('<25I',data))
    if words[:6]!=[0x53493332,4,36,1,0x715e7a11,0x715e7a11] or words[7]!=25:raise ValueError('system information layout, stack or guards differ')
    fields=words[8:17]
    if fields[0]&0xffff!=0 or not fields[1] or fields[1]&(fields[1]-1) or not 0<fields[2]<fields[3] or not fields[4] or not 0<fields[5]<=4096 or not fields[7] or fields[7]&(fields[7]-1):raise ValueError('incomplete or unsupported native x86 system information')
    return {'words':words,'information_words':fields,'cpuid0':words[17:21],'cpuid1':words[21:25],'last_error':words[6]}

def native(d):
    identity=verify(d);reports=[];path=d/'native-system-info.json'
    def save():path.write_text(json.dumps({'backend':'native Windows PE32 SYSTEM_INFO/CPUID observer','inputs':identity,'reports':reports},indent=2)+'\n')
    for name in FILES[:2]:
        r=subprocess.run([str((d/name).resolve())],capture_output=True,timeout=20);report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report);save()
        if r.returncode or r.stderr:raise ValueError('native system information observer failed; raw result retained')
        report['record']=decode(r.stdout);save()
    print('Observed complete ordinary/LAA SYSTEM_INFO bytes and actual CPUID leaves; backend support is separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
