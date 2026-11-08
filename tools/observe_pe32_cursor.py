#!/usr/bin/env python3
"""Read-only cursor ABI observations; per-host positions are not cross-host values."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
FILES=[f'cursor-{i}.exe' for i in range(2)]
SOURCES=['cursor.c','cursor.def','cursor-kernel32.def','cursor-layout.cpp']
REASONS={1:b'cursor POINT outside aligned writable guest storage'}
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'cursor-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('cursor source/PE identity changed')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    for name in ['cursor','cursor-kernel32']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetCursorPos',4)]]
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/'cursor.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*aliases,'/out:'+str(d/name)],check=True)
    (d/'cursor-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned read-only cursor witnesses')

def decode(data):
    if len(data)!=44:raise ValueError('incomplete cursor transport')
    words=list(struct.unpack('<11I',data))
    if words[:8]!=[0x43503332,4,8,1,1,0x715e7a11,0x715e7a11,0xface1234] or words[-1]!=11:
        raise ValueError(f'cursor return, stack, guard or error observations differ: {words}')
    return {'abi_words':words[:8]+words[10:],'x':struct.unpack('<i',data[32:36])[0],'y':struct.unpack('<i',data[36:40])[0]}

def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-cursor.json' if native else 'cursor-host.json')
    for mode,name in enumerate(FILES):
        if native and mode:continue
        command=[str((d/name).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins']
        r=subprocess.run(command,capture_output=True,timeout=20);report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report)
        def save():path.write_text(json.dumps({'backend':'native Windows cursor observer' if native else 'Rosetta actual Quartz cursor query','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or native and r.stderr:raise ValueError('cursor witness failed; raw report retained')
            report['record']=decode(r.stdout)
        elif r.returncode!=78 or r.stdout or REASONS[mode] not in r.stderr:raise ValueError('cursor restriction did not stop for its actual reason')
        save()
    if native:
        h=json.loads((d/'cursor-host.json').read_text());positive=h['reports'][0]
        if h['inputs']!=identity or positive['status'] or decode(bytes.fromhex(positive['stdout_hex']))!=positive['record'] or positive['record']['abi_words']!=reports[0]['record']['abi_words']:raise ValueError('native/host cursor records or input identity differ')
        print('Identical-PE cursor ABI words agree; actual per-host desktop coordinates recorded separately')
    else:print('Actual Quartz cursor query, guest POINT guards and stdcall stack pass; native comparison separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
