#!/usr/bin/env python3
"""Owned native observations of DAO's remaining empty-DLL/token/file layouts.

No original runtime/game files are accepted. Build is not execution evidence.
"""
import argparse
import ctypes
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest,resource_record
from observe_pe32_activation_startup import APIS,decode
from observe_pe32_assembly_metadata import manifests as metadata_manifests,verify as verify_metadata

CASES=['empty-dll','three-files','token-exact']
FILES=['ctx-layout-empty-importer.dll','ctx-layout-three-files.manifest','ctx-layout-token-exact.manifest']

def manifests():
    original=(SOURCE/'ctx-assembly.manifest').read_text()
    return {'three-files':original.replace('<file name="ctx-provider.dll"/>','<file name="ctx-provider.dll"/><file name="ctx-secondary.dll"/><file name="ctx-unused.dll"/>'),
            'token-exact':metadata_manifests()['token-newer-hash'].replace('2.0.0.0','1.0.0.0')}

def inputs(directory):
    return {'contract':'owned activation empty-DLL/token/three-file layouts v1','metadata_base':verify_metadata(directory),
            'files':{name:digest(directory/name) for name in FILES}}

def verify(directory):
    value=json.loads((directory/'ctx-layout-inputs.json').read_text())
    if value!=inputs(directory):raise ValueError('owned activation layout inputs changed')
    for name,xml in manifests().items():
        if (directory/f'ctx-layout-{name}.manifest').read_bytes()!=xml.encode():raise ValueError('changed owned activation layout declaration')
    return value

def build(directory):
    verify_metadata(directory)
    for name,xml in manifests().items():(directory/f'ctx-layout-{name}.manifest').write_bytes(xml.encode())
    res=directory/'ctx-layout-empty-importer.res'
    res.write_bytes(resource_record(0,0,0,b'')+resource_record(24,2,1033,(SOURCE/'activation-empty.manifest').read_bytes()))
    subprocess.run([rust_tool('rust-lld'),'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no',
        '/timestamp:0','/safeseh:no','/manifest:no',str(directory/'ctx-kernel32.lib'),str(directory/'ctx-observe.obj'),
        *[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n,size in APIS],'/dll','/entry:DllMain@12',str(directory/'ctx-importer.obj'),str(res),
        str(directory/'ctx-provider.lib'),'/alternatename:__imp__ProviderValue@0=__imp__ProviderValue','/export:ImporterValue=_ImporterValue@0',
        '/implib:'+str(directory/'ctx-layout-empty-generated.lib'),'/out:'+str(directory/'ctx-layout-empty-importer.dll')],check=True)
    (directory/'ctx-layout-inputs.json').write_text(json.dumps(inputs(directory),indent=2)+'\n');verify(directory)
    print('Built owned empty-DLL, token-exact and three-file cases; native observations pending')

def native_cases(directory):
    identity=verify(directory);reports=[];output=directory/'native-ctx-layout.json'
    for case in CASES:
        run=directory/('ctx-layout-run-'+case);assembly=run/'Alkyhol.OwnedCRT';assembly.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(directory/'ctx-startup.exe',run/'ctx-startup.exe')
        importer='ctx-layout-empty-importer.dll' if case=='empty-dll' else 'ctx-meta-token-importer.dll' if case=='token-exact' else 'ctx-importer.dll'
        shutil.copyfile(directory/importer,run/'ctx-importer.dll')
        provider=run/'ctx-provider.dll' if case=='empty-dll' else assembly/'ctx-provider.dll'
        shutil.copyfile(directory/'ctx-provider.dll',provider)
        if case!='empty-dll':
            if (run/'ctx-provider.dll').exists():raise ValueError('flat provider invalidates assembly observation')
            shutil.copyfile(directory/f'ctx-layout-{case}.manifest',assembly/'Alkyhol.OwnedCRT.manifest')
        if case=='three-files':
            for name in ['ctx-secondary.dll','ctx-unused.dll']:shutil.copyfile(directory/'ctx-provider.dll',assembly/name)
        report={'case':case,'backend':'native Windows PE32 owned assembly layout observer','inputs':identity}
        try:
            r=subprocess.run([str((run/'ctx-startup.exe').resolve())],cwd=run,capture_output=True,timeout=20)
            report.update(status=r.returncode,stdout_hex=r.stdout.hex(),stderr_hex=r.stderr.hex())
        except subprocess.TimeoutExpired as e:
            report.update(timed_out=True,stdout_hex=(e.stdout or b'').hex(),stderr_hex=(e.stderr or b'').hex())
            reports.append(report);output.write_text(json.dumps(reports,indent=2)+'\n');raise RuntimeError('layout observation exhausted its budget; no completed outcome claimed')
        except OSError as e:
            report['process_creation_error']={'winerror':getattr(e,'winerror',None),'message':str(e)}
            reports.append(report);output.write_text(json.dumps(reports,indent=2)+'\n');raise
        reports.append(report);output.write_text(json.dumps(reports,indent=2)+'\n')
        if r.returncode or r.stderr:raise ValueError('native layout did not complete; no compatible execution claimed')
        report['records']=decode(r.stdout,{1:str(provider.resolve()),2:str((run/'ctx-importer.dll').resolve()),3:str((run/'ctx-startup.exe').resolve())},require_explicit_key=case!='empty-dll')
        output.write_text(json.dumps(reports,indent=2)+'\n')
        print(case+': captured complete native activation lifecycle and section bytes')

def native(directory):
    old=ctypes.windll.kernel32.SetErrorMode(0x8003)
    try:native_cases(directory)
    finally:ctypes.windll.kernel32.SetErrorMode(old)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true')
    a=p.parse_args();(native if a.native else build)(a.output.resolve())
