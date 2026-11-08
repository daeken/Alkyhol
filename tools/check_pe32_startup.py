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
    for mode in [0,1,3,4,5,6,7,8,9,10,11,12,13]:
        obj=output/(f'case{mode}.obj')
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(source/'guard.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/entry:entry',*(['/merge:.rdata=.text'] if mode==3 else []),str(obj),*libs,
                        *[f'/alternatename:__imp__{name}@{size}=__imp__{name}' for name,size in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('DeferredFunction',4),('GetProcessHeap',0),('HeapAlloc',12),('HeapFree',12),('GetVersionExA',4),('HeapCreate',12),('GetEnvironmentVariableA',12),('SetLastError',4),('GetModuleHandleA',4),('GetModuleHandleW',4),('GetProcAddress',8),('FlsAlloc',4),('FlsFree',4),('FlsGetValue',4),('FlsSetValue',8),('TlsAlloc',0),('TlsFree',4),('TlsGetValue',4),('TlsSetValue',8),('EncodePointer',4),('DecodePointer',4),('InitializeCriticalSectionAndSpinCount',8),('InitializeCriticalSection',4),('EnterCriticalSection',4),('LeaveCriticalSection',4),('DeleteCriticalSection',4),('TryEnterCriticalSection',4),('InterlockedIncrement',4),('GetCurrentThreadId',0),('GetStartupInfoA',4),('GetFileType',4),('SetHandleCount',4),('GetEnvironmentStringsW',0),('FreeEnvironmentStringsW',4),('GetCommandLineA',0),('GetCommandLineW',0),('GetACP',0),('IsValidCodePage',4),('GetCPInfo',8),('WideCharToMultiByte',32),('MultiByteToWideChar',24),('GetStringTypeW',16)]],'/out:'+str(output/f'case{mode}.exe')],check=True)
    inputs={'files':{f'case{mode}.exe':hashlib.sha256((output/f'case{mode}.exe').read_bytes()).hexdigest() for mode in [0,1,3,4,5,6,7,8,9,10,11,12,13]},'sources':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source/'guard.c',source/'kernel32.def',source/'service.def']}}
    (output/'microtest-inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo');rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc');target='x86_64-apple-darwin'
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline'],check=True)
    build=ROOT/'pe/target'/target/'debug';probe=output/'startup-macos'
    subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_startup32.rs'),'--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),'-L','dependency='+str(build/'deps'),'-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(probe)],check=True)
    observations=[]
    environment=dict(os.environ,ALKYHOL_STARTUP_MICRO="active",ALKYHOL_STARTUP_UNICODE="λ🌸",ALKYHOL_STARTUP_EMPTY="");environment.pop("ALKYHOL_STARTUP_MICRO_ABSENT",None)
    for mode in [0,1,3,4,5,6,7,8,9,10,11,12]:
        result=subprocess.run([str(probe),str(output/f'case{mode}.exe'),'--external','external-service.dll'],env=environment,input=b'',capture_output=True,timeout=15)
        expected=struct.pack('<I',0x53543332) if mode in [0,1,7,8] else expected_positive(mode)
        if result.returncode!=(78 if mode in [0,1,7,8] else 0) or result.stdout!=expected:raise ValueError(f'startup microtest {mode}: {result.returncode}, {result.stdout.hex()}, {result.stderr!r}')
        if mode<2:
            label='DeferredFunction' if mode==0 else 'DeferredData'
            if b'diagnostic stop: unimplemented import' not in result.stderr or label.encode() not in result.stderr or b'source_module=case' not in result.stderr:raise ValueError('missing real guarded-import fault attribution')
            strict=subprocess.run([str(probe),str(output/f'case{mode}.exe'),'--strict'],capture_output=True,timeout=15)
            if strict.returncode!=78 or strict.stdout:raise ValueError('strict loader entered a missing-dependency program')
        if mode in [7,8] and (b'unsupported' not in result.stderr or (mode==7 and b'UnsupportedStartupMicroExport' not in result.stderr)):raise ValueError('unknown dynamic export did not stop explicitly')
        observations.append({'case':mode,'status':result.returncode,'stdout_hex':result.stdout.hex(),'stderr_hex':result.stderr.hex()})
    (output/'microtest-results.json').write_text(json.dumps({'backend':'Alkyhol PE32 host instrumentation microtests','inputs':verify(output),'observations':observations},indent=2)+'\n')
    print('Passed 12 host startup microtests; built the separate native NLS observer')

def verify(directory):
    inputs=json.loads((directory/'microtest-inputs.json').read_text(encoding='utf-8'))
    if set(inputs['files'])!={f'case{m}.exe' for m in [0,1,3,4,5,6,7,8,9,10,11,12,13]} or set(inputs['sources'])!={'pe/fixtures/pe32-startup/'+n for n in ['guard.c','kernel32.def','service.def']}:raise ValueError('changed microtest input roster')
    for name,digest in inputs['files'].items():
        if hashlib.sha256((directory/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed microtest executable')
    for name,digest in inputs['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed microtest source')
    return inputs

def expected_positive(mode):
    words={3:[0x4833324d,1,1,1,1,4,1,1],4:[0x56333249,1,148,6,2,9200,2,1],5:[0x45333249,7,7,6,1,0,203,4]}
    words[6]=[0x4c533332]+[1]*18+[4]
    words[9]=[0x43533332]+[1]*6+[4]
    words[10]=[0x53493332]+[1]*11
    words[11]=[0x45423332]+[1]*10+[4]
    words[12]=[0x414e3332]+[1]*14+[4]
    return struct.pack('<'+str(len(words[mode]))+'I',*words[mode])

def check_native(directory):
    import ctypes
    ctypes.windll.kernel32.SetErrorMode(0x8003)
    # The native job receives these exact PE files from the Mac artifact. The
    # guarded-import/dynamic-query cases are instrumentation only and never run natively.
    observations=[]
    environment=dict(os.environ,ALKYHOL_STARTUP_MICRO="active",ALKYHOL_STARTUP_UNICODE="λ🌸",ALKYHOL_STARTUP_EMPTY="");environment.pop("ALKYHOL_STARTUP_MICRO_ABSENT",None)
    verify(directory)
    for mode in [3,4,5,6,9,10,11,12]:
        r=subprocess.run([str((directory/f'case{mode}.exe').resolve())],env=environment,input=b'',capture_output=True,timeout=15)
        if r.returncode!=0 or r.stdout!=expected_positive(mode) or r.stderr:raise ValueError(f'native startup microtest {mode}: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        observations.append({'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    # Native-only capture for GetStringTypeW, the next observed DAO blocker.
    # No host execution or equality is claimed for this observation.
    r=subprocess.run([str((directory/'case13.exe').resolve())],capture_output=True,timeout=15)
    if r.returncode!=0 or len(r.stdout)!=516 or r.stdout[:4]!=struct.pack('<I',0x4e4c5331) or r.stderr:raise ValueError('native NLS observation failed')
    native_only=[{'case':13,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}]
    (directory/'native-microtest-results.json').write_text(json.dumps({'backend':'native Windows PE32 targeted startup microtests','inputs':verify(directory),'observations':observations,'native_only_observations':native_only},indent=2)+'\n')
    print('Passed 8 identical-PE native startup cases; captured separate native NLS observation')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();check(a.output,a.native)
