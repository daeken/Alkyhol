#!/usr/bin/env python3
"""One stateful PE exercises the DAO core API batch across shared subsystems."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
FILES=[f'runtime-family-{i}.exe' for i in range(9)]
SOURCES=['runtime-family.c','runtime-family.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleFileNameA',12),('GetCurrentThread',0),('InterlockedExchangeAdd',8),('CreateEventA',16),('CreateSemaphoreA',16),('ReleaseSemaphore',12),('WaitForSingleObject',8),('CloseHandle',4),('SetEvent',4),('SetCurrentDirectoryA',4),('GetCurrentDirectoryA',8),('CreateDirectoryA',8),('GetFileAttributesA',4),('GetFileAttributesW',4),('CreateFileA',28),('GetFileSize',8),('GetFileSizeEx',8),('WritePrivateProfileStringW',16),('GetPrivateProfileStringW',24)]
FIXTURE=b'12345678'
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES},'fixture_hex':FIXTURE.hex()}
def verify(d):
    r=json.loads((d/'runtime-family-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('runtime family source/PE identity changed')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'runtime-family.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'runtime-family.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function','-Wno-unused-variable',f'-DMODE={mode}','-c',str(SOURCE/'runtime-family.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
    (d/'runtime-family-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')

def decode(data):
    if len(data)%4:raise ValueError('truncated runtime family report')
    w=list(struct.unpack('<'+'I'*(len(data)//4),data));g=0x71717171
    expected=[0x46414d32,4,1,0x7fffffff,0x80000000,0x80000000,0x7fffffff,0xffffffff,1,
              1,0,258,1,0,1,1,1,0,0,0,258,1,0,298,g,0,87,g,0,87,g,1,
              1,1,1,1,1,0,183,16,16,32,32,1,8,0,8,1,8,0,0x715e7a11,0x715e7a11,1,
              0xffffffff,2,0xffffffff,2,0xffffffff,3,1,8,2,1,0,2,1,1,0]
    expected[-1]=len(expected)
    if w!=expected:raise ValueError('runtime family shared state, ABI, errors or output guards differ: '+repr(w))
    return w

def execute(d,native=False):
    identity=verify(d);reports=[];probe=None if native else runner(d);path=d/('native-runtime-family.json' if native else 'runtime-family-host.json')
    def save():path.write_text(json.dumps({'backend':'native Windows DAO core family' if native else 'Rosetta DAO core family','inputs':identity,'reports':reports},indent=2)+'\n')
    for mode in ([0] if native else range(9)):
        run=Path(tempfile.mkdtemp(prefix='family-',dir=d));root=run/'root';root.mkdir();(root/'file').write_bytes(FIXTURE);exe=run/FILES[mode];shutil.copyfile(d/exe.name,exe)
        command=[str(exe.resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins','--documents-directory',str(root),'--private-profile-files']
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row);save()
        if mode==0:
            if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('runtime family PE failed; raw output retained')
            row['words']=decode(r.stdout);row['filesystem']={'created':(root/'new').is_dir(),'file_hex':(root/'file').read_bytes().hex(),'ini_hex':(root/'query.ini').read_bytes().hex()};save()
            if row['filesystem']!={'created':True,'file_hex':FIXTURE.hex(),'ini_hex':b'[S]\r\nK=value\r\n'.hex()}:raise ValueError('core family effects differ from reported state')
        else:
            reasons={1:b'semaphore release requires an owned live counting semaphore',2:b'file-size output outside checked writable guest storage',3:b'interlocked add operand outside aligned writable guest storage',4:b'ordinary file path requires a checked current directory',5:b'ANSI family paths currently require ASCII',6:b'ANSI current-directory output outside writable guest storage',7:b'PE32 CreateEventW security attributes and named events are unsupported',8:b'semaphore previous-count output outside writable guest storage'}
            if r.returncode!=78 or r.stdout or reasons[mode] not in r.stderr:raise ValueError('core family exclusion failed for a different reason: '+str(mode))
    if native:
        peer=json.loads((d/'runtime-family-host.json').read_text());row=peer['reports'][0]
        if peer['inputs']!=identity or row['status'] or decode(bytes.fromhex(row['stdout_hex']))!=reports[0]['words'] or row['filesystem']!=reports[0]['filesystem']:raise ValueError('native/host runtime family identity or evidence differs')
    print('Atomic wraparound, live semaphore/event state, ANSI aliases, file metadata/sizes and checked buffers agree')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
