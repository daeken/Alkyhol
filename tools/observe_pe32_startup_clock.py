#!/usr/bin/env python3
"""Per-host PID/boot-clock/counter/atomic observations for actual FMOD startup."""
import argparse,ctypes,json,os,shutil,struct,subprocess,time
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetCurrentProcessId',0),('GetCurrentThreadId',0),('GetTickCount',0),('QueryPerformanceCounter',4),('QueryPerformanceFrequency',4),('InterlockedCompareExchange',12),('InterlockedExchange',8)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['startup-clock.c','startup-clock.def']},'files':{'startup-clock.exe':digest(d/'startup-clock.exe')}}
def verify(d):
    v=json.loads((d/'startup-clock-inputs.json').read_text())
    if v!=inputs(d):raise ValueError('startup clock source/PE identity changed')
    return v
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');obj=d/'startup-clock.obj';lib=d/'startup-clock.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'startup-clock.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'startup-clock.c'),'-o',str(obj)],check=True)
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/'startup-clock.exe')],check=True)
    (d/'startup-clock-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned startup clock/atomic observer; native execution remains separate')
def decode(data,pid,before,after,duration):
    if len(data)%4 or len(data)<4:raise ValueError('truncated startup clock transport')
    words=list(struct.unpack('<'+'I'*(len(data)//4),data))
    if len(words)!=47 or words[:2]!=[0x434c3332,4] or words[-1]!=len(words):raise ValueError('startup clock order/layout differs')
    if words[2]!=pid or words[4]!=pid or not words[5] or words[5]!=words[7]:raise ValueError('PID is not actual child PID or thread identity is unstable')
    if words[8]!=1 or words[11]!=0 or not words[10] or words[15]!=1 or words[21]!=1:raise ValueError('counter/frequency result differs')
    for index in [3,6,9,14,16,22,27,32,37,42]:
        if words[index]!=0xface1234:raise ValueError('startup clock/atomic last error differs')
    for index in [12,19,25,29,30,34,35,39,40,44,45]:
        if words[index]!=0x715e7a11:raise ValueError('startup clock/atomic guard damaged')
    for value in [words[13],words[20]]:
        if (value-((before-32)&0xffffffff))&0xffffffff>after-before+64:raise ValueError('tick count is not bracketed by actual per-host boot clock')
    ticks=(words[20]-words[13])&0xffffffff;delta=(words[23]+(words[24]<<32))-(words[17]+(words[18]<<32));elapsed=delta/words[10]
    if ticks<32 or not max(0.001,ticks/1000-0.025)<=elapsed<=ticks/1000+0.025 or elapsed>duration+0.1:raise ValueError('counter rate is inconsistent with independent boot clock and parent interval')
    if [(words[i],words[i+2]) for i in [26,31,36]]!=[(0,0xffffffff),(0xffffffff,0xffffffff),(0xffffffff,0x80000000)]:raise ValueError('compare-exchange old/new bit semantics differ')
    if (words[41],words[43])!=(0x80000000,0xffffffff):raise ValueError('exchange old/new bit semantics differ')
    return {'words':words,'frequency':words[10],'tick_delta_ms':ticks,'counter_elapsed_seconds':elapsed}
def execute(d,native=False):
    identity=verify(d);command=[str((d/'startup-clock.exe').resolve())]
    if native:
        get_ticks=ctypes.windll.kernel32.GetTickCount64;get_ticks.restype=ctypes.c_uint64
    else:
        probe=runner(d);reader=d/'startup-clock-macos';subprocess.run([shutil.which('clang'),'-std=c11','-Wall','-Wextra','-Werror',str(SOURCE/'startup-clock-macos.c'),'-o',str(reader)],check=True)
        get_ticks=lambda:int(subprocess.check_output([str(reader)]));command=[str(probe),*command,'--strict','--file-origins']
    before=get_ticks();start=time.monotonic();p=subprocess.Popen(command,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    try:stdout,stderr=p.communicate(timeout=20)
    except subprocess.TimeoutExpired:p.kill();p.communicate();raise RuntimeError('startup clock budget exhausted; no successful observation claimed')
    duration=time.monotonic()-start;after=get_ticks();r={'backend':'native Windows PE32 startup clock observer' if native else 'physical M4 host startup clock assertions','inputs':identity,'status':p.returncode,'pid':p.pid,'boot_ms_before':before,'boot_ms_after':after,'parent_elapsed_seconds':duration,'stdout_hex':stdout.hex(),'stderr_hex':stderr.hex()};path=d/('native-startup-clock.json' if native else 'startup-clock-host.json');path.write_text(json.dumps(r,indent=2)+'\n')
    if p.returncode or native and stderr:raise ValueError('startup clock observer failed')
    r['decoded']=decode(stdout,p.pid,before,after,duration);path.write_text(json.dumps(r,indent=2)+'\n');print('Actual child PID, live independent clocks, real compare-exchange bits and guards checked; native comparison separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
