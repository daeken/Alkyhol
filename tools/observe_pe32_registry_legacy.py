#!/usr/bin/env python3
"""Identical PE32 legacy registry balances, changed modes and actual worker isolation."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/registry-legacy.c'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/'registry-legacy.exe')}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4)],'advapi32.dll':[('RegOpenKeyA',12)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'registry-legacy.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/'registry-legacy.exe')],check=True)
 (d/'registry-legacy-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/'registry-legacy-inputs.json').read_text())
 if expected_identity!=ident:
  (d/('native-registry-legacy.json' if native else 'registry-legacy-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('legacy registry PE/source identity changed')
 catalog=d/'missing-legacy-registry.xml'
 catalog.write_text('<registry view="32"><scope hive="HKCU" path="Software\\AlkyholOwnedMissingLegacy" exists="false"/></registry>')
 if native:
  import winreg
  try:key=winreg.OpenKey(winreg.HKEY_CURRENT_USER,r'Software\AlkyholOwnedMissingLegacy\{5B40D859-43AF-4528-9A38-6EA2F41759E8}',0,winreg.KEY_READ|winreg.KEY_WOW64_32KEY)
  except FileNotFoundError:pass
  else:key.Close();raise ValueError('owned expected-absent key already exists; refusing to assume absence')
 command=[str(d/'registry-legacy.exe')] if native else [str(runner(d)),str(d/'registry-legacy.exe'),'--file-origins','--external','advapi32.dll','--registry-catalog',str(catalog)]
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-registry-legacy.json' if native else 'registry-legacy-host.json')).write_text(json.dumps({'backend':'native Windows legacy registry calls' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows legacy registry calls' if native else 'Alkyhol PE32 legacy registry calls','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-registry-legacy.json' if native else 'registry-legacy-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('legacy registry witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated legacy registry report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 expected=[0x4c474552,1,2,1,0xface1234,0,1,0xface1234,0,1,0xface1234,1]
 expected += [len(expected)+1]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'registry-legacy-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical legacy registry PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('legacy registry witness differs: '+repr(words))
 print('Identical PE32 legacy ANSI registry missing-key, same-hive, output guard and stack observations pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/registry-legacy-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
