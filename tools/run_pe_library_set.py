#!/usr/bin/env python3
"""Execute unchanged named/ordinal consumers sharing one loader-owned dependency."""
import argparse
import contextlib
import ctypes
import hashlib
import io
import json
from pathlib import Path
import platform
import shutil
import subprocess
import sys
from build_pe_fixture import ROOT
from build_pe_library_set import ORDER, expected_identity
from run_pe_dependent import native_child as named_native, EXPECTED_CALLS, EXPECTED_TRACE
from run_pe_ordinal import native as ordinal_native, EXPECTED as EXPECTED_ORDINAL

REFERENCE = ROOT / 'pe/reference/library-set-windows-v1.json'
EXPECTED = {'dependent': EXPECTED_CALLS, 'ordinal': EXPECTED_ORDINAL, 'shared_leaf_targets': True}


def native_child(directory):
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.LoadLibraryExW.argtypes, kernel.LoadLibraryExW.restype = [ctypes.c_wchar_p, ctypes.c_void_p, ctypes.c_uint32], ctypes.c_void_p
    kernel.GetProcAddress.argtypes, kernel.GetProcAddress.restype = [ctypes.c_void_p, ctypes.c_char_p], ctypes.c_void_p
    kernel.FreeLibrary.argtypes, kernel.FreeLibrary.restype = [ctypes.c_void_p], ctypes.c_int
    handles = []
    try:
        for name in ORDER[:2]:
            handle = kernel.LoadLibraryExW(str(directory / name), None, 0x1100)
            if not handle:
                raise ctypes.WinError(ctypes.get_last_error())
            handles.append(handle)
        # Reuse the already frozen independent observers, against cached native
        # module instances kept alive by the two outer handles. C trace remains
        # actual pipe output; only Python's JSON diagnostic is captured here.
        captured = io.StringIO()
        with contextlib.redirect_stderr(captured):
            named_native(directory)
        dependent = json.loads(captured.getvalue())
        ordinal = ordinal_native(directory)
        a = kernel.GetProcAddress(handles[0], b'Through')
        b = kernel.GetProcAddress(handles[1], b'ForwardDirect')
        if not a or not b:
            raise ctypes.WinError(ctypes.get_last_error())
        result = {'common': {'dependent': dependent['calls'], 'ordinal': ordinal['common'],
                             'shared_leaf_targets': a == b}, 'relocated': None}
    finally:
        for handle in reversed(handles):
            if not kernel.FreeLibrary(handle):
                raise ctypes.WinError(ctypes.get_last_error())
    print(json.dumps(result), file=sys.stderr)


def check_execution(run, windows=False, rebase=False):
    if run.returncode != 0 or run.stdout != EXPECTED_TRACE:
        raise ValueError(f'closed DLL bundle failed/lifecycle differs: {run.returncode}, {run.stdout.hex()}, {run.stderr!r}')
    value = json.loads(run.stderr.decode('utf-8'))
    if set(value) != {'common', 'relocated'} or value['common'] != EXPECTED or value['common']['shared_leaf_targets'] is not True:
        raise ValueError('shared DLL calls or actual target identity differs')
    for body, keys in [('dependent', ['through_matches_dependency','error_matches_dependency']),
                       ('ordinal', ['through_matches_ordinal','data_matches_ordinal','direct_matches_ordinal'])]:
        if any(value['common'][body][key] is not True for key in keys):
            raise ValueError('missing actual resolved target identity')
    flags = value['relocated']
    if windows:
        if flags is not None:
            raise ValueError('invented native forced rebase')
    elif not isinstance(flags, list) or len(flags) != 3 or any(type(v) is not bool for v in flags) or (rebase and flags != [True]*3):
        raise ValueError('missing actual three-DLL relocation observations')
    return {'results': value['common'], 'trace_hex': run.stdout.hex()}, flags


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--directory', type=Path, required=True)
    p.add_argument('--output', type=Path)
    p.add_argument('--native-child', action='store_true', help=argparse.SUPPRESS)
    args = p.parse_args()
    directory = args.directory.resolve()
    manifest = json.loads((directory / 'library-set-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version'] != 1 or manifest['provided_modules'] != ORDER:
        raise ValueError('missing closed bundle manifest/order')
    for field, identity in expected_identity().items():
        if manifest[field] != identity:
            raise ValueError('bundle changed existing frozen DLL/source identities')
    if REFERENCE.exists():
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
        for field in ['binaries', 'sources', 'provided_modules']:
            if manifest[field] != reference[field]:
                raise ValueError('changed frozen shared DLL fixture')
    for name, digest in manifest['binaries'].items():
        if hashlib.sha256((directory / name).read_bytes()).hexdigest() != digest:
            raise ValueError('changed bundle binary')
    for name, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest:
            raise ValueError('changed bundle source')
    windows = platform.system() == 'Windows'
    if args.native_child:
        if not windows:
            raise RuntimeError('native child requires Windows')
        native_child(directory)
        return
    if args.output is None:
        raise ValueError('output path is required')
    if windows:
        command = [sys.executable, str(Path(__file__).resolve()), '--native-child', '--directory', str(directory)]
        backend = 'native Windows shared DLL bundle'
    elif platform.system() == 'Darwin':
        cargo = shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
        subprocess.run([cargo, 'build', '--manifest-path', str(ROOT / 'pe/Cargo.toml'), '--target', 'x86_64-apple-darwin',
                        '--locked', '--offline', '--bin', 'probe_library_set'], check=True)
        command = [str(ROOT / 'pe/target/x86_64-apple-darwin/debug/probe_library_set'), str(directory)]
        backend = 'Alkyhol closed DLL bundle'
    else:
        raise RuntimeError('requires Windows or macOS/Rosetta')
    common, flags = check_execution(subprocess.run(command, capture_output=True, timeout=30), windows)
    rebased, rebased_flags = None, None
    if not windows:
        rebased, rebased_flags = check_execution(subprocess.run(command+['--rebase'], capture_output=True, timeout=30), rebase=True)
        if rebased != common:
            raise ValueError('rebased shared bundle results differ')
    report = {'schema_version': 1, 'backend': backend,
              **{k: manifest[k] for k in ['binaries','sources','provided_modules']}, 'common': common,
              'relocated': flags, 'rebased_common': rebased, 'rebased_relocated': rebased_flags,
              'revision': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True)
    args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Passed actual closed DLL bundle: '+backend)


if __name__ == '__main__':
    main()
