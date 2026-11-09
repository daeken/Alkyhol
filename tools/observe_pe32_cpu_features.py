#!/usr/bin/env python3
"""Identical PE32 instruction feature balances, changed modes and actual worker isolation."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/cpu-features.c'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/'cpu-features.exe')}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('IsProcessorFeaturePresent',4)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'cpu-features.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/'cpu-features.exe')],check=True)
 (d/'cpu-features-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/'cpu-features-inputs.json').read_text())
 if expected_identity!=ident:
  (d/('native-cpu-features.json' if native else 'cpu-features-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('instruction feature PE/source identity changed')
 command=[str(d/'cpu-features.exe')] if native else [str(runner(d)),str(d/'cpu-features.exe'),'--file-origins']
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-cpu-features.json' if native else 'cpu-features-host.json')).write_text(json.dumps({'backend':'native Windows instruction features' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows instruction features' if native else 'Alkyhol PE32 instruction features','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-cpu-features.json' if native else 'cpu-features-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('instruction feature witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated instruction feature report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 expected=[0x46555043,1]+[1,0xface1234]*4+[1,1]
 expected += [len(expected)+1]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'cpu-features-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical instruction feature PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('instruction feature witness differs: '+repr(words))
 print('Identical PE32 instruction features against actual CPUID, LastError, XMM execution and guest stack pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/cpu-features-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
