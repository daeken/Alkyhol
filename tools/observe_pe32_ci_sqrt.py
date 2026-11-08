#!/usr/bin/env python3
"""Owned native x87 _CIsqrt evidence; no assumed host math implementation."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['ci-sqrt.c','ci-sqrt.S','ci-sqrt.def','crt-kernel32.def']},'files':{'ci-sqrt.exe':digest(d/'ci-sqrt.exe')}}
def verify(d):
    r=json.loads((d/'ci-sqrt-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('x87 observer source/PE identity changed')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[];objects=[]
    if not clang:raise RuntimeError('open Clang is required')
    for name in ['ci-sqrt','crt-kernel32']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    for suffix in ['c','S']:
        obj=d/('ci-sqrt-'+suffix+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/('ci-sqrt.'+suffix)),'-o',str(obj)],check=True);objects.append(str(obj))
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('SetLastError',4),('GetLastError',0)]]
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',*objects,*libs,*aliases,'/out:'+str(d/'ci-sqrt.exe')],check=True)
    (d/'ci-sqrt-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned direct x87 observer; no native/host result is assumed')
def decode(data):
    if len(data)!=24+204*68 or struct.unpack_from('<6I',data)!=(0x53513332,4,60,68,17,12):raise ValueError('incomplete x87 observer transport')
    records=[]
    for row in range(204):
        b=data[24+row*68:24+(row+1)*68];r=b[4:64];index,requested=struct.unpack_from('<2I',r)
        expected=0x7f|[0,0x200,0x300][row//68]|(((row//17)%4)<<10)
        if index!=row%17 or requested!=expected or any(struct.unpack_from('<I',b,at)[0]!=0x715e7a11 for at in [0,60,64]) or r[18:20]!=b'\xa5\xa5':raise ValueError('x87 observer order/control/guard damaged')
        cw,sw,cwa,swa,swp=struct.unpack_from('<5H',r,8);mx,mxa,error,errno=struct.unpack_from('<4I',r,40)
        records.append({'input':index,'requested':requested,'cw_before':cw,'sw_before':sw,'cw_after':cwa,'sw_after':swa,'sw_pop':swp,'result80':r[20:30].hex(),'below80':r[30:40].hex(),'mx_before':mx,'mx_after':mxa,'last_error':error,'errno':errno})
    return records
def native(d):
    identity=verify(d);r=subprocess.run([str((d/'ci-sqrt.exe').resolve())],capture_output=True,timeout=20)
    report={'backend':'native Windows direct x87 _CIsqrt observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};p=d/'native-ci-sqrt.json';p.write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or r.stderr:raise ValueError('native x87 observer failed; no completed result claimed')
    report['records']=decode(r.stdout);p.write_text(json.dumps(report,indent=2)+'\n');print('Captured actual 80-bit results, stack witness, control/status, MXCSR, last error and errno; host parity separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
