#!/usr/bin/env python3
"""Identical PE32 indexed DIB palette, stride, orientation and exact-color operations."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/palette-dib.c'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/'palette-dib.exe')}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('GetStdHandle',4),('WriteFile',20),('ExitProcess',4)],'gdi32.dll':[('GdiFlush',0),('CreateCompatibleDC',4),('CreateDIBSection',24),('SelectObject',8),('GetObjectW',12),('GetPixel',12),('CreateSolidBrush',4),('BitBlt',36),('DeleteObject',4),('DeleteDC',4)],'user32.dll':[('FillRect',12)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'palette-dib.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/'palette-dib.exe')],check=True)
 (d/'palette-dib-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/'palette-dib-inputs.json').read_text())
 if expected_identity!=ident:
  (d/('native-palette-dib.json' if native else 'palette-dib-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('palette DIB PE/source identity changed')
 command=[str(d/'palette-dib.exe')] if native else [str(runner(d)),str(d/'palette-dib.exe'),'--file-origins','--external','gdi32.dll']
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-palette-dib.json' if native else 'palette-dib-host.json')).write_text(json.dumps({'backend':'native Windows indexed DIB' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows indexed DIB' if native else 'Alkyhol PE32 indexed DIB','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-palette-dib.json' if native else 'palette-dib-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('palette DIB witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated palette DIB report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 expected=[0x38424944,1]+[1]*5+[0x070d1d,0x070707,1]+[1]*2+[0x1d0d07,0x020202,0x111111,0x040404,0x050505,0x060606,0x070707,0x080808,0x090909,0x0a0a0a]+[1,1,0x020202,0x0a0a0a]+[1]*6
 expected += [len(expected)+1]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'palette-dib-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical palette DIB PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('palette DIB witness differs: '+repr(words))
 print('Identical PE32 8-bit palette, DWORD stride/padding, orientation, RGB copy and exact-color writes pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/palette-dib-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
