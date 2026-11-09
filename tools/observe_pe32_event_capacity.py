#!/usr/bin/env python3
"""Identical-PE live synchronization capacity/state/lifetime comparison for DAO."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
PREFIX='event-capacity'
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('CreateEventW',16),('CreateEventA',16),('CreateSemaphoreW',16),('SetEvent',4),('ResetEvent',4),('WaitForSingleObjectEx',12),('ReleaseSemaphore',12),('CloseHandle',4)]
def inputs(d):return {'sources':{'pe/fixtures/pe32-startup/event-capacity.c':digest(SOURCE/(PREFIX+'.c'))},'files':{PREFIX+'.exe':digest(d/(PREFIX+'.exe'))}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld')
 definition=d/(PREFIX+'.def');definition.write_text('LIBRARY KERNEL32.dll\nEXPORTS\n'+'\n'.join(n for n,_ in SYMBOLS)+'\n')
 lib=d/(PREFIX+'.lib');obj=d/(PREFIX+'.obj')
 subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(definition),'/out:'+str(lib)],check=True)
 subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/(PREFIX+'.c')),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/(PREFIX+'.exe'))],check=True)
 (d/(PREFIX+'-inputs.json')).write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated event capacity report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data))
 if words!=[0x45564350,1]+[1]*11+[14]:raise ValueError('live synchronization state, lifetime or stack differs: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/(PREFIX+'-inputs.json')).read_text())!=identity:raise ValueError('capacity source/PE identity changed')
 run=Path(tempfile.mkdtemp(prefix='event-capacity-',dir=d));exe=run/(PREFIX+'.exe');shutil.copyfile(d/(PREFIX+'.exe'),exe)
 command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins']
 r=subprocess.run(command,capture_output=True,timeout=30)
 report={'backend':'native Windows' if native else 'Rosetta PE32 real kernel synchronization','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
 path=d/(('native-' if native else '')+PREFIX+('.json' if native else '-host.json'));path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('capacity PE failed; raw evidence retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/(PREFIX+'-host.json')).read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical native/host live object states differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('512 live events and 512 semaphores, state isolation, close/replacement and stdcall stack pass')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--host',action='store_true');m.add_argument('--native',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.host or a.native else build(d)
