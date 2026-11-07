#!/usr/bin/env python3
"""Fetch pinned open headers/toolchain source without modifying the sample checkout."""
import json
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]
MANIFEST=ROOT/'cpp/upstream.json'

def ensure(name, clone=True):
    record=json.loads(MANIFEST.read_text())['repositories'][name]
    path=ROOT/'local/upstream'/name
    if not path.exists():
        if not clone: raise ValueError('missing dependency: '+name)
        subprocess.run(['git','clone','--depth','1','--filter=blob:none','--no-checkout',record['url'],str(path)],check=True)
        subprocess.run(['git','-C',str(path),'fetch','--depth','1','origin',record['revision']],check=True)
        if name=='llvm-project':
            subprocess.run(['git','-C',str(path),'sparse-checkout','set','clang/lib/Sema','clang/lib/CodeGen','clang/tools/driver'],check=True)
        subprocess.run(['git','-C',str(path),'checkout','--detach',record['revision']],check=True)
    actual=subprocess.check_output(['git','-C',str(path),'rev-parse','HEAD'],text=True).strip()
    if actual!=record['revision']:raise ValueError('dependency revision differs: '+name)
    if subprocess.run(['git','-C',str(path),'diff','--quiet','HEAD']).returncode:
        raise ValueError('dependency has tracked edits: '+name)
    return path

if __name__=='__main__':
    for name in json.loads(MANIFEST.read_text())['repositories']:print(name,ensure(name))
