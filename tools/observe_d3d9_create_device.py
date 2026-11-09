#!/usr/bin/env python3
"""Compile unchanged Microsoft Tutorial 01 into PE32 and compare its GPU pixels."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/d3d9'
FILES=['d3d9-create-device.exe']
PREFIX='d3d9-create-device'
VERTICES=False
SYMBOLS={'kernel32.dll':[('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20)],'user32.dll':[('RegisterClassExW',4),('CreateWindowExW',48),('DefWindowProcW',16),('PostQuitMessage',4),('ValidateRect',8),('ShowWindow',8),('UpdateWindow',4),('GetMessageW',16),('TranslateMessage',4),('DispatchMessageW',4),('UnregisterClassW',8),('DestroyWindow',4),('PeekMessageW',20)],'d3d9.dll':[('Direct3DCreate9',4)]}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE/'create-device-pe.cpp',SOURCE/'include/d3d9.h',SOURCE/'include/strsafe.h',ROOT/'samples/d3d9/create_device/CreateDevice.cpp',ROOT/'samples/d3d9/vertices/Vertices.cpp',ROOT/'samples/d3d9/provenance.json']},'pe':sha(d/FILES[0])}
def build(d):
 d.mkdir(parents=True,exist_ok=True)
 manifest=json.loads((ROOT/'samples/d3d9/provenance.json').read_text())
 for name,record in manifest['files'].items():
  if sha(ROOT/'samples/d3d9'/name)!=record['sha256']:raise ValueError('original Microsoft sample changed')
 lld=rust_tool('rust-lld');libs=[]
 for module,entries in SYMBOLS.items():
  deffile=d/(module+'.def');deffile.write_text('LIBRARY '+module+'\nEXPORTS\n'+'\n'.join(name for name,_ in entries)+'\n');lib=d/(module+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/(PREFIX+'.obj');subprocess.run([shutil.which('clang++'),'--target=i686-pc-windows-msvc','-std=c++17','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-exceptions','-fno-rtti','-ffunction-sections','-Wall','-Wextra','-Werror','-Wno-unknown-pragmas',*(['-DVERTICES'] if VERTICES else []),'-I'+str(SOURCE/'include'),'-c',str(SOURCE/'create-device-pe.cpp'),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for entries in SYMBOLS.values() for n,s in entries],'/out:'+str(d/FILES[0])],check=True)
 (d/(PREFIX+'-inputs.json')).write_text(json.dumps(identity(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated D3D9 report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data))
 if VERTICES:
  if len(words)!=90032 or words[:23]!=[0x44394433,4,0]+[1]*20 or words[-9:]!=[1]*8+[90032]:raise ValueError('triangle COM/state/guard/stack differs')
  pixels=words[23:-9]
  if not 19000<sum(n!=0xff for n in pixels)<21001 or pixels[150*300+150]==0xff or any(pixels[n]!=0xff for n in [0,299,299*300,300*300-1]):raise ValueError('triangle coverage/background failed')
  return words
 expected=[0x44394433,4,0]+[1]*10+[0xff]*195+[1]*8+[217]
 if words!=expected:raise ValueError('D3D9 COM, scene state, full GPU blue pixels, cleanup or stack differ: '+repr(words))
 return words

def execute(d,native):
 ident=identity(d)
 expected=json.loads((d/(PREFIX+'-inputs.json')).read_text())
 if expected!=ident:
  (d/(('native-' if native else '')+PREFIX+'-identity-mismatch.json')).write_text(json.dumps({'expected':expected,'observed':ident},indent=2)+'\n')
  raise ValueError('D3D9 identity mismatch')
 run=Path(tempfile.mkdtemp(prefix='d3d9-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe)
 command=[str(exe)] if native else [str(runner(d)),str(exe),'--file-origins','--external','user32.dll']
 r=subprocess.run(command,capture_output=True,timeout=90)
 report={'backend':'native Windows D3D9' if native else 'Alkyhol PE32 wgpu D3D9','platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','inputs':ident,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/(('native-'+PREFIX+'.json') if native else (PREFIX+'-host.json'));path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('D3D9 sample failed; raw evidence retained: '+r.stderr.decode(errors='replace')[-2000:])
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/(PREFIX+'-host.json')).read_text())
  if peer['inputs']!=ident or peer['status']:raise ValueError('identical PE native/host identity/status mismatch')
  other=decode(bytes.fromhex(peer['stdout_hex']))
  if not VERTICES and other!=report['words']:raise ValueError('exact clear/COM comparison failed')
  if VERTICES:
   for a,b in zip(other[23:-9],report['words'][14:-9]):
    if (a==0xff)!=(b==0xff) or any(abs(((a>>n)&255)-((b>>n)&255))>1 for n in [0,8,16]):raise ValueError('native triangle coverage or RGB tolerance 1 failed')
   report['comparison']='all 90000 coverage decisions exact; RGB channel error <= 1'
 path.write_text(json.dumps(report,indent=2)+'\n');print(('Original Tutorial 02 vertex buffers/DrawPrimitive' if VERTICES else 'Original Tutorial 01 clear/present')+' executes; GPU pixels and finite COM witness pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/d3d9-microtests');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');p.add_argument('--vertices',action='store_true');a=p.parse_args();d=a.output.resolve()
 VERTICES=a.vertices
 if VERTICES:PREFIX='d3d9-vertices';FILES=[PREFIX+'.exe']
 execute(d,a.native) if a.host or a.native else build(d)
