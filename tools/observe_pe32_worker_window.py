#!/usr/bin/env python3
"""Physical/native window and four-argument x86 callback capability witness."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['worker-window.c','worker-window.def','window-family.c','window-family-user.def'];FILES=['worker-window.exe'];SYMBOLS=[('CreateThread',24),('WaitForSingleObject',8),('GetExitCodeThread',8),('CloseHandle',4),('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('RegisterClassExW',4),('CreateWindowExW',48),('DefWindowProcW',16),('AdjustWindowRect',12),('ShowWindow',8),('UpdateWindow',4),('GetClientRect',8),('GetWindowRect',8),('GetWindowLongW',8),('SetWindowLongW',12),('SetWindowTextW',8),('IsWindowVisible',4),('DestroyWindow',4),('UnregisterClassA',8),('GetSystemMetrics',4),('GetClassLongA',8),('ClientToScreen',8),('ScreenToClient',8)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['worker-window','window-family-user']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'worker-window.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'worker-window-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated window family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x57494e32,4]+[1]*23+[1412]+[1]*4+[0xcf0000,32]
 if words!=expected:raise ValueError('window creation, callbacks, state, coordinates, teardown or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'worker-window-inputs.json').read_text())!=identity:raise ValueError('window family identity changed')
 run=Path(tempfile.mkdtemp(prefix='window-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins']
 r=subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows guest-worker window' if native else 'physical guest-worker AppKit window','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-worker-window.json' if native else 'worker-window-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned window PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'worker-window-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Real guest-worker window, original callbacks and synchronous main-thread handoff validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
