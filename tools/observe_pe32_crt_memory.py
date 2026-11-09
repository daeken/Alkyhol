#!/usr/bin/env python3
"""Identical PE32 CRT allocation, memory and narrow string calls reached by D3DX."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/crt-memory.c'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/'crt-memory.exe')}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20),('ExitProcess',4)],'msvcrt.dll':[(n,0) for n in ['malloc','free','??2@YAPAXI@Z','??3@YAXPAX@Z','??_U@YAPAXI@Z','??_V@YAXPAX@Z','memcpy','memmove','memset','strchr','strncmp']]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'crt-memory.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/'crt-memory.exe')],check=True)
 (d/'crt-memory-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/'crt-memory-inputs.json').read_text())
 if expected_identity!=ident:
  (d/('native-crt-memory.json' if native else 'crt-memory-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('CRT PE/source identity changed')
 command=[str(d/'crt-memory.exe')] if native else [str(runner(d)),str(d/'crt-memory.exe'),'--file-origins','--external','msvcrt.dll']
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-crt-memory.json' if native else 'crt-memory-host.json')).write_text(json.dumps({'backend':'native Windows MSVCRT' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows MSVCRT' if native else 'Alkyhol PE32 CRT allocation and memory','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-crt-memory.json' if native else 'crt-memory-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('CRT witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated CRT report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 expected=[0x4d545243,1]
 sizes=[0,1,24,4097,65539]
 for kind in range(3):
  expected += [1]*15
  for i,size in enumerate(sizes):expected += [1,sum((j*17+i*23+kind*7)&255 for j in range(size)),1]
  expected += [1]
 expected += [1]*9+[0xffffffff,1,0,0,254,0xffffff02,0,0xffffff9e,25]+[0,2,4,0xffffffff,0,0]
 expected += [1,len(expected)+2]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'crt-memory-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical CRT PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('CRT allocation/memory/string witness differs: '+repr(words))
 print('Identical PE32 scalar/array/malloc ownership, copy/move/set, narrow strings and cdecl stack pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/crt-memory-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
