#!/usr/bin/env python3
"""Identical PE32 real SO_BROADCAST set/get reached by DAO LAN setup."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/socket-broadcast.c'
PREFIX='socket-broadcast'
ASYNC=False
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/(PREFIX+'.exe'))}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('GetModuleHandleA',4),('GetProcAddress',8),('GetStdHandle',4),('WriteFile',20),('ExitProcess',4),('Sleep',4)],'ws2_32.dll':[('WSAStartup',8),('WSACleanup',0),('WSAGetLastError',0),('socket',12),('closesocket',4),('WSARecvFrom',36),('sendto',24),('recvfrom',24),('ioctlsocket',12),('bind',12),('getsockname',12),('setsockopt',20),('getsockopt',20)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/(PREFIX+'.obj');subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror',*(['-DASYNC_EXCLUSION'] if ASYNC else []),'-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/(PREFIX+'.exe'))],check=True)
 (d/(PREFIX+'-inputs.json')).write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/(PREFIX+'-inputs.json')).read_text())
 if expected_identity!=ident:
  (d/('native-'+PREFIX+'.json' if native else PREFIX+'-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('SO_BROADCAST PE/source identity changed')
 command=[str(d/(PREFIX+'.exe'))] if native else [str(runner(d)),str(d/(PREFIX+'.exe')),'--file-origins','--external','ws2_32.dll']
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-'+PREFIX+'.json' if native else PREFIX+'-host.json')).write_text(json.dumps({'backend':'native Windows SO_BROADCASTs' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows SO_BROADCASTs' if native else 'Alkyhol PE32 SO_BROADCASTs','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-'+PREFIX+'.json' if native else PREFIX+'-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('SO_BROADCAST witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated SO_BROADCAST report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 expected=[0x42534f53,1,0,1,1]+[0,0,1,1]+[0,0,1,1]*2+[0,5,5,1,1,0xffffffff,10035]+[5,0,5,1,1]+[0,0xffffffff,10038,0,1]
 if ASYNC:expected=[0x42534f53,1,0,1,1,0xffffffff,10045,1,0,0,1]
 expected += [len(expected)+1]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/(PREFIX+'-host.json')).read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical SO_BROADCAST PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('SO_BROADCAST witness differs: '+repr(words))
 print('Unsupported asynchronous receives fail explicitly without fabricated pending I/O or completion' if ASYNC else 'Identical PE32 real UDP endpoints, options, datagrams, scatter receive and guards pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/socket-broadcast-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');p.add_argument('--async-exclusion',action='store_true');a=p.parse_args();d=a.output.resolve();ASYNC=a.async_exclusion;PREFIX='socket-async-exclusion' if ASYNC else 'socket-broadcast';
 if ASYNC and a.native:p.error('the unsupported asynchronous profile is an Alkyhol exclusion, not a native Windows semantic comparison')
 execute(d,a.native) if a.host or a.native else build(d)
