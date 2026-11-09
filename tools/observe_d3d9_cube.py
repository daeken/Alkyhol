#!/usr/bin/env python3
"""DAO's finite cube resource COM witness; transfer the identical PE to Windows."""
import argparse,json,os,platform,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
from observe_d3d9_create_device import SYMBOLS,SOURCE,sha
PREFIX='d3d9-cube'
HEADERS=False
SOURCES=[SOURCE/'cube-pe.cpp',SOURCE/'include/d3d9.h',SOURCE/'include/strsafe.h',ROOT/'samples/d3d9/create_device/CreateDevice.cpp',ROOT/'samples/d3d9/provenance.json']
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in SOURCES},'pe':sha(d/(PREFIX+'.exe'))}
def build(d):
 d.mkdir(parents=True,exist_ok=True)
 pin=json.loads((ROOT/'samples/d3d9/provenance.json').read_text())
 for name,record in pin['files'].items():
  if sha(ROOT/'samples/d3d9'/name)!=record['sha256']:raise ValueError('pinned original Microsoft sample changed')
 lld=rust_tool('rust-lld');libs=[]
 for module,entries in SYMBOLS.items():
  definition=d/(module+'.def');definition.write_text('LIBRARY '+module+'\nEXPORTS\n'+'\n'.join(n for n,_ in entries)+'\n');lib=d/(module+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(definition),'/out:'+str(lib)],check=True)
 obj=d/(PREFIX+'.obj')
 subprocess.run([shutil.which('clang++'),'--target=i686-pc-windows-msvc','-std=c++17','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-exceptions','-fno-rtti','-ffunction-sections','-Wall','-Wextra','-Werror','-Wno-unknown-pragmas','-I'+str(SOURCE/'include'),'-c',str(SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for entries in SYMBOLS.values() for n,s in entries],'/out:'+str(d/(PREFIX+'.exe'))],check=True)
 (d/(PREFIX+'-inputs.json')).write_text(json.dumps(identity(d),indent=2)+'\n')
def decode(data,headers=False):
 if len(data)!=32:raise ValueError('cube resource report length differs')
 words=list(struct.unpack('<8I',data))
 expected=[0x434f4d48,1,0,1,1,1,2,8] if headers else [0x43554245,1,0,1,1,1,1,8]
 if words!=expected:raise ValueError('resource COM, storage, lifetime or stack contract differs: '+repr(words))
 return words
def execute(d,native=False):
 ident=identity(d)
 if json.loads((d/(PREFIX+'-inputs.json')).read_text())!=ident:raise ValueError('cube source/PE identity changed')
 directory=Path(tempfile.mkdtemp(prefix='cube-',dir=d));exe=directory/(PREFIX+'.exe');shutil.copyfile(d/(PREFIX+'.exe'),exe)
 command=[str(exe)] if native else [str(runner(d)),str(exe),'--file-origins','--external','user32.dll']
 r=subprocess.run(command,capture_output=True,timeout=90)
 report={'backend':'native Windows D3D9' if native else 'Alkyhol PE32 wgpu D3D9','platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','inputs':ident,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/(('native-' if native else '')+PREFIX+('.json' if native else '-host.json'));path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('cube PE failed; raw evidence retained: '+r.stderr.decode(errors='replace')[-1500:])
 report['words']=decode(r.stdout,HEADERS)
 if native:
  peer=json.loads((d/(PREFIX+'-host.json')).read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical cube PE native/host contract differs')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Two cycles of 4094 live COM declarations, all descriptors, retained binding and rollover validate' if HEADERS else 'Six cube faces, all mips, COM identities, surface aliases, raw locks and bound lifetime validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/d3d9-microtests');p.add_argument('--com-headers',action='store_true');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve()
 if a.com_headers:HEADERS=True;PREFIX='d3d9-com-headers';SOURCES[0]=SOURCE/'com-headers-pe.cpp'
 execute(d,a.native) if a.host or a.native else build(d)
