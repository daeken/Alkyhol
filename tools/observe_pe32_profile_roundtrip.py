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
    if r!=frozen()['inputs']:raise ValueError('profile lifecycle differs from literal native identity')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'profile-roundtrip.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'profile-roundtrip.def'),'/out:'+str(lib)],check=True)
    for step,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DSTEP={step}','-c',str(SOURCE/'profile-roundtrip.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
        literal=ROOT/f'pe/reference/pe32-profile-roundtrip-{step}-v1.exe'
        if digest(literal)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native profile PE')
        shutil.copyfile(literal,d/name)
    (d/'profile-roundtrip-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')

def decode(data,step):
    if len(data)!=168:raise ValueError('incomplete profile lifecycle report')
    magic,pointers,index,wrote,we,count,re,balanced,before=struct.unpack_from('<9I',data)
    after=struct.unpack_from('<I',data,164)[0];expected=VALUES[step].encode('utf-16-le')+b'\0\0'
    if [magic,pointers,index,wrote,count,balanced,before,after]!=[0x50524632,4,step,1,len(expected)//2-1,1,0x715e7a11,0x715e7a11] or data[36:36+len(expected)]!=expected:raise ValueError('profile read/write/ABI/buffer observation differs at step '+str(step))
    return {'wrote':wrote,'write_error_observed':we,'count':count,'read_error_observed':re,'value':VALUES[step],'stack_balanced':balanced}

def frozen():
    path=ROOT/'pe/reference/pe32-profile-roundtrip-windows-v1.json'
    if digest(path)!='f114d548fc190551f33d1eb30fc1e1e918fff0e4e7b11611760a5af18a015b38':raise ValueError('changed native profile lifecycle report')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows scalar profile lifecycle' or len(r['reports'])!=7:raise ValueError('incomplete native profile lifecycle')
    for i,row in enumerate(r['reports']):
        if row['step']!=i or row['status'] or row['stderr_hex'] or decode(bytes.fromhex(row['stdout_hex']),i)!=row['decoded'] or not row['file_hex']:raise ValueError('failed native profile lifecycle')
    return r

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
        reference=frozen()['reports'][step]
        if row['stdout_hex']!=reference['stdout_hex'] or row['file_hex']!=reference['file_hex']:raise ValueError('profile output or persistent bytes differ from identical native PE')
    if not native:restrictions(d,probe)
    print('Captured seven persistent profile write/read transitions and exact file bytes')
def restrictions(d,probe):
    rows=[]
    for kind in ['no-profile','outside','syntax','symlink','hardlink']:
        run=Path(tempfile.mkdtemp(prefix='profile-reject-',dir=d));root=run/'root';root.mkdir();outside=run/'outside';outside.mkdir();exe=run/FILES[0];shutil.copyfile(d/FILES[0],exe);target=root/'alkyhol-owned-profile.ini';external=outside/'known';external.write_bytes(b'outside preserved')
        if kind=='syntax':target.write_bytes(b'[S]\nK=unmeasured\n')
        if kind=='symlink':target.symlink_to(external)
        if kind=='hardlink':target.hardlink_to(external)
        before=target.read_bytes() if target.exists() else None
        command=[str(probe),str(exe),'--strict','--file-origins','--documents-directory',str(outside if kind=='outside' else root)]
        if kind!='no-profile':command+=['--private-profile-files']
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':kind,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};rows.append(row);(d/'profile-roundtrip-restrictions.json').write_text(json.dumps(rows,indent=2)+'\n')
        reasons={'no-profile':b'private-profile writes require an explicit file-only INI profile','outside':b'INI file access is restricted','syntax':b'INI contents require measured CRLF termination','symlink':b'PE32 find symbolic links/reparse points','hardlink':b'INI backing requires a bounded regular non-aliased file'}
        if r.returncode!=78 or r.stdout or reasons[kind] not in r.stderr:raise ValueError('INI rejection stopped for another reason: '+kind)
        if (target.read_bytes() if target.exists() else None)!=before or external.read_bytes()!=b'outside preserved':raise ValueError('rejected INI write changed existing bytes')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
