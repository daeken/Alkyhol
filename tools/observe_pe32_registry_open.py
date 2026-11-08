#!/usr/bin/env python3
"""Owned scoped registry observations for the RegOpenKeyExW DAO startup frontier."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
FILES=[f'registry-open-{i}.exe' for i in range(5)]
SOURCES=['registry-open.c','registry-open.def','registry-open-kernel32.def','registry-open.xml','registry-open-layout.cpp']
SCOPE=r'Software\AlkyholOwnedDaoStartup\{E97B4171-7A2B-4A82-A867-9AF56BFF70F4}'
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    value=json.loads((d/'registry-open-inputs.json').read_text())
    if value!=inputs(d):raise ValueError('registry-open source/PE identity changed')
    if value!=frozen()['inputs']:raise ValueError('changed frozen registry source/PE identities')
    return value

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    for kind in ['registry-open','registry-open-kernel32']:
        lib=d/(kind+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(kind+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('RegOpenKeyExW',20),('RegCloseKey',4)]]
    for mode,name in enumerate(FILES):
        if mode==0:
            literal=ROOT/'pe/reference/pe32-registry-open-v1.exe'
            if digest(literal)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native registry PE')
            shutil.copyfile(literal,d/name);continue
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}','-c',str(SOURCE/'registry-open.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*aliases,'/out:'+str(d/name)],check=True)
    (d/'registry-open-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Built owned registry open/close observers; native execution remains separate')

def expected_words():
    e=0xface1234
    return [0x52473332,4,0,1,1,e,0,1,1,e,2,0,1,e,0,1,1,e,0,e,0,e,0,e,1,26]
def decode(data):
    if len(data)!=104:raise ValueError('incomplete registry transport')
    words=list(struct.unpack('<26I',data))
    if words!=expected_words():raise ValueError(f'registry status/output/handle lifecycle, caller stack or error observations differ: {words}')
    return words

def frozen():
    path=ROOT/'pe/reference/pe32-registry-open-windows-v1.json'
    if digest(path)!='980f920609e04f17c4d05d414083b7372e16dd7e42f053b9fbbd6bfd6efe42bc':raise ValueError('changed frozen native registry observation')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows owned HKCU registry observer' or r['status'] or r['stderr_hex'] or decode(bytes.fromhex(r['stdout_hex']))!=r['words']:raise ValueError('incomplete native registry observation')
    return r

def host(d):
    from observe_pe32_activation_startup import runner
    identity=verify(d);probe=runner(d);reports=[];path=d/'registry-open-host.json'
    reasons={1:b'admitted read access in the configured 32-bit view',2:b'registry result handle outside aligned writable guest storage',3:b'registry lookup outside authoritative configured scopes',4:b'RegCloseKey requires an owned live key handle'}
    for mode in [0,1,2,3,4,5]:
        name=FILES[mode] if mode<5 else FILES[0];command=[str(probe),str((d/name).resolve()),'--strict','--file-origins']
        if mode!=5:command+=['--registry-catalog',str(SOURCE/'registry-open.xml')]
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row)
        def save():path.write_text(json.dumps({'backend':'Rosetta scoped file-backed registry','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode:raise ValueError('host registry observer failed; raw result retained')
            row['words']=decode(r.stdout);save()
            if row['words']!=frozen()['words']:raise ValueError('host registry differs from frozen identical-PE native observation')
        else:
            reason=reasons[mode] if mode<5 else b'registry API requires an explicit scoped backing catalog'
            if r.returncode!=78 or r.stdout or reason not in r.stderr:raise ValueError('registry exclusion did not stop for its actual reason')
    print('Real registry key tokens, relative/case lookup, known absence and close pass; five exclusions stop')

def native(d):
    import winreg
    identity=verify(d);access=winreg.KEY_READ|winreg.KEY_WOW64_32KEY
    try:
        previous=winreg.OpenKey(winreg.HKEY_CURRENT_USER,SCOPE,0,access)
    except FileNotFoundError:pass
    else:
        previous.Close();raise ValueError('owned native registry fixture already exists; refusing to overwrite it')
    parent=r'Software\AlkyholOwnedDaoStartup';created_parent=False
    try:
        key=winreg.OpenKey(winreg.HKEY_CURRENT_USER,parent,0,access);key.Close()
    except FileNotFoundError:created_parent=True
    try:
        with winreg.CreateKeyEx(winreg.HKEY_CURRENT_USER,SCOPE,0,winreg.KEY_ALL_ACCESS|winreg.KEY_WOW64_32KEY) as base:
            with winreg.CreateKeyEx(base,'Present'):pass
        r=subprocess.run([str((d/FILES[0]).resolve())],capture_output=True,timeout=20)
        report={'backend':'native Windows owned HKCU registry observer','inputs':identity,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};path=d/'native-registry-open.json';path.write_text(json.dumps(report,indent=2)+'\n')
        if r.returncode or r.stderr:raise ValueError('native registry observer failed; raw result retained')
        report['words']=decode(r.stdout);path.write_text(json.dumps(report,indent=2)+'\n');print('Observed real native open, relative/case lookup, missing-key output and close semantics')
        host_path=d/'registry-open-host.json'
        if not host_path.exists():raise ValueError('missing peer host registry observation')
        h=json.loads(host_path.read_text());positive=h['reports'][0]
        if h['inputs']!=identity or positive['status'] or decode(bytes.fromhex(positive['stdout_hex']))!=positive['words'] or positive['words']!=report['words']:raise ValueError('native/host registry observations or source/PE identities differ')
    finally:
        for suffix in ['\\Present','']:
            try:winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,SCOPE+suffix,winreg.KEY_WOW64_32KEY)
            except FileNotFoundError:pass
        if created_parent:
            try:winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,parent,winreg.KEY_WOW64_32KEY)
            except OSError:pass
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(native if a.native else host if a.host else build)(a.output.resolve())
