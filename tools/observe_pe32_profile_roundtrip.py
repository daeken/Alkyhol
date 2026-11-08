#!/usr/bin/env python3
"""Native-first evidence for DAO's scalar INI write/read family; no game inputs."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
FILES=[f'profile-roundtrip-{i}.exe' for i in range(7)]
SOURCES=['profile-roundtrip.c','profile-roundtrip.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleFileNameW',12),('WritePrivateProfileStringW',16),('GetPrivateProfileStringW',24)]
VALUES=['alpha','beta','','1','gamma','tail','final']
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'profile-roundtrip-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('profile lifecycle source/PE identity changed')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'profile-roundtrip.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'profile-roundtrip.def'),'/out:'+str(lib)],check=True)
    for step,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DSTEP={step}','-c',str(SOURCE/'profile-roundtrip.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
    (d/'profile-roundtrip-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')

def decode(data,step):
    if len(data)!=168:raise ValueError('incomplete profile lifecycle report')
    magic,pointers,index,wrote,we,count,re,balanced,before=struct.unpack_from('<9I',data)
    after=struct.unpack_from('<I',data,164)[0];expected=VALUES[step].encode('utf-16-le')+b'\0\0'
    if [magic,pointers,index,wrote,count,balanced,before,after]!=[0x50524632,4,step,1,len(expected)//2-1,1,0x715e7a11,0x715e7a11] or data[36:36+len(expected)]!=expected:raise ValueError('profile read/write/ABI/buffer observation differs at step '+str(step))
    return {'wrote':wrote,'write_error_observed':we,'count':count,'read_error_observed':re,'value':VALUES[step],'stack_balanced':balanced}

def execute(d,native=False):
    identity=verify(d);run=Path(tempfile.mkdtemp(prefix='profile-',dir=d));root=run/'root';root.mkdir();reports=[];probe=None if native else runner(d);path=d/('native-profile-roundtrip.json' if native else 'profile-roundtrip-host.json')
    def save():path.write_text(json.dumps({'backend':'native Windows scalar profile lifecycle' if native else 'Rosetta scalar profile lifecycle','inputs':identity,'reports':reports},indent=2)+'\n')
    for step,name in enumerate(FILES):
        exe=run/name;shutil.copyfile(d/name,exe);command=[str(exe.resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins','--private-profile-files','--documents-directory',str(root)]
        r=subprocess.run(command,capture_output=True,timeout=20);row={'step':step,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row);save()
        if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('profile lifecycle failed; raw capture retained')
        row['decoded']=decode(r.stdout,step);file=root/'alkyhol-owned-profile.ini'
        if not file.is_file() or file.stat().st_size>65536:raise ValueError('profile API did not leave a bounded real file')
        row['file_hex']=file.read_bytes().hex();save()
    print('Captured seven persistent profile write/read transitions and exact file bytes')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
