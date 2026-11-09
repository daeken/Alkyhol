#!/usr/bin/env python3
"""Identical PE32 large asset reads, complete bytes, guards and EOF reached by DAO."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/large-read.c'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
LENGTH=80*1024*1024+3
DATA=(bytes((i*37+11)&255 for i in range(256))*(LENGTH//256+1))[:LENGTH]
def ensure_data(d):
 p=d/'owned-large-read.bin'
 if not p.exists():p.write_bytes(DATA)
 if sha(p)!=hashlib.sha256(DATA).hexdigest():raise ValueError('owned large-read data changed')
def identity(d):
 ensure_data(d)
 return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/'large-read.exe'),'data':sha(d/'owned-large-read.bin')}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('GetModuleFileNameA',12),('SetCurrentDirectoryA',4),('CreateFileA',28),('ReadFile',20),('CloseHandle',4),('GetStdHandle',4),('WriteFile',20),('ExitProcess',4)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'large-read.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/'large-read.exe')],check=True)
 (d/'large-read-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/'large-read-inputs.json').read_text())
 if expected_identity!=ident:
  (d/('native-large-read.json' if native else 'large-read-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('large read PE/source identity changed')
 command=[str(d/'large-read.exe')] if native else [str(runner(d)),str(d/'large-read.exe'),'--file-origins']
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-large-read.json' if native else 'large-read-host.json')).write_text(json.dumps({'backend':'native Windows large reads' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows large reads' if native else 'Alkyhol PE32 large reads','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-large-read.json' if native else 'large-read-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('large read witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated large read report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 digest=2166136261
 for byte in DATA:digest=((digest^byte)*16777619)&0xffffffff
 expected=[0x4c524746,1,1,1,1,1,LENGTH,digest,1,1,1,1,1]
 expected += [len(expected)+1]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'large-read-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical large read PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('large read witness differs: '+repr(words))
 print('Identical PE32 complete 80MiB asset read, full byte digest, untouched guards and EOF pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/large-read-80-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
