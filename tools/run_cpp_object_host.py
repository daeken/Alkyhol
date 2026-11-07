#!/usr/bin/env python3
"""Execute the frozen metadata call sequence through the real Rust COM vtable.

The explicit CPU metadata constructor represents no GPU/device/fence capability.
The independent Windows SDK source and measured acceptance data stay unchanged.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
if __package__:
    from .fetch_cpp_deps import ROOT,ensure
    from .run_cpp_object_reference import compare_frozen,parse_observation
else:
    from fetch_cpp_deps import ROOT,ensure
    from run_cpp_object_reference import compare_frozen,parse_observation


def compare_reports(reports,revision):
    expected={'native Windows SDK D3D12 WARP object metadata','Rust ID3D12Object metadata base (CPU; no GPU)'}
    if len(reports)!=2 or {r.get('backend') for r in reports}!=expected:
        raise ValueError('one independent native reference and one Rust CPU object report are required')
    reference=json.loads((ROOT/'cpp/reference/object-private-data-windows-v1.json').read_text())
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True or report.get('matches_frozen_contract') is not True:
            raise ValueError('unverified, dirty or stale metadata execution')
        observation=parse_observation(json.dumps(report['observation']))
        compare_frozen(observation,report['artifacts']['cpp/tests/object_private_data.cpp'],reference)
    if reports[0]['observation']!=reports[1]['observation']:
        raise ValueError('metadata reports differ')


def run(output):
    if platform.system()!='Darwin':raise RuntimeError('UTF-32 native C++ host probe currently requires macOS')
    headers=ensure('DirectX-Headers');mingw=ensure('mingw-w64')
    build=ROOT/'local/cpp-tests';build.mkdir(parents=True,exist_ok=True)
    overlay=build/'dependency-include';overlay.mkdir(exist_ok=True)
    for p in (mingw/'mingw-w64-headers/include').glob('dxgi*.h'):
        target=overlay/p.name
        if not target.exists():target.symlink_to(p)
    helper=overlay/'_mingw_dxhelper.h'
    if not helper.exists():helper.symlink_to(mingw/'mingw-w64-headers/include/_mingw_dxhelper.h')
    source=ROOT/'cpp/tests/object_metadata_host.cpp'
    original=ROOT/'cpp/tests/object_private_data.cpp'
    inputs=[source,original,Path(__file__).resolve(),ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'cpp/upstream.json',
            *sorted((ROOT/'src').glob('*.rs')),*sorted(p for p in (ROOT/'cpp/include').rglob('*') if p.is_file())]
    hashes=lambda:{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    before=hashes()
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    subprocess.run([cargo,'build','--offline','--target','x86_64-apple-darwin'],cwd=ROOT,check=True,timeout=120)
    library=ROOT/'target/x86_64-apple-darwin/debug/libalkyhol.a'
    library_hash=hashlib.sha256(library.read_bytes()).hexdigest()
    executable=build/'object_metadata_host'
    command=['clang++','-arch','x86_64','-std=c++14','-fms-extensions','-I'+str(ROOT/'cpp/include'),
             '-I'+str(headers/'include'),'-I'+str(headers/'include/directx'),'-I'+str(overlay),str(source),str(library),
             '-framework','Security','-framework','CoreFoundation','-lresolv','-o',str(executable)]
    identity=subprocess.check_output(['clang++','--version'],text=True)
    compiled=subprocess.run(command,capture_output=True,text=True,timeout=120)
    (build/'object_metadata_host.build.log').write_text(compiled.stdout+compiled.stderr)
    if compiled.returncode:raise RuntimeError('metadata host C++ build failed: '+compiled.stderr)
    digest=hashlib.sha256(executable.read_bytes()).hexdigest()
    result=subprocess.run([str(executable)],capture_output=True,text=True,timeout=30,check=True)
    observation=parse_observation(result.stdout)
    reference=json.loads((ROOT/'cpp/reference/object-private-data-windows-v1.json').read_text())
    compare_frozen(observation,before['cpp/tests/object_private_data.cpp'],reference)
    if hashes()!=before or digest!=hashlib.sha256(executable.read_bytes()).hexdigest() or library_hash!=hashlib.sha256(library.read_bytes()).hexdigest():
        raise RuntimeError('host metadata inputs changed during execution')
    report={'schema_version':1,'backend':'Rust ID3D12Object metadata base (CPU; no GPU)','observation':observation,
            'source_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':subprocess.run(['git','diff','--quiet','HEAD'],cwd=ROOT).returncode==0,
            'host':platform.platform(),'machine':platform.machine(),'execution_arch':'x86_64','compiler':identity,
            'build_command':command,'artifacts':before,'executable_sha256':digest,'rust_library_sha256':library_hash,
            'matches_frozen_contract':True,'diagnostics':result.stderr}
    output.write_text(json.dumps(report,indent=2)+'\n');return report

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();run(args.output)
