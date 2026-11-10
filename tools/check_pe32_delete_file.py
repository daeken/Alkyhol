#!/usr/bin/env python3
"""Owned identical PE checks for DAO's reached ordinary save-file deletion."""
import argparse,hashlib,json,os,platform,shutil,stat,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
SOURCE=ROOT/'pe/fixtures/pe32-startup'
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetEnvironmentVariableA',12),('SetCurrentDirectoryW',4),('DeleteFileW',4),('DeleteFileA',4),('CreateFileW',28),('CloseHandle',4),('GetLastError',0),('SetLastError',4)]
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def identity(d):return dict(sources={n:digest(SOURCE/n) for n in ['delete-file.c','delete-file.def']},pe_sha256=digest(d/'delete-file.exe'))
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');lib=d/'kernel32.lib';obj=d/'delete-file.obj'
 subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'delete-file.def'),'/out:'+str(lib)],check=True)
 subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'delete-file.c'),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/'delete-file.exe')],check=True)
 (d/'inputs.json').write_text(json.dumps(identity(d),indent=2)+'\n')
def check(d,host,native):
 if identity(d)!=json.loads((d/'inputs.json').read_text()):raise ValueError('changed deletion witness source/PE')
 if native and os.name!='nt':raise ValueError('native observation requires Windows')
 report=dict(inputs=identity(d),backend='native Windows' if native else 'physical Rosetta host assertions',platform=platform.platform(),observations=[])
 for rebase in ([False] if native else [False,True]):
  for mode in range(7 if native else 12):
   parent=d/f'case-{mode}-{int(rebase)}';root=parent/'Documents';root.mkdir(parents=True,exist_ok=False)
   target=root/'target.bin';outside=parent/'outside.bin';outside.write_bytes(b'outside retained');target.write_bytes(b'owned save test bytes')
   if mode==3:target.chmod(stat.S_IRUSR|stat.S_IRGRP|stat.S_IROTH)
   if mode==4:(root/'folder').mkdir()
   if mode in [9,10]:
    target.unlink();os.link(outside,target) if mode==9 else target.symlink_to(outside.resolve())
   guest=str(root.resolve()) if native else 'Z:'+str(root.resolve()).replace('/','\\')
   command=[str((d/'delete-file.exe').resolve())] if native else [str(host.resolve()),str((d/'delete-file.exe').resolve()),'--strict','--file-origins','--documents-directory',str(root.resolve()),*(['--rebase'] if rebase else [])]
   r=subprocess.run(command,env=dict(os.environ,ALKYHOL_DELETE_ROOT=guest,ALKYHOL_DELETE_MODE=str(mode)),capture_output=True,timeout=30)
   row=dict(mode=mode,rebased=rebase,status=r.returncode,stdout_hex=r.stdout.hex(),stderr=r.stderr.decode(errors='replace'));report['observations'].append(row)
   (d/'report.json').write_text(json.dumps(report,indent=2)+'\n')
   if mode>=7:
    if r.returncode!=78 or r.stdout!=struct.pack('<2I',0x444c3332,mode):raise ValueError('missing explicit deletion exclusion')
   else:
    if r.returncode or len(r.stdout)!=32:raise ValueError('delete ABI witness failed')
    magic,m,result,error,again,again_error,stack,closed=struct.unpack('<8I',r.stdout)
    if magic!=0x444c3332 or m!=mode or stack!=1 or closed!=1:raise ValueError('delete ABI/stack/handle result differs')
    if mode in [0,6]:
     if not result or again or again_error!=2 or target.exists():raise ValueError('real deletion/repeated absence differs')
    elif result or error!={1:2,2:3,3:5,4:5,5:32}[mode] or again or again_error:raise ValueError('defined delete failure differs')
   if outside.read_bytes()!=b'outside retained':raise ValueError('outside capability bytes changed')
   if mode not in [0,6] and target.read_bytes()!=b'owned save test bytes' and mode not in [9,10]:raise ValueError('failed deletion changed target')
   if mode in [9,10] and target.read_bytes()!=b'outside retained':raise ValueError('unsupported alias changed')
 print('DeleteFile checks pass:',len(report['observations']))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/delete-file');p.add_argument('--build',action='store_true');p.add_argument('--native',action='store_true');p.add_argument('--host',type=Path);a=p.parse_args();d=a.output.resolve()
 if a.build:build(d)
 else:
  if not a.native and not a.host:p.error('--host is required for host checks')
  check(d,a.host,a.native)
