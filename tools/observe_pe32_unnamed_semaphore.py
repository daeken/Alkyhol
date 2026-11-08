#!/usr/bin/env python3
"""Actual DAO counting-semaphore constructor: real permits and signed bounds."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
FILES=[f'unnamed-semaphore-{i}.exe' for i in range(5)]
SOURCES=['unnamed-semaphore.c','unnamed-semaphore.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('CreateSemaphoreW',16),('WaitForSingleObjectEx',12),('CloseHandle',4)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'unnamed-semaphore-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('semaphore source/PE identity changed')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'unnamed-semaphore.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'unnamed-semaphore.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function',f'-DMODE={mode}','-c',str(SOURCE/'unnamed-semaphore.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
    (d/'unnamed-semaphore-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
def decode(data):
    e=0xface1234;expected=[0x53454d32,4,1,0,258,e,1,0,0,e,0,e,258,e,0,87,0,87,0,87,258,e,1,e,1,e,1,28]
    if len(data)!=112 or list(struct.unpack('<28I',data))!=expected:raise ValueError('semaphore permit/count/error/stack observations differ: '+data.hex())
    return expected
def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-unnamed-semaphore.json' if native else 'unnamed-semaphore-host.json')
    for mode in ([0] if native else range(5)):
        command=[str((d/FILES[mode]).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins']
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row)
        def save():path.write_text(json.dumps({'backend':'native Windows unnamed semaphores' if native else 'Rosetta checked counting semaphores','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('semaphore observer failed; raw output retained')
            row['words']=decode(r.stdout);save()
        else:
            reason=b'PE32 CreateSemaphoreW security attributes and named semaphores are unsupported' if mode in [1,2] else b'semaphore waits require zero timeout and alertable FALSE'
            if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('semaphore exclusion failed for a different reason')
    if native:
        peer=json.loads((d/'unnamed-semaphore-host.json').read_text());row=peer['reports'][0]
        if peer['inputs']!=identity or row['status'] or decode(bytes.fromhex(row['stdout_hex']))!=reports[0]['words']:raise ValueError('native/host semaphore identity or permits differ')
    print('Counting permits, zero-timeout consumption, signed invalid-count failures and close state agree')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
