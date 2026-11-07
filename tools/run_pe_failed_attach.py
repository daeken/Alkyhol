#!/usr/bin/env python3
"""Observe real DLL attach rejection/cleanup and IAT calls, with raw guest trace."""
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
from build_pe_fixture import ROOT
from build_pe_failed_attach import SOURCES

EXPECTED = struct.pack('<8I', 1, 1, 1, 0xa17ac0de, 0, 2, 1, 0xa17ac0de)


def native_child(dll):
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.LoadLibraryW.argtypes, kernel.LoadLibraryW.restype = [ctypes.c_wchar_p], ctypes.c_void_p
    kernel.FreeLibrary.argtypes, kernel.FreeLibrary.restype = [ctypes.c_void_p], ctypes.c_int
    module = kernel.LoadLibraryW(str(dll))
    error = ctypes.get_last_error()
    if module:
        kernel.FreeLibrary(module)
        raise RuntimeError('intentionally failed DLL loaded on Windows')
    print(json.dumps({'loaded': False, 'attach_rejected': True, 'windows_last_error': error}), file=sys.stderr)


def check_execution(run, windows):
    if run.returncode != 0:
        raise ValueError(f'failed-attach observer crashed/failed: {run.returncode}, {run.stderr!r}')
    if run.stdout != EXPECTED:
        raise ValueError(f'actual attach/detach trace differs: {run.stdout.hex()}')
    result = json.loads(run.stderr.decode('utf-8'))
    key = 'windows_last_error' if windows else 'host_last_error'
    if set(result) != {'loaded', 'attach_rejected', key} or result['loaded'] is not False or result['attach_rejected'] is not True:
        raise ValueError('DLL rejection was not observed')
    if type(result[key]) is not int or not 0 <= result[key] <= 0xffffffff:
        raise ValueError('invalid observed last-error DWORD')
    if not windows and result[key] != 0x1234:
        raise ValueError('guest IAT SetLastError sentinel changed')
    return {'loaded': False, 'trace_hex': run.stdout.hex()}, {key: result[key]}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--dll', type=Path, required=True)
    p.add_argument('--output', type=Path)
    p.add_argument('--native-child', action='store_true', help=argparse.SUPPRESS)
    args = p.parse_args()
    dll = args.dll.resolve()
    manifest = json.loads((dll.parent / 'failed-attach-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version'] != 1 or set(manifest['sources']) != set(SOURCES):
        raise ValueError('missing failed-attach source manifest')
    if hashlib.sha256(dll.read_bytes()).hexdigest() != manifest['dll_sha256']:
        raise ValueError('failed-attach DLL identity changed')
    for path, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT / path).read_bytes()).hexdigest() != digest:
            raise ValueError('failed-attach source identity changed')
    windows = platform.system() == 'Windows'
    if args.native_child:
        if not windows:
            raise RuntimeError('native child requires Windows')
        native_child(dll)
        return
    if args.output is None:
        raise ValueError('output path is required')
    if windows:
        executable = [sys.executable, str(Path(__file__).resolve()), '--native-child', '--dll', str(dll)]
        backend = 'native Windows DLL attach rejection'
    elif platform.system() == 'Darwin':
        cargo = shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
        subprocess.run([cargo, 'build', '--manifest-path', str(ROOT / 'pe/Cargo.toml'), '--target', 'x86_64-apple-darwin',
                        '--locked', '--offline', '--bin', 'probe_failed_library'], check=True)
        executable = [str(ROOT / 'pe/target/x86_64-apple-darwin/debug/probe_failed_library'), str(dll)]
        backend = 'Alkyhol DLL attach rejection'
    else:
        raise RuntimeError('requires Windows or macOS')
    normal, only = check_execution(subprocess.run(executable, capture_output=True, timeout=30), windows)
    rebased = None
    if not windows:
        rebased, rebase_only = check_execution(subprocess.run(executable + ['--rebase'], capture_output=True, timeout=30), False)
        if rebased != normal or rebase_only != only:
            raise ValueError('rebased DLL failure behavior differs')
    report = {'schema_version': 1, 'backend': backend, 'dll_sha256': manifest['dll_sha256'], 'sources': manifest['sources'],
              'common': normal, 'rebased_common': rebased, 'backend_only': only,
              'revision': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Passed actual DLL attach rejection and ordered cleanup trace: ' + backend)


if __name__ == '__main__':
    main()
