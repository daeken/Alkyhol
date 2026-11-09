#!/usr/bin/env python3
"""Identical PE32 real socket readiness and Win32 fd_sets reached by DAO."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/socket-select.c'
PREFIX='socket-select'
NEGATIVE=False
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/(PREFIX+'.exe'))}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('GetModuleHandleA',4),('GetProcAddress',8),('GetStdHandle',4),('WriteFile',20),('ExitProcess',4),('Sleep',4),('CreateThread',24),('WaitForSingleObject',8),('GetExitCodeThread',8),('CloseHandle',4)],'ws2_32.dll':[('WSAStartup',8),('WSACleanup',0),('WSAGetLastError',0),('socket',12),('closesocket',4),('select',20),('sendto',24),('recvfrom',24),('ioctlsocket',12),('bind',12),('getsockname',12),('setsockopt',20),('getsockopt',20)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/(PREFIX+'.obj');subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror',*(['-DNEGATIVE_EXCLUSION'] if NEGATIVE else []),'-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/(PREFIX+'.exe'))],check=True)
 (d/(PREFIX+'-inputs.json')).write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/(PREFIX+'-inputs.json')).read_text())
 if expected_identity!=ident:
  (d/('native-'+PREFIX+'.json' if native else PREFIX+'-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('socket select PE/source identity changed')
 command=[str(d/(PREFIX+'.exe'))] if native else [str(runner(d)),str(d/(PREFIX+'.exe')),'--file-origins','--external','ws2_32.dll']
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-'+PREFIX+'.json' if native else PREFIX+'-host.json')).write_text(json.dumps({'backend':'native Windows socket selects' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows socket selects' if native else 'Alkyhol PE32 socket selects','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-'+PREFIX+'.json' if native else PREFIX+'-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if NEGATIVE:
  if r.returncode!=78 or b'non-normalized select timeouts require a measured Windows normalization profile' not in r.stderr:raise ValueError('non-normalized select exclusion did not stop explicitly; raw capture retained')
  print('Non-normalized select timeout is explicitly excluded; no fabricated WSAEINVAL or successful readiness')
  return
 if r.returncode:raise ValueError('socket select witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated socket select report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 expected=[0x4c455353,3,0,1,0,0,1,0,1,1,1,1,0,1,0xffffffff,10022,0xffffffff,10038,1,5,1,1,2,1,5,1,1,1,1,4,1,1,0,1,1,0,0,1]
 expected += [len(expected)+1]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/(PREFIX+'-host.json')).read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical socket select PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('socket select witness differs: '+repr(words))
 print('Identical PE32 real readiness, counted fd_sets, worker datagrams, timeout storage and guards pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/socket-select-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');p.add_argument('--negative-exclusion',action='store_true');a=p.parse_args();d=a.output.resolve();NEGATIVE=a.negative_exclusion;PREFIX='socket-select-negative-exclusion' if NEGATIVE else 'socket-select'
 if NEGATIVE and a.native:p.error('negative-timeout exclusion is host-only; the raw successful Windows observation is preserved in pe/reference')
 execute(d,a.native) if a.host or a.native else build(d)
