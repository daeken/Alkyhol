#!/usr/bin/env python3
"""Original x86 worker functions, distinct TEB/TLS and real suspend/wait/exit lifecycle."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['thread-family.c','thread-family.def'];FILES=['thread-family.exe'];SYMBOLS=[('FlsAlloc',4),('FlsFree',4),('FlsGetValue',4),('FlsSetValue',8),('InterlockedExchangeAdd',8),('InitializeCriticalSectionAndSpinCount',8),('EnterCriticalSection',4),('TryEnterCriticalSection',4),('LeaveCriticalSection',4),('DeleteCriticalSection',4),('CreateEventW',16),('SetEvent',4),('CreateThread',24),('ResumeThread',4),('GetCurrentThreadId',0),('GetThreadId',4),('GetExitCodeThread',8),('WaitForSingleObject',8),('CloseHandle',4),('ExitThread',4),('Sleep',4),('TlsAlloc',0),('TlsFree',4),('TlsGetValue',4),('TlsSetValue',8),('SetLastError',4),('GetLastError',0),('GetStdHandle',4),('WriteFile',20)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['thread-family']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'thread-family.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'thread-family-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated process family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x54485232,4]+[1]*23+[26]
 if words!=expected:raise ValueError('original worker execution, TEB/TLS, suspend/wait/exit state or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'thread-family-inputs.json').read_text())!=identity:raise ValueError('process family identity changed')
 run=Path(tempfile.mkdtemp(prefix='window-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins','--host-machine-info']
 r=subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows current-process family' if native else 'physical Rosetta current-process family','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-thread-family.json' if native else 'thread-family-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned window PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'thread-family-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Original x86 workers, isolated TEB/TLS, LastError and suspend/wait/exit validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
