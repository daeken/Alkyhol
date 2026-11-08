#!/usr/bin/env python3
"""Actual heap ownership reached by original DAO EXE startup; no game files."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
FILES=[f'heap-lock-{i}.exe' for i in range(4)]
SOURCES=['heap-lock.c','heap-lock.def']
REASONS={1:b'unknown PE32 heap lock handle',2:b'critical section leave without ownership',3:b'heap lock requires a serialized owned heap'}
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'heap-lock-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('heap lock source/PE identity changed')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    for name in ['heap-lock']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetProcessHeap',0),('HeapCreate',12),('HeapAlloc',12),('HeapReAlloc',16),('HeapSize',12),('HeapFree',12),('HeapLock',4),('HeapUnlock',4)]]
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/'heap-lock.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*aliases,'/out:'+str(d/name)],check=True)
    (d/'heap-lock-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned serialized heap lock witnesses')

def expected_words():
    row=[1,1,1,1,1,1,48,1,1,1,1,0xface1234]
    return [0x484c3332,4]+row+[1]+row+[28]

def decode(data):
    if len(data)!=112:raise ValueError('incomplete heap lock transport')
    words=list(struct.unpack('<28I',data))
    if words!=expected_words():raise ValueError(f'heap lock/recursion, allocation, caller stack or error observations differ: {words}')
    return words

def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-heap-lock.json' if native else 'heap-lock-host.json')
    for mode,name in enumerate(FILES):
        if native and mode:continue
        command=[str((d/name).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins']
        r=subprocess.run(command,capture_output=True,timeout=20);report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report)
        def save():path.write_text(json.dumps({'backend':'native Windows heap lock observer' if native else 'Rosetta actual heap ownership assertions','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or native and r.stderr:raise ValueError('heap lock witness failed; raw report retained')
            report['words']=decode(r.stdout)
        elif r.returncode!=78 or r.stdout or REASONS[mode] not in r.stderr:raise ValueError('heap lock restriction did not stop for its actual reason')
        save()
    if native:
        h=json.loads((d/'heap-lock-host.json').read_text());positive=h['reports'][0]
        if h['inputs']!=identity or positive['status'] or decode(bytes.fromhex(positive['stdout_hex']))!=positive['words'] or positive['words']!=reports[0]['words']:raise ValueError('native/host heap lock records or input identity differ')
        print('All 28 identical-PE heap lock words agree')
    else:print('Actual serialized heap recursion, allocation while owned and matching release pass; native comparison separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
