#!/usr/bin/env python3
"""DAO's INI read: defaults only after real file/path absence is established."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
FILES=[f'private-profile-missing-{i}.exe' for i in range(8)]
SOURCES=['private-profile-missing.c','private-profile-missing.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleFileNameW',12),('GetPrivateProfileStringW',24),('SetCurrentDirectoryW',4)]
FIXTURE=b'[Section]\r\nKey=actual-owned-value\r\n'
DEFAULTS=['fallback','','  fallback','','π桜🌸','abc','','fallback','','abcdef']
SIZES=[32,32,32,32,32,4,1,32,32,7]
ERRORS=[2,2,2,2,2,2,2,3,2,2]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES},'fixture_hex':FIXTURE.hex()}
def verify(d):
    r=json.loads((d/'private-profile-missing-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('private-profile source/PE identity changed')
    if r!=frozen()['inputs']:raise ValueError('private-profile identity differs from literal native observation')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'private-profile-missing.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'private-profile-missing.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function',f'-DMODE={mode}','-c',str(SOURCE/'private-profile-missing.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
        literal=ROOT/f'pe/reference/pe32-private-profile-missing-{mode}-v1.exe'
        if digest(literal)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native private-profile PE')
        shutil.copyfile(literal,d/name)
    (d/'private-profile-missing-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')

def decode(data):
    if len(data)!=936 or struct.unpack_from('<4I',data)!=(0x494e4932,4,10,92):raise ValueError('incomplete private-profile report')
    rows=[]
    for i,(text,size) in enumerate(zip(DEFAULTS,SIZES)):
        start=16+i*92;index,capacity,result,error,balanced,before=struct.unpack_from('<6I',data,start);units=struct.unpack_from('<32H',data,start+24);after=struct.unpack_from('<I',data,start+88)[0]
        expected=text.encode('utf-16-le')+b'\0\0';n=len(expected)//2-1
        if [index,capacity,result,error,balanced,before,after]!=[i,size,n,ERRORS[i],1,0x715e7a11,0x715e7a11]:raise ValueError('private-profile return/error/ABI/guard mismatch at case '+str(i))
        if data[start+24:start+24+len(expected)]!=expected or units[size:]!=(0x7171,)*(32-size):raise ValueError('private-profile default/truncation/termination/output bounds differ')
        rows.append({'case':i,'capacity':capacity,'result':result,'error':error,'balanced':balanced,'text':text,'buffer_units':list(units)})
    return rows

def frozen():
    path=ROOT/'pe/reference/pe32-private-profile-missing-windows-v1.json'
    if digest(path)!='b5cf4426db5fdaca66b129614614cb3f7a21b3d9002274f63ca4e4f7de0e48e8':raise ValueError('changed raw native private-profile record')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows absent-file private profile' or len(r['reports'])!=1 or r['native_relative_anchor']!={'result':18,'text':'actual-owned-value'}:raise ValueError('incomplete native private-profile record')
    row=r['reports'][0]
    if row['case'] or row['status'] or row['stderr_hex']:raise ValueError('failed native private-profile execution')
    decode(bytes.fromhex(row['stdout_hex']))
    return r

def execute(d,native=False):
    identity=verify(d);reports=[];probe=None if native else runner(d)
    run=d/'private-profile-run';run.mkdir(exist_ok=True)
    for name in FILES:shutil.copyfile(d/name,run/name)
    for name in ['alkyhol-absent-profile.ini','absent-profile-parent']:
        if (run/name).exists():raise ValueError('owned absent profile fixture already exists')
    fixture=run/'alkyhol-owned-profile.ini'
    if fixture.exists() and fixture.read_bytes()!=FIXTURE:raise ValueError('refusing to overwrite unknown INI contents')
    fixture.write_bytes(FIXTURE)
    anchor=None
    if native:
        import ctypes,os
        before=os.getcwd()
        try:
            os.chdir(run)
            output=ctypes.create_unicode_buffer(64)
            query=ctypes.windll.kernel32.GetPrivateProfileStringW
            query.argtypes=[ctypes.c_wchar_p,ctypes.c_wchar_p,ctypes.c_wchar_p,ctypes.c_wchar_p,ctypes.c_uint,ctypes.c_wchar_p]
            result=query('Section','Key','fallback',output,64,'.\\alkyhol-owned-profile.ini')
            anchor={'result':result,'text':output.value}
            if anchor!={'result':18,'text':'actual-owned-value'}:raise ValueError('native dot-relative INI did not resolve against controlled current directory')
        finally:os.chdir(before)
    path=d/('native-private-profile-missing.json' if native else 'private-profile-missing-host.json')
    def save():path.write_text(json.dumps({'backend':'native Windows absent-file private profile' if native else 'Rosetta explicit absent-file private profile','inputs':identity,'native_relative_anchor':anchor,'reports':reports},indent=2)+'\n')
    for mode in ([0] if native else range(9)):
        command=[str((run/FILES[0 if mode==8 else mode]).resolve())]
        if not native:
            command=[str(probe),*command,'--strict','--file-origins']
            if mode!=8:command+=['--private-profile-files']
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row);save()
        if mode==0:
            if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('private-profile observer failed; raw results retained')
            row['records']=decode(r.stdout);save()
            if r.stdout!=bytes.fromhex(frozen()['reports'][0]['stdout_hex']):raise ValueError('private-profile output differs from complete raw native record')
        else:
            reasons={1:b'private-profile enumeration is unsupported',2:b'private-profile enumeration is unsupported',3:b'private-profile buffer sizes outside 1..32768',4:b'private-profile output outside writable UTF-16 guest storage',5:b'private-profile paths require absolute Z: or explicit dot-relative leaf spelling',6:b'existing private-profile files require measured INI parsing',7:b'private-profile trailing control whitespace is unmeasured',8:b'private-profile reads require an explicit file-only INI profile'}
            if r.returncode!=78 or r.stdout or reasons[mode] not in r.stderr:raise ValueError('private-profile exclusion failed for a different reason: '+str(mode))
    if native:
        peer=json.loads((d/'private-profile-missing-host.json').read_text());row=peer['reports'][0]
        if peer['inputs']!=identity or row['status'] or decode(bytes.fromhex(row['stdout_hex']))!=row['records'] or row['records']!=reports[0]['records']:raise ValueError('native/host private-profile identities or complete records differ')
    print('Real absence, scalar defaults, Unicode, trailing spaces, truncation, error and stack evidence validated')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
