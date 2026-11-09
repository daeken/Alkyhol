#!/usr/bin/env python3
"""Owned asynchronous directory notifications, completion and cancellation witness."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['watch-family.c','watch-family.def'];FILES=['watch-family.exe'];SYMBOLS=[('GetStdHandle',4),('GetModuleFileNameA',12),('SetCurrentDirectoryA',4),('CreateFileA',28),('CloseHandle',4),('GetLastError',0),('WriteFile',20),('CreateEventW',16),('ReadDirectoryChangesW',32),('GetOverlappedResult',16),('CancelIo',4),('WaitForSingleObject',8)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['watch-family']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'watch-family.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'watch-family-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated window family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x57415432,4]+[1]*25+[28]
 if words!=expected:raise ValueError('directory completion, cancellation, guards or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'watch-family-inputs.json').read_text())!=identity:raise ValueError('window family identity changed')
 run=Path(tempfile.mkdtemp(prefix='file-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);(run/'root').mkdir();command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins','--documents-directory',str(run/'root')]
 r=subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows owned file' if native else 'physical Rosetta PE32 file','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-watch-family.json' if native else 'watch-family-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned file PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if not (run/'root/created.tmp').is_file():raise ValueError('real created file absent')
 if native:
  peer=json.loads((d/'watch-family-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Real directory notifications, pending completion, cancellation and buffer guards validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
