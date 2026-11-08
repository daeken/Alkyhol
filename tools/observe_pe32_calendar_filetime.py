#!/usr/bin/env python3
"""Native observations for the next real CRT FILETIME/calendar conversion."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('FileTimeToSystemTime',8)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['calendar-filetime.c','calendar-filetime.def','calendar-layout.cpp']},'files':{'calendar-filetime.exe':digest(d/'calendar-filetime.exe')}}
def verify(d):
    value=json.loads((d/'calendar-filetime-inputs.json').read_text())
    if value!=inputs(d):raise ValueError('calendar FILETIME source/PE identity changed')
    return value
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');obj=d/'calendar-filetime.obj';lib=d/'calendar-filetime.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'calendar-filetime.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'calendar-filetime.c'),'-o',str(obj)],check=True)
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/'calendar-filetime.exe')],check=True)
    (d/'calendar-filetime-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned calendar/century/submillisecond/boundary observer; native execution pending')
def decode(data):
    if len(data)!=12+9*48 or struct.unpack('<3I',data[:12])!=(0x53543332,4,16):raise ValueError('incomplete calendar FILETIME transport')
    records=[]
    for i in range(9):
        words=list(struct.unpack_from('<8I',data,12+i*48));fields=list(struct.unpack_from('<8H',data,12+i*48+32))
        if words[0]!=i or words[3] not in [0,1] or words[5:7]!=[0x715e7a11]*2 or words[7]!=1:raise ValueError('calendar FILETIME source/output guards or order differ')
        records.append({'words':words,'fields':fields})
    return records
def native(d):
    identity=verify(d);r=subprocess.run([str((d/'calendar-filetime.exe').resolve())],capture_output=True,timeout=20)
    report={'backend':'native Windows PE32 FILETIME calendar observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};p=d/'native-calendar-filetime.json';p.write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or r.stderr:raise ValueError('native calendar observer failed; no completed calendar evidence claimed')
    report['records']=decode(r.stdout);p.write_text(json.dumps(report,indent=2)+'\n');print('Captured real native calendar fields, invalid-range errors and preserved guards; host parity remains separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
