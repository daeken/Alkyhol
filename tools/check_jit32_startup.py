#!/usr/bin/env python3
"""Compare existing unchanged TLS/heap witnesses on JIT, Rosetta and frozen Windows.

This is finite loader/service evidence, not DAO or whole-CPU parity. Game data is
not accepted by this comparator. It never rewrites native observations/fixtures.
"""
import argparse
import json
import platform
import re
import shutil
import subprocess
import tempfile
from pathlib import Path
from check_jit32 import ROOT, sha, verify_sharpretro

SUMMARY=re.compile(rb'JIT32 startup: phase=Ready modules=1 stop=Returned\(0\)\nJIT32 ARM64: instructions=([0-9]+) services=([0-9]+) blocks=([0-9]+) stop=Returned\(0\)\n')


def decode(status,stdout,stderr,expected):
    match=SUMMARY.fullmatch(stderr)
    if type(status) is not int or status!=0 or stdout!=expected or match is None:
        raise ValueError(f'JIT startup/output differs: {status}, {stdout.hex()}, {stderr!r}')
    stats=list(map(int,match.groups()))
    if not all(value>0 for value in stats):raise ValueError('missing actual native execution/service evidence')
    return {'status':status,'stdout_hex':stdout.hex(),'instructions':stats[0],'services':stats[1],'blocks':stats[2]}


def check(output):
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires actual Apple Silicon JIT execution')
    verify_sharpretro()
    from build_pe32_tls_data import build
    build(ROOT/'local/pe/pe32-tls-data')
    tls=ROOT/'pe/reference/pe32-tls-data-windows-v1.json'
    heap=ROOT/'pe/reference/pe32-heap-realloc-windows-v1.json'
    cases={
        'tls':(ROOT/'local/pe/pe32-tls-data/pe32-tls-data.exe',tls),
        'heap':(ROOT/'pe/reference/pe32-heap-realloc-v1.exe',heap),
    }
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--locked','--offline'],check=True,cwd=ROOT)
    exe=ROOT/'jit32/target/debug/alkyhol-jit32'
    output.mkdir(parents=True,exist_ok=True)
    from translate_pe32 import translate
    from run_pe32_tls_data import decode as decode_rosetta_tls
    from observe_pe32_activation_startup import runner
    heap_host=runner(output)
    reports={}
    for label,(pe,reference) in cases.items():
        frozen=json.loads(reference.read_text())
        expected_row=frozen['normal'] if label=='tls' else frozen['reports'][0]
        expected=bytes.fromhex(expected_row['stdout_hex'])
        digest=frozen['pe_sha256'] if label=='tls' else frozen['inputs']['files']['heap-realloc-0.exe']
        sources=frozen['sources'] if label=='tls' else frozen['inputs']['sources']
        for source,identity in sources.items():
            if sha(ROOT/source)!=identity:raise ValueError('frozen source identity differs')
        if sha(pe)!=digest or expected_row['status']!=0 or len(expected)!=(96 if label=='tls' else 256):raise ValueError('changed/invalid native fixture')
        snapshot=(sha(pe),sha(reference))
        row={'pe_sha256':digest,'native_reference_sha256':sha(reference),'native_stdout_hex':expected.hex(),'jit':{},'rosetta':{}}
        host=translate(pe,output/'tls-rosetta') if label=='tls' else heap_host
        for forced in [False,True]:
            mode='rebased' if forced else 'normal'
            with tempfile.TemporaryFile() as stream:
                result=subprocess.run([str(exe),str(pe),'--process']+(['--rebase'] if forced else []),stdout=stream,stderr=subprocess.PIPE,timeout=30)
                stream.seek(0);row['jit'][mode]=decode(result.returncode,stream.read(),result.stderr,expected)
            args=[str(host)]+(['--loader-info','--teb-info'] if label=='tls' else [str(pe),'--strict','--file-origins'])+(['--rebase'] if forced else [])
            result=subprocess.run(args,capture_output=True,timeout=30)
            if label=='tls':decode_rosetta_tls(result,windows=False,forced=forced)
            if result.returncode!=0 or result.stdout!=expected:raise ValueError('actual Rosetta execution differs')
            row['rosetta'][mode]={'status':result.returncode,'stdout_hex':result.stdout.hex(),'stderr_hex':result.stderr.hex()}
        if snapshot!=(sha(pe),sha(reference)):raise ValueError('input/reference changed during execution')
        reports[label]=row
    verify_sharpretro()
    report={'schema_version':1,'host':platform.platform(),'backend':'native ARM64 SharpRetro process startup',
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip(),
            'source_manifest_sha256':sha(ROOT/'jit32/sharpretro-source.json'),'jit_executable_sha256':sha(exe),'cases':reports}
    (output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
    print('Native ARM JIT and actual Rosetta agree with frozen Windows: 24 TLS + 64 heap words, normal and rebased')
    return report


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/jit32/startup-comparison');check(p.parse_args().output.resolve())
