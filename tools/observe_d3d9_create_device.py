#!/usr/bin/env python3
"""Compile unchanged Microsoft Tutorial 01 into PE32 and compare its GPU pixels."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/d3d9'
FILES=['d3d9-create-device.exe']
SYMBOLS={'kernel32.dll':[('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20)],'user32.dll':[('RegisterClassExW',4),('CreateWindowExW',48),('DefWindowProcW',16),('PostQuitMessage',4),('ValidateRect',8),('ShowWindow',8),('UpdateWindow',4),('GetMessageW',16),('TranslateMessage',4),('DispatchMessageW',4),('UnregisterClassW',8),('DestroyWindow',4)],'d3d9.dll':[('Direct3DCreate9',4)]}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def identity(d):return {'source':{str(p.relative_to(ROOT)):sha(p) for p in [SOURCE/'create-device-pe.cpp',SOURCE/'include/d3d9.h',SOURCE/'include/strsafe.h',ROOT/'samples/d3d9/create_device/CreateDevice.cpp',ROOT/'samples/d3d9/provenance.json']},'pe':sha(d/FILES[0])}
def build(d):
 d.mkdir(parents=True,exist_ok=True)
 manifest=json.loads((ROOT/'samples/d3d9/provenance.json').read_text())
 for name,record in manifest['files'].items():
  if sha(ROOT/'samples/d3d9'/name)!=record['sha256']:raise ValueError('original Microsoft sample changed')
 lld=rust_tool('rust-lld');libs=[]
 for module,entries in SYMBOLS.items():
  deffile=d/(module+'.def');deffile.write_text('LIBRARY '+module+'\nEXPORTS\n'+'\n'.join(name for name,_ in entries)+'\n');lib=d/(module+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'d3d9-create-device.obj';subprocess.run([shutil.which('clang++'),'--target=i686-pc-windows-msvc','-std=c++17','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-exceptions','-fno-rtti','-ffunction-sections','-Wall','-Wextra','-Werror','-Wno-unknown-pragmas','-I'+str(SOURCE/'include'),'-c',str(SOURCE/'create-device-pe.cpp'),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for entries in SYMBOLS.values() for n,s in entries],'/out:'+str(d/FILES[0])],check=True)
 (d/'d3d9-create-device-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated D3D9 report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data))
 expected=[0x44394433,4,0]+[1]*9+[0xff]*195+[1]*8+[216]
 if words!=expected:raise ValueError('D3D9 COM, scene state, full GPU blue pixels, cleanup or stack differ: '+repr(words))
 return words

def execute(d,native):
 ident=identity(d)
 if json.loads((d/'d3d9-create-device-inputs.json').read_text())!=ident:raise ValueError('D3D9 identity mismatch')
 run=Path(tempfile.mkdtemp(prefix='d3d9-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe)
 command=[str(exe)] if native else [str(runner(d)),str(exe),'--file-origins','--external','user32.dll']
 r=subprocess.run(command,capture_output=True,timeout=90)
 report={'backend':'native Windows D3D9' if native else 'Alkyhol PE32 wgpu D3D9','platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','inputs':ident,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-d3d9-create-device.json' if native else 'd3d9-create-device-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('D3D9 sample failed; raw evidence retained: '+r.stderr.decode(errors='replace')[-2000:])
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'d3d9-create-device-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE native/host D3D9 result mismatch')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Original Tutorial 01 InitD3D/Render/Cleanup executes; every GPU pixel and finite COM witness passes')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/d3d9-microtests');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve()
 execute(d,a.native) if a.host or a.native else build(d)
