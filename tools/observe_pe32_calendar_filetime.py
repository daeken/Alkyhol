#!/usr/bin/env python3
"""Native observations for the next real CRT FILETIME/calendar conversion."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('FileTimeToSystemTime',8)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['calendar-filetime.c','calendar-filetime.def','calendar-layout.cpp']},'files':{'calendar-filetime.exe':digest(d/'calendar-filetime.exe')}}
def verify(d):
    value=json.loads((d/'calendar-filetime-inputs.json').read_text())
    if value!=inputs(d):raise ValueError('calendar FILETIME source/PE identity changed')
    if value!=frozen()['inputs']:raise ValueError('changed frozen calendar FILETIME inputs')
    return value
def frozen():
    p=ROOT/'pe/reference/pe32-calendar-filetime-windows-v1.json'
    if digest(p)!='472812221c1704d1bdb0727fb5becc562ffba9235dc7c6ba5c8240d8c1237fa5':raise ValueError('changed frozen native calendar report')
    r=json.loads(p.read_bytes())
    if r['status'] or r['stderr_hex'] or r['backend']!='native Windows PE32 FILETIME calendar observer':raise ValueError('invalid native calendar evidence')
    if decode(bytes.fromhex(r['stdout_hex']))!=r['records']:raise ValueError('native calendar raw/decoded record differs')
    return r
def compare(records):
    if records!=frozen()['records']:raise ValueError('calendar fields, invalid-range error, source or guards differ from native Windows')
def fixture(d):
    d.mkdir(parents=True,exist_ok=True);p=ROOT/'pe/reference/pe32-calendar-filetime-v1.exe'
    if digest(p)!=frozen()['inputs']['files']['calendar-filetime.exe']:raise ValueError('changed literal owned calendar PE')
    shutil.copyfile(p,d/'calendar-filetime.exe');(d/'calendar-filetime-inputs.json').write_text(json.dumps(frozen()['inputs'],indent=2)+'\n');verify(d)
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
    report['records']=decode(r.stdout);p.write_text(json.dumps(report,indent=2)+'\n');compare(report['records']);print('Native calendar fields, invalid-range errors and preserved guards agree with the frozen capture')
def host(d):
    identity=verify(d);probe=runner(d);reports=[]
    for forced in [False,True]:
        r=subprocess.run([str(probe),str(d/'calendar-filetime.exe'),'--strict','--file-origins',*(['--rebase'] if forced else [])],capture_output=True,timeout=20)
        report={'backend':'Rosetta PE32 calendar conversion','forced_relocation':forced,'inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report);p=d/'calendar-filetime-host.json';p.write_text(json.dumps(reports,indent=2)+'\n')
        if r.returncode:raise ValueError('host calendar observer did not complete')
        report['records']=decode(r.stdout);compare(report['records']);p.write_text(json.dumps(reports,indent=2)+'\n')
    print('All 444 calendar transport bytes agree with native Windows at normal and forced placement')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');m.add_argument('--frozen-fixture',action='store_true');a=p.parse_args();(native if a.native else host if a.host else fixture if a.frozen_fixture else build)(a.output.resolve())
