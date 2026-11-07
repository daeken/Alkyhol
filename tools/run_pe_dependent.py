#!/usr/bin/env python3
"""Observe genuine DLL-to-DLL IAT calls and lifecycle, retaining exact raw trace."""
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
from build_pe_dependent import SOURCES

REFERENCE = ROOT / 'pe/reference/dependent-windows-v1.json'
DEPENDENCY_REFERENCE = ROOT / 'pe/reference/exports-windows-v1.json'
DIRECT_INIT = (3 * 33) ^ 5 ^ 0x31415926 ^ 0x13579bdf
DIRECT_CALL = (7 * 33) ^ 11 ^ 0x31415926 ^ 0x13579bdf
EXPECTED_CALLS = {'evaluation': DIRECT_CALL ^ 0x31415926 ^ DIRECT_INIT ^ 1,
                  'through_call': DIRECT_CALL, 'read_error': 0x3456,
                  'double_forward_error': 0x4567, 'through_matches_dependency': True,
                  'error_matches_dependency': True}
EXPECTED_TRACE = struct.pack('<12I', 1, 1, 1, DIRECT_INIT, 0x31415926, 0x13579bdf,
                             0, 2, 1, DIRECT_INIT, 0x31415926, 0x13579bdf)


def native_child(directory):
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.LoadLibraryExW.argtypes, kernel.LoadLibraryExW.restype = [ctypes.c_wchar_p, ctypes.c_void_p, ctypes.c_uint32], ctypes.c_void_p
    kernel.GetModuleHandleW.argtypes, kernel.GetModuleHandleW.restype = [ctypes.c_wchar_p], ctypes.c_void_p
    kernel.GetProcAddress.argtypes, kernel.GetProcAddress.restype = [ctypes.c_void_p, ctypes.c_char_p], ctypes.c_void_p
    kernel.FreeLibrary.argtypes, kernel.FreeLibrary.restype = [ctypes.c_void_p], ctypes.c_int
    # DLL_LOAD_DIR | DEFAULT_DIRS makes the explicit fixture directory available
    # for the actual Windows dependency search, without changing global paths.
    module = kernel.LoadLibraryExW(str(directory / 'alkyhol_dependent.dll'), None, 0x1100)
    if not module:
        raise ctypes.WinError(ctypes.get_last_error())
    def address(mod, name):
        result = kernel.GetProcAddress(mod, name.encode('ascii'))
        if not result:
            raise ctypes.WinError(ctypes.get_last_error())
        return result
    try:
        dependency = kernel.GetModuleHandleW('alkyhol_exports.dll')
        if not dependency:
            raise RuntimeError('Windows did not load the real dependency')
        binary = ctypes.WINFUNCTYPE(ctypes.c_uint32, ctypes.c_uint32, ctypes.c_uint32, use_last_error=True)
        unary = ctypes.WINFUNCTYPE(ctypes.c_uint32, use_last_error=True)
        through, error = address(module, 'Through'), address(module, 'Error')
        evaluation = binary(address(module, 'Evaluate'))(7, 11)
        through_call = binary(through)(7, 11)
        read_error_fn = unary(address(module, 'ReadError'))
        error_fn = unary(error)
        ctypes.set_last_error(0x3456)
        read_error = read_error_fn()
        ctypes.set_last_error(0x4567)
        double_forward_error = error_fn()
        calls = {'evaluation': evaluation, 'through_call': through_call, 'read_error': read_error,
                 'double_forward_error': double_forward_error,
                 'through_matches_dependency': through == address(dependency, 'Direct'),
                 'error_matches_dependency': error == address(dependency, 'Forwarded')}
    finally:
        if not kernel.FreeLibrary(module):
            raise ctypes.WinError(ctypes.get_last_error())
    print(json.dumps({'calls': calls, 'relocated': None}), file=sys.stderr)


def check_execution(run, windows, rebase=False):
    if run.returncode != 0 or run.stdout != EXPECTED_TRACE:
        raise ValueError(f'dependent DLL observer failed/trace differs: {run.returncode}, {run.stdout.hex()}, {run.stderr!r}')
    result = json.loads(run.stderr.decode('utf-8'))
    if set(result) != {'calls', 'relocated'} or result['calls'] != EXPECTED_CALLS:
        raise ValueError('actual cross-DLL calls/forwarders differ')
    for key in ['through_matches_dependency', 'error_matches_dependency']:
        if result['calls'][key] is not True:
            raise ValueError('missing actual forwarded target identity')
    relocated = result['relocated']
    if windows:
        if relocated is not None:
            raise ValueError('invented native rebase observation')
    elif not isinstance(relocated, list) or len(relocated) != 2 or any(type(v) is not bool for v in relocated) or (rebase and relocated != [True, True]):
        raise ValueError('missing actual DLL relocation observations')
    return {'calls': result['calls'], 'trace_hex': run.stdout.hex()}, relocated


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--directory', type=Path, required=True)
    p.add_argument('--output', type=Path)
    p.add_argument('--native-child', action='store_true', help=argparse.SUPPRESS)
    args = p.parse_args()
    directory = args.directory.resolve()
    manifest = json.loads((directory / 'dependent-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version'] != 1 or set(manifest['sources']) != set(SOURCES) or set(manifest['binaries']) != {'alkyhol_exports.dll', 'alkyhol_dependent.dll'}:
        raise ValueError('missing exact dependent fixture manifest')
    dependency_reference = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
    if manifest['binaries']['alkyhol_exports.dll'] != dependency_reference['dll_sha256']:
        raise ValueError('changed frozen dependency DLL')
    if any(manifest['sources'][s] != digest for s, digest in dependency_reference['sources'].items()):
        raise ValueError('changed frozen dependency source')
    if REFERENCE.exists():
        reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
        for field in ['binaries', 'sources']:
            if manifest[field] != reference[field]:
                raise ValueError(f'changed frozen dependent fixture: {field}')
    for name, digest in manifest['binaries'].items():
        if hashlib.sha256((directory / name).read_bytes()).hexdigest() != digest:
            raise ValueError('dependent DLL binary identity changed')
    for name, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest:
            raise ValueError('dependent DLL source identity changed')
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
        backend = 'native Windows dependent DLL LoadLibraryEx'
    elif platform.system() == 'Darwin':
        cargo = shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
        subprocess.run([cargo, 'build', '--manifest-path', str(ROOT / 'pe/Cargo.toml'), '--target', 'x86_64-apple-darwin',
                        '--locked', '--offline', '--bin', 'probe_dependent'], check=True)
        command = [str(ROOT / 'pe/target/x86_64-apple-darwin/debug/probe_dependent'), str(directory)]
        backend = 'Alkyhol explicit dependent DLLs'
    else:
        raise RuntimeError('requires native Windows or macOS/Rosetta')
    common, relocated = check_execution(subprocess.run(command, capture_output=True, timeout=30), windows)
    rebased, rebased_relocated = None, None
    if not windows:
        rebased, rebased_relocated = check_execution(subprocess.run(command + ['--rebase'], capture_output=True, timeout=30), False, True)
        if common != rebased:
            raise ValueError('rebased dependent DLL behavior differs')
    report = {'schema_version': 1, 'backend': backend, 'binaries': manifest['binaries'], 'sources': manifest['sources'],
              'common': common, 'relocated': relocated, 'rebased_common': rebased, 'rebased_relocated': rebased_relocated,
              'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=ROOT, text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Passed actual DLL-to-DLL calls and ordered lifecycle: ' + backend)


if __name__ == '__main__':
    main()
