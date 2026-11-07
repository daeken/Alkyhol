#!/usr/bin/env python3
"""Execute actual C++ DXGI discovery through the Rust/direct-Metal COM runtime."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
if __package__:
    from .fetch_cpp_deps import ROOT,ensure
else:
    from fetch_cpp_deps import ROOT,ensure


def validate_observation(text):
    result=json.loads(text)
    expected={'scenario','adapters','parent_lifetime','unsupported_boundaries','descriptor_bytes'}
    if not isinstance(result,dict) or set(result)!=expected or result['scenario']!='metal-dxgi-factory-v1':raise ValueError('unknown factory observation')
    if result['parent_lifetime'] is not True or result['unsupported_boundaries'] is not True or result['descriptor_bytes']!=568:raise ValueError('factory ABI/lifetime checks failed')
    if not isinstance(result['adapters'],list) or not result['adapters']:raise ValueError('actual Metal adapters required')
    for adapter in result['adapters']:
        if set(adapter)!= {'name','registry_id','virtual_video_budget'} or not isinstance(adapter['name'],str) or not adapter['name']:raise ValueError('invalid Metal descriptor')
        for key in ['registry_id','virtual_video_budget']:
            if type(adapter[key]) is not int or not 0<=adapter[key]<2**64:raise ValueError('invalid Metal identity/budget')
    return result


def run(output):
    if platform.system()!='Darwin':raise RuntimeError('direct Metal runtime requires macOS; no software enumeration substitute')
    headers=ensure('DirectX-Headers');mingw=ensure('mingw-w64')
    build=ROOT/'local/cpp-metal';build.mkdir(parents=True,exist_ok=True);overlay=build/'include';overlay.mkdir(exist_ok=True)
    for source in (mingw/'mingw-w64-headers/include').glob('dxgi*.h'):
        target=overlay/source.name
        if not target.exists():target.symlink_to(source)
    helper=overlay/'_mingw_dxhelper.h'
    if not helper.exists():helper.symlink_to(mingw/'mingw-w64-headers/include/_mingw_dxhelper.h')
    inputs=[ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'cpp/upstream.json',ROOT/'cpp/runtime/Cargo.toml',ROOT/'cpp/runtime/Cargo.lock',
            ROOT/'cpp/runtime/src/lib.rs',ROOT/'cpp/tests/metal_factory.cpp',Path(__file__).resolve(),
            *sorted((ROOT/'src').glob('*.rs')),*sorted(p for p in (ROOT/'cpp/include').rglob('*') if p.is_file())]
    hashes=lambda:{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
    before=hashes()
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'cpp/runtime/Cargo.toml'),'--locked','--target','x86_64-apple-darwin'],cwd=ROOT,check=True,timeout=300)
    library=ROOT/'cpp/runtime/target/x86_64-apple-darwin/debug/libalkyhol_cpp_metal.a';library_hash=hashlib.sha256(library.read_bytes()).hexdigest()
    executable=build/'metal_factory'
    command=['clang++','-arch','x86_64','-std=c++14','-fms-extensions','-I'+str(ROOT/'cpp/include'),'-I'+str(headers/'include'),'-I'+str(headers/'include/directx'),'-I'+str(overlay),str(ROOT/'cpp/tests/metal_factory.cpp'),str(library),
             '-framework','Metal','-framework','Foundation','-framework','CoreGraphics','-framework','Security','-framework','CoreFoundation','-lobjc','-lresolv','-o',str(executable)]
    result=subprocess.run(command,capture_output=True,text=True,timeout=120);(build/'build.log').write_text(result.stdout+result.stderr)
    if result.returncode:raise RuntimeError('C++ factory build failed: '+result.stderr)
    digest=hashlib.sha256(executable.read_bytes()).hexdigest()
    executed=subprocess.run([str(executable)],cwd=build,capture_output=True,text=True,check=True,timeout=30)
    (build/'execution.log').write_text(executed.stdout+executed.stderr)
    observation=validate_observation(executed.stdout)
    native_lines=[line for line in executed.stderr.splitlines() if line.startswith('Metal adapter: ')]
    if len(native_lines)!=len(observation['adapters']):raise RuntimeError('missing or ambiguous actual Metal device diagnostics')
    if before!=hashes() or library_hash!=hashlib.sha256(library.read_bytes()).hexdigest() or digest!=hashlib.sha256(executable.read_bytes()).hexdigest():raise RuntimeError('factory inputs changed during observation')
    report={'schema_version':1,'backend':'Rust DXGI COM over direct Metal (discovery only)','observation':observation,'diagnostics':executed.stderr,
            'source_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':subprocess.run(['git','diff','--quiet','HEAD'],cwd=ROOT).returncode==0,
            'host':platform.platform(),'machine':platform.machine(),'execution_arch':'x86_64','compiler':subprocess.check_output(['clang++','--version'],text=True),
            'build_command':command,'artifacts':before,'rust_library_sha256':library_hash,'executable_sha256':digest}
    output.write_text(json.dumps(report,indent=2)+'\n');return report

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();run(args.output)
