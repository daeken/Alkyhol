#!/usr/bin/env python3
"""Owned deferred DLL initialization, binary exports and thread notifications."""
import platform,os
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['deferred-family.c','deferred-family.def','deferred-library.c'];FILES=['deferred-family.exe','owned-deferred.dll','owned-rejected.dll'];SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetModuleHandleA',4),('GetModuleHandleW',4),('GetProcAddress',8),('LoadLibraryA',4),('LoadLibraryW',4),('FreeLibrary',4),('GetLastError',0),('SetLastError',4),('GetModuleFileNameA',12),('SwitchToThread',0),('CreateThread',24),('WaitForSingleObject',8),('GetExitCodeThread',8),('CloseHandle',4)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['deferred-family']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'deferred-family.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 for rejected,name in enumerate(FILES[1:]):
  obj=d/(name+'.obj');subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DREJECT={rejected}','-c',str(SOURCE/'deferred-library.c'),'-o',str(obj)],check=True)
  subprocess.run([lld,'-flavor','link','/nodefaultlib','/dll','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:DllMain@12',str(obj),'/out:'+str(d/name)],check=True)
 (d/'deferred-family-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated window family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x444c4c32,4]+[1]*20+[23]
 if words!=expected:raise ValueError('file contents, sharing, descriptor metadata or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'deferred-family-inputs.json').read_text())!=identity:raise ValueError('window family identity changed')
 run=Path(tempfile.mkdtemp(prefix='file-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);(run/'root').mkdir();[shutil.copyfile(d/n,run/n) for n in FILES[1:]];command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins',*sum([['--deferred-dll',n+'='+str(run/n)] for n in FILES[1:]],[])]
 r=subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows PE32 deferred DLLs' if native else 'Alkyhol macOS PE32 deferred DLLs','hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','host_platform':platform.platform(),'inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-deferred-family.json' if native else 'deferred-family-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned file PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'deferred-family-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Real deferred DLL initialization, references, binary exports, worker callbacks and failed attach validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
