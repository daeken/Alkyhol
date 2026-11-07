#!/usr/bin/env python3
"""Execute an identical DLL's ordinal code/data IAT and export forwarders."""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
from build_pe_fixture import ROOT
from build_pe_ordinal import SOURCES, IMPORTS
from pe_imports import PE

REFERENCE = ROOT / 'pe/reference/ordinal-windows-v1.json'
DEPENDENCY_REFERENCE = ROOT / 'pe/reference/exports-windows-v1.json'
EXPECTED = {'evaluation': (7 + 0x12345678) ^ ((7 * 33) ^ 11 ^ 0x31415926 ^ 0x13579bdf) ^ 0x31415926 ^ (9 + 0x12345678),
            'initial': 9 + 0x12345678, 'through_call': 9 + 0x12345678, 'forward_data': 0x31415926,
            'through_matches_ordinal': True, 'data_matches_ordinal': True, 'direct_matches_ordinal': True}
POINTER_FLAGS = ['through_matches_ordinal', 'data_matches_ordinal', 'direct_matches_ordinal']


def native(directory):
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.LoadLibraryExW.argtypes, kernel.LoadLibraryExW.restype = [ctypes.c_wchar_p, ctypes.c_void_p, ctypes.c_uint32], ctypes.c_void_p
    kernel.GetModuleHandleW.argtypes, kernel.GetModuleHandleW.restype = [ctypes.c_wchar_p], ctypes.c_void_p
    kernel.GetProcAddress.argtypes, kernel.GetProcAddress.restype = [ctypes.c_void_p, ctypes.c_void_p], ctypes.c_void_p
    kernel.FreeLibrary.argtypes, kernel.FreeLibrary.restype = [ctypes.c_void_p], ctypes.c_int
    module = kernel.LoadLibraryExW(str(directory / 'alkyhol_ordinal.dll'), None, 0x1100)
    if not module:
        raise ctypes.WinError(ctypes.get_last_error())
    def address(mod, symbol):
        if isinstance(symbol, int):
            pointer = ctypes.c_void_p(symbol)
        else:
            buffer = ctypes.create_string_buffer(symbol.encode('ascii'))
            pointer = ctypes.cast(buffer, ctypes.c_void_p)
        value = kernel.GetProcAddress(mod, pointer)
        if not value:
            raise ctypes.WinError(ctypes.get_last_error())
        return value
    try:
        leaf = kernel.GetModuleHandleW('alkyhol_exports.dll')
        if not leaf:
            raise RuntimeError('actual ordinal dependency was not loaded')
        through, data, direct = (address(module, n) for n in ['Through', 'ForwardData', 'ForwardDirect'])
        binary = ctypes.WINFUNCTYPE(ctypes.c_uint32, ctypes.c_uint32, ctypes.c_uint32)
        unary = ctypes.WINFUNCTYPE(ctypes.c_uint32, ctypes.c_uint32)
        return {'common': {'evaluation': binary(address(module, 'Evaluate'))(7,11),
                           'initial': ctypes.c_uint32.from_address(address(module, 'Initial')).value,
                           'through_call': unary(through)(9), 'forward_data': ctypes.c_uint32.from_address(data).value,
                           'through_matches_ordinal': through == address(leaf, 23),
                           'data_matches_ordinal': data == address(leaf, 21),
                           'direct_matches_ordinal': direct == address(leaf, 17)}, 'relocated': None}
    finally:
        if not kernel.FreeLibrary(module):
            raise ctypes.WinError(ctypes.get_last_error())


def check_result(value, windows=False, rebase=False):
    if set(value) != {'common', 'relocated'} or value['common'] != EXPECTED or any(value['common'][key] is not True for key in POINTER_FLAGS):
        raise ValueError('actual ordinal calls/data/forwarded identities differ')
    flags = value['relocated']
    if windows:
        if flags is not None:
            raise ValueError('invented native rebase observation')
    elif not isinstance(flags, list) or len(flags) != 2 or any(type(v) is not bool for v in flags) or (rebase and flags != [True, True]):
        raise ValueError('missing actual relocation of both DLLs')


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--directory', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    directory = args.directory.resolve()
    manifest = json.loads((directory / 'ordinal-fixture.json').read_text(encoding='utf-8'))
    if manifest['schema_version'] != 1 or set(manifest['sources']) != set(SOURCES) or set(manifest['binaries']) != {'alkyhol_exports.dll', 'alkyhol_ordinal.dll'} or manifest['imports'] != IMPORTS:
        raise ValueError('missing exact ordinal fixture manifest')
    old = json.loads(DEPENDENCY_REFERENCE.read_text(encoding='utf-8'))
    if manifest['binaries']['alkyhol_exports.dll'] != old['dll_sha256'] or any(manifest['sources'][s] != digest for s, digest in old['sources'].items()):
        raise ValueError('changed frozen dependency identity')
    reference = json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['binaries', 'sources', 'imports']:
        if manifest[field] != reference[field]:
            raise ValueError(f'changed frozen ordinal fixture: {field}')
    for name, digest in manifest['binaries'].items():
        if hashlib.sha256((directory / name).read_bytes()).hexdigest() != digest:
            raise ValueError('changed ordinal DLL identity')
    for name, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest:
            raise ValueError('changed ordinal source identity')
    if PE((directory / 'alkyhol_ordinal.dll').read_bytes()).imports() != IMPORTS:
        raise ValueError('input does not have the actual expected mixed ordinal thunks')
    windows = platform.system() == 'Windows'
    if windows:
        normal = native(directory)
        rebased = None
    elif platform.system() == 'Darwin':
        cargo = shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
        subprocess.run([cargo, 'build', '--manifest-path', str(ROOT / 'pe/Cargo.toml'), '--target', 'x86_64-apple-darwin',
                        '--locked', '--offline', '--bin', 'probe_ordinal'], check=True)
        command = [str(ROOT / 'pe/target/x86_64-apple-darwin/debug/probe_ordinal'), str(directory)]
        def execute(flags):
            run = subprocess.run(command + flags, capture_output=True, check=True, timeout=30)
            if run.stderr:
                raise ValueError('unexpected ordinal probe diagnostic')
            return json.loads(run.stdout.decode('utf-8'))
        normal, rebased = execute([]), execute(['--rebase'])
        check_result(rebased, rebase=True)
    else:
        raise RuntimeError('requires Windows or macOS/Rosetta')
    check_result(normal, windows=windows)
    report = {'schema_version': 1, 'backend': 'native Windows ordinal DLLs' if windows else 'Alkyhol ordinal DLLs',
              **{k: manifest[k] for k in ['binaries', 'sources', 'imports']}, 'normal': normal, 'rebased': rebased,
              'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(['git', 'status', '--porcelain', '--untracked-files=no'], cwd=ROOT, text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Passed actual mixed ordinal IAT calls/data/forwarders: ' + report['backend'])


if __name__ == '__main__':
    main()
