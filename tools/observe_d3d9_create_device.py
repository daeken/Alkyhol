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
QUERIES=False
SHADERS=False
TEXTURES=False
MODEL2=False
STATES=False
TARGETS=False
DEPTH=False
INDEXED=False
BC3=False
PACKED=False
SYMBOLS={'kernel32.dll':[('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20)],'user32.dll':[('RegisterClassExW',4),('CreateWindowExW',48),('DefWindowProcW',16),('PostQuitMessage',4),('ValidateRect',8),('ShowWindow',8),('UpdateWindow',4),('GetMessageW',16),('TranslateMessage',4),('DispatchMessageW',4),('UnregisterClassW',8),('DestroyWindow',4),('PeekMessageW',20),('GetWindowInfo',8),('GetWindowRect',8),('GetClientRect',8),('ClientToScreen',8),('GetWindowLongW',8),('SetWindowPos',28),('SetWindowLongW',12)],'d3d9.dll':[('Direct3DCreate9',4)]}
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE/'create-device-pe.cpp',SOURCE/'include/d3d9.h',SOURCE/'include/strsafe.h',ROOT/'samples/d3d9/create_device/CreateDevice.cpp',ROOT/'samples/d3d9/vertices/Vertices.cpp',ROOT/'samples/d3d9/provenance.json',*([SOURCE/'legacy-shaders-pe.cpp',SOURCE/'legacy-color.vs.inc',SOURCE/'legacy-color.ps.inc',ROOT/'d3d9/tests/fixtures/legacy-color.vs.bin',ROOT/'d3d9/tests/fixtures/legacy-color.ps.bin',*([SOURCE/'legacy-texture.ps.inc',ROOT/'d3d9/tests/fixtures/legacy-texture.ps.bin'] if TEXTURES else []),*([SOURCE/'legacy-model2.vs.inc',SOURCE/'legacy-slots128.ps.inc',ROOT/'d3d9/tests/fixtures/legacy-model2.vs.bin',ROOT/'d3d9/tests/fixtures/legacy-slots128.ps.bin'] if MODEL2 else [])] if SHADERS else [])]},'pe':sha(d/FILES[0])}
def build(d):
 d.mkdir(parents=True,exist_ok=True)
 manifest=json.loads((ROOT/'samples/d3d9/provenance.json').read_text())
 for name,record in manifest['files'].items():
  if sha(ROOT/'samples/d3d9'/name)!=record['sha256']:raise ValueError('original Microsoft sample changed')
 lld=rust_tool('rust-lld');libs=[]
 for module,entries in SYMBOLS.items():
  deffile=d/(module+'.def');deffile.write_text('LIBRARY '+module+'\nEXPORTS\n'+'\n'.join(name for name,_ in entries)+'\n');lib=d/(module+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/(PREFIX+'.obj');subprocess.run([shutil.which('clang++'),'--target=i686-pc-windows-msvc','-std=c++17','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-exceptions','-fno-rtti','-ffunction-sections','-Wall','-Wextra','-Werror','-Wno-unknown-pragmas',*(['-DVERTICES'] if VERTICES else []),*(['-DQUERIES'] if QUERIES else []),*(['-DTEXTURES'] if TEXTURES else []),*(['-DMODEL2'] if MODEL2 else []),*(['-DSTATES'] if STATES else []),*(['-DTARGETS'] if TARGETS else []),*(['-DDEPTH'] if DEPTH else []),*(['-DINDEXED'] if INDEXED else []),*(['-DBC3'] if BC3 else []),*(['-DPACKED'] if PACKED else []),'-I'+str(SOURCE/'include'),'-c',str(SOURCE/('legacy-shaders-pe.cpp' if SHADERS else 'create-device-pe.cpp')),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for entries in SYMBOLS.values() for n,s in entries],'/out:'+str(d/FILES[0])],check=True)
 (d/(PREFIX+'-inputs.json')).write_text(json.dumps(identity(d),indent=2)+'\n')
def triangle_pixels(words):
 if len(words)!=90032:raise ValueError('triangle report length differs')
 return words[23:-9]
def compare_triangle_pixels(first,second):
 if len(first)!=90000 or len(second)!=90000:raise ValueError('triangle pixel count differs')
 for i,(a,b) in enumerate(zip(first,second)):
  if (a==0xff)!=(b==0xff) or any(abs(((a>>n)&255)-((b>>n)&255))>1 for n in [0,8,16]):raise ValueError(f'native triangle coverage or RGB tolerance 1 failed at {i%300},{i//300}')
def shader_words(words,textured=False,model2=False,states=False):
 prefix=37 if BC3 else 23 if INDEXED else 23 if DEPTH else 37 if TARGETS else 44 if states else 23 if model2 else 36 if textured else 22;checks=14 if states else 10 if textured else 8
 block=1+checks+1024+1;footer=prefix+2*block;length=footer+9
 if len(words)!=length or words[:prefix]!=[0x39534844,9 if PACKED else 8 if BC3 else 7 if INDEXED else 6 if DEPTH else 5 if TARGETS else 4 if states else 3 if model2 else 2 if textured else 1]+[1]*(prefix-2) or words[footer:]!=[0xfeedffff]+[1]*7+[length]:raise ValueError('legacy shader COM/metadata/stack witness differs')
 for frame in range(2):
  start=prefix+block*frame
  if words[start:start+checks+1]!=[0xfeed0000+frame]+[1]*checks or words[start+block-1]!=1:raise ValueError('legacy shader constant/draw/readback failed')
  pixels=words[start+checks+1:start+block-1]
  color=([0x848284,0x3b393b][frame] if BC3 else [0xff,0xff0000][frame] if DEPTH else [0x4040bf,0x1c00ff][frame] if states else [0x808080,0x373737][frame] if textured else 0xa347eb if model2 else 0xff0000)
  if any(v not in [0xff,color] for v in pixels):raise ValueError('legacy shader color differs')
  if pixels[0]!=0xff or pixels[20*32+22]!=color or pixels[20*32+10]!=(color if frame==0 else 0xff):raise ValueError('legacy shader matrix/coverage did not change as specified')
 return words

def decode(data):
 if len(data)%4:raise ValueError('truncated D3D9 report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data))
 if SHADERS:return shader_words(words,TEXTURES,MODEL2,STATES)
 if VERTICES:
  if len(words)!=90032 or words[:23]!=[0x44394433,4,0]+[1]*20 or words[-9:]!=[1]*8+[90032]:raise ValueError('triangle COM/state/guard/stack differs')
  pixels=triangle_pixels(words)
  if not 19000<sum(n!=0xff for n in pixels)<21001 or pixels[150*300+150]==0xff or any(pixels[n]!=0xff for n in [0,299,299*300,300*300-1]):raise ValueError('triangle coverage/background failed')
  return words
 expected=[0x44394433,8 if QUERIES else 4,0]+[1]*(28 if QUERIES else 10)+[0xff]*195+[1]*8+[235 if QUERIES else 217]
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
 if not native:
  paths=[*sorted((ROOT/'pe/src').glob('*.*')),*sorted((ROOT/'d3d9/src').glob('*.*')),ROOT/'pe/Cargo.lock',ROOT/'d3d9/Cargo.lock']
  report['runtime']={'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'sources':{p.relative_to(ROOT).as_posix():sha(p) for p in paths if p.is_file()},'tracked_runtime_clean':not subprocess.check_output(['git','status','--porcelain','--','pe/src','d3d9/src','pe/Cargo.lock','d3d9/Cargo.lock'],cwd=ROOT,text=True).strip()}
 path=d/(('native-'+PREFIX+'.json') if native else (PREFIX+'-host.json'));path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('D3D9 sample failed; raw evidence retained: '+r.stderr.decode(errors='replace')[-2000:])
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/(PREFIX+'-host.json')).read_text())
  if peer['inputs']!=ident or peer['status']:raise ValueError('identical PE native/host identity/status mismatch')
  other=decode(bytes.fromhex(peer['stdout_hex']))
  if not VERTICES and other!=report['words']:raise ValueError('exact clear/COM comparison failed')
  if VERTICES:
   compare_triangle_pixels(triangle_pixels(other),triangle_pixels(report['words']))
   report['comparison']='all 90000 coverage decisions exact; RGB channel error <= 1'
 path.write_text(json.dumps(report,indent=2)+'\n');print(('Owned legacy bytecode shaders' if SHADERS else 'Original Tutorial 02 vertex buffers/DrawPrimitive' if VERTICES else 'Original Tutorial 01 clear/present')+' executes; GPU pixels and finite COM witness pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/d3d9-microtests');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');p.add_argument('--vertices',action='store_true');p.add_argument('--queries',action='store_true');p.add_argument('--shaders',action='store_true');p.add_argument('--textures',action='store_true');p.add_argument('--model2',action='store_true');p.add_argument('--states',action='store_true');p.add_argument('--targets',action='store_true');p.add_argument('--depth',action='store_true');p.add_argument('--indexed',action='store_true');p.add_argument('--bc3',action='store_true');p.add_argument('--packed',action='store_true');a=p.parse_args();d=a.output.resolve()
 if sum([a.vertices,a.queries,a.shaders,a.textures,a.model2,a.states,a.targets,a.depth,a.indexed,a.bc3,a.packed])>1:p.error('choose one witness variant')
 VERTICES=a.vertices;QUERIES=a.queries;SHADERS=a.shaders or a.textures or a.model2 or a.states or a.targets or a.depth or a.indexed or a.bc3 or a.packed;TEXTURES=a.textures or a.states or a.targets or a.bc3;MODEL2=a.model2;STATES=a.states;TARGETS=a.targets;DEPTH=a.depth;INDEXED=a.indexed;BC3=a.bc3;PACKED=a.packed
 if VERTICES:PREFIX='d3d9-vertices';FILES=[PREFIX+'.exe']
 if QUERIES:PREFIX='d3d9-queries';FILES=[PREFIX+'.exe']
 if SHADERS:PREFIX='d3d9-shaders';FILES=[PREFIX+'.exe']
 if TEXTURES:PREFIX='d3d9-textures';FILES=[PREFIX+'.exe']
 if MODEL2:PREFIX='d3d9-model2';FILES=[PREFIX+'.exe']
 if STATES:PREFIX='d3d9-states';FILES=[PREFIX+'.exe']
 if TARGETS:PREFIX='d3d9-targets';FILES=[PREFIX+'.exe']
 if DEPTH:PREFIX='d3d9-depth';FILES=[PREFIX+'.exe']
 if INDEXED:PREFIX='d3d9-indexed';FILES=[PREFIX+'.exe']
 if BC3:PREFIX='d3d9-bc3';FILES=[PREFIX+'.exe']
 if PACKED:PREFIX='d3d9-packed';FILES=[PREFIX+'.exe']
 execute(d,a.native) if a.host or a.native else build(d)
