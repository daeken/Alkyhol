#!/usr/bin/env python3
"""Identical PE32 unnamed mutex ownership, thread contention and abandonment."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from observe_pe32_activation_startup import runner
SOURCE=ROOT/'pe/fixtures/pe32-startup/unnamed-mutex.c'
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def identity(d):return {'source':{p.relative_to(ROOT).as_posix():sha(p) for p in [SOURCE]},'pe':sha(d/'unnamed-mutex.exe')}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 symbols={'kernel32.dll':[('CreateMutexA',12),('CreateMutexW',12),('ReleaseMutex',4),('WaitForSingleObject',8),('CreateEventW',16),('SetEvent',4),('CreateThread',24),('GetExitCodeThread',8),('CloseHandle',4),('SetLastError',4),('GetLastError',0),('GetStdHandle',4),('WriteFile',20),('ExitProcess',4)]}
 for dll,entries in symbols.items():
  deffile=d/(dll+'.def');deffile.write_text('LIBRARY '+dll+'\nEXPORTS\n'+'\n'.join(chr(34)+n+chr(34) for n,_ in entries)+'\n');lib=d/(dll+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(deffile),'/out:'+str(lib)],check=True)
 obj=d/'unnamed-mutex.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-std=c11','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-Wall','-Wextra','-Werror','-c',str(SOURCE),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/opt:ref','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{size}=__imp__{n}' for dll,entries in symbols.items() if dll!='msvcrt.dll' for n,size in entries],'/out:'+str(d/'unnamed-mutex.exe')],check=True)
 (d/'unnamed-mutex-inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def execute(d,native):
 ident=identity(d)
 expected_identity=json.loads((d/'unnamed-mutex-inputs.json').read_text())
 if expected_identity!=ident:
  (d/('native-unnamed-mutex.json' if native else 'unnamed-mutex-host.json')).write_text(json.dumps({'status':'rejected before execution','expected':expected_identity,'observed':ident},indent=2)+'\n')
  raise ValueError('mutex PE/source identity changed')
 command=[str(d/'unnamed-mutex.exe')] if native else [str(runner(d)),str(d/'unnamed-mutex.exe'),'--file-origins','--host-machine-info']
 try:r=subprocess.run(command,capture_output=True,timeout=60)
 except subprocess.TimeoutExpired as error:
  (d/('native-unnamed-mutex.json' if native else 'unnamed-mutex-host.json')).write_text(json.dumps({'backend':'native Windows mutex/thread' if native else 'Alkyhol PE32','inputs':ident,'status':'timeout','stdout_hex':(error.stdout or b'').hex(),'stderr_hex':(error.stderr or b'').hex()},indent=2)+'\n')
  raise
 report={'backend':'native Windows mutex/thread' if native else 'Alkyhol PE32 guest-thread mutex','inputs':ident,'platform':platform.platform(),'hosted_ci':os.getenv('GITHUB_ACTIONS')=='true','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/('native-unnamed-mutex.json' if native else 'unnamed-mutex-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('mutex witness failed; raw capture retained: '+r.stderr.decode(errors='replace')[-1500:])
 if len(r.stdout)%4:raise ValueError('truncated mutex report')
 words=list(struct.unpack('<'+'I'*(len(r.stdout)//4),r.stdout));report['words']=words
 expected=[0x584d5541,1]+[1,0,0,0,1,1,0,1,258,1,0,1]+[258,0,288,258,1,0,1]+[0x80,1,0,288,1,1,1,1]+[1,0,1,1,0,288,1]+[1,1]
 expected += [len(expected)+1]
 path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'unnamed-mutex-host.json').read_text())
  if peer['inputs']!=ident or peer['status'] or peer['stdout_hex']!=r.stdout.hex():raise ValueError('identical mutex PE differs; raw native and host observations retained')
 if words!=expected:raise ValueError('mutex allocation/memory/string witness differs: '+repr(words))
 print('Identical PE32 recursive ownership, real contention/waits, worker exit abandonment and stack pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/unnamed-mutex-microtest');g=p.add_mutually_exclusive_group();g.add_argument('--host',action='store_true');g.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
