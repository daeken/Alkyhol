#!/usr/bin/env python3
"""Targeted PE32 microtests for APIs reached by actual DAO CRT startup."""
import argparse
import json
import hashlib
import os
from pathlib import Path
import shutil
import struct
import subprocess
import time
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
    lib=output/'lcmap.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(source/'lcmap.def'),'/out:'+str(lib)],check=True)
    for mode in range(5):
        obj=output/f'map{mode}.obj'
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(source/'lcmap.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{name}@{size}=__imp__{name}' for name,size in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('LCMapStringW',24),('InterlockedDecrement',4),('IsProcessorFeaturePresent',4)]],'/out:'+str(output/f'map{mode}.exe')],check=True)
    lib=output/'clock-heap.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(source/'clock-heap.def'),'/out:'+str(lib)],check=True)
    for mode in range(3):
        obj=output/f'clock{mode}.obj'
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(source/'clock-heap.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{name}@{size}=__imp__{name}' for name,size in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetSystemTimeAsFileTime',4),('GetProcessHeap',0),('HeapCreate',12),('HeapAlloc',12),('HeapFree',12),('HeapSize',12)]],'/out:'+str(output/f'clock{mode}.exe')],check=True)
    inputs={'files':{name:hashlib.sha256((output/name).read_bytes()).hexdigest() for name in input_files()},'sources':{str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [source/n for n in ['guard.c','kernel32.def','service.def','nls.c','nls.def','lcmap.c','lcmap.def','clock-heap.c','clock-heap.def']]}}
    (output/'microtest-inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo');rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc');target='x86_64-apple-darwin'
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline'],check=True)
    build=ROOT/'pe/target'/target/'debug';probe=output/'startup-macos'
    subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_startup32.rs'),'--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),'-L','dependency='+str(build/'deps'),'-L','dependency='+str(build.parent.parent/'debug/deps'),'-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(probe)],check=True)
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
    for name,expected in [('nls3.exe',lcmap_expected())]+[(f'map{mode}.exe',map_expected() if mode==0 else struct.pack('<I',0x53543332)) for mode in range(5)]:
        r=subprocess.run([str(probe),str(output/name)],input=b'',capture_output=True,timeout=15)
        status=78 if name.startswith('map') and name!='map0.exe' else 0
        if r.returncode!=status or r.stdout!=expected:raise ValueError(f'LCMap host microtest {name}: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        if status and b'unsupported' not in r.stderr and b'outside measured repertoire' not in r.stderr:raise ValueError('missing explicit mapping/feature restriction')
        observations.append({'case':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    for mode in range(3):
        before=time.time_ns();r=subprocess.run([str(probe),str(output/f'clock{mode}.exe')],input=b'',capture_output=True,timeout=15);after=time.time_ns()
        if mode==0:
            if r.returncode!=0:raise ValueError('host clock/heap query failed: '+r.stderr.decode(errors='replace'))
            validate_clock(r.stdout,before,after)
        elif r.returncode!=78 or r.stdout!=struct.pack('<I',0x53543332) or b'unsupported' not in r.stderr:raise ValueError('missing explicit clock/heap host restriction')
        observations.append({'case':f'clock{mode}','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex(),'host_clock_bounds_ns':[before,after]})
    (output/'microtest-results.json').write_text(json.dumps({'backend':'Alkyhol PE32 host instrumentation microtests','inputs':verify(output),'observations':observations},indent=2)+'\n')
    print('Passed 25 host startup cases; wall clock is checked against host UTC bounds, not cross-machine bit equality')

def input_files():
    return {f'case{m}.exe' for m in [0,1,3,4,5,6,7,8,9,10,11,12,13]}|{f'nls{m}.exe' for m in range(4)}|{f'map{m}.exe' for m in range(5)}|{f'clock{m}.exe' for m in range(3)}

def validate_clock(data,before_ns,after_ns):
    if len(data)!=40:raise ValueError('wrong clock/heap record width')
    words=struct.unpack('<10I',data)
    if words[0]!=0x54483332 or words[3:]!=(0x715e7a11,37,128,128,1,1,4):raise ValueError('clock output guard or heap query contract differs')
    ticks=words[1]|(words[2]<<32)
    # Windows' non-precise wall clock can trail the precise parent clock by a
    # scheduler tick. Bracket each machine independently with 100 ms tolerance.
    low=min(before_ns,after_ns)//100+116444736000000000-1_000_000
    high=max(before_ns,after_ns)//100+116444736000000000+1_000_000
    if not low<=ticks<=high:raise ValueError('guest FILETIME outside actual host UTC bounds')
    return words

def frozen():
    raw=(ROOT/'pe/reference/pe32-startup-windows-v1.json').read_bytes()
    if hashlib.sha256(raw).hexdigest()!='e872d29c86d34d392d8481941e593d38c5cfab7e472fd802c6071d556dda8015':raise ValueError('changed frozen native startup report')
    value=json.loads(raw)
    if value['backend']!='native Windows PE32 targeted startup microtests':raise ValueError('wrong frozen native backend')
    return value

def frozen_lcmap():
    raw=(ROOT/'pe/reference/pe32-lcmap-windows-v1.json').read_bytes()
    if hashlib.sha256(raw).hexdigest()!='e22977b5e30c49163aeeb8577f88d7ebc4bb6cc0919809c729c5f025b959635a':raise ValueError('changed frozen native LCMap report')
    return json.loads(raw)

def lcmap_expected():
    return bytes.fromhex(frozen_lcmap()['observations'][0]['stdout_hex'])

def map_expected():
    return struct.pack('<24I',0x4c434d33,4,4,0x49,0xdf,0x191,0,0xbeef,0,122,3,0x61,0x62,0,1,1,1,4,1,1,1,1,1,1)

def nls_expected():
    return struct.pack('<12I',0x4e4c5332,1,0x382,0x381,0x220,0x268,0x382,0xbeef,1,0x381,0x220,0xbeef)

def verify(directory):
    inputs=json.loads((directory/'microtest-inputs.json').read_text(encoding='utf-8'))
    if set(inputs['files'])!=input_files() or set(inputs['sources'])!={'pe/fixtures/pe32-startup/'+n for n in ['guard.c','kernel32.def','service.def','nls.c','nls.def','lcmap.c','lcmap.def','clock-heap.c','clock-heap.def']}:raise ValueError('changed microtest input roster')
    for previous in [frozen(),frozen_lcmap()]:
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
    for mode in [3,4,5,6,9,10,11,12,13]:
        r=subprocess.run([str((directory/f'case{mode}.exe').resolve())],env=environment,input=b'',capture_output=True,timeout=15)
        if r.returncode!=0 or r.stdout!=expected_positive(mode) or r.stderr:raise ValueError(f'native startup microtest {mode}: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        observations.append({'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    r=subprocess.run([str((directory/'nls0.exe').resolve())],capture_output=True,timeout=15)
    if r.returncode!=0 or r.stdout!=nls_expected() or r.stderr:raise ValueError(f'native NLS order/termination microtest failed: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
    observations.append({'case':'nls0','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    for name,expected in [('nls3.exe',lcmap_expected()),('map0.exe',map_expected())]:
        r=subprocess.run([str((directory/name).resolve())],capture_output=True,timeout=15)
        if r.returncode!=0 or r.stdout!=expected or r.stderr:raise ValueError(f'native LCMap case {name}: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        observations.append({'case':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
    before=time.time_ns();r=subprocess.run([str((directory/'clock0.exe').resolve())],capture_output=True,timeout=15);after=time.time_ns()
    if r.returncode!=0 or r.stderr:raise ValueError('native clock/heap microtest failed')
    validate_clock(r.stdout,before,after)
    observations.append({'case':'clock0','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex(),'host_clock_bounds_ns':[before,after]})
    (directory/'native-microtest-results.json').write_text(json.dumps({'backend':'native Windows PE32 targeted startup microtests','inputs':verify(directory),'observations':observations,'native_only_observations':[]},indent=2)+'\n')
    print('Passed 12 exact-output native startup cases plus independently bracketed clock/heap evidence')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();check(a.output,a.native)
