#!/usr/bin/env python3
"""Real software exception delivery through original x86 handlers."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['software-exception.c','software-exception.def'];FILES=['software-exception.exe']+[f'software-exception-{i}.exe' for i in range(1,4)];SYMBOLS=[('RaiseException',16),('GetStdHandle',4),('WriteFile',20)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['software-exception']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 for mode,name in enumerate(FILES):
  obj=d/(name+'.obj');subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
  subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
 (d/'software-exception-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated software exception report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x53454832,4,1,1,1,1,7]
 if words!=expected:raise ValueError('software SEH order, parameters, context or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'software-exception-inputs.json').read_text())!=identity:raise ValueError('software exception identity changed')
 run=Path(tempfile.mkdtemp(prefix='seh-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);(run/'root').mkdir();command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins','--documents-directory',str(run/'root')]
 r=subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows software SEH' if native else 'physical Rosetta software SEH','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-software-exception.json' if native else 'software-exception-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned software exception PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'software-exception-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 if not native:
  report['restrictions']=[]
  for mode,reason in [(1,b'unhandled software exception'),(2,b'noncontinuable or oversized software exception'),(3,b'mutated context/record/chain')]:
   r=subprocess.run([str(runner(d)),str((d/FILES[mode]).resolve()),'--strict','--file-origins'],capture_output=True,timeout=20)
   report['restrictions'].append({'mode':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()})
   path.write_text(json.dumps(report,indent=2)+'\n')
   if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('software SEH exclusion did not fail honestly')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Original x86 search and continuation handlers received the actual exception')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
