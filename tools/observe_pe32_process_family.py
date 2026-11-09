#!/usr/bin/env python3
"""Current process identity and explicit host-machine processor relationship."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['process-family.c','process-family.def'];FILES=['process-family.exe'];SYMBOLS=[('GetCurrentProcess',0),('GetCurrentProcessId',0),('GetProcessId',4),('GetProcessAffinityMask',12),('GetSystemInfo',4),('GetLastError',0),('GetStdHandle',4),('WriteFile',20)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['process-family']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'process-family.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'process-family-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated process family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x50524332,4,1,1,1,1,1,6,1,10]
 if words!=expected:raise ValueError('current process identity, mask relationship or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'process-family-inputs.json').read_text())!=identity:raise ValueError('process family identity changed')
 run=Path(tempfile.mkdtemp(prefix='window-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins','--host-machine-info']
 r=subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows current-process family' if native else 'physical Rosetta current-process family','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-process-family.json' if native else 'process-family-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned window PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'process-family-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Current process identity and available processor relationship validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
