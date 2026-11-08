#!/usr/bin/env python3
"""Native SYSTEM_INFO and actual CPUID observations for the reached DAO blocker."""
import argparse,json,os,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
SOURCES=['system-info.c','system-info.def','system-info-layout.cpp','system-info-reservations.c','system-info-reservations.def']
FILES=['system-info.exe','system-info-large.exe','system-info-bad.exe','system-info-reservations.exe','system-info-reservations-large.exe']
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    value=json.loads((d/'system-info-inputs.json').read_text())
    if value!=inputs(d):raise ValueError('system information source/PE identities changed')
    original=frozen()['inputs']
    if any(value[category].get(name)!=checksum for category in ['sources','files'] for name,checksum in original[category].items()):raise ValueError('changed frozen SYSTEM_INFO producer/PE identities')
    return value

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');obj=d/'system-info.obj';lib=d/'system-info.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'system-info.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'system-info.c'),'-o',str(obj)],check=True)
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetSystemInfo',4)]]
    common=[lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(lib),*aliases]
    for name,literal in zip(FILES[:2],['pe32-system-info-v1.exe','pe32-system-info-large-v1.exe']):
        path=ROOT/'pe/reference'/literal
        if digest(path)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native SYSTEM_INFO PE')
        shutil.copyfile(path,d/name)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-DMODE=1','-c',str(SOURCE/'system-info.c'),'-o',str(d/'system-info-bad.obj')],check=True)
    subprocess.run([*common,str(d/'system-info-bad.obj'),'/out:'+str(d/FILES[2])],check=True)
    extra=d/'system-info-reservations.lib';bounds_obj=d/'system-info-reservations.obj'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'system-info-reservations.def'),'/out:'+str(extra)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-c',str(SOURCE/'system-info-reservations.c'),'-o',str(bounds_obj)],check=True)
    extras=[f'/alternatename:__imp__{name}@{n}=__imp__{name}' for name,n in [('GetModuleHandleA',4),('GetCommandLineW',0),('HeapCreate',12),('HeapAlloc',12),('HeapFree',12)]]
    for name,flag in zip(FILES[3:],['/largeaddressaware:no','/largeaddressaware']):subprocess.run([*common,flag,str(extra),str(bounds_obj),*extras,'/out:'+str(d/name)],check=True)
    (d/'system-info-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Selected pinned ordinary/LAA SYSTEM_INFO PEs and built the bad-pointer restriction')

def decode(data):
    if len(data)!=100:raise ValueError('incomplete system information transport')
    words=list(struct.unpack('<25I',data))
    if words[:6]!=[0x53493332,4,36,1,0x715e7a11,0x715e7a11] or words[7]!=25:raise ValueError('system information layout, stack or guards differ')
    fields=words[8:17]
    if fields[0]&0xffff!=0 or not fields[1] or fields[1]&(fields[1]-1) or not 0<fields[2]<fields[3] or not fields[4] or not 0<fields[5]<=4096 or not fields[7] or fields[7]&(fields[7]-1):raise ValueError('incomplete or unsupported native x86 system information')
    return {'words':words,'information_words':fields,'cpuid0':words[17:21],'cpuid1':words[21:25],'last_error':words[6]}

def validate_machine(record,large,count):
    if not 1<=count<=32:raise ValueError('CPU group outside the admitted machine profile')
    vendor=record['cpuid0'];signature=record['cpuid1'][0];base=(signature>>8)&15
    family=base+((signature>>20)&255 if base==15 else 0)
    intel=vendor[1:]==[0x756e6547,0x6c65746e,0x49656e69];amd=vendor[1:]==[0x68747541,0x444d4163,0x69746e65]
    if not vendor[0] or not (intel and family==6 or amd and family==25):raise ValueError('CPU family/vendor outside the admitted observed profile')
    model=((signature>>4)&15)|((signature>>12)&0xf0 if base in [6,15] else 0)
    revision=(model<<8)|(signature&15)
    expected=[0,4096,65536,0xfffeffff if large else 0x7ffeffff,(1<<count)-1,count,586,65536,(revision<<16)|family]
    if record['information_words']!=expected or record['last_error']!=0xface1234:raise ValueError('SYSTEM_INFO differs from actual per-host CPU observations or selected address model')
    return expected

def frozen():
    p=ROOT/'pe/reference/pe32-system-info-windows-v1.json'
    if digest(p)!='9e6d3399fb6d2c2798aa3bad982b0a89ea83cd8eadccb1d4065dc96d3ab6a6ee':raise ValueError('changed frozen native system information evidence')
    value=json.loads(p.read_bytes())
    if value['backend']!='native Windows PE32 SYSTEM_INFO/CPUID observer' or len(value['reports'])!=2:raise ValueError('incomplete native system information evidence')
    for i,row in enumerate(value['reports']):
        if row['file']!=FILES[i] or row['status'] or row['stderr_hex'] or decode(bytes.fromhex(row['stdout_hex']))!=row['record']:raise ValueError('native SYSTEM_INFO raw/decoded records differ')
        validate_machine(row['record'],i==1,row['record']['information_words'][5])
    return value

def decode_storage(data,large):
    if len(data)!=72:raise ValueError('incomplete real-storage machine-profile transport')
    w=list(struct.unpack('<18I',data));minimum=65536;maximum=0xfffeffff if large else 0x7ffeffff
    if w[:4]!=[0x424d3332,4,minimum,maximum] or w[8]!=65540 or w[10:]!=[0x71,0x19,1,1,0xface1234,0x715e7a11,0x715e7a11,18]:raise ValueError('real-storage machine profile, guards or memory effects differ')
    for index,length in [(4,1),(5,4),(6,36),(7,w[8]),(9,2)]:
        if not minimum<=w[index] or w[index]+length>maximum+1:raise ValueError('actual guest storage falls outside the reported application band')
    if w[4]%65536 or w[5]%4 or w[6]%4 or w[7]%8 or w[9]%2:raise ValueError('actual image/data allocation alignment differs')
    return {'words':w,'contract':w[:4]+[w[8]]+w[10:]}

def host(d):
    from observe_pe32_activation_startup import runner
    identity=verify(d);probe=runner(d);reports=[];path=d/'system-info-host.json'
    for name,enabled,reason in [(FILES[0],True,None),(FILES[1],True,None),(FILES[2],True,b'SYSTEM_INFO outside aligned writable guest storage'),(FILES[0],False,b'GetSystemInfo requires the explicit host-machine profile'),(FILES[3],True,None),(FILES[4],True,None)]:
        command=[str(probe),str((d/name).resolve()),'--strict','--file-origins']
        if enabled:command+=['--host-machine-info']
        r=subprocess.run(command,capture_output=True,timeout=20);row={'file':name,'enabled':enabled,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row)
        def save():path.write_text(json.dumps({'backend':'Rosetta host-backed x86 machine profile','inputs':identity,'parent_cpu_count':os.cpu_count(),'reports':reports},indent=2)+'\n')
        save()
        if reason:
            if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('machine profile restriction failed for its actual reason')
        else:
            if r.returncode:raise ValueError('host SYSTEM_INFO witness failed; raw result retained')
            if name in FILES[3:]:row['record']=decode_storage(r.stdout,name==FILES[4])
            else:
                row['record']=decode(r.stdout);validate_machine(row['record'],name==FILES[1],os.cpu_count())
            save()
    print('All ordinary/LAA structure fields agree with actual host CPUID/count and enforced address bands; two restrictions stop')

def native(d):
    identity=verify(d);reports=[];path=d/'native-system-info.json'
    def save():path.write_text(json.dumps({'backend':'native Windows PE32 SYSTEM_INFO/CPUID observer','inputs':identity,'reports':reports},indent=2)+'\n')
    for name in [*FILES[:2],*FILES[3:]]:
        r=subprocess.run([str((d/name).resolve())],capture_output=True,timeout=20);report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report);save()
        if r.returncode or r.stderr:raise ValueError('native system information observer failed; raw result retained')
        if name in FILES[3:]:report['record']=decode_storage(r.stdout,name==FILES[4])
        else:
            report['record']=decode(r.stdout);validate_machine(report['record'],name==FILES[1],os.cpu_count())
        save()
    host_path=d/'system-info-host.json'
    if not host_path.exists():raise ValueError('missing peer host SYSTEM_INFO observation')
    h=json.loads(host_path.read_text())
    if h['inputs']!=identity or len(h['reports'])!=6:raise ValueError('native/host SYSTEM_INFO input identities or complete roster differ')
    for i,row in enumerate(h['reports'][:2]):
        if row['file']!=FILES[i] or row['status'] or decode(bytes.fromhex(row['stdout_hex']))!=row['record']:raise ValueError('host raw/decoded system information differs')
        validate_machine(row['record'],i==1,h['parent_cpu_count'])
    for i,name in enumerate(FILES[3:]):
        row=h['reports'][4+i]
        if row['file']!=name or row['status'] or decode_storage(bytes.fromhex(row['stdout_hex']),i==1)!=row['record'] or row['record']['contract']!=reports[2+i]['record']['contract']:raise ValueError('native/host real-storage machine-profile contracts differ')
    print('Ordinary/LAA SYSTEM_INFO semantics agree with each host CPU and parent count; all raw per-host bytes retained')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');mode=p.add_mutually_exclusive_group();mode.add_argument('--native',action='store_true');mode.add_argument('--host',action='store_true');a=p.parse_args();(native if a.native else host if a.host else build)(a.output.resolve())
