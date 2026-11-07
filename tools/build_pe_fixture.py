#!/usr/bin/env python3
"""Build an actual no-CRT AMD64 Windows executable using freely available LLVM."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SOURCES = ['pe/fixtures/contract.c', 'pe/fixtures/abi_canary.S', 'pe/fixtures/kernel32.def']


def rust_tool(name):
    rustc = shutil.which('rustc') or str(Path.home() / '.cargo/bin/rustc')
    sysroot = Path(subprocess.check_output([rustc, '--print', 'sysroot'], text=True).strip())
    host = next(line.split(': ', 1)[1] for line in subprocess.check_output([rustc, '-vV'], text=True).splitlines() if line.startswith('host: '))
    tool = sysroot / 'lib/rustlib' / host / 'bin' / name
    if not tool.is_file():
        raise RuntimeError(f'Rust toolchain is missing {name}: {tool}')
    return str(tool)


def build(output):
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    clang = shutil.which('clang')
    if not clang:
        raise RuntimeError('Open LLVM clang is required')
    lld = rust_tool('rust-lld')
    def run(args):
        subprocess.run(args, cwd=ROOT, check=True)
    common = [clang, '--target=x86_64-pc-windows-msvc']
    run(common + ['-O2', '-ffreestanding', '-fno-builtin', '-fno-stack-protector',
                  '-fno-unwind-tables', '-fno-asynchronous-unwind-tables', '-Wall', '-Wextra', '-Werror',
                  '-c', str(ROOT / SOURCES[0]), '-o', str(output / 'contract.obj')])
    run(common + ['-c', str(ROOT / SOURCES[1]), '-o', str(output / 'abi_canary.obj')])
    run([lld, '-flavor', 'link', '/lib', '/def:' + str(ROOT / SOURCES[2]), '/machine:x64', '/out:' + str(output / 'kernel32.lib')])
    exe = output / 'contract.exe'
    run([lld, '-flavor', 'link', '/nodefaultlib', '/entry:entry', '/subsystem:console',
         '/machine:x64', '/dynamicbase', '/fixed:no', '/timestamp:0', '/out:' + str(exe),
         str(output / 'contract.obj'), str(output / 'abi_canary.obj'), str(output / 'kernel32.lib')])
    manifest = {'schema_version': 1, 'pe_sha256': hashlib.sha256(exe.read_bytes()).hexdigest(),
                'sources': {s: hashlib.sha256((ROOT / s).read_bytes()).hexdigest() for s in SOURCES},
                'clang': subprocess.check_output([clang, '--version'], text=True).splitlines()[0],
                'lld': subprocess.check_output([lld, '-flavor', 'link', '--version'], text=True).strip()}
    (output / 'fixture.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(f'Built Windows PE64: {exe} ({manifest["pe_sha256"]})')
    return exe


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'local/pe/fixture')
    args = parser.parse_args()
    build(args.output)


if __name__ == '__main__':
    main()
