#!/usr/bin/env python3
"""Owned PE evidence for actual DAO CRT allocation/configuration blockers."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['crt-state.c','crt-exit.c','crt-state.def','crt-kernel32.def']
FILES=['crt-state.exe',*[f'crt-exit-{i}.exe' for i in range(1,10)]]
UNSUPPORTED={'crt-exit-1.exe':b'unsupported abort/reporting behavior','crt-exit-3.exe':b'unsupported abort/reporting behavior','crt-exit-6.exe':b'handler outside executable guest storage','crt-exit-7.exe':b'errno/new-handler policy is unsupported','crt-exit-8.exe':b'invalid-parameter/errno handling is unsupported','crt-exit-9.exe':b'unsupported CRT application type'}
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'crt-state-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('CRT source/PE identity changed')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld')
    if not clang:raise RuntimeError('open Clang is required')
    libraries=[]
    for name in ['crt-state','crt-kernel32']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libraries.append(str(lib))
    for name,source,defines in [('crt-state','crt-state.c',[]),*[(f'crt-exit-{i}','crt-exit.c',[f'-DMODE={i}']) for i in range(1,10)]]:
        obj=d/(name+'.obj')
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',*defines,'-c',str(SOURCE/source),'-o',str(obj)],check=True)
        aliases=[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n,size in [('GetStdHandle',4),('WriteFile',20),('SetLastError',4),('GetLastError',0),('ExitProcess',4)]]
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libraries,*aliases,'/out:'+str(d/(name+'.exe'))],check=True)
    (d/'crt-state-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
    print('Built owned CRT state/storage and real callback/exit PEs; native execution is separate')
def decode(data):
    if len(data)!=73*4:raise ValueError('incomplete CRT state/storage transport')
    words=list(struct.unpack('<73I',data))
    if words[:2]!=[0x43523332,4] or words[-1]!=73:raise ValueError('CRT transport order/width differs')
    return words
def assertions(words):
    for i,old in enumerate([0,0,1,1,2,2,0]):
        if words[2+i*3:5+i*3]!=[old,1,0xface1234]:raise ValueError('CRT error policy, cdecl stack or last error differs')
    for start in [23,32]:
        if words[start] not in [0,1] or words[start+1:start+8]!=[1]*7 or words[start+8]!=0xface1234:raise ValueError('CRT handler state, cdecl cleanup or last error differs')
    for start in range(41,61,4):
        if words[start:start+4]!=[1,0,1,0xface1234]:raise ValueError('CRT allocation, alignment, cdecl cleanup or last error differs')
    expected=[sum((j*17+i)&255 for j in range(n)) for i,n in enumerate([0,1,17,4099,65539])]
    if words[61:66]!=expected:raise ValueError('CRT allocation storage overlaps, is truncated or loses data')
    if words[66] not in [0,1,2] or words[67:72]!=[1,1,2,1,0xface1234]:raise ValueError('CRT application type mutation or cdecl stack differs')
def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[]
    for name in FILES:
        if native and name in UNSUPPORTED:continue
        command=[str((d/name).resolve())]
        if not native:command=[str(probe),*command,'--strict']
        r=subprocess.run(command,capture_output=True,timeout=20)
        report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report)
        path=d/('native-crt-state.json' if native else 'crt-state-host.json');path.write_text(json.dumps({'backend':'native Windows CRT observer' if native else 'Rosetta CRT host assertions','inputs':identity,'reports':reports},indent=2)+'\n')
        if name=='crt-state.exe':
            if r.returncode or native and r.stderr:raise ValueError('CRT observer failed; raw report retained')
            report['words']=decode(r.stdout);assertions(report['words'])
        elif name in UNSUPPORTED:
            if r.returncode!=78 or r.stdout or UNSUPPORTED[name] not in r.stderr:raise ValueError('unsupported CRT branch did not stop for its actual reason')
        elif r.returncode!=({'crt-exit-2.exe':37,'crt-exit-4.exe':38,'crt-exit-5.exe':37}[name]) or r.stdout or native and r.stderr:
            raise ValueError('real CRT callback/ExitProcess status differs')
        path.write_text(json.dumps({'backend':'native Windows CRT observer' if native else 'Rosetta CRT host assertions','inputs':identity,'reports':reports},indent=2)+'\n')
    if native:
        h=json.loads((d/'crt-state-host.json').read_text())
        if h['backend']!='Rosetta CRT host assertions' or h['inputs']!=identity:raise ValueError('CRT host/native inputs or backend differ')
        host=next(r for r in h['reports'] if r['file']=='crt-state.exe')
        if host['status'] or decode(bytes.fromhex(host['stdout_hex']))!=host['words']:raise ValueError('invalid actual host CRT observation')
        if reports[0]['words']!=host['words']:raise ValueError('native and Rust CRT state/ABI/storage words differ')
    print('CRT cdecl stacks, allocation storage, state changes and real callback exits checked; raw per-host observations retained')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
