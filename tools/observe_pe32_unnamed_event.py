#!/usr/bin/env python3
"""Real unnamed event state at the actually reached DAO startup constructor."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
FILES=[f'unnamed-event-{i}.exe' for i in range(6)]
SOURCES=['unnamed-event.c','unnamed-event.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('CreateEventW',16),('SetEvent',4),('ResetEvent',4),('WaitForSingleObjectEx',12),('CloseHandle',4)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'unnamed-event-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('unnamed-event source/PE identity changed')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'unnamed-event.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'unnamed-event.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function',f'-DMODE={mode}','-c',str(SOURCE/'unnamed-event.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
    (d/'unnamed-event-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
def decode(data):
    e=0xface1234
    results=[258,0,258,1,0,0,1,1,258,1,1,0,258,1,258,1,1]
    expected=[0x45563332,4,1,0,1,0,*[w for x in results for w in [x,e]],1,42]
    if len(data)!=len(expected)*4:raise ValueError('incomplete event transport')
    got=list(struct.unpack('<'+'I'*len(expected),data))
    if got!=expected:raise ValueError('event state/error/stack observations differ: '+repr(got))
    return got
def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-unnamed-event.json' if native else 'unnamed-event-host.json')
    for mode in ([0] if native else range(6)):
        command=[str((d/FILES[mode]).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins']
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row)
        def save():path.write_text(json.dumps({'backend':'native Windows unnamed events' if native else 'Rosetta typed unnamed events','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('event observer execution failed; raw result retained')
            row['words']=decode(r.stdout);save()
        else:
            reason=b'PE32 CreateEventW security attributes and named events are unsupported' if mode in [1,2] else b'event waits require zero timeout and alertable FALSE' if mode in [3,4] else b'event state change requires an owned live event'
            if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('event exclusion did not fail for its actual reason')
    if native:
        peer=json.loads((d/'unnamed-event-host.json').read_text());row=peer['reports'][0]
        if peer['inputs']!=identity or row['status'] or decode(bytes.fromhex(row['stdout_hex']))!=reports[0]['words']:raise ValueError('native/host event identity or state differs')
    print('Real manual/automatic event state, zero-timeout consumption, set/reset and close checked')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
