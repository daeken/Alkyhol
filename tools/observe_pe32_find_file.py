#!/usr/bin/env python3
"""Owned identical-PE exact-file evidence for DAO's actual CRT blocker."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner

SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleFileNameW',12),('FindFirstFileW',8),('FindNextFileW',8),('FindClose',4)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in ['find-file.c','find-file.def','find-layout.cpp']},'files':{'find-file.exe':digest(d/'find-file.exe')}}
def verify(d):
    v=json.loads((d/'find-file-inputs.json').read_text())
    if v!=inputs(d):raise ValueError('owned find-file source/PE identity changed')
    if v!=frozen()['inputs']:raise ValueError('changed frozen native exact-file inputs')
    return v
def frozen():
    p=ROOT/'pe/reference/pe32-find-file-windows-v1.json'
    if digest(p)!='b09164fb590477d5bb6bb8d20eca305387ab99446443ada0d29944eee9297c82':raise ValueError('changed frozen native exact-file report')
    r=json.loads(p.read_bytes())
    if r['status'] or r['stderr_hex'] or r['backend']!='native Windows PE32 exact-file observer':raise ValueError('invalid native exact-file evidence')
    if decode(bytes.fromhex(r['stdout_hex']),r['metadata_before'],r['metadata_after'])!=r['decoded']:raise ValueError('native decoded record differs from its raw transport')
    return r
def comparable(value):
    raw=bytearray.fromhex(value['record_hex']);raw[4:28]=bytes(24)
    return {'words':value['words'],'record_without_host_times':raw.hex()}
def compare(value):
    if comparable(value)!=comparable(frozen()['decoded']):raise ValueError('exact-file fields/holes/guards/handle lifecycle differ from native Windows')
def build(d):
    d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');clang=shutil.which('clang');obj=d/'find-file.obj';lib=d/'find-file.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'find-file.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'find-file.c'),'-o',str(obj)],check=True)
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/'find-file.exe')],check=True)
    (d/'find-file-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
    print('Built owned exact-file ABI/metadata/handle observer; no host execution claimed')
def frozen_fixture(d):
    # Literal owned reference input, not a recompiled substitute. This is useful
    # when another open Clang version emits different instruction bytes.
    d.mkdir(parents=True,exist_ok=True);source=ROOT/'pe/reference/pe32-find-file-v1.exe'
    if digest(source)!=frozen()['inputs']['files']['find-file.exe']:raise ValueError('changed literal owned reference PE')
    shutil.copyfile(source,d/'find-file.exe');(d/'find-file-inputs.json').write_text(json.dumps(frozen()['inputs'],indent=2)+'\n');verify(d)
    print('Selected literal owned PE executed in the frozen Windows capture; source/PE identities verified')
def metadata(path,reader=None):
    if reader is not None:return json.loads(subprocess.check_output([str(reader),str(path)]))
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
def host(d):
    identity=verify(d);probe=runner(d);reader=d/'find-metadata-macos';source=SOURCE/'find-metadata-macos.c'
    subprocess.run([shutil.which('clang'),'-std=c11','-Wall','-Wextra','-Werror',str(source),'-o',str(reader)],check=True)
    run=d/'find-host λ🌸';run.mkdir(exist_ok=True);shutil.copyfile(d/'find-file.exe',run/'find-file.exe');file=run/'probe.dat';file.write_bytes(b'Alkyhol')
    if (run/'absent.dat').exists() or (run/'absent-dir').exists():raise ValueError('owned missing controls are not absent')
    reports=[]
    for forced in [False,True]:
        before=metadata(file,reader);r=subprocess.run([str(probe),str(run/'find-file.exe'),'--strict','--file-origins',*(['--rebase'] if forced else [])],cwd=run,capture_output=True,timeout=20);after=metadata(file,reader)
        report={'backend':'Rosetta PE32 exact-file metadata and handles','inputs':identity,'host_metadata_reader_sha256':digest(source),'forced_relocation':forced,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex(),'metadata_before':before,'metadata_after':after}
        reports.append(report);path=d/'find-file-host.json';path.write_text(json.dumps(reports,indent=2)+'\n')
        if r.returncode:raise ValueError('host exact-file observer did not complete')
        report['decoded']=decode(r.stdout,before,after);compare(report['decoded']);path.write_text(json.dumps(reports,indent=2)+'\n')
    print('Both M4 exact-file runs match native fields/holes/errors/handles; timestamps match independent per-host SDK stat data')
def native(d):
    identity=verify(d);run=d/'find-native';run.mkdir(exist_ok=True);shutil.copyfile(d/'find-file.exe',run/'find-file.exe');file=run/'probe.dat';file.write_bytes(b'Alkyhol')
    if (run/'absent.dat').exists() or (run/'absent-dir').exists():raise ValueError('owned missing-path controls are not absent')
    before=metadata(file);r=subprocess.run([str((run/'find-file.exe').resolve())],cwd=run,capture_output=True,timeout=20);after=metadata(file)
    report={'backend':'native Windows PE32 exact-file observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex(),'metadata_before':before,'metadata_after':after};path=d/'native-find-file.json';path.write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or r.stderr:raise ValueError('native exact-file observer did not complete')
    report['decoded']=decode(r.stdout,before,after);path.write_text(json.dumps(report,indent=2)+'\n')
    compare(report['decoded'])
    print('Captured actual native metadata, success/error/output behavior and handle lifecycle; host support remains separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');m.add_argument('--frozen-fixture',action='store_true');a=p.parse_args();(native if a.native else lambda d:host(d) if a.host else frozen_fixture(d) if a.frozen_fixture else build(d))(a.output.resolve())
