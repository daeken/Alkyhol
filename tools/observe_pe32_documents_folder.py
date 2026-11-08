#!/usr/bin/env python3
"""Current-user backed Documents query, HRESULT/guards and stdcall evidence.

Native and host paths are independently backed, intentionally platform-specific.
LastError is recorded, never normalized or treated as an HRESULT API guarantee.
"""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
FILES=[f'documents-folder-{i}.exe' for i in range(6)]
SOURCES=['documents-folder.c','documents-folder-kernel.def','documents-folder-shell.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleHandleW',4),('GetProcAddress',8),('SHGetFolderPathW',20)]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'documents-folder-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('Documents source/PE identity changed')
    if r!=frozen()['inputs']:raise ValueError('Documents identity differs from frozen native observation')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    for role in ['kernel','shell']:
        lib=d/f'documents-folder-{role}.lib';libs.append(str(lib))
        subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/f'documents-folder-{role}.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function',f'-DMODE={mode}','-c',str(SOURCE/'documents-folder.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
        literal=ROOT/f'pe/reference/pe32-documents-folder-{mode}-v1.exe'
        if digest(literal)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native Documents PE')
        shutil.copyfile(literal,d/name)
    (d/'documents-folder-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')

def decode(data,expected):
    if len(data)!=1096:raise ValueError('incomplete Documents report')
    magic,pointers,hr0,hr1,gle0,gle1,balanced,n0,n1=struct.unpack_from('<9I',data)
    if [magic,pointers,hr0,hr1,balanced]!=[0x444f4332,4,0,0,1] or struct.unpack_from('<I',data,1092)[0]!=0x656e6432:raise ValueError('Documents HRESULT/ABI/stack mismatch')
    paths=[]
    for i,n in enumerate([n0,n1]):
        start=36+i*528
        if not 0<n<260 or struct.unpack_from('<I',data,start)[0]!=0x715e7a11 or struct.unpack_from('<I',data,start+524)[0]!=0x715e7a11:raise ValueError('Documents output length/guard mismatch')
        text=data[start+4:start+4+(n+1)*2].decode('utf-16-le')
        if text!=expected+'\0' or '\0' in text[:-1]:raise ValueError('Documents path differs from independently verified backing')
        paths.append(text[:-1])
    return {'hresults':[hr0,hr1],'last_errors_observed_only':[gle0,gle1],'stack_balanced':balanced,'paths':paths,'path_units':[n0,n1]}

def frozen():
    path=ROOT/'pe/reference/pe32-documents-folder-windows-v1.json'
    if digest(path)!='72ca6264014946d0d2dccb937c585c4d1b74c9b02e7d0330720d8e1f302f045e':raise ValueError('changed native Documents report')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows current-user Documents' or len(r['reports'])!=1:raise ValueError('incomplete native Documents report')
    row=r['reports'][0]
    if row['case'] or row['status'] or row['stderr_hex'] or decode(bytes.fromhex(row['stdout_hex']),r['expected_path'])!=row['decoded']:raise ValueError('failed native Documents observation')
    return r

def native_path():
    import ctypes
    buffer=ctypes.create_unicode_buffer(260)
    query=ctypes.windll.shell32.SHGetFolderPathW
    # Establish this runner's real current-user directory; the PE itself asks
    # for the existing current path, without CREATE or DONT_VERIFY flags.
    if query(None,0x8005,None,0,buffer)!=0 or not Path(buffer.value).is_dir():raise ValueError('native Documents backing cannot be established')
    return buffer.value

def execute(d,native=False):
    identity=verify(d);reports=[];probe=None if native else runner(d)
    if native:expected=native_path();directory=None
    else:
        directory=d/'profile'/'Documents-桜🌸';directory.mkdir(parents=True,exist_ok=True)
        expected='Z:'+str(directory.resolve()).replace('/','\\')
    path=d/('native-documents-folder.json' if native else 'documents-folder-host.json')
    def save():path.write_text(json.dumps({'backend':'native Windows current-user Documents' if native else 'Rosetta explicit backed Documents','inputs':identity,'expected_path':expected,'reports':reports},indent=2)+'\n')
    for mode in ([0] if native else [0,1,2,3,4,5,6]):
        command=[str((d/FILES[min(mode,5) if mode!=6 else 0]).resolve())]
        if not native:
            command=[str(probe),*command,'--strict','--file-origins']
            if mode!=6:command+=['--documents-directory',str(directory)]
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row);save()
        if mode==0:
            if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('Documents observer failed; raw results retained')
            row['decoded']=decode(r.stdout,expected);save()
        else:
            reason=b'SHGetFolderPathW output requires writable MAX_PATH WCHAR storage' if mode==4 else b'SHGetFolderPathW requires an explicit backed Documents directory' if mode==6 else b'SHGetFolderPathW requires NULL owner/token, CSIDL_PERSONAL and current-path flags'
            if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('Documents exclusion failed for a different reason')
    if native:
        peer=json.loads((d/'documents-folder-host.json').read_text());row=peer['reports'][0]
        if peer['inputs']!=identity or row['status'] or decode(bytes.fromhex(row['stdout_hex']),peer['expected_path'])!=row['decoded']:raise ValueError('native/host Documents identities or bounded behavior differ')
    print('Direct and dynamically resolved same-PE Documents calls return their real per-host backing with intact guards and balanced stack')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
