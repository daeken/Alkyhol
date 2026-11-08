#!/usr/bin/env python3
"""Identical-PE complete ANSI directory queries reached by original Bink startup."""
import argparse,json,os,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
KINDS={'windows':('GetWindowsDirectoryA','--windows-directory','user Windows directory'),
       'system':('GetSystemDirectoryA','--system-directory','system directory')}
FILES=[f'{kind}-directory-a-{mode}.exe' for kind in KINDS for mode in range(3)]
SOURCES=['windows-directory-a.c','windows-directory-a.def','system-directory-a.def']
def inputs(d):
    return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    recorded=json.loads((d/'ansi-directories-inputs.json').read_text())
    if recorded!=inputs(d):raise ValueError('ANSI directory source/PE identity changed')
    return recorded

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld')
    if not clang:raise RuntimeError('open Clang is required')
    for kind,(api,_,_) in KINDS.items():
        lib=d/f'{kind}-directory-a.lib'
        subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/f'{kind}-directory-a.def'),'/out:'+str(lib)],check=True)
        aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),(api,8)]]
        for mode in range(3):
            name=f'{kind}-directory-a-{mode}';obj=d/(name+'.obj')
            subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}',f'-DGetWindowsDirectoryA={api}','-c',str(SOURCE/'windows-directory-a.c'),'-o',str(obj)],check=True)
            subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*aliases,'/out:'+str(d/(name+'.exe'))],check=True)
    (d/'ansi-directories-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');print('Built complete ANSI user-Windows/system-directory witnesses')

def decode(data,expected_path=None):
    if len(data)<41:raise ValueError('incomplete ANSI directory transport')
    words=list(struct.unpack('<10I',data[:40]));n=words[2]
    if not 0<n<1023 or len(data)!=40+n+1:raise ValueError('ANSI directory extent differs')
    contract=words[:2]+words[3:]
    if contract!=[0x57443332,4,1,0x715e7a11,0x715e7a11,1,1,0xface1234,10]:raise ValueError('ANSI directory stack/guard/error observations differ')
    path=data[40:].decode('ascii')
    if not path.endswith('\0') or '\0' in path[:-1]:raise ValueError('invalid ANSI directory termination')
    path=path[:-1]
    if expected_path is not None and path!=expected_path:raise ValueError('ANSI directory does not identify the independently configured backed role')
    return {'contract':contract,'path':path,'length':n}

def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[]
    report_path=d/('native-ansi-directories.json' if native else 'ansi-directories-host.json')
    def save():report_path.write_text(json.dumps({'backend':'native Windows ANSI directory observer' if native else 'Rosetta explicit backed ASCII directory profiles','inputs':identity,'reports':reports},indent=2)+'\n')
    for kind,(api,option,role) in KINDS.items():
        root=d/'ANSI directory profiles';user=root/'PrivateWindows';system=root/'OtherSystem';shared=root/'SharedWindows'
        for p in [user,system,shared]:
            if not native:p.mkdir(parents=True,exist_ok=True)
        directory=user if kind=='windows' else system
        cases=[('positive',0,directory,None)]
        if not native:
            unicode_path=root/'non-ASCII-λ';unicode_path.mkdir(exist_ok=True)
            cases += [('bad-output',1,directory,b'output outside writable guest storage'),('small-output',2,directory,b'sizing/truncation profile'),('unconfigured',0,None,b'requires an explicitly configured backed guest directory'),('non-ASCII',0,unicode_path,b'requires an ASCII-backed origin')]
        for label,mode,profile,reason in cases:
            name=f'{kind}-directory-a-{mode}.exe';command=[str((d/name).resolve())]
            if not native:
                command=[str(probe),*command,'--strict','--file-origins','--system-windows-directory',str(shared)]
                if profile is not None:command += [option,str(profile)]
            r=subprocess.run(command,capture_output=True,timeout=20);row={'case':label,'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row);save()
            if reason is not None:
                if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('ANSI directory restriction did not stop for its actual reason: '+label)
                continue
            if r.returncode or native and r.stderr:raise ValueError('ANSI directory observation failed; raw report retained')
            row['record']=decode(r.stdout,None if native else 'Z:'+str(profile.resolve()).replace('/','\\'))
            if native:
                actual=row['record']['path'];expected=Path(os.environ['WINDIR'])/('System32' if kind=='system' else '')
                if not Path(actual).is_dir() or actual.casefold()!=str(expected.resolve()).casefold():raise ValueError('native directory differs from its actual verified Windows root/role')
            save()
    if native:
        host=json.loads((d/'ansi-directories-host.json').read_text())
        if host['inputs']!=identity:raise ValueError('native/host directory input identity differs')
        for row in reports:
            h=next(h for h in host['reports'] if h['case']=='positive' and h['file']==row['file'])
            if h['status'] or decode(bytes.fromhex(h['stdout_hex']))!=h['record'] or h['record']['contract']!=row['record']['contract']:raise ValueError('native/host directory ABI contract differs')
        print('Both identical-PE complete ANSI directory ABI/guard/error contracts agree; actual backed paths verified separately')
    else:print('Distinct backed user/shared/system roles and eight explicit restrictions pass; native comparisons separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
