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
    nls=output/'nls.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(source/'nls.def'),'/out:'+str(nls)],check=True)
    for mode in range(4):
        obj=output/f'nls{mode}.obj'
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(source/'nls.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/entry:entry',str(obj),str(nls),*[f'/alternatename:__imp__{name}@{size}=__imp__{name}' for name,size in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetStringTypeW',16),('MultiByteToWideChar',24),('LCMapStringW',24)]],'/out:'+str(output/f'nls{mode}.exe')],check=True)
    inputs={'files':{name:hashlib.sha256((output/name).read_bytes()).hexdigest() for name in input_files()},'sources':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source/n for n in ['guard.c','kernel32.def','service.def','nls.c','nls.def']]}}
    (output/'microtest-inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo');rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc');target='x86_64-apple-darwin'
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline'],check=True)
    build=ROOT/'pe/target'/target/'debug';probe=output/'startup-macos'
    subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_startup32.rs'),'--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),'-L','dependency='+str(build/'deps'),'-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(probe)],check=True)
    observations=[]
    environment=dict(os.environ,ALKYHOL_STARTUP_MICRO="active",ALKYHOL_STARTUP_UNICODE="λ🌸",ALKYHOL_STARTUP_EMPTY="");environment.pop("ALKYHOL_STARTUP_MICRO_ABSENT",None)
    for mode in [0,1,3,4,5,6,7,8,9,10,11,12,13]:
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
    for mode in range(3):
        r=subprocess.run([str(probe),str(output/f'nls{mode}.exe')],input=b'',capture_output=True,timeout=15)
        expected=nls_expected() if mode==0 else struct.pack('<I',0x53543332)
        if r.returncode!=(0 if mode==0 else 78) or r.stdout!=expected:raise ValueError(f'NLS host microtest {mode}: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        if mode and b'unsupported' not in r.stderr and b'outside measured repertoire' not in r.stderr:raise ValueError('missing explicit NLS restriction')
        observations.append({'case':f'nls{mode}','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    (output/'microtest-results.json').write_text(json.dumps({'backend':'Alkyhol PE32 host instrumentation microtests','inputs':verify(output),'observations':observations},indent=2)+'\n')
    print('Passed 16 host startup microtests, including frozen native CTYPE1; built LCMap observer only')

def input_files():
    return {f'case{m}.exe' for m in [0,1,3,4,5,6,7,8,9,10,11,12,13]}|{f'nls{m}.exe' for m in range(4)}

def frozen():
    raw=(ROOT/'pe/reference/pe32-startup-windows-v1.json').read_bytes()
    if hashlib.sha256(raw).hexdigest()!='e872d29c86d34d392d8481941e593d38c5cfab7e472fd802c6071d556dda8015':raise ValueError('changed frozen native startup report')
    value=json.loads(raw)
    if value['backend']!='native Windows PE32 targeted startup microtests':raise ValueError('wrong frozen native backend')
    return value

def nls_expected():
    return struct.pack('<12I',0x4e4c5332,1,0x382,0x381,0x220,0x268,0x382,0xbeef,1,0x381,0x220,0xbeef)

def verify(directory):
    inputs=json.loads((directory/'microtest-inputs.json').read_text(encoding='utf-8'))
    if set(inputs['files'])!=input_files() or set(inputs['sources'])!={'pe/fixtures/pe32-startup/'+n for n in ['guard.c','kernel32.def','service.def','nls.c','nls.def']}:raise ValueError('changed microtest input roster')
    previous=frozen()
    for kind in ['files','sources']:
        if any(inputs[kind].get(name)!=digest for name,digest in previous['inputs'][kind].items()):raise ValueError('changed frozen native startup inputs')
    for name,digest in inputs['files'].items():
        if hashlib.sha256((directory/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed microtest executable')
    for name,digest in inputs['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed microtest source')
    return inputs

def expected_positive(mode):
    if mode==13:return bytes.fromhex(frozen()['native_only_observations'][0]['stdout_hex'])
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
    # Native-only capture for LCMapStringW, the next actual original CRT blocker.
    r=subprocess.run([str((directory/'nls3.exe').resolve())],capture_output=True,timeout=15)
    if r.returncode!=0 or len(r.stdout)<20 or r.stderr:raise ValueError('native LCMap observation failed')
    magic,lo,up,le,ue=struct.unpack('<5I',r.stdout[:20])
    if magic!=0x4c434d31 or max(lo,up)>512:raise ValueError('invalid native LCMap batch capture')
    at=20+2*(lo+up)
    if r.stdout[at:at+8]!=struct.pack('<2I',0x4c434d32,256):raise ValueError('missing native LCMap per-unit capture')
    at+=8
    for _ in range(256):
        if at+20>len(r.stdout):raise ValueError('truncated LCMap row')
        unit,lo,up,le,ue=struct.unpack('<5I',r.stdout[at:at+20]);at+=20
        if unit>65535 or max(lo,up)>8:raise ValueError('invalid LCMap row widths')
        at+=2*(lo+up)
    if at!=len(r.stdout):raise ValueError('truncated or extra LCMap observations')
    native_only=[{'case':'nls3','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}]
    (directory/'native-lcmap-observation.json').write_text(json.dumps({'backend':'native Windows LCMapStringW observation only','inputs':verify(directory),'observations':native_only},indent=2)+'\n')
    for mode in [3,4,5,6,9,10,11,12,13]:
        r=subprocess.run([str((directory/f'case{mode}.exe').resolve())],env=environment,input=b'',capture_output=True,timeout=15)
        if r.returncode!=0 or r.stdout!=expected_positive(mode) or r.stderr:raise ValueError(f'native startup microtest {mode}: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        observations.append({'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    r=subprocess.run([str((directory/'nls0.exe').resolve())],capture_output=True,timeout=15)
    if r.returncode!=0 or r.stdout!=nls_expected() or r.stderr:raise ValueError(f'native NLS order/termination microtest failed: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
    observations.append({'case':'nls0','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    (directory/'native-microtest-results.json').write_text(json.dumps({'backend':'native Windows PE32 targeted startup microtests','inputs':verify(directory),'observations':observations,'native_only_observations':native_only},indent=2)+'\n')
    print('Passed 10 identical-PE native startup cases; captured separate native LCMap observation')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();check(a.output,a.native)
