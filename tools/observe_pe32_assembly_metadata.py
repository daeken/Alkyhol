#!/usr/bin/env python3
"""Native-only metadata experiments for DAO's remaining CRT binding blocker.

All files are owned fixtures. Test private assembly version matching and whether
file hash/token metadata changes real Windows binding. A recorded nonzero status
is an observation, never successful compatibility. No game/runtime data enters CI.
"""
import argparse
import ctypes
import json
from pathlib import Path
import shutil
import subprocess
from build_pe_fixture import ROOT, rust_tool
from check_pe32_activation import SOURCE, digest, resource_record
from observe_pe32_activation_startup import APIS, comparable, decode, frozen, verify as verify_base

CASES=['newer-version','hash-mismatch','token-newer-hash']

def manifests():
    assembly=(SOURCE/'ctx-assembly.manifest').read_text()
    importer=(SOURCE/'ctx-importer.manifest').read_text()
    # Deliberately incorrect digests, using the relevant public XML schema. This
    # is a test of enforcement, not a certificate or trusted publisher assertion.
    file='''<file name="ctx-provider.dll" hash="0000000000000000000000000000000000000000" hashalg="SHA1">
    <asmv2:hash xmlns:asmv2="urn:schemas-microsoft-com:asm.v2" xmlns:dsig="http://www.w3.org/2000/09/xmldsig#">
      <dsig:Transforms><dsig:Transform Algorithm="urn:schemas-microsoft-com:HashTransforms.Identity"/></dsig:Transforms>
      <dsig:DigestMethod Algorithm="http://www.w3.org/2000/09/xmldsig#sha1"/>
      <dsig:DigestValue>AAAAAAAAAAAAAAAAAAAAAAAAAAA=</dsig:DigestValue>
    </asmv2:hash>
  </file>'''
    hashed=assembly.replace('<assemblyIdentity','<noInheritable/>\n  <assemblyIdentity',1).replace('<file name="ctx-provider.dll"/>',file)
    token='processorArchitecture="x86" publicKeyToken="0123456789abcdef"'
    return {'newer-version':assembly.replace('1.0.0.0','2.0.0.0'),
            'hash-mismatch':hashed,
            'token-newer-hash':hashed.replace('1.0.0.0','2.0.0.0').replace('processorArchitecture="x86"',token),
            'token-importer':importer.replace('processorArchitecture="x86"',token)}

def names():
    return [f'ctx-meta-{name}.manifest' for name in [*CASES,'token-importer']]+['ctx-meta-token-importer.dll']

def inputs(directory):
    return {'contract':'owned private assembly metadata experiment v1','base':verify_base(directory),
            'files':{name:digest(directory/name) for name in names()}}

def verify(directory):
    value=json.loads((directory/'ctx-meta-inputs.json').read_text())
    if value!=inputs(directory):raise ValueError('metadata experiment source/PE/manifest identities changed')
    for name,xml in manifests().items():
        if (directory/f'ctx-meta-{name}.manifest').read_bytes()!=xml.encode('utf-8'):
            raise ValueError('metadata experiment manifest differs from its declared owned case')
    return value

def build(directory):
    verify_base(directory)
    for name,xml in manifests().items():(directory/f'ctx-meta-{name}.manifest').write_bytes(xml.encode('utf-8'))
    res=directory/'ctx-meta-token-importer.res'
    res.write_bytes(resource_record(0,0,0,b'')+resource_record(24,2,1033,(directory/'ctx-meta-token-importer.manifest').read_bytes()))
    lld=rust_tool('rust-lld')
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no',
                    '/timestamp:0','/safeseh:no','/manifest:no',str(directory/'ctx-kernel32.lib'),str(directory/'ctx-observe.obj'),
                    *[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for n,size in APIS],
                    '/dll','/entry:DllMain@12',str(directory/'ctx-importer.obj'),str(res),str(directory/'ctx-provider.lib'),
                    '/alternatename:__imp__ProviderValue@0=__imp__ProviderValue','/export:ImporterValue=_ImporterValue@0',
                    '/implib:'+str(directory/'ctx-meta-token-generated.lib'),'/out:'+str(directory/'ctx-meta-token-importer.dll')],check=True)
    (directory/'ctx-meta-inputs.json').write_text(json.dumps(inputs(directory),indent=2)+'\n')
    verify(directory)
    print('Built owned private version/hash/token cases; no execution or acceptance inferred')

def native(directory):
    identity=verify(directory);reports=[];path=directory/'native-ctx-metadata.json'
    # Suppress interactive loader-error dialogs in this observer subprocess tree.
    old=ctypes.windll.kernel32.SetErrorMode(0x8003)
    try:
        for case in CASES:
            run=directory/('ctx-meta-run-'+case);assembly=run/'Alkyhol.OwnedCRT';assembly.mkdir(parents=True,exist_ok=True)
            shutil.copyfile(directory/'ctx-startup.exe',run/'ctx-startup.exe')
            shutil.copyfile(directory/('ctx-meta-token-importer.dll' if case=='token-newer-hash' else 'ctx-importer.dll'),run/'ctx-importer.dll')
            shutil.copyfile(directory/'ctx-provider.dll',assembly/'ctx-provider.dll')
            shutil.copyfile(directory/f'ctx-meta-{case}.manifest',assembly/'Alkyhol.OwnedCRT.manifest')
            if (run/'ctx-provider.dll').exists():raise ValueError('metadata observer has a flat-directory provider')
            report={'case':case,'backend':'native Windows owned private assembly metadata observer','inputs':identity}
            try:
                result=subprocess.run([str((run/'ctx-startup.exe').resolve())],cwd=run,capture_output=True,timeout=20)
                report.update(status=result.returncode,stdout_hex=result.stdout.hex(),stderr_hex=result.stderr.hex())
            except subprocess.TimeoutExpired as e:
                report.update(timed_out=True,stdout_hex=(e.stdout or b'').hex(),stderr_hex=(e.stderr or b'').hex())
                reports.append(report);path.write_text(json.dumps(reports,indent=2)+'\n')
                raise RuntimeError('native metadata observation exhausted its budget; no binding outcome claimed')
            except OSError as e:
                report['process_creation_error']={'winerror':getattr(e,'winerror',None),'errno':e.errno,'message':str(e)}
                reports.append(report);path.write_text(json.dumps(reports,indent=2)+'\n')
                raise RuntimeError('Windows did not create the observer process; exact error retained, no completed experiment claimed') from e
            reports.append(report);path.write_text(json.dumps(reports,indent=2)+'\n')
            if result.returncode==0:
                if result.stderr:raise ValueError('unexpected successful metadata observer diagnostics')
                report['records']=decode(result.stdout,{1:str((assembly/'ctx-provider.dll').resolve()),2:str((run/'ctx-importer.dll').resolve()),3:str((run/'ctx-startup.exe').resolve())})
                report['same_scope_and_section_as_baseline']=comparable(report['records'])==comparable(frozen()['records'])
            elif result.stdout:
                raise ValueError('metadata process failed after partial observer execution; no complete binding outcome claimed')
            path.write_text(json.dumps(reports,indent=2)+'\n')
            print(case+': Windows process status '+str(result.returncode)+'; native-only observation, not host support')
    finally:
        ctypes.windll.kernel32.SetErrorMode(old)

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true')
    a=p.parse_args();(native if a.native else build)(a.output.resolve())
