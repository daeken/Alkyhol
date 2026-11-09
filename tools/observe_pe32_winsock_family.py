#!/usr/bin/env python3
"""Owned Winsock ordinal ABI and real loopback TCP byte transport."""
import platform,os
import argparse,json,shutil,struct,subprocess,tempfile,os,socket,threading
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['winsock-family.c','winsock-family.def','winsock-family-net.def'];FILES=['winsock-family.exe'];SYMBOLS=[('GetStdHandle',4),('GetModuleHandleA',4),('GetProcAddress',8),('WriteFile',20),('GetEnvironmentVariableA',12),('WSAStartup',8),('WSACleanup',0),('WSAGetLastError',0),('socket',12),('closesocket',4),('connect',12),('htons',4),('inet_addr',4),('ioctlsocket',12),('recv',16),('send',16),('gethostbyname',4)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['winsock-family','winsock-family-net']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'winsock-family.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'winsock-family-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated window family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x57534132,4]+[1]*21+[24]
 if words!=expected:raise ValueError('file contents, sharing, descriptor metadata or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'winsock-family-inputs.json').read_text())!=identity:raise ValueError('window family identity changed')
 run=Path(tempfile.mkdtemp(prefix='file-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);(run/'root').mkdir();command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins']
 r=loopback(command);report={'backend':'native Windows PE32 Winsock' if native else 'Alkyhol macOS PE32 Winsock','hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','host_platform':platform.platform(),'inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-winsock-family.json' if native else 'winsock-family-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned file PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'winsock-family-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Winsock ordinal calls, real loopback TCP bytes, hostent layout, state and guards validate')
def loopback(command):
 listener=socket.socket();listener.bind(('127.0.0.1',0));listener.listen(1);listener.settimeout(25);errors=[];received=[]
 def server():
  try:
   with listener.accept()[0] as peer:
    peer.settimeout(20);payload=b''
    while len(payload)<5:
     chunk=peer.recv(5-len(payload))
     if not chunk:raise ValueError('client closed before original bytes arrived')
     payload+=chunk
    received.append(payload);peer.sendall(b'reply')
  except Exception as e:errors.append(repr(e))
 thread=threading.Thread(target=server,daemon=True);thread.start()
 try:
  env=dict(os.environ,ALKYHOL_TEST_PORT=str(listener.getsockname()[1]));r=subprocess.run(command,capture_output=True,timeout=30,env=env)
 finally:
  listener.close();thread.join(timeout=5)
 if r.returncode==0 and (errors or received!=[b'hello']):raise ValueError('independent server did not observe exact owned bytes: '+repr(errors))
 return r
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
