#!/usr/bin/env python3
"""Real wildcard search snapshots, A/W layouts and shared encoding-independent handles."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
SOURCES=['search-family.c','search-family.def'];FILES=['search-family.exe'];SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetModuleFileNameA',12),('GetLastError',0),('FindFirstFileA',8),('FindFirstFileW',8),('FindNextFileA',8),('FindNextFileW',8),('FindClose',4)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def build(d):
 d.mkdir(parents=True,exist_ok=True);lld=rust_tool('rust-lld');lib=d/'search-family.lib';obj=d/'search-family.obj'
 subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/SOURCES[1]),'/out:'+str(lib)],check=True)
 subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/SOURCES[0]),'-o',str(obj)],check=True)
 subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/FILES[0])],check=True)
 (d/'search-family-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')
def decode(data):
 expected=[0x53454332,4]+[x for i in range(6) for x in ([63,6,1,1,18,1,1] if i<3 else [12,2,1,1,18,1,1])]+[1,46]
 if len(data)!=184 or list(struct.unpack('<46I',data))!=expected:raise ValueError('wildcard entries, metadata, A/W layouts, handles, end state or stack differ')
 return expected
def execute(d,native=False):
 identity=inputs(d)
 if json.loads((d/'search-family-inputs.json').read_text())!=identity:raise ValueError('search family identity changed')
 run=Path(tempfile.mkdtemp(prefix='search-',dir=d));root=run/'root';root.mkdir();(root/'alpha.txt').write_bytes(b'abc');(root/'Beta.TXT').write_bytes(b'12345');(root/'plain').write_bytes(b'12');(root/'folder').mkdir();exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);command=[str(exe.resolve())]
 if not native:command=[str(runner(d)),*command,'--strict','--file-origins']
 r=subprocess.run(command,capture_output=True,timeout=20);report={'backend':'native Windows search family' if native else 'Rosetta search family','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/('native-search-family.json' if native else 'search-family-host.json');path.write_text(json.dumps(report,indent=2)+'\n')
 if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('search family PE failed; raw output retained')
 report['words']=decode(r.stdout);path.write_text(json.dumps(report,indent=2)+'\n')
 if native:
  peer=json.loads((d/'search-family-host.json').read_text())
  if peer['inputs']!=identity or peer['status'] or peer['words']!=report['words']:raise ValueError('identical-PE search family differs')
 print('Real all/extension searches agree across A/W and mixed handles with directory/file metadata')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
