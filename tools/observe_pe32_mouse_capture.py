#!/usr/bin/env python3
"""Same-PE native/Alkyhol witness for DAO's mouse capture lifecycle and callback."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['mouse-capture.c','mouse-capture.def','mouse-capture-user.def']
PREFIX='mouse-capture'
BACKGROUND=False
CLIENT_DEFAULT=False
CLIENT_BUTTONS=False
CLIP_QUERY=False
FORCE_REBASE=False
FILES=['mouse-capture.exe']
SYMBOLS=[('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20),('Sleep',4),('RegisterClassExW',4),('CreateWindowExW',48),('DefWindowProcW',16),('ShowWindow',8),('DestroyWindow',4),('SetCapture',4),('ReleaseCapture',0),('GetCapture',0),('PeekMessageW',20),('DispatchMessageW',4)]
def inputs(d):
 return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in [Path(n).stem for n in SOURCES[1:]]:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/(PREFIX+'.obj')
 subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',*(['-DBACKGROUND'] if BACKGROUND else []),*(['-DCLIENT_DEFAULT'] if CLIENT_DEFAULT else []),'-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/(PREFIX+'-inputs.json')).write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated capture report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data))
 if CLIP_QUERY:
  expected=[0x434c5032,1]+[1]*113+[116]
  if words!=expected:raise ValueError('actual desktop clip bounds, output guards or stack differ: '+repr(words))
  return words
 if CLIENT_BUTTONS:
  expected=[0x4d444633,1,1,1,1]
  for msg in [0x200,0x201,0x202,0x203,0x204,0x206,0x207,0x208,0x209]:expected += [msg,0,0,1]
  for point in [0x00040003,0xfff3fff9,0x80007fff,0x00040003]:expected += [point,0,1,1,1,1]
  expected += [0,1,1,1,70]
  if words!=expected:raise ValueError('default client button/context callback observations differ: '+repr(words))
  return words
 if words!=[0x43415032,3 if CLIENT_DEFAULT else 2 if BACKGROUND else 1]+[1]*(19 if CLIENT_DEFAULT else 18 if BACKGROUND else 16)+[22 if CLIENT_DEFAULT else 21 if BACKGROUND else 19]:raise ValueError('capture lifecycle, callback, query or stack differs: '+repr(words))
 return words
def execute(d,native=False,host=None):
 identity=inputs(d)
 if json.loads((d/(PREFIX+'-inputs.json')).read_text())!=identity:raise ValueError('capture source/PE identity changed')
 run=Path(tempfile.mkdtemp(prefix='capture-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);command=[str(exe.resolve())]
 if not native:command=[str(host.resolve() if host else runner(d)),*command,'--strict','--file-origins',*(['--rebase'] if FORCE_REBASE else [])]
 r=subprocess.run(command,capture_output=True,timeout=30)
 report={'backend':'native Windows' if native else 'physical AppKit PE32','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 if CLIENT_BUTTONS or CLIP_QUERY:
  import platform
  report.update(platform=platform.platform(),rebased=FORCE_REBASE)
  if host is not None:report['host_sha256']=digest(host)
  if native:
   import ctypes
   kernel=ctypes.WinDLL('kernel32',use_last_error=True);kernel.GetCurrentProcess.restype=ctypes.c_void_p;kernel.IsWow64Process2.argtypes=[ctypes.c_void_p,ctypes.POINTER(ctypes.c_ushort),ctypes.POINTER(ctypes.c_ushort)];process,machine=ctypes.c_ushort(),ctypes.c_ushort()
   if not kernel.IsWow64Process2(kernel.GetCurrentProcess(),ctypes.byref(process),ctypes.byref(machine)) or machine.value!=0x8664:raise ValueError('native AMD64 Windows required')
   report.update(observer_process_machine=process.value,native_machine=machine.value)
 path=d/(('native-' if native else '')+PREFIX+('.json' if native else '-host.json'));path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('capture PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native and not (CLIENT_BUTTONS or CLIP_QUERY):
  peer=json.loads((d/(PREFIX+'-host.json')).read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE native/host capture observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Owned PE32 actual unconfined clip query validates' if CLIP_QUERY else 'Owned PE32 default client buttons/context callbacks validate' if CLIENT_BUTTONS else 'Owned PE32 capture/release/query, WM_CAPTURECHANGED and stdcall stack validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--background',action='store_true');p.add_argument('--client-default',action='store_true');p.add_argument('--client-buttons',action='store_true');p.add_argument('--clip-query',action='store_true');p.add_argument('--runner',type=Path);p.add_argument('--rebase',action='store_true');p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--host',action='store_true');m.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve()
 if sum([a.client_default,a.background,a.client_buttons,a.clip_query])>1:p.error('choose one mouse witness variant')
 FORCE_REBASE=a.rebase
 if a.client_default:CLIENT_DEFAULT=True;PREFIX='mouse-client-default';FILES=[PREFIX+'.exe']
 if a.background:BACKGROUND=True;PREFIX='mouse-background-capture';FILES=[PREFIX+'.exe']
 if a.client_buttons:CLIENT_BUTTONS=True;PREFIX='mouse-buttons-default';FILES=[PREFIX+'.exe'];SOURCES=['mouse-defaults.c','mouse-capture.def','mouse-defaults-user.def'];SYMBOLS=SYMBOLS+[('GetWindowInfo',8)]
 if a.clip_query:CLIP_QUERY=True;PREFIX='cursor-clip-query';FILES=[PREFIX+'.exe'];SOURCES=['cursor-clip-query.c','mouse-capture.def','cursor-clip-query-user.def'];SYMBOLS=SYMBOLS+[('GetClipCursor',4),('ClipCursor',4),('GetSystemMetrics',4),('ShowCursor',4)]
 execute(d,a.native,a.runner) if a.host or a.native else build(d)
