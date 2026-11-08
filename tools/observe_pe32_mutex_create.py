#!/usr/bin/env python3
"""Native named-mutex lifetime/other-process evidence for the actual DAO frontier."""
import argparse,ctypes,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
FILES=[f'mutex-create-{i}.exe' for i in range(3)]
SOURCES=['mutex-create.c','mutex-create.def']
NAME=r'Local\AlkyholOwnedPe32Mutex-{FFEFAC52-53A9-48CC-9660-FBD4EB3B465F}'
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'mutex-create-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('mutex source/PE identity changed')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'mutex-create.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'mutex-create.def'),'/out:'+str(lib)],check=True)
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('CreateMutexW',12),('CloseHandle',4)]]
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/'mutex-create.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*aliases,'/out:'+str(d/name)],check=True)
    (d/'mutex-create-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');print('Built native mutex lifecycle/other-process observers; no backend support claimed')

def decode(data,case):
    e=0xface1234
    expected=[0x4d583332,4,1,0,1,183,1,e,1,e,1,0,1,e,1,16] if case=='lifecycle' else [0x4d583332,4,1,183 if case=='other-process-live' else 0,1,e,1,8]
    if len(data)!=len(expected)*4:raise ValueError('incomplete mutex transport')
    words=list(struct.unpack('<'+'I'*len(expected),data))
    if words!=expected:raise ValueError(f'mutex existence/last-close/caller-stack/error observations differ: {words}')
    return words

def native(d):
    identity=verify(d);reports=[];path=d/'native-mutex-create.json'
    def save():path.write_text(json.dumps({'backend':'native Windows named mutex observer','inputs':identity,'reports':reports},indent=2)+'\n')
    def run(case,file):
        r=subprocess.run([str((d/file).resolve())],capture_output=True,timeout=20);row={'case':case,'file':file,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row);save()
        if r.returncode or r.stderr:raise ValueError('native mutex observer failed; raw result retained')
        row['words']=decode(r.stdout,case);save()
    run('lifecycle',FILES[0])
    kernel=ctypes.WinDLL('kernel32',use_last_error=True);create=kernel.CreateMutexW;create.argtypes=[ctypes.c_void_p,ctypes.c_int,ctypes.c_wchar_p];create.restype=ctypes.c_void_p;close=kernel.CloseHandle;close.argtypes=[ctypes.c_void_p];close.restype=ctypes.c_int
    ctypes.set_last_error(ctypes.c_int32(0xface1234).value);handle=create(None,False,NAME);error=ctypes.get_last_error()
    if not handle:raise ValueError('independent native parent mutex creation failed')
    try:
        if error!=0:raise ValueError('native parent found a pre-existing mutex fixture; not claiming new ownership')
        run('other-process-live',FILES[1])
    finally:
        if not close(handle):raise ValueError('native parent mutex close failed')
    run('after-other-process-close',FILES[1]);print('Observed named mutex creation, live other-process reference and actual last-close destruction')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
