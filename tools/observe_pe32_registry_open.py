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
    return value

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    for kind in ['registry-open','registry-open-kernel32']:
        lib=d/(kind+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(kind+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('RegOpenKeyExW',20),('RegCloseKey',4)]]
    for mode,name in enumerate(FILES):
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
    finally:
        for suffix in ['\\Present','']:
            try:winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,SCOPE+suffix,winreg.KEY_WOW64_32KEY)
            except FileNotFoundError:pass
        if created_parent:
            try:winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER,parent,winreg.KEY_WOW64_32KEY)
            except OSError:pass
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else build)(a.output.resolve())
