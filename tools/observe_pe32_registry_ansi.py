#!/usr/bin/env python3
"""Owned mixed ANSI/Unicode registry handles and trailing separator witness."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
SOURCES=['registry-ansi.c','registry-ansi.def','registry-ansi-kernel.def','registry-ansi.xml'];FILES=['registry-ansi.exe'];SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('RegOpenKeyExA',20),('RegOpenKeyExW',20),('RegCloseKey',4),('RegQueryValueExA',24),('RegQueryValueExW',24)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');libs=[]
 for role in ['registry-ansi','registry-ansi-kernel']:
  lib=d/(role+'.lib');libs.append(str(lib));subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(role+'.def')),'/out:'+str(lib)],check=True)
 obj=d/'registry-ansi.obj';subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'registry-ansi-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 if len(data)%4:raise ValueError('truncated window family report')
 words=list(struct.unpack('<'+'I'*(len(data)//4),data));expected=[0x52474132,4]+[1]*13+[16]
 if words!=expected:raise ValueError('file contents, sharing, descriptor metadata or stack differ: '+repr(words))
 return words
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'registry-ansi-inputs.json').read_text())!=identity:raise ValueError('window family identity changed')
 run=Path(tempfile.mkdtemp(prefix='file-',dir=d));exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);(run/'root').mkdir();command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins','--registry-catalog',str(SOURCE/'registry-ansi.xml')]
 r=run_native(command) if native else subprocess.run(command,capture_output=True,timeout=30);report={'backend':'native Windows owned file' if native else 'physical Rosetta PE32 file','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-registry-ansi.json' if native else 'registry-ansi-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode:raise ValueError('owned file PE failed; raw output retained')
 report['words']=decode(r.stdout)
 if native:
  peer=json.loads((d/'registry-ansi-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['stdout_hex']!=report['stdout_hex']:raise ValueError('identical PE host/native observations differ')
 path.write_text(json.dumps(report,indent=2)+'\n');print('Mixed A/W registry handles, trailing separators, known absence and x86 stack validate')
def run_native(command):
 import winreg
 scope=r'Software\AlkyholAnsiDaoStartup\{E97B4171-7A2B-4A82-A867-9AF56BFF70F4}'
 access=winreg.KEY_ALL_ACCESS|winreg.KEY_WOW64_32KEY
 try:
  key=winreg.OpenKey(winreg.HKEY_CURRENT_USER,scope,0,access)
 except FileNotFoundError:pass
 else:
  key.Close();raise ValueError('owned registry test scope already exists')
 parent=r'Software\AlkyholAnsiDaoStartup'
 try:
  key=winreg.OpenKey(winreg.HKEY_CURRENT_USER,parent,0,access);key.Close();created_parent=False
 except FileNotFoundError:created_parent=True
 try:
  with winreg.CreateKeyEx(winreg.HKEY_CURRENT_USER,scope,0,access) as key:
   with winreg.CreateKeyEx(key,'Present'):pass
   winreg.SetValueEx(key,'Path',0,winreg.REG_SZ,r'Z:\owned')
   winreg.SetValueEx(key,'Number',0,winreg.REG_DWORD,0x12345678)
  return subprocess.run(command,capture_output=True,timeout=30)
 finally:
  for suffix in ['\\Present','']:
   winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,scope+suffix,winreg.KEY_WOW64_32KEY)
  if created_parent:winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,parent,winreg.KEY_WOW64_32KEY)
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
