#!/usr/bin/env python3
"""Owned lexical full-path A/W and caller-buffer pointer observations."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
SOURCES=['full-path.c','full-path.def'];FILES=['full-path.exe']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetModuleFileNameW',12),('SetCurrentDirectoryW',4),('GetFullPathNameW',16),('GetFullPathNameA',16)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'full-path.lib';obj=d/'full-path.obj'
 subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'full-path.def'),'/out:'+str(lib)],check=True)
 subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'full-path.c'),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'full-path-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)!=244:raise ValueError('incomplete full-path report')
 words=list(struct.unpack('<61I',data))
 if words[:2]!=[0x50415432,4] or words[-2:]!=[1,61]:raise ValueError('full-path ABI/stack differs')
 for i in range(9):
  row=words[2+i*6:8+i*6]
  if row[:3]!=[1,1,1] or row[4:]!=[1,1]:raise ValueError('full-path sizing/content/pointer/guards differ: '+str(row))
 if words[-5]!=1 or words[-3]!=1:raise ValueError('full-path insufficient-buffer bounds differ')
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'full-path-inputs.json').read_text())!=identity:raise ValueError('full-path identity changed')
 run=Path(tempfile.mkdtemp(prefix='path-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);probe=None if native else runner(d);command=[str(exe.resolve())]
 if not native:command=[str(probe),*command,'--strict','--file-origins']
 r=subprocess.run(command,capture_output=True,timeout=20);report={'backend':'native Windows full path' if native else 'Rosetta full path','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-full-path.json' if native else 'full-path-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('full-path PE failed; raw output retained')
 report['words']=decode(r.stdout);path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'full-path-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['words']!=report['words']:raise ValueError('identical-PE full-path observations differ')
 print('Full-path A/W lexical paths, size queries, file-part pointers and buffers validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
