#!/usr/bin/env python3
"""Test the explicit backed system-directory role reached by DAO's CRT."""
import argparse
import json
import os
from pathlib import Path
import shutil
import struct
import subprocess
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest

SOURCES=['system-directory.c','system-directory.def']
FILES=[f'system-directory{i}.exe' for i in range(5)]
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetSystemDirectoryW',8)]
BOOT=struct.pack('<I',0x53443332)

def expected():
    seed=0xface1234
    return [0x53443332,4,1,seed,1,1]+[1,seed,1]*4

def decode(data,known_path=None):
    if len(data)<76 or list(struct.unpack('<18I',data[:72]))!=expected():
        raise ValueError('system-directory size/copy/error/guard contract differs')
    length=struct.unpack_from('<I',data,72)[0]
    if not 0<length<1024 or len(data)!=76+(length+1)*2:
        raise ValueError('system-directory UTF-16 record width differs')
    text=data[76:].decode('utf-16-le')
    if not text.endswith('\0') or '\0' in text[:-1]:raise ValueError('invalid directory terminator')
    text=text[:-1]
    if known_path is not None and text!=known_path:raise ValueError('directory does not identify its explicit backed profile role')
    return text

def identities(directory):
    return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},
            'files':{n:digest(directory/n) for n in FILES}}

def verify(directory):
    value=json.loads((directory/'system-directory-inputs.json').read_text())
    if value!=identities(directory):raise ValueError('system-directory source/PE identities changed')
    return value

def host(directory):
    directory.mkdir(parents=True,exist_ok=True)
    clang=shutil.which('clang')
    if not clang:raise RuntimeError('open Clang is required')
    lld=rust_tool('rust-lld');lib=directory/'system-directory.lib'
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'system-directory.def'),'/out:'+str(lib)],check=True)
    for mode in range(5):
        obj=directory/f'system-directory{mode}.obj'
        subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector',
                        '-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror',f'-DMODE={mode}',
                        '-c',str(SOURCE/'system-directory.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no',
                        '/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),
                        *[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(directory/f'system-directory{mode}.exe')],check=True)
    (directory/'system-directory-inputs.json').write_text(json.dumps(identities(directory),indent=2)+'\n')
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo');rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
    target='x86_64-apple-darwin';build=ROOT/'pe/target'/target/'debug';probe=directory/'system-directory-macos'
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target',target,'--locked','--offline'],check=True)
    subprocess.run([rustc,'--edition=2024','--target',target,str(ROOT/'pe/probe_startup32.rs'),'--extern','alkyhol_pe='+str(build/'libalkyhol_pe.rlib'),
                    '-L','dependency='+str(build/'deps'),'-C','link-arg=-Wl,-pagezero_size,0x1000','-o',str(probe)],check=True)
    system=directory/'system profile λ🌸/Windows/System32';system.mkdir(parents=True,exist_ok=True)
    records=[]
    for mode in range(5):
        exe=directory/f'system-directory{mode}.exe';command=[str(probe),str(exe),'--file-origins']
        profile=exe if mode==3 else directory/'missing-system-directory' if mode==4 else system
        if mode!=1:command+=['--system-directory',str(profile)]
        result=subprocess.run(command,capture_output=True,timeout=15)
        row={'case':mode,'status':result.returncode,'stdout_hex':result.stdout.hex(),'stderr_hex':result.stderr.hex()}
        if mode==0:
            if result.returncode:raise ValueError('host system-directory query failed: '+repr(row))
            row['system_directory']=decode(result.stdout,'Z:'+str(system.resolve()).replace('/','\\'))
        else:
            reasons={1:b'requires an explicitly configured backed guest directory',2:b'system directory output outside checked writable',
                     3:b'system directory must identify an existing host directory',4:b'system directory:'}
            if result.returncode!=78 or result.stdout!=(BOOT if mode in [1,2] else b'') or reasons[mode] not in result.stderr:
                raise ValueError('system-directory restriction stopped for the wrong reason: '+repr(row))
        records.append(row)
    (directory/'system-directory-host.json').write_text(json.dumps({'backend':'Rosetta PE32 explicit backed system-directory profile',
        'inputs':verify(directory),'observations':records},indent=2)+'\n')
    print('Explicit backed directory copy/sizing passes; absent role, read-only output and invalid profile paths stop')

def native(directory):
    inputs=verify(directory)
    result=subprocess.run([str((directory/'system-directory0.exe').resolve())],capture_output=True,timeout=15)
    report={'backend':'native Windows PE32 system-directory observer','inputs':inputs,'status':result.returncode,
            'stdout_hex':result.stdout.hex(),'stderr_hex':result.stderr.hex()}
    path=directory/'native-system-directory.json';path.write_text(json.dumps(report,indent=2)+'\n')
    if result.returncode or result.stderr:raise ValueError('native system-directory observer failed')
    text=decode(result.stdout);root=str(Path(os.environ['SystemRoot']).resolve())
    if not Path(text).is_dir() or not text.casefold().startswith(root.casefold()+'\\'):
        raise ValueError('native reported directory is not a real directory under its Windows root')
    report.update(system_directory=text,windows_root=root)
    path.write_text(json.dumps(report,indent=2)+'\n')
    print('18 normalized words agree; actual Windows directory recorded separately from the host profile path')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests')
    p.add_argument('--native',action='store_true');a=p.parse_args();(native if a.native else host)(a.output.resolve())
