#!/usr/bin/env python3
"""Observe static DLL records through real normal/failed dynamic initialization."""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import platform
import shutil
import struct
import subprocess
import sys
import tempfile
from build_pe_fixture import ROOT
from build_pe_registered_dll import SOURCES

REFERENCE=ROOT/'pe/reference/registered-dll-windows-v1.json'
NATIVE,HOST='native Windows static DLL lifecycle','Alkyhol static DLL lifecycle / Rosetta'
TRACE=struct.pack('<20I',0x44535441,1,1,1,1,2,1,1,0,1,0x44535441,0,2,1,1,2,1,1,0,1)
EXPECTED=[{'case':case,'loaded':not reject,'probe':None if reject else 0x42a11c0d,
           'alias':None if reject else True,'trace_hex':TRACE.hex()}
          for case,reject in [('accepted',False),('rejected',True)]]


def native_child(dll,reject):
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    kernel.LoadLibraryW.argtypes,kernel.LoadLibraryW.restype=[ctypes.c_wchar_p],ctypes.c_void_p
    kernel.GetProcAddress.argtypes,kernel.GetProcAddress.restype=[ctypes.c_void_p,ctypes.c_void_p],ctypes.c_void_p
    kernel.FreeLibrary.argtypes,kernel.FreeLibrary.restype=[ctypes.c_void_p],ctypes.c_int
    module=kernel.LoadLibraryW(str(dll))
    if reject:
        if module:
            kernel.FreeLibrary(module)
            raise RuntimeError('rejected DLL unexpectedly attached')
        result={'loaded':False,'probe':None,'alias':None}
    else:
        if not module:raise ctypes.WinError(ctypes.get_last_error())
        try:
            name=ctypes.create_string_buffer(b'Probe')
            address=kernel.GetProcAddress(module,ctypes.cast(name,ctypes.c_void_p))
            ordinal=kernel.GetProcAddress(module,ctypes.c_void_p(17))
            if not address:raise RuntimeError('missing native Probe')
            result={'loaded':True,'probe':ctypes.WINFUNCTYPE(ctypes.c_uint32)(address)(),'alias':address==ordinal}
        finally:
            if not kernel.FreeLibrary(module):raise ctypes.WinError(ctypes.get_last_error())
    print(json.dumps(result),file=sys.stderr)


def decode(run,case):
    if run.returncode!=0 or run.stdout!=TRACE:
        raise ValueError(f'DLL lifecycle failed or raw trace differs: {run.returncode}, {run.stdout.hex()}, {run.stderr!r}')
    result=json.loads(run.stderr.decode('utf-8'))
    if set(result)!={'loaded','probe','alias'}:raise ValueError('unexpected DLL observer fields')
    value={'case':case,**result,'trace_hex':run.stdout.hex()}
    expected=next(e for e in EXPECTED if e['case']==case)
    if value!=expected or type(value['loaded']) is not bool:
        raise ValueError('actual DLL attach/export/rejection observations differ')
    if case=='accepted' and (type(value['probe']) is not int or value['alias'] is not True):
        raise ValueError('missing actual exported Win64 call/name-ordinal identity')
    return value


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--directory',type=Path)
    p.add_argument('--output',type=Path)
    p.add_argument('--native-child',type=Path,help=argparse.SUPPRESS)
    p.add_argument('--expect-reject',action='store_true',help=argparse.SUPPRESS)
    args=p.parse_args()
    windows=platform.system()=='Windows'
    if args.native_child:
        if not windows:raise RuntimeError('native child requires Windows')
        native_child(args.native_child.resolve(),args.expect_reject);return
    if args.directory is None or args.output is None:raise ValueError('directory/output are required')
    directory=args.directory.resolve()
    manifest=json.loads((directory/'registered-dll-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES) or set(manifest['binaries'])!={'accepted','rejected'}:
        raise ValueError('missing exact DLL/source manifest')
    snapshots={}
    for case,digest in manifest['binaries'].items():
        snapshots[case]=(directory/(case+'.dll')).read_bytes()
        if hashlib.sha256(snapshots[case]).hexdigest()!=digest:raise ValueError('DLL identity changed')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('DLL source changed')
    reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['binaries','sources']:
        if manifest[field]!=reference[field]:raise ValueError('changed frozen registered DLL fixture')
    if windows:backend=NATIVE
    elif platform.system()=='Darwin':
        backend=HOST
        cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
        subprocess.run([cargo,'build','--manifest-path',str(ROOT/'pe/Cargo.toml'),'--target','x86_64-apple-darwin',
                        '--locked','--offline','--bin','probe_registered_library'],check=True)
        executable=ROOT/'pe/target/x86_64-apple-darwin/debug/probe_registered_library'
    else:raise RuntimeError('requires Windows or macOS/Rosetta')
    observations=[]
    with tempfile.TemporaryDirectory(prefix='alkyhol-static-dll-') as temp:
        for rebase in [False] if windows else [False,True]:
            values=[]
            for case in ['accepted','rejected']:
                dll=Path(temp)/(case+'.dll');dll.write_bytes(snapshots[case])
                command=([sys.executable,str(Path(__file__).resolve()),'--native-child',str(dll)] if windows else [str(executable),str(dll)])
                if case=='rejected':command.append('--expect-reject')
                if rebase:command.append('--rebase')
                values.append(decode(subprocess.run(command,cwd=temp,capture_output=True,timeout=30),case))
            observations.append(values)
    if not windows and observations[0]!=observations[1]:raise ValueError('forced-rebase lifecycle differs')
    report={'schema_version':1,'backend':backend,'binaries':manifest['binaries'],'sources':manifest['sources'],
            'normal':observations[0],'rebased':None if windows else observations[1],
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Passed actual static DLL attach/export/detach and rejected-attach cleanup: '+backend)


if __name__=='__main__':main()
