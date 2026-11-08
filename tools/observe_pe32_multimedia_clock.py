#!/usr/bin/env python3
"""Actual Winmm boot-clock import at the reached original DAO startup call."""
import argparse,ctypes,json,shutil,subprocess,time
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_startup_clock import SYMBOLS,decode
SOURCES=['multimedia-clock.c','multimedia-clock.def','startup-clock.c','startup-clock.def','startup-clock-macos.c']
FILE='multimedia-clock.exe'
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{FILE:digest(d/FILE)}}
def verify(d):
    v=json.loads((d/'multimedia-clock-inputs.json').read_text())
    if v!=inputs(d):raise ValueError('multimedia source/PE identity changed')
    if v!=frozen()['inputs']:raise ValueError('multimedia source/PE differs from frozen native record')
    return v
def frozen():
    path=ROOT/'pe/reference/pe32-multimedia-clock-windows-v1.json'
    if digest(path)!='5b2fea4322e53c7147e8f7c0ebdbc16c668236e4b92e2a33a9e355328114d0bd':raise ValueError('changed frozen native multimedia report')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows Winmm observer' or r['status'] or r['stderr_hex'] or decode(bytes.fromhex(r['stdout_hex']),r['pid'],r['boot_ms_before'],r['boot_ms_after'],r['parent_elapsed_seconds'])!=r['decoded']:raise ValueError('incomplete frozen native multimedia observation')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');obj=d/'multimedia-clock.obj';libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    for source in ['startup-clock','multimedia-clock']:
        lib=d/(source+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(source+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'multimedia-clock.c'),'-o',str(obj)],check=True)
    symbols=[('timeGetTime' if n=='GetTickCount' else n,s) for n,s in SYMBOLS]
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in symbols],'/out:'+str(d/FILE)],check=True)
    literal=ROOT/'pe/reference/pe32-multimedia-clock-v1.exe'
    if digest(literal)!=frozen()['inputs']['files'][FILE]:raise ValueError('changed literal native multimedia PE')
    shutil.copyfile(literal,d/FILE)
    (d/'multimedia-clock-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
def execute(d,native=False):
    identity=verify(d);command=[str((d/FILE).resolve())]
    if native:
        ticks=ctypes.windll.kernel32.GetTickCount64;ticks.restype=ctypes.c_uint64
    else:
        probe=runner(d);reader=d/'multimedia-clock-reader'
        subprocess.run([shutil.which('clang'),'-std=c11','-Wall','-Wextra','-Werror',str(SOURCE/'startup-clock-macos.c'),'-o',str(reader)],check=True)
        ticks=lambda:int(subprocess.check_output([str(reader)]));command=[str(probe),*command,'--strict','--file-origins']
    before=ticks();start=time.monotonic();p=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    try:stdout,stderr=p.communicate(timeout=20)
    except subprocess.TimeoutExpired:p.kill();p.communicate();raise RuntimeError('multimedia clock budget exhausted; no successful observation')
    duration=time.monotonic()-start;after=ticks()
    r={'backend':'native Windows Winmm observer' if native else 'Rosetta host multimedia clock assertions','inputs':identity,'status':p.returncode,'pid':p.pid,'boot_ms_before':before,'boot_ms_after':after,'parent_elapsed_seconds':duration,'stdout_hex':stdout.hex(),'stderr_hex':stderr.hex()}
    path=d/('native-multimedia-clock.json' if native else 'multimedia-clock-host.json');path.write_text(json.dumps(r,indent=2)+'\n')
    if p.returncode or (native and stderr):raise ValueError('multimedia clock observer failed; raw result retained')
    r['decoded']=decode(stdout,p.pid,before,after,duration);path.write_text(json.dumps(r,indent=2)+'\n')
    if native:
        peer=json.loads((d/'multimedia-clock-host.json').read_text())
        if peer['inputs']!=identity or peer['status'] or decode(bytes.fromhex(peer['stdout_hex']),peer['pid'],peer['boot_ms_before'],peer['boot_ms_after'],peer['parent_elapsed_seconds'])!=peer['decoded']:raise ValueError('invalid or changed peer multimedia observation')
    print('Actual Winmm import executes; raw per-host boot interval, counter rate, error state and guards validate independently')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
