#!/usr/bin/env python3
"""Small host microtests for guarded function/data imports and low PE32 memory."""
import argparse
import json
import hashlib
import os
from pathlib import Path
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT,rust_tool

def check(output,native=False):
    if native:return check_native(output)
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    clang=shutil.which('clang');lld=rust_tool('rust-lld')
    if not clang:raise RuntimeError('open Clang is required')
    source=ROOT/'pe/fixtures/pe32-startup';libs=[]
    for name in ['kernel32','service']:
        lib=output/(name+'.lib');libs.append(str(lib))
        subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(source/(name+'.def')),'/out:'+str(lib)],check=True)
    for mode in [0,1,3,4,5]:
        obj=output/(f'case{mode}.obj')
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(source/'guard.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/entry:entry',*(['/merge:.rdata=.text'] if mode==3 else []),str(obj),*libs,
                        *[f'/alternatename:__imp__{name}@{size}=__imp__{name}' for name,size in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('DeferredFunction',4),('GetProcessHeap',0),('HeapAlloc',12),('HeapFree',12),('GetVersionExA',4),('HeapCreate',12),('GetEnvironmentVariableA',12)]],'/out:'+str(output/f'case{mode}.exe')],check=True)
    inputs={'files':{f'case{mode}.exe':hashlib.sha256((output/f'case{mode}.exe').read_bytes()).hexdigest() for mode in [0,1,3,4,5]},'sources':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source/'guard.c',source/'kernel32.def',source/'service.def']}}
    (output/'microtest-inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo');rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc');target='x86_64-apple-darwin'
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline'],check=True)
    build=ROOT/'pe/target'/target/'debug';probe=output/'startup-macos'
    subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_startup32.rs'),'--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),'-L','dependency='+str(build/'deps'),'-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(probe)],check=True)
    observations=[]
    environment=dict(os.environ,ALKYHOL_STARTUP_MICRO="active");environment.pop("ALKYHOL_STARTUP_MICRO_ABSENT",None)
    for mode in [0,1,3,4,5]:
        result=subprocess.run([str(probe),str(output/f'case{mode}.exe'),'--external','external-service.dll'],env=environment,capture_output=True,timeout=15)
        expected=struct.pack('<I',0x53543332) if mode<2 else expected_positive(mode)
        if result.returncode!=(78 if mode<2 else 0) or result.stdout!=expected:raise ValueError(f'startup microtest {mode}: {result.returncode}, {result.stdout.hex()}, {result.stderr!r}')
        if mode<2:
            label='DeferredFunction' if mode==0 else 'DeferredData'
            if b'diagnostic stop: unimplemented import' not in result.stderr or label.encode() not in result.stderr or b'source_module=case' not in result.stderr:raise ValueError('missing real guarded-import fault attribution')
            strict=subprocess.run([str(probe),str(output/f'case{mode}.exe'),'--strict'],capture_output=True,timeout=15)
            if strict.returncode!=78 or strict.stdout:raise ValueError('strict loader entered a missing-dependency program')
        observations.append({'case':mode,'status':result.returncode,'stdout_hex':result.stdout.hex(),'stderr_hex':result.stderr.hex()})
    (output/'microtest-results.json').write_text(json.dumps({'backend':'Alkyhol PE32 host instrumentation microtests','inputs':verify(output),'observations':observations},indent=2)+'\n')
    print('Passed targeted startup guard, RX-IAT, heap, version and environment microtests')

def verify(directory):
    inputs=json.loads((directory/'microtest-inputs.json').read_text(encoding='utf-8'))
    if set(inputs['files'])!={f'case{m}.exe' for m in [0,1,3,4,5]} or set(inputs['sources'])!={'pe/fixtures/pe32-startup/'+n for n in ['guard.c','kernel32.def','service.def']}:raise ValueError('changed microtest input roster')
    for name,digest in inputs['files'].items():
        if hashlib.sha256((directory/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed microtest executable')
    for name,digest in inputs['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed microtest source')
    return inputs

def expected_positive(mode):
    words={3:[0x4833324d,1,1,1,1,4,1,1],4:[0x56333249,1,148,6,2,9200,2,1],5:[0x45333249,7,7,6,1,0,203,4]}
    return struct.pack('<8I',*words[mode])

def check_native(directory):
    import ctypes
    ctypes.windll.kernel32.SetErrorMode(0x8003)
    # The native job receives these exact PE files from the Mac artifact. The
    # two guarded-import cases are instrumentation only and never run natively.
    observations=[]
    environment=dict(os.environ,ALKYHOL_STARTUP_MICRO="active");environment.pop("ALKYHOL_STARTUP_MICRO_ABSENT",None)
    verify(directory)
    for mode in [3,4,5]:
        r=subprocess.run([str((directory/f'case{mode}.exe').resolve())],env=environment,capture_output=True,timeout=15)
        if r.returncode!=0 or r.stdout!=expected_positive(mode) or r.stderr:raise ValueError(f'native startup microtest {mode}: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        observations.append({'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    (directory/'native-microtest-results.json').write_text(json.dumps({'backend':'native Windows PE32 targeted startup microtests','inputs':verify(directory),'observations':observations},indent=2)+'\n')
    print('Passed identical-PE native process heap, legacy version and environment microtests')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();check(a.output,a.native)
