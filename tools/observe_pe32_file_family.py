#!/usr/bin/env python3
"""Owned synchronous file operations, sharing and descriptor metadata witness."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['file-family.c','file-family.def'];FILES=['file-family.exe'];SYMBOLS=[('GetStdHandle',4),('GetModuleFileNameA',12),('SetCurrentDirectoryA',4),('CreateFileA',28),('CloseHandle',4),('GetLastError',0),('WriteFile',20),('ReadFile',20),('SetFilePointer',16),('SetFilePointerEx',20),('SetEndOfFile',4),('FlushFileBuffers',4),('GetFileInformationByHandle',8),('GetFileSize',8),('GetHandleInformation',8)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['file-family']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'file-family.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'file-family-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated window family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x46494c32,5]+[1]*33+[36]
 if words!=expected:raise ValueError('file contents, sharing, descriptor metadata or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'file-family-inputs.json').read_text())!=identity:raise ValueError('window family identity changed')
 run=Path(tempfile.mkdtemp(prefix='file-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);(run/'root').mkdir();command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins','--documents-directory',str(run/'root')]
 r=subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows owned file' if native else 'physical Rosetta PE32 file','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-file-family.json' if native else 'file-family-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned file PE failed; raw output retained')
 report['words']=decode(r.stdout);report['file_hex']=(run/'root/owned.dat').read_bytes().hex();
 if report['file_hex']:raise ValueError('final actual file was not truncated')
 if native:
  peer=json.loads((d/'file-family-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Owned file contents, sharing, seeking, truncation and live metadata validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
