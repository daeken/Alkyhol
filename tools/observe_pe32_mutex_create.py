#!/usr/bin/env python3
"""Native named-mutex lifetime/other-process evidence for the actual DAO frontier."""
import argparse,ctypes,json,os,select,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
FILES=[f'mutex-create-{i}.exe' for i in range(3)]
SOURCES=['mutex-create.c','mutex-create.def']
NAME=r'Local\AlkyholOwnedPe32Mutex-{FFEFAC52-53A9-48CC-9660-FBD4EB3B465F}'
HOST_BANNER=b'PE32 diagnostic mode: guarded external imports; every fault stops; no exception recovery\n'
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'mutex-create-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('mutex source/PE identity changed')
    if r!=frozen()['inputs']:raise ValueError('changed frozen mutex source/PE identity')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'mutex-create.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'mutex-create.def'),'/out:'+str(lib)],check=True)
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('CreateMutexW',12),('CloseHandle',4)]]
    for mode,name in enumerate(FILES):
        if mode<2:
            literal=ROOT/f'pe/reference/pe32-mutex-create-{mode}-v1.exe'
            if digest(literal)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native mutex PE')
            shutil.copyfile(literal,d/name);continue
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/'mutex-create.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*aliases,'/out:'+str(d/name)],check=True)
    (d/'mutex-create-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
    # Separate holder source never rewrites the source-pinned native observers.
    lib=d/'mutex-holder.lib';obj=d/'mutex-holder.obj'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'mutex-holder.def'),'/out:'+str(lib)],check=True)
    subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(SOURCE/'mutex-holder.c'),'-o',str(obj)],check=True)
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*aliases,'/alternatename:__imp__ReadFile@20=__imp__ReadFile','/out:'+str(d/'mutex-holder.exe')],check=True)
    (d/'mutex-holder-inputs.json').write_text(json.dumps({'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in [*SOURCES,'mutex-holder.c','mutex-holder.def']},'pe_sha256':digest(d/'mutex-holder.exe')},indent=2)+'\n')
    print('Selected identical native mutex PEs and built the separate live-reference holder')

def decode(data,case):
    if case not in ['lifecycle','other-process-live','after-other-process-close']:raise ValueError('unknown mutex observation case')
    e=0xface1234
    expected=[0x4d583332,4,1,0,1,183,1,e,1,e,1,0,1,e,1,16] if case=='lifecycle' else [0x4d583332,4,1,183 if case=='other-process-live' else 0,1,e,1,8]
    if len(data)!=len(expected)*4:raise ValueError('incomplete mutex transport')
    words=list(struct.unpack('<'+'I'*len(expected),data))
    if words!=expected:raise ValueError(f'mutex existence/last-close/caller-stack/error observations differ: {words}')
    return words

def frozen():
    path=ROOT/'pe/reference/pe32-mutex-create-windows-v1.json'
    if digest(path)!='ac8dd7be930dadb38097535fcdbcd3303a42219a1fcdbe2a87734cf0872a1eee':raise ValueError('changed frozen native mutex report')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows named mutex observer' or [x['case'] for x in r['reports']]!=['lifecycle','other-process-live','after-other-process-close']:raise ValueError('incomplete frozen mutex evidence')
    for row in r['reports']:
        if row['status'] or row['stderr_hex'] or decode(bytes.fromhex(row['stdout_hex']),row['case'])!=row['words']:raise ValueError('invalid frozen mutex record')
    return r

def host(d):
    from observe_pe32_activation_startup import runner
    identity=verify(d);probe=runner(d);reports=[];path=d/'mutex-create-host.json'
    holder=json.loads((d/'mutex-holder-inputs.json').read_text())
    if holder!={'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in [*SOURCES,'mutex-holder.c','mutex-holder.def']},'pe_sha256':digest(d/'mutex-holder.exe')}:raise ValueError('changed holder sources/PE')
    def save():path.write_text(json.dumps({'backend':'Rosetta explicit session mutex leases','inputs':identity,'holder':json.loads((d/'mutex-holder-inputs.json').read_text()),'reports':reports},indent=2)+'\n')
    with tempfile.TemporaryDirectory(prefix='alkyhol-mutex-') as directory:
        def command(file,configured=True):
            return [str(probe),str((d/file).resolve()),'--strict','--file-origins']+(['--object-directory',directory] if configured else [])
        def run(case,file,configured=True,reason=None):
            r=subprocess.run(command(file,configured),capture_output=True,timeout=20)
            row={'case':case,'file':file,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row);save()
            if reason is not None:
                if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('mutex exclusion failed for a different reason')
            else:
                if r.returncode or r.stderr!=HOST_BANNER:raise ValueError('host mutex execution failed; raw result retained')
                row['words']=decode(r.stdout,case);save()
                if case in [x['case'] for x in frozen()['reports']]:
                    native=next(x for x in frozen()['reports'] if x['case']==case)
                    if row['words']!=native['words']:raise ValueError('host differs from identical native mutex PE')
        run('lifecycle',FILES[0])
        for crash in [False,True]:
            child=subprocess.Popen(command('mutex-holder.exe'),stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
            try:
                if not select.select([child.stdout],[],[],20)[0]:raise ValueError('guest holder readiness timed out')
                ready=os.read(child.stdout.fileno(),8)
                reports.append({'case':'holder-crash' if crash else 'holder-close','ready_hex':ready.hex()});save()
                if ready!=struct.pack('<2I',1,0):raise ValueError('guest holder failed to create a new object')
                run('other-process-live',FILES[1])
                if crash:child.kill()
                out,err=child.communicate(None if crash else b'x',timeout=20)
                if out or err!=HOST_BANNER or (child.returncode!=0 if not crash else child.returncode>=0):raise ValueError('unexpected guest holder teardown')
            finally:
                if child.poll() is None:child.kill();child.communicate(timeout=20)
            run('after-other-process-close',FILES[1])
        run('initial-owner',FILES[2],reason=b'mutex security attributes or initial ownership are unsupported')
        run('unconfigured',FILES[0],configured=False,reason=b'mutex creation requires an explicit object directory')
    print('Identical native mutex words agree; separate guest-process references survive one close and disappear on close/crash')

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
    run('after-other-process-close',FILES[1])
    h=json.loads((d/'mutex-create-host.json').read_text())
    if h['inputs']!=identity:raise ValueError('native/host mutex identities differ')
    for row in reports:
        peers=[x for x in h['reports'] if x['case']==row['case']]
        if not peers or any(x['status'] or bytes.fromhex(x['stderr_hex'])!=HOST_BANNER or decode(bytes.fromhex(x['stdout_hex']),x['case'])!=row['words'] for x in peers):raise ValueError('native/host mutex words differ')
    print('Observed native named mutex creation, live other-process reference and last-close destruction; identical host words agree')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(native if a.native else host if a.host else build)(a.output.resolve())
