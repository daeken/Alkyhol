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
from observe_pe32_activation_startup import APIS,decode,comparable,runner
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
    if value!=frozen_layout()[0]['inputs']:raise ValueError('changed frozen owned activation layout inputs')
    return value

def frozen_layout():
    path=ROOT/'pe/reference/pe32-assembly-layout-windows-v1.json'
    if digest(path)!='447013659b9d99cc5e25bcb62daf7328cf547333c904c04044aaae6416622b79':raise ValueError('changed frozen native assembly layout report')
    rows=json.loads(path.read_bytes())
    if [r['case'] for r in rows]!=CASES or any(r['status'] or r['stderr_hex'] for r in rows):raise ValueError('invalid frozen native assembly layout observation')
    return rows

def compare(report):
    reference=next(r for r in frozen_layout() if r['case']==report['case'])
    if report['status'] or comparable(report['records'])!=comparable(reference['records']):raise ValueError('activation layout differs from the frozen native observation')

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

def execute(directory,host=False):
    identity=verify(directory);reports=[];output=directory/('ctx-layout-host.json' if host else 'native-ctx-layout.json');probe=runner(directory) if host else None
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
        report={'case':case,'backend':'Rosetta PE32 original assembly layouts' if host else 'native Windows PE32 owned assembly layout observer','inputs':identity}
        command=[str((run/'ctx-startup.exe').resolve())]
        if host:
            command=[str(probe),*command,'--strict','--file-origins','--activation','--dll','ctx-importer.dll='+str(run/'ctx-importer.dll'),'--dll','ctx-provider.dll='+str(provider)]
            if case!='empty-dll':command+=['--assembly-manifest',str(assembly/'Alkyhol.OwnedCRT.manifest')]
        try:
            r=subprocess.run(command,cwd=run,capture_output=True,timeout=20)
            report.update(status=r.returncode,stdout_hex=r.stdout.hex(),stderr_hex=r.stderr.hex())
        except subprocess.TimeoutExpired as e:
            report.update(timed_out=True,stdout_hex=(e.stdout or b'').hex(),stderr_hex=(e.stderr or b'').hex())
            reports.append(report);output.write_text(json.dumps(reports,indent=2)+'\n');raise RuntimeError('layout observation exhausted its budget; no completed outcome claimed')
        except OSError as e:
            report['process_creation_error']={'winerror':getattr(e,'winerror',None),'message':str(e)}
            reports.append(report);output.write_text(json.dumps(reports,indent=2)+'\n');raise
        reports.append(report);output.write_text(json.dumps(reports,indent=2)+'\n')
        if r.returncode or not host and r.stderr:raise ValueError('layout did not complete; no compatible execution claimed')
        paths={1:provider,2:run/'ctx-importer.dll',3:run/'ctx-startup.exe'}
        report['records']=decode(r.stdout,{k:('Z:'+str(p.resolve()).replace('/','\\') if host else str(p.resolve())) for k,p in paths.items()},require_explicit_key=case!='empty-dll')
        compare(report)
        output.write_text(json.dumps(reports,indent=2)+'\n')
        print(case+': complete activation lifecycle and section bytes agree with frozen native Windows')
    if host:policy_cases(directory,probe)

def policy_cases(directory,probe):
    # This is an explicit trusted-policy HOST model, not a native Windows policy
    # installation or signature-validation observation. Original PE code is kept.
    run=directory/'ctx-layout-policy-model';assembly=run/'Alkyhol.OwnedCRT';assembly.mkdir(parents=True,exist_ok=True)
    for source,target in [('ctx-startup.exe',run/'ctx-startup.exe'),('ctx-meta-token-importer.dll',run/'ctx-importer.dll'),('ctx-provider.dll',assembly/'ctx-provider.dll')]:shutil.copyfile(directory/source,target)
    manifest=assembly/'Alkyhol.OwnedCRT.manifest';manifest.write_bytes(manifests()['token-exact'].replace('version="1.0.0.0"','version="1.0.5.0"').encode())
    policy='''<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
      <assemblyIdentity type="win32-policy" name="policy.1.0.Alkyhol.OwnedCRT" version="1.0.5.0" processorArchitecture="x86" publicKeyToken="0123456789abcdef"/>
      <dependency><dependentAssembly><assemblyIdentity type="win32" name="Alkyhol.OwnedCRT" processorArchitecture="x86" publicKeyToken="0123456789abcdef"/>
      <bindingRedirect oldVersion="1.0.0.0-1.0.4.0" newVersion="1.0.5.0"/></dependentAssembly></dependency></assembly>'''
    paths={}
    for label,text in [('valid',policy),('outside',policy.replace('oldVersion="1.0.0.0-','oldVersion="1.0.0.1-')),('other-token',policy.replace('0123456789abcdef','fedcba9876543210')),('unknown-field',policy.replace('newVersion="1.0.5.0"','newVersion="1.0.5.0" unknown="x"'))]:
        paths[label]=run/(label+'.policy.manifest');paths[label].write_bytes(text.encode())
    base=[str(probe),str(run/'ctx-startup.exe'),'--strict','--file-origins','--assembly-manifest',str(manifest),'--dll','ctx-importer.dll='+str(run/'ctx-importer.dll'),'--dll','ctx-provider.dll='+str(assembly/'ctx-provider.dll')]
    reports=[]
    for label in ['valid','missing','outside','other-token','duplicate','unknown-field']:
        command=base+(['--assembly-policy',str(paths['valid' if label=='duplicate' else label])] if label!='missing' else [])
        if label=='duplicate':command+=['--assembly-policy',str(paths['valid'])]
        r=subprocess.run(command,cwd=run,capture_output=True,timeout=20)
        report={'case':label,'backend':'host explicit installed-policy model; no native policy installation claim','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
        reports.append(report);(directory/'ctx-policy-host.json').write_text(json.dumps(reports,indent=2)+'\n')
        if label=='valid':
            if r.returncode:raise ValueError('explicit policy model failed to bind its actual target')
            known={1:assembly/'ctx-provider.dll',2:run/'ctx-importer.dll',3:run/'ctx-startup.exe'}
            report['records']=decode(r.stdout,{k:'Z:'+str(p.resolve()).replace('/','\\') for k,p in known.items()})
            compare(dict(report,case='token-exact'))
        else:
            reason=b'ambiguous installed publisher policy' if label=='duplicate' else b'unsupported activation manifest attributes' if label=='unknown-field' else b'unresolved exact assembly dependency'
            if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('policy restriction failed for its actual reason: '+label)
        (directory/'ctx-policy-host.json').write_text(json.dumps(reports,indent=2)+'\n')
    print('Explicit trusted-policy model passes; missing/outside/other-token/duplicate/unknown policies stop before entry')

def native(directory):
    old=ctypes.windll.kernel32.SetErrorMode(0x8003)
    try:execute(directory)
    finally:ctypes.windll.kernel32.SetErrorMode(old)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');mode=p.add_mutually_exclusive_group();mode.add_argument('--native',action='store_true');mode.add_argument('--host',action='store_true')
    a=p.parse_args();(native if a.native else lambda d:execute(d,True) if a.host else build(d))(a.output.resolve())
