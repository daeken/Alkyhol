#!/usr/bin/env python3
"""Identical PE32 Win32 and Winsock event-set waits reached by DAO."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/wsa-events.c'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/'wsa-events.exe')}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('CreateEventA',16),('CreateThread',24),('WaitForSingleObject',8),('WaitForMultipleObjects',16),('GetExitCodeThread',8),('CloseHandle',4),('Sleep',4),('GetStdHandle',4),('WriteFile',20),('ExitProcess',4)],'ws2_32.dll':[('WSAStartup',8),('WSACleanup',0),('WSAGetLastError',0),('WSACreateEvent',0),('WSASetEvent',4),('WSAResetEvent',4),('WSACloseEvent',4),('WSAWaitForMultipleEvents',20)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'wsa-events.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/'wsa-events.exe')],check=True)
 (d/'wsa-events-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/'wsa-events-inputs.json').read_text())
 if expected_identity!=ident:
  (d/('native-wsa-events.json' if native else 'wsa-events-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('WSA event PE/source identity changed')
 command=[str(d/'wsa-events.exe')] if native else [str(runner(d)),str(d/'wsa-events.exe'),'--file-origins','--external','ws2_32.dll']
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-wsa-events.json' if native else 'wsa-events-host.json')).write_text(json.dumps({'backend':'native Windows WSA events' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows WSA events' if native else 'Alkyhol PE32 WSA events','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-wsa-events.json' if native else 'wsa-events-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('WSA event witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated WSA event report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 expected=[0x45534157,4,1,0,1,0,1,258,1,0,0,1,1,0,1,0xffffffff,87,258]+[1,258,1,1,1,0,0,1,258,1,0,0,0xffffffff,87,1,1,0,1]+[1,1,258,1,258,0,258,1,0,1,1,0,0,258,1,0,258,1,0,1,1,1]+[1,1,0,6,0,1,1]
 expected += [len(expected)+1]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'wsa-events-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical WSA event PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('WSA event witness differs: '+repr(words))
 print('Identical PE32 WSA event aliases, real worker signaling, manual/automatic any/all sets, timeouts and stack balance pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/wsa-events-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
