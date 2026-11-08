#!/usr/bin/env python3
"""Process-owned exception filter registration reached by original DAO startup."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
FILES=[f'exception-filter-{i}.exe' for i in range(2)]
SOURCES=['exception-filter.c','exception-filter.def']
REASONS={1:b'top-level exception filter outside executable guest storage'}
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'exception-filter-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('filter registration source/PE identity changed')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    for name in ['exception-filter']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('SetUnhandledExceptionFilter',4)]]
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/'exception-filter.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*aliases,'/out:'+str(d/name)],check=True)
    (d/'exception-filter-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned top-level exception filter registration witnesses')

def expected_words():
    return [0x45523332,4,1,1,1,1,1,1,1,0xface1234,11]

def decode(data):
    if len(data)!=44:raise ValueError('incomplete exception filter registration transport')
    words=list(struct.unpack('<11I',data))
    if words!=expected_words():raise ValueError(f'filter previous-value, registration, caller-stack or error observations differ: {words}')
    return words

def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-exception-filter.json' if native else 'exception-filter-host.json')
    for mode,name in enumerate(FILES):
        if native and mode:continue
        command=[str((d/name).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins']
        r=subprocess.run(command,capture_output=True,timeout=20);report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report)
        def save():path.write_text(json.dumps({'backend':'native Windows exception filter registration observer' if native else 'Rosetta process-owned filter registration','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or native and r.stderr:raise ValueError('filter registration witness failed; raw report retained')
            report['words']=decode(r.stdout)
        elif r.returncode!=78 or r.stdout or REASONS[mode] not in r.stderr:raise ValueError('filter registration restriction did not stop for its actual reason')
        save()
    if native:
        h=json.loads((d/'exception-filter-host.json').read_text());positive=h['reports'][0]
        if h['inputs']!=identity or positive['status'] or decode(bytes.fromhex(positive['stdout_hex']))!=positive['words'] or positive['words']!=reports[0]['words']:raise ValueError('native/host filter registration records or input identity differ')
        print('All 11 identical-PE filter registration words agree; exception delivery is separate')
    else:print('Actual previous filter values, replacement/clear and no eager callback execution pass; exception delivery is separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
