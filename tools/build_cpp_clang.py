#!/usr/bin/env python3
"""Rebuild small pinned Clang components for MiniEngine's MSVC extensions."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
if __package__:
    from .fetch_cpp_deps import ROOT,ensure
else:
    from fetch_cpp_deps import ROOT,ensure
VERSION='21.1.6'

def build(prefix=Path('/opt/homebrew/opt/llvm'),jobs=3):
    prefix=prefix.resolve()
    actual=subprocess.check_output([str(prefix/'bin/llvm-config'),'--version'],text=True).strip()
    if actual!=VERSION:raise ValueError(f'Clang compatibility components require LLVM {VERSION}, found {actual}')
    source=ensure('llvm-project');out=ROOT/'local/clang-compat/build';out.mkdir(parents=True,exist_ok=True)
    executable=ROOT/'local/clang-compat/bin/clang++';executable.parent.mkdir(parents=True,exist_ok=True)
    patches=sorted((ROOT/'cpp/bridge').glob('clang-msvc-*.patch'))
    inputs={'llvm_prefix':str(prefix),'llvm_version':VERSION,
            'patches':{p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in patches},
            'main':hashlib.sha256((ROOT/'cpp/bridge/clang_main.cpp').read_bytes()).hexdigest()}
    record=out/'compiler-build.json'
    if record.exists() and executable.exists():
        old=json.loads(record.read_text())
        if old['inputs']==inputs and old['executable_sha256']==hashlib.sha256(executable.read_bytes()).hexdigest():return executable
    for name in ['SemaCast.cpp','SemaInit.cpp','SemaOverload.cpp']:
        (out/name).write_bytes(subprocess.check_output(['git','-C',str(source),'show','HEAD:clang/lib/Sema/'+name]))
    for patch in patches:subprocess.run(['patch','--batch','-p4','-i',str(patch)],cwd=out,check=True)
    work=[(out/name,out/(name+'.o')) for name in ['SemaCast.cpp','SemaInit.cpp','SemaOverload.cpp']]
    work += [(p,out/('Builtin-'+p.name+'.o')) for p in sorted((source/'clang/lib/CodeGen/TargetBuiltins').glob('*.cpp'))]
    work += [(source/'clang/tools/driver'/name,out/(name+'.o')) for name in ['driver.cpp','cc1_main.cpp','cc1as_main.cpp','cc1gen_reproducer_main.cpp']]
    work += [(ROOT/'cpp/bridge/clang_main.cpp',out/'main.cpp.o')]
    def compile(pair):
        src,obj=pair
        command=[str(prefix/'bin/clang++'),'-std=c++17','-O1','-DNDEBUG','-fno-exceptions','-I'+str(prefix/'include'),
                 '-I'+str(source/'clang/lib/Sema'),'-I'+str(source/'clang/lib/CodeGen'),'-c',str(src),'-o',str(obj)]
        result=subprocess.run(command,capture_output=True,text=True)
        obj.with_suffix('.log').write_text(result.stdout+result.stderr)
        if result.returncode:raise RuntimeError(f'Clang component failed: {src}\n{result.stderr}')
    with ThreadPoolExecutor(max_workers=jobs) as pool:list(pool.map(compile,work))
    archive=out/'libclangSema.a';shutil.copy2(prefix/'lib/libclangSema.a',archive)
    sema_objects=[out/(name+'.o') for name in ['SemaCast.cpp','SemaInit.cpp','SemaOverload.cpp']]
    subprocess.run([str(prefix/'bin/llvm-ar'),'r',str(archive),*map(str,sema_objects)],check=True)
    libraries=[str(archive) if p.name=='libclangSema.a' else str(p) for p in sorted((prefix/'lib').glob('libclang*.a'))]
    objects=[str(obj) for _,obj in work if obj not in sema_objects]
    subprocess.run([str(prefix/'bin/clang++'),*objects,*libraries,'-L'+str(prefix/'lib'),'-lLLVM',
                    '-Wl,-rpath,'+str(prefix/'lib'),'-o',str(executable)],check=True)
    for relative,target in [('lib/libLTO.dylib',prefix/'lib/libLTO.dylib'),('lib/clang',prefix/'lib/clang'),('include/c++/v1',prefix/'include/c++/v1')]:
        dest=executable.parent.parent/relative;dest.parent.mkdir(parents=True,exist_ok=True)
        if dest.is_symlink():dest.unlink()
        dest.symlink_to(target)
    record.write_text(json.dumps({'inputs':inputs,'executable_sha256':hashlib.sha256(executable.read_bytes()).hexdigest()},indent=2)+'\n')
    return executable

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--llvm-prefix',type=Path,default=Path('/opt/homebrew/opt/llvm'));p.add_argument('--jobs',type=int,default=3)
    args=p.parse_args();print(build(args.llvm_prefix,args.jobs))
