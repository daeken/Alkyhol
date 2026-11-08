#!/usr/bin/env python3
"""Actual DLL onexit/CRT lock and backed PhysX startup blockers, without game files."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
FILES=[f'crt-onexit-{i}.exe' for i in range(6)]
SOURCES=['crt-onexit.c','crt-onexit.def','crt-onexit-kernel32.def','crt-onexit-layout.cpp']
REASONS={1:b'unlock requires an owned initialized lock',2:b'lock index outside the observed onexit lock profile',3:b'onexit callback outside executable guest storage',4:b'onexit output cells outside separate writable guest storage',5:b'onexit requires a preallocated owned table'}
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'crt-onexit-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('onexit source/PE identity changed')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    for name in ['crt-onexit','crt-onexit-kernel32']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetStartupInfoA',4),('GetStartupInfoW',4),('GetModuleFileNameA',12),('GetModuleFileNameW',12)]]
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/'crt-onexit.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*aliases,'/out:'+str(d/name)],check=True)
    (d/'crt-onexit-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned DLL exit registration and actual startup API witnesses')

def decode(data):
    if len(data)!=53*4:raise ValueError('incomplete onexit/startup transport')
    words=list(struct.unpack('<53I',data));error=0xface1234
    expected=[0x4f583332,4,1,error,1,1,0,1,1,error,1,2,1,1,1,1,error,1,3,1,1,1,1,1,10,20,10,121,1,1,2,1,1,1,1,21,error,1,68,68,1,1,1,1,1,1,error,error,1,1,1,0,53]
    if words!=expected:raise ValueError(f'onexit/startup state, callback order, caller stack or guards differ: {words}')
    return words

def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-crt-onexit.json' if native else 'crt-onexit-host.json')
    for mode,name in enumerate(FILES):
        if native and mode:continue
        command=[str((d/name).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins']
        r=subprocess.run(command,capture_output=True,timeout=20);report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report)
        def save():path.write_text(json.dumps({'backend':'native Windows onexit/startup observer' if native else 'Rosetta onexit/startup assertions','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or native and r.stderr:raise ValueError('onexit/startup witness failed; raw report retained')
            report['words']=decode(r.stdout)
        elif r.returncode!=78 or r.stdout or REASONS[mode] not in r.stderr:raise ValueError('onexit restriction did not stop for its actual reason')
        save()
    if native:
        h=json.loads((d/'crt-onexit-host.json').read_text());positive=h['reports'][0]
        if h['inputs']!=identity or positive['status'] or decode(bytes.fromhex(positive['stdout_hex']))!=positive['words'] or positive['words']!=reports[0]['words']:raise ValueError('native/host onexit records or input identity differ')
        print('All 53 identical-PE lock/table/callback/startup words agree')
    else:print('Owned recursive lock, real callback arrays, guards and A/W startup/path relations pass; native comparison separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
