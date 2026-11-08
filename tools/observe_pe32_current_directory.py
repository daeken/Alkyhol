#!/usr/bin/env python3
"""Backed working-directory behavior at the reached original DAO call."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
FILES=[f'current-directory-{i}.exe' for i in range(3)]
SOURCES=['current-directory.c','current-directory.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('ReadFile',20),('CreateFileW',28),('CloseHandle',4),('GetLastError',0),('SetLastError',4),('GetModuleFileNameW',12),('GetCurrentDirectoryW',8),('SetCurrentDirectoryW',4)]
NAME='cwd-owned-λ🌸'
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES},'fixture':{NAME+'/sentinel.bin':'41',NAME+'/inner/sentinel.bin':'42'}}
def verify(d):
    r=json.loads((d/'current-directory-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('current-directory source/PE/fixture identity changed')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'current-directory.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'current-directory.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj')
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function',f'-DMODE={mode}','-c',str(SOURCE/'current-directory.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
    (d/'current-directory-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
def decode(data,a,b):
    if len(data)%4:raise ValueError('truncated current-directory transport')
    words=list(struct.unpack('<'+'I'*(len(data)//4),data));e=0xface1234;g=0x715e7a11
    def observe(n,byte):return [n+1,e,n+1,0x7171,e,n,e,1,0,g,g,1,0,1,1,byte,1,e]
    expected=[0x43445732,4,1,e,*observe(a,65),1,e,*observe(b,66),0,2,0,267,*observe(b,66),a,b,0]
    expected[-1]=len(expected)
    if words!=expected:raise ValueError(f'current directory change/query/error/buffer/relative-file evidence differs: {words}')
    # Path lengths differ by host; all remaining observed words must agree.
    return {'words':words,'path_units':[a,b]}
def execute(d,native=False):
    identity=verify(d);root=d/NAME;(root/'inner').mkdir(parents=True,exist_ok=True)
    for path,value in [(root/'sentinel.bin',b'A'),(root/'inner/sentinel.bin',b'B')]:
        if path.exists() and path.read_bytes()!=value:raise ValueError('refusing to overwrite unknown owned-directory fixture')
        path.write_bytes(value)
    reports=[];path=d/('native-current-directory.json' if native else 'current-directory-host.json')
    probe=None if native else runner(d)
    text=str(root.resolve()) if native else 'Z:'+str(root.resolve()).replace('/','\\')
    a=len(text.encode('utf-16-le'))//2;b=a+len('\\inner')
    for mode in ([0] if native else [0,1,2]):
        command=[str((d/FILES[mode]).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins']
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row)
        def save():path.write_text(json.dumps({'backend':'native Windows directory observer' if native else 'Rosetta process-owned directory','inputs':identity,'path_units':[a,b],'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or (native and r.stderr):raise ValueError('directory observer failed; raw result retained')
            row['decoded']=decode(r.stdout,a,b);save()
        elif r.returncode!=78 or r.stdout or b'current directory requires an exact absolute Z: backed path' not in r.stderr:raise ValueError('unmeasured directory namespace did not stop explicitly')
    if native:
        peer=json.loads((d/'current-directory-host.json').read_text())
        if peer['inputs']!=identity or peer['reports'][0]['status'] or decode(bytes.fromhex(peer['reports'][0]['stdout_hex']),*peer['path_units'])!=peer['reports'][0]['decoded']:raise ValueError('native/host directory identities or semantics differ')
    print('Real directory transitions redirect relative reads; sizing, guards and failed-change atomicity validate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
