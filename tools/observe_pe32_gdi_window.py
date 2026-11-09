#!/usr/bin/env python3
"""Owned resource bitmap metadata, RGB readback, selection and deletion witness."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest,resource_record
from observe_pe32_activation_startup import runner
SOURCES=['gdi-window.c','gdi-window.def','gdi-window-user.def','gdi-window-gdi.def'];FILES=['gdi-window.exe','gdi-window-worker.exe'];SYMBOLS=[('CreateDIBSection',24),('GetObjectW',12),('GetModuleHandleW',4),('GetStdHandle',4),('WriteFile',20),('CreateThread',24),('WaitForSingleObject',8),('GetExitCodeThread',8),('CloseHandle',4),('RegisterClassExW',4),('CreateWindowExW',48),('DefWindowProcW',16),('ShowWindow',8),('UpdateWindow',4),('DestroyWindow',4),('UnregisterClassA',8),('LoadBitmapW',8),('GetDC',4),('ReleaseDC',8),('CreateCompatibleDC',4),('SelectObject',8),('DeleteObject',4),('DeleteDC',4),('BitBlt',36),('GetPixel',12)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 header=struct.pack('<IiiHHIIiiII',40,3,2,1,24,0,26,0,0,0,0);rows=bytes(range(1,10))+b'\0'*3+bytes(range(10,19))+b'\0'*3+b'\x73\x25'
 resource=d/'gdi-window.res';resource.write_bytes(resource_record(0,0,0,b'')+resource_record(2,101,1033,header+rows))
 for role in ['gdi-window','gdi-window-user','gdi-window-gdi']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 for mode,name in enumerate(FILES):
  obj=d/(name+'.obj');subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
  subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(resource),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
 (d/'gdi-window-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated window family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x47444932,4]+[1]*8+[0x332211,0x010203]+[1]*7+[20]
 if words!=expected:raise ValueError('bitmap header, RGB rows, ownership or stack differ: '+repr(words))
 return words
def execute(d,native=False,mode=0):
 identity=inputs(d)
 if json.loads((d/'gdi-window-inputs.json').read_text())!=identity:raise ValueError('window family identity changed')
 run=Path(tempfile.mkdtemp(prefix='window-',dir=d));exe=run/FILES[mode];shutil.copyfile(d/FILES[mode],exe);command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins']
 r=subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows resource bitmap' if native else 'physical Rosetta resource bitmap','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/(f'native-gdi-window-{mode}.json' if native else f'gdi-window-{mode}-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned window PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/f'gdi-window-{mode}-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Real native-window GDI copy, pixel values and primary/worker UI delivery validate')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();[execute(d,a.native,i) for i in range(2)] if a.native or a.host else build(d)
