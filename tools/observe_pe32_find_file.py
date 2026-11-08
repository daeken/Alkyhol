#!/usr/bin/env python3
"""Owned native-only exact-file evidence for DAO's next actual CRT blocker."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest

SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleFileNameW',12),('FindFirstFileW',8),('FindNextFileW',8),('FindClose',4)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['find-file.c','find-file.def','find-layout.cpp']},'files':{'find-file.exe':digest(d/'find-file.exe')}}
def verify(d):
    v=json.loads((d/'find-file-inputs.json').read_text())
    if v!=inputs(d):raise ValueError('owned find-file source/PE identity changed')
    return v
def build(d):
    d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');clang=shutil.which('clang');obj=d/'find-file.obj';lib=d/'find-file.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'find-file.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'find-file.c'),'-o',str(obj)],check=True)
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/'find-file.exe')],check=True)
    (d/'find-file-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
    print('Built owned exact-file ABI/metadata/handle observer; no host execution claimed')
def metadata(path):
    st=path.stat()
    return {'size':st.st_size,'birth_ns':st.st_birthtime_ns,'access_ns':st.st_atime_ns,'write_ns':st.st_mtime_ns}
def decode(data,before,after):
    prefix=struct.pack('<7I',0x46465354,0x10,0x20,0x30,0x40,0x50,0x60)
    if not data.startswith(prefix):raise ValueError('find-file lifecycle stopped before all actual API stages')
    data=data[len(prefix):]
    if len(data)<596 or (len(data)-592)%4:raise ValueError('incomplete find-file transport')
    n=(len(data)-592)//4;words=list(struct.unpack('<'+'I'*n,data[:n*4]));record=data[n*4:]
    if words[:4]!=[0x464e3332,4,592,1] or words[-1]!=n:raise ValueError('find-file layout/complete-lifecycle contract differs')
    fields=list(struct.unpack('<11I',record[:44]));times={}
    for name,index in [('birth_ns',1),('access_ns',3),('write_ns',5)]:
        ticks=fields[index]+(fields[index+1]<<32)
        if ticks not in [v[name]//100+116444736000000000 for v in [before,after]]:raise ValueError('find-file time is not backed by actual filesystem metadata: '+name)
        times[name]=ticks
    if fields[8]+(fields[7]<<32)!=7:raise ValueError('find-file size is not actual owned fixture size')
    name=record[44:564].decode('utf-16-le').split('\0',1)[0]
    if name!='probe.dat':raise ValueError('find-file name differs from actual fixture')
    return {'words':words,'record_hex':record.hex(),'fields':fields,'timestamps':times}
def native(d):
    identity=verify(d);run=d/'find-native';run.mkdir(exist_ok=True);shutil.copyfile(d/'find-file.exe',run/'find-file.exe');file=run/'probe.dat';file.write_bytes(b'Alkyhol')
    if (run/'absent.dat').exists() or (run/'absent-dir').exists():raise ValueError('owned missing-path controls are not absent')
    before=metadata(file);r=subprocess.run([str((run/'find-file.exe').resolve())],cwd=run,capture_output=True,timeout=20);after=metadata(file)
    report={'backend':'native Windows PE32 exact-file observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex(),'metadata_before':before,'metadata_after':after};path=d/'native-find-file.json';path.write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or r.stderr:raise ValueError('native exact-file observer did not complete')
    report['decoded']=decode(r.stdout,before,after);path.write_text(json.dumps(report,indent=2)+'\n')
    print('Captured actual native metadata, success/error/output behavior and handle lifecycle; host support remains separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
