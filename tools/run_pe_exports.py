#!/usr/bin/env python3
"""Compare Rust DLL metadata with actual Windows GetProcAddress; no host DLL entry."""
import argparse
import ctypes
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
from build_pe_fixture import ROOT

# Independent source/.def contract; queries never come from the Rust parser.
SPEC = [(17, ['Direct'], None), (18, ['Alias'], None),
        (20, ['Forwarded'], 'KERNEL32.GetLastError'), (21, ['Data'], None),
        (22, ['Bss'], None), (23, [], None)]


def native(dll):
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.LoadLibraryW.argtypes, kernel.LoadLibraryW.restype = [ctypes.c_wchar_p], ctypes.c_void_p
    kernel.GetProcAddress.argtypes, kernel.GetProcAddress.restype = [ctypes.c_void_p, ctypes.c_void_p], ctypes.c_void_p
    kernel.FreeLibrary.argtypes, kernel.FreeLibrary.restype = [ctypes.c_void_p], ctypes.c_int
    kernel.GetModuleHandleW.argtypes, kernel.GetModuleHandleW.restype = [ctypes.c_wchar_p], ctypes.c_void_p
    module = kernel.LoadLibraryW(str(dll))
    if not module:
        raise ctypes.WinError(ctypes.get_last_error())
    def get(mod, symbol):
        if isinstance(symbol, int):
            return kernel.GetProcAddress(mod, ctypes.c_void_p(symbol))
        buffer = ctypes.create_string_buffer(symbol.encode('ascii'))
        return kernel.GetProcAddress(mod, ctypes.cast(buffer, ctypes.c_void_p))
    try:
        entries = []
        for ordinal, names, forwarder in SPEC:
            address = get(module, ordinal)
            if not address:
                raise RuntimeError(f'native ordinal {ordinal} did not resolve')
            for name in names:
                if get(module, name) != address:
                    raise RuntimeError('native named/ordinal query differs')
            if forwarder:
                if address != get(kernel.GetModuleHandleW('kernel32.dll'), 'GetLastError'):
                    raise RuntimeError('native forwarder did not resolve to real kernel32')
                target = {'forwarder': forwarder}
            else:
                target = {'rva': address - module}
            entries.append({'ordinal': ordinal, 'names': names, 'target': target})
        if get(module, 'missing_export'):
            raise RuntimeError('missing named export resolved unexpectedly')
        if get(module, 'Direct') != get(module, 'Alias'):
            raise RuntimeError('native aliases have different target addresses')
        data = ctypes.c_uint32.from_address(get(module, 'Data')).value
        bss = ctypes.c_uint32.from_address(get(module, 'Bss')).value
        result = ctypes.WINFUNCTYPE(ctypes.c_uint32, ctypes.c_uint32, ctypes.c_uint32)(get(module, 'Direct'))(7, 11)
        ordinal_result = ctypes.WINFUNCTYPE(ctypes.c_uint32, ctypes.c_uint32)(get(module, 23))(9)
        if (data, bss, result, ordinal_result) != (0x31415926, 0x13579bdf, (7 * 33) ^ 11 ^ 0x31415926 ^ 0x13579bdf, 0x12345681):
            raise RuntimeError('actual native DLL initialization/calls differ from source contract')
        hole = get(module, 19)
        native_only = {'initialized_data': data, 'bss_after_attach': bss, 'direct_call': result,
                       'ordinal_call': ordinal_result,
                       'hole19': 'NULL' if not hole else 'image-base' if hole == module else 'other-nonnull'}
        common = {'dll_name': 'alkyhol_exports.dll', 'ordinal_base': 17, 'entries': entries}
        return common, native_only
    finally:
        if not kernel.FreeLibrary(module):
            raise ctypes.WinError(ctypes.get_last_error())


def inspect(dll):
    cargo = shutil.which('cargo') or str(Path.home() / '.cargo/bin/cargo')
    result = subprocess.run([cargo, 'run', '--manifest-path', str(ROOT / 'pe/Cargo.toml'),
                             '--locked', '--offline', '--bin', 'inspect_exports', '--', str(dll)],
                            capture_output=True, check=True)
    common = json.loads(result.stdout.decode('utf-8'))
    expected = [(ordinal, names, forwarder) for ordinal, names, forwarder in SPEC]
    actual = [(e['ordinal'], e['names'], e['target'].get('forwarder')) for e in common['entries']]
    if actual != expected or common['dll_name'] != 'alkyhol_exports.dll' or common['ordinal_base'] != 17:
        raise RuntimeError(f'Rust export metadata differs from independent .def contract: {common}')
    if common['entries'][0]['target'] != common['entries'][1]['target']:
        raise RuntimeError('Rust alias target RVAs differ')
    return common, None


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--dll', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    dll = args.dll.resolve()
    manifest = json.loads((dll.parent / 'exports-fixture.json').read_text(encoding='utf-8'))
    if hashlib.sha256(dll.read_bytes()).hexdigest() != manifest['dll_sha256']:
        raise RuntimeError('DLL changed after build')
    for name, digest in manifest['sources'].items():
        if hashlib.sha256((ROOT / name).read_bytes()).hexdigest() != digest:
            raise RuntimeError('DLL source identity changed')
    windows = platform.system() == 'Windows'
    common, native_only = native(dll) if windows else inspect(dll)
    report = {'schema_version': 1, 'backend': 'native Windows LoadLibrary/GetProcAddress' if windows else 'Rust metadata only (no DLL entry)',
              'dll_sha256': manifest['dll_sha256'], 'sources': manifest['sources'],
              'common': common, 'native_only': native_only,
              'revision': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print('Passed owned DLL observer: ' + report['backend'])


if __name__ == '__main__':
    main()
