#!/usr/bin/env python3
"""Owned native-only DLL activation observations for DAO's actual CRT blocker.

macOS builds the exact PEs/manifests; only Windows executes this observer. No
original DLLs, game manifests or derived game inventories are accepted by it.
"""
import argparse
import json
from pathlib import Path
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest,resource_record

SOURCES=['ctx-observe.h','ctx-observe.c','ctx-provider.c','ctx-importer.c','ctx-startup.c',
         'ctx-kernel32.def','ctx-provider.def','ctx-importer.def','ctx-importer.manifest',
         'ctx-assembly.manifest','activation-empty.manifest']
FILES=['ctx-startup.exe','ctx-provider.dll','ctx-importer.dll','ctx-importer.manifest','ctx-assembly.manifest']
APIS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleFileNameW',12),
      ('GetModuleHandleW',4),('GetCurrentActCtx',4),('ReleaseActCtx',4),('FindActCtxSectionStringW',20),
      ('CreateActCtxW',4),('ActivateActCtx',8),('DeactivateActCtx',8)]
EVENTS=[(1,1),(2,1),(3,3),(3,4),(3,5),(2,0),(1,0)]

class Reader:
    def __init__(self,data):
        if len(data)>1024*1024:raise ValueError('activation observation budget exceeded')
        self.data=data;self.at=0
    def read(self,n):
        if n<0 or n>len(self.data)-self.at:raise ValueError('truncated activation observation')
        out=self.data[self.at:self.at+n];self.at+=n;return out
    def words(self,n):return list(struct.unpack('<'+'I'*n,self.read(n*4)))
    def blob(self,limit):
        n=self.words(1)[0]
        if n>limit:raise ValueError('activation blob exceeds observation budget')
        return self.read(n)
    def wide(self):
        n=self.words(1)[0]
        if not 0<n<1024:raise ValueError('invalid module path unit count')
        value=self.read((n+1)*2).decode('utf-16-le')
        if not value.endswith('\0') or '\0' in value[:-1]:raise ValueError('invalid path termination')
        return value[:-1]

def decode(data,paths):
    r=Reader(data);handles={0:0};records=[]
    def identity(raw):
        if raw not in handles:handles[raw]=len(handles)
        return handles[raw]
    for actor,reason in EVENTS:
        header=r.words(7)
        if header[:4]!=[0x43545832,actor,reason,1] or header[6]!=4:
            raise ValueError('incomplete/out-of-order native activation lifecycle or API failure')
        path=r.wide()
        if path!=paths[actor]:raise ValueError('native module did not load from its intended private-assembly/source path')
        row={'actor':actor,'event':reason,'current_error':header[4],'current_context_id':identity(header[5]),'path':path,'queries':[]}
        for flags in [0,1]:
            q=r.words(19);body=r.blob(4096);section=r.blob(65536)
            if q[0]!=flags or q[1] not in [0,1] or q[3]!=64 or q[15]!=0x715e7a11:
                raise ValueError('query ABI/output guard differs')
            if q[1]:
                if len(body)!=q[5] or len(section)!=q[7] or q[10]>len(section) or body!=section[q[10]:q[10]+len(body)]:
                    raise ValueError('DLL redirection data is not a complete section-backed payload')
                if q[6] and (q[18]>len(section) or q[6]>len(section)-q[18]):raise ValueError('global data outside captured section')
            elif body or section:raise ValueError('failed query emitted a valid-data payload')
            row['queries'].append({'words':q[:17]+q[18:],'found_context_id':identity(q[17]) if flags and q[1] else None,
                                   'data_hex':body.hex(),'section_hex':section.hex()})
        records.append(row)
    if r.at!=len(data):raise ValueError('extra or dropped native activation events')
    if any(q['words'][1]!=1 for q in records[3]['queries']):
        raise ValueError('explicit activation did not expose the owned private assembly')
    return records

def identities(directory):
    return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},
            'files':{n:digest(directory/n) for n in FILES}}

def verify(directory):
    value=json.loads((directory/'ctx-startup-inputs.json').read_text())
    if value!=identities(directory):raise ValueError('native activation source/PE/manifest identities changed')
    return value

def build(directory):
    directory.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang is required')
    lld=rust_tool('rust-lld')
    compile_args=[clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector',
                  '-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror']
    common=[lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no',
            '/timestamp:0','/safeseh:no','/manifest:no',str(directory/'ctx-kernel32.lib'),str(directory/'ctx-observe.obj'),
            *[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n,size in APIS]]
    for kind in ['ctx-kernel32','ctx-provider','ctx-importer']:
        subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(kind+'.def')),'/out:'+str(directory/(kind+'.lib'))],check=True)
    for name in ['ctx-observe','ctx-provider','ctx-importer','ctx-startup']:
        subprocess.run([*compile_args,'-c',str(SOURCE/(name+'.c')),'-o',str(directory/(name+'.obj'))],check=True)
    null=resource_record(0,0,0,b'')
    for name,rid,manifest in [('ctx-startup',1,'activation-empty.manifest'),('ctx-importer',2,'ctx-importer.manifest')]:
        (directory/(name+'.res')).write_bytes(null+resource_record(24,rid,1033,(SOURCE/manifest).read_bytes()))
    subprocess.run([*common,'/dll','/entry:DllMain@12',str(directory/'ctx-provider.obj'),'/export:ProviderValue=_ProviderValue@0',
                    '/implib:'+str(directory/'ctx-provider-generated.lib'),'/out:'+str(directory/'ctx-provider.dll')],check=True)
    subprocess.run([*common,'/dll','/entry:DllMain@12',str(directory/'ctx-importer.obj'),str(directory/'ctx-importer.res'),
                    str(directory/'ctx-provider.lib'),'/alternatename:__imp__ProviderValue@0=__imp__ProviderValue',
                    '/export:ImporterValue=_ImporterValue@0','/implib:'+str(directory/'ctx-importer-generated.lib'),'/out:'+str(directory/'ctx-importer.dll')],check=True)
    subprocess.run([*common,'/entry:entry',str(directory/'ctx-startup.obj'),str(directory/'ctx-startup.res'),str(directory/'ctx-importer.lib'),
                    '/alternatename:__imp__ImporterValue@0=__imp__ImporterValue','/out:'+str(directory/'ctx-startup.exe')],check=True)
    for name in ['ctx-importer.manifest','ctx-assembly.manifest']:shutil.copyfile(SOURCE/name,directory/name)
    (directory/'ctx-startup-inputs.json').write_text(json.dumps(identities(directory),indent=2)+'\n')
    print('Built owned native-only activation/DLL observer; no host activation execution or compatibility claim')

def native(directory):
    inputs=verify(directory);run=directory/'ctx-sxs-run';assembly=run/'Alkyhol.OwnedCRT';assembly.mkdir(parents=True,exist_ok=True)
    for name in ['ctx-startup.exe','ctx-importer.dll']:shutil.copyfile(directory/name,run/name)
    shutil.copyfile(directory/'ctx-provider.dll',assembly/'ctx-provider.dll')
    shutil.copyfile(directory/'ctx-assembly.manifest',assembly/'Alkyhol.OwnedCRT.manifest')
    # The provider deliberately exists only inside the private assembly folder.
    if (run/'ctx-provider.dll').exists():raise ValueError('unisolated provider would invalidate the loader observation')
    result=subprocess.run([str((run/'ctx-startup.exe').resolve())],cwd=run,capture_output=True,timeout=20)
    report={'backend':'native Windows PE32 loader-scoped activation observer','inputs':inputs,'status':result.returncode,
            'stdout_hex':result.stdout.hex(),'stderr_hex':result.stderr.hex()}
    path=directory/'native-ctx-startup.json';path.write_text(json.dumps(report,indent=2)+'\n')
    if result.returncode or result.stderr:raise ValueError('native activation startup failed; no compatible loader/context result claimed')
    report['records']=decode(result.stdout,{1:str((assembly/'ctx-provider.dll').resolve()),2:str((run/'ctx-importer.dll').resolve()),3:str((run/'ctx-startup.exe').resolve())})
    path.write_text(json.dumps(report,indent=2)+'\n')
    print('Captured native DLL attach/detach, default/explicit/restored contexts and complete redirection payloads')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests')
    p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
