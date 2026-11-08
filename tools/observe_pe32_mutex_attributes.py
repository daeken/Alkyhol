#!/usr/bin/env python3
"""Explicit default Win32 SECURITY_ATTRIBUTES reached by original DAO startup."""
import argparse,json,shutil,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import decode,HOST_BANNER
FILES=[f'mutex-default-attributes-{i}.exe' for i in range(4)]
SOURCES=['mutex-default-attributes.c','mutex-create.c','mutex-create.def']
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    v=json.loads((d/'mutex-default-attributes-inputs.json').read_text())
    if v!=inputs(d):raise ValueError('mutex-attributes source/PE identity changed')
    return v
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'mutex-attributes.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'mutex-create.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DCASE={mode}','-c',str(SOURCE/'mutex-default-attributes.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('CreateMutexW',12),('CloseHandle',4)]],'/out:'+str(d/name)],check=True)
    (d/'mutex-default-attributes-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-mutex-default-attributes.json' if native else 'mutex-default-attributes-host.json')
    directory=d/'default-attributes-objects';directory.mkdir(exist_ok=True)
    for mode in ([0] if native else range(4)):
        command=[str((d/FILES[mode]).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins','--object-directory',str(directory)]
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row)
        def save():path.write_text(json.dumps({'backend':'native Windows default mutex attributes' if native else 'Rosetta checked default mutex attributes','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('default attributes execution failed; raw result retained')
            row['words']=decode(r.stdout,'lifecycle');save()
        elif r.returncode!=78 or r.stdout or b'mutex security attributes or initial ownership are unsupported' not in r.stderr:raise ValueError('unsupported security/inheritance/size did not stop explicitly')
    if native:
        peer=json.loads((d/'mutex-default-attributes-host.json').read_text());row=peer['reports'][0]
        if peer['inputs']!=identity or row['status'] or decode(bytes.fromhex(row['stdout_hex']),'lifecycle')!=reports[0]['words']:raise ValueError('native/host mutex attribute identity or lifecycle differs')
    print('Explicit default attributes and NULL attributes share real object identity, close lifecycle and error/stack behavior')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
