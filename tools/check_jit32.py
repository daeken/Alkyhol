#!/usr/bin/env python3
"""Run one frozen PE32 on native ARM JIT and Rosetta; compare frozen Windows bytes.

All generated executables/reports stay under local/. Never accepts a changed
native reference or a changed guest/source identity as a successful comparison.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import shutil
import struct
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SHARP = ROOT.parent/'SharpRetro'
PIN = ROOT/'jit32/sharpretro-source.json'
REFERENCE = ROOT/'pe/reference/pe32-windows-v1.json'
SUMMARY = re.compile(rb'JIT32 ARM64: instructions=([0-9]+) services=([0-9]+) blocks=([0-9]+) stop=Returned\(0\)\n')


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_sharpretro():
    pin = json.loads(PIN.read_text())
    revision = subprocess.check_output(['git','rev-parse','HEAD'],cwd=SHARP,text=True).strip()
    if revision != pin['revision']:
        raise ValueError('SharpRetro checkout revision differs from audited pin')
    for name, digest in pin['files'].items():
        if sha(SHARP/name) != digest:
            raise ValueError(f'SharpRetro source/artifact differs from pin: {name}')
    return pin


def decode_jit(status, stdout, stderr, expected):
    match = SUMMARY.fullmatch(stderr)
    if type(status) is not int or status != 0 or stdout != expected or match is None:
        raise ValueError(f'JIT execution differs: status={status}, stdout={stdout.hex()}, stderr={stderr!r}')
    instructions, services, blocks = map(int, match.groups())
    if instructions < 100000 or services != 20005 or blocks < 50 or blocks > 4096:
        raise ValueError('missing cumulative native JIT/import evidence')
    return {'status':status,'stdout_hex':stdout.hex(),'instructions':instructions,'services':services,'blocks':blocks}


def check(fixture, output):
    if platform.system() != 'Darwin' or platform.machine() != 'arm64':
        raise RuntimeError('requires physical Apple Silicon/macOS; no interpreter substitute')
    pin = verify_sharpretro()
    reference = json.loads(REFERENCE.read_text())
    if reference['backend'] != 'native Windows PE32 i386' or reference['tracked_tree_clean'] is not True:
        raise ValueError('invalid native reference')
    if sha(fixture) != reference['pe_sha256'] or reference['executed_pe_sha256'] != reference['pe_sha256']:
        raise ValueError('guest PE differs from frozen native Windows input')
    for name,digest in reference['sources'].items():
        if sha(ROOT/name) != digest:
            raise ValueError(f'owned PE source changed: {name}')
    expected = bytes.fromhex(reference['normal']['stdout_hex'])
    if len(expected) != 32 or struct.unpack('<8I',expected) != tuple(reference['normal']['words']) or reference['normal']['status'] != 0:
        raise ValueError('invalid frozen native result')
    cargo = shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--locked','--offline'],cwd=ROOT,check=True)
    executable = ROOT/'jit32/target/debug/alkyhol-jit32'
    runs = {}
    for mode in ['normal','rebased']:
        # The initial shared kernel contract admits regular-file stdout explicitly.
        with tempfile.TemporaryFile() as stream:
            result = subprocess.run([str(executable),str(fixture.resolve())]+(['--rebase'] if mode=='rebased' else []),stdout=stream,stderr=subprocess.PIPE,timeout=30)
            stream.seek(0)
            runs[mode] = decode_jit(result.returncode,stream.read(),result.stderr,expected)
    from translate_pe32 import translate, embedded_payload
    from run_pe32_probe import observe
    rosetta = translate(fixture,ROOT/'local/jit32/pe32-rosetta')
    if hashlib.sha256(embedded_payload(rosetta.read_bytes())).hexdigest() != reference['pe_sha256']:
        raise ValueError('Rosetta embedded guest differs')
    native_observation = reference['normal']
    ros = {'normal':observe(rosetta),'rebased':observe(rosetta,rebase=True)}
    if any(value != native_observation for value in ros.values()):
        raise ValueError('Rosetta differs from frozen Windows observation')
    # Reject concurrent changes after execution, not just at initial preflight.
    verify_sharpretro()
    if sha(fixture) != reference['pe_sha256']:
        raise ValueError('guest changed during execution')
    report = {'schema_version':1,'backend':'SharpRetro Bits32 -> native ARM64 / Apple MAP_JIT',
              'host':platform.platform(),'pe_sha256':reference['pe_sha256'],
              'native_reference_sha256':sha(REFERENCE),'sharpretro_source_manifest_sha256':sha(PIN),
              'sharpretro_revision':pin['revision'],'jit_executable_sha256':sha(executable),
              'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
              'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip(),
              'jit':runs,'rosetta':ros,'native_windows':native_observation}
    output.parent.mkdir(parents=True,exist_ok=True)
    output.write_text(json.dumps(report,indent=2)+'\n')
    print('Unchanged PE32: native ARM JIT and Rosetta agree with all 8 frozen Windows words, normal and rebased')
    return report


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--fixture',type=Path,default=ROOT/'local/pe/pe32/pe32.exe')
    parser.add_argument('--output',type=Path,default=ROOT/'local/jit32/comparison.json')
    args=parser.parse_args()
    check(args.fixture,args.output)
