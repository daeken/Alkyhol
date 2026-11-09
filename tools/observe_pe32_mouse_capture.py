#!/usr/bin/env python3
"""Same-PE native/Alkyhol witness for DAO's mouse capture lifecycle and callback."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['mouse-capture.c','mouse-capture.def','mouse-capture-user.def']
FILES=['mouse-capture.exe']
SYMBOLS=[('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20),('Sleep',4),('RegisterClassExW',4),('CreateWindowExW',48),('DefWindowProcW',16),('ShowWindow',8),('DestroyWindow',4),('SetCapture',4),('ReleaseCapture',0),('GetCapture',0),('PeekMessageW',20),('DispatchMessageW',4)]
def inputs(d):
 return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['mouse-capture','mouse-capture-user']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'mouse-capture.obj'
 subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'mouse-capture-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated capture report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data))
 if words!=[0x43415032,1]+[1]*16+[19]:raise ValueError('capture lifecycle, callback, query or stack differs: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'mouse-capture-inputs.json').read_text())!=identity:raise ValueError('capture source/PE identity changed')
 run=Path(tempfile.mkdtemp(prefix='capture-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins']
 r=subprocess.run(command,capture_output=True,timeout=30)
 report={'backend':'native Windows' if native else 'physical AppKit PE32','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-mouse-capture.json' if native else 'mouse-capture-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('capture PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'mouse-capture-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE native/host capture observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Owned PE32 capture/release/query, WM_CAPTURECHANGED and stdcall stack validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--host',action='store_true');m.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
