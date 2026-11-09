#!/usr/bin/env python3
"""Identical PE32 DirectInput lifecycle/format/property witness; no live input claim."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/input8/lifecycle.cpp'
HEADER=ROOT/'pe/fixtures/d3d9/include/d3d9.h'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE,HEADER]},'pe':sha(d/'input8.exe')}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20)],'dinput8.dll':[('DirectInput8Create',20)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(n for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'input8.obj';subprocess.run([shutil.which('clang++'),'--target=i686-pc-windows-msvc','-std=c++17','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-exceptions','-fno-rtti','-Wall','-Wextra','-Werror','-I'+str(HEADER.parent),'-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for entries in symbols.values() for n,size in entries],'/out:'+str(d/'input8.exe')],check=True)
 (d/'input8-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/'input8-inputs.json').read_text())
 if expected_identity!=ident:
  (d/('native-input8.json' if native else 'input8-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('input PE/source identity changed')
 command=[str(d/'input8.exe')] if native else [str(runner(d)),str(d/'input8.exe'),'--file-origins','--external','dinput8.dll']
 r=subprocess.run(command,capture_output=True,timeout=60)
 report={'backend':'native Windows DirectInput8' if native else 'Alkyhol PE32 DirectInput8 lifecycle','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-input8.json' if native else 'input8-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('input witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated input report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 # Each A/W factory emits HRESULT and three booleans; each device emits 12
 # records, two of which deliberately retain the APPDATA HRESULT/default.
 expected=[0x38494441,1]
 for _ in range(2):
  expected += [0,1,1,1]
  for _ in range(2):expected += [1]*5+[0,0xffffffff]+[1]*5
 expected += [1,len(expected)+2]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'input8-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical input PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('input lifecycle/format/property witness differs: '+repr(words))
 print('Identical PE32 input lifecycle, A/W layouts, properties, nonacquired state and stack pass; no live event evidence')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/input8-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
