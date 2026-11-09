#!/usr/bin/env python3
"""Identical-PE witness for the system CRT's initial C locale used by D3DX."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['c-locale.c','c-locale.def','c-locale-crt.def']
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{'c-locale.exe':digest(d/'c-locale.exe')}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['c-locale','c-locale-crt']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'c-locale.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,'/alternatename:__imp__GetStdHandle@4=__imp__GetStdHandle','/alternatename:__imp__WriteFile@20=__imp__WriteFile','/out:'+str(d/'c-locale.exe')],check=True)
 (d/'c-locale-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)!=88:raise ValueError('C locale report length differs')
 words=list(struct.unpack('<22I',data))
 if words!=[0x434c4f43,1]+[1]*19+[22]:raise ValueError('C locale query/selection, raw string duplication or cdecl stack differs: '+repr(words))
 return words
def execute(d,native=False):
 ident=inputs(d)
 if json.loads((d/'c-locale-inputs.json').read_text())!=ident:raise ValueError('C locale source/PE identity changed')
 directory=Path(tempfile.mkdtemp(prefix='locale-',dir=d));exe=directory/'c-locale.exe';shutil.copyfile(d/'c-locale.exe',exe)
 command=[str(exe)] if native else [str(runner(d)),str(exe),'--strict','--file-origins']
 r=subprocess.run(command,capture_output=True,timeout=30)
 report={'backend':'native Windows system msvcrt' if native else 'Alkyhol system CRT C locale','inputs':ident,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-c-locale.json' if native else 'c-locale-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('C locale PE failed; raw evidence retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'c-locale-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE native/host C locale differs')
 path.write_text(json.dumps(report,indent=2)+'\n');print('All six initial C categories, C selection, failed locale preservation and cdecl stack validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
