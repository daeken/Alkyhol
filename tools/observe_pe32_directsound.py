#!/usr/bin/env python3
"""Owned DirectSound PE32 COM/PCM playback evidence on an actual audio endpoint."""
import argparse,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
SOURCE=ROOT/'pe/fixtures/pe32-startup'
SOURCES=['directsound.c','directsound.def','directsound-kernel.def','directsound-user.def']
SYMBOLS=[('DirectSoundCreate8',12),('DirectSoundEnumerateA',8),('GetDesktopWindow',0),('CreateEventW',16),('WaitForSingleObject',8),('CloseHandle',4),('Sleep',4),('GetStdHandle',4),('WriteFile',20),('LoadLibraryA',4),('GetProcAddress',8),('FreeLibrary',4)]
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def inputs(d):return dict(sources={n:digest(SOURCE/n) for n in SOURCES},pe_sha256=digest(d/'directsound.exe'))
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for name in SOURCES[1:]:
  lib=d/(Path(name).stem+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/name),'/out:'+str(lib)],check=True)
 obj=d/'directsound.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/'directsound.exe')],check=True)
 (d/'inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)!=63*4:raise ValueError('incomplete DirectSound playback witness')
 words=list(struct.unpack('<63I',data))
 if words!=[0x44534e32,2]+[1]*60+[63]:raise ValueError('DirectSound COM/PCM/notification/stack assertions differ: '+repr(words))
 return words
def execute(d,host,native):
 identity=inputs(d)
 if identity!=json.loads((d/'inputs.json').read_text()):raise ValueError('changed sound witness source/PE')
 if native and os.name!='nt':raise ValueError('native playback requires actual Windows')
 report=dict(inputs=identity,backend='native Windows DirectSound' if native else 'physical Rosetta/CoreAudio',platform=platform.platform(),observations=[])
 for rebase in ([False] if native else [False,True]):
  command=[str((d/'directsound.exe').resolve())] if native else [str(host.resolve()),str((d/'directsound.exe').resolve()),'--file-origins','--host-machine-info','--external','dsound.dll',*(['--rebase'] if rebase else [])]
  r=subprocess.run(command,env=dict(os.environ,ALKYHOL_PE32_TRACE_CALLS='first'),capture_output=True,timeout=30)
  row=dict(rebased=rebase,status=r.returncode,stdout_hex=r.stdout.hex(),stderr=r.stderr.decode(errors='replace'));report['observations'].append(row)
  (d/('native-report.json' if native else 'report.json')).write_text(json.dumps(report,indent=2)+'\n')
  if r.returncode:raise ValueError('sound witness failed; raw evidence retained')
  row['words']=decode(r.stdout)
 (d/('native-report.json' if native else 'report.json')).write_text(json.dumps(report,indent=2)+'\n');print('Actual audio endpoint/COM/PCM witness passes:',len(report['observations']))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/audio/witness');p.add_argument('--build',action='store_true');p.add_argument('--native',action='store_true');p.add_argument('--runner',type=Path);a=p.parse_args();d=a.output.resolve()
 if a.build:build(d)
 else:
  if not a.native and not a.runner:p.error('--runner is required')
  execute(d,a.runner,a.native)
