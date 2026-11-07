#!/usr/bin/env python3
"""Compare native WARP mapped buffers with the restricted Rust/Metal COM device.

Metal uses an explicit subset constructor. D3D12CreateDevice stays unsupported;
no feature-level conformance is manufactured for the native API name.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import tempfile
if __package__:
    from .fetch_cpp_deps import ROOT,ensure
else:
    from fetch_cpp_deps import ROOT,ensure


def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!= {'scenario','resources','parent_lifetime','gpu_addresses_nonzero'} or result['scenario']!='device-shared-buffers-v1':raise ValueError('unknown buffer observation')
    if result['parent_lifetime'] is not True or result['gpu_addresses_nonzero'] is not True:raise ValueError('device/resource ownership/address checks failed')
    rows=result['resources']
    if not isinstance(rows,list) or len(rows)!=4:raise ValueError('four complete buffer observations required')
    for row in rows:
        if set(row)!= {'heap','heap_flags','desc','checksum'}:raise ValueError('unknown buffer fields')
        for key,count in [('heap',5),('desc',11)]:
            if not isinstance(row[key],list) or len(row[key])!=count or any(type(v) is not int or not 0<=v<2**64 for v in row[key]):raise ValueError('invalid descriptor words')
        if any(type(row[key]) is not int or not 0<=row[key]<2**32 for key in ['heap_flags','checksum']):raise ValueError('invalid returned flags/checksum')
    return result


def compare_reports(reports,revision):
    expected={'native Windows SDK WARP device buffers','Rust restricted Metal COM device buffers'}
    if len(reports)!=2 or {r.get('backend') for r in reports}!=expected:raise ValueError('one independent native and one Metal buffer report required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:raise ValueError('stale or dirty buffer execution')
        validate_observation(json.dumps(report['observation']))
    if reports[0]['artifacts']['cpp/tests/device_buffers.cpp']!=reports[1]['artifacts']['cpp/tests/device_buffers.cpp']:raise ValueError('probe source bytes differ')
    if reports[0]['observation']!=reports[1]['observation']:raise ValueError('native and Metal buffer observations differ')
    frozen=json.loads((ROOT/'cpp/reference/device-shared-buffers-windows-v1.json').read_text())
    if reports[0]['observation']!=frozen['observation']:raise ValueError('live buffer behavior differs from frozen native contract')


def run(output):
    return run_probe(output,'device_buffers','device buffers',validate_observation,
                     ['trap-view','trap-heap','trap-copy','trap-footprints','trap-tiling'])

def run_probe(output,name,label,validate,traps=()):
    native=os.name=='nt'
    if not native and platform.system()!='Darwin':raise RuntimeError('Windows SDK or actual Metal host required')
    source=ROOT/f'cpp/tests/{name}.cpp';inputs=[source,Path(__file__).resolve()]
    if name!='device_buffers':inputs.append(ROOT/f'tools/run_cpp_{name}.py')
    if name=='cbv_copies':inputs += [ROOT/'cpp/tests/cbv_probe.hlsl',ROOT/'cpp/tests/cbv_probe_dxil.h',ROOT/'tools/build_cpp_cbv_probe.py']
    if name in ('compute_pipelines','compute_dispatch'):inputs += [ROOT/'cpp/tests/compute_probe.dxil',ROOT/'cpp/tests/compute_probe_dxil.h',ROOT/'tools/build_cpp_compute_probe.py',ROOT/'shader/tests/fixtures/compute.hlsl']
    with tempfile.TemporaryDirectory(prefix='alkyhol-device-buffers-') as scratch:
        executable=Path(scratch)/(name+'.exe' if native else name)
        if native:
            command=['cl','/nologo','/W4','/WX','/EHsc','/std:c++17',str(source),'/Fe:'+str(executable),'/link','d3d12.lib','dxgi.lib','dxguid.lib']
            identity=subprocess.run(['cl'],capture_output=True,text=True,timeout=30);compiler=identity.stdout+identity.stderr
            backend='native Windows SDK WARP '+label
        else:
            headers=ensure('DirectX-Headers');mingw=ensure('mingw-w64');overlay=Path(scratch)/'include';overlay.mkdir()
            for path in (mingw/'mingw-w64-headers/include').glob('dxgi*.h'):(overlay/path.name).symlink_to(path)
            (overlay/'_mingw_dxhelper.h').symlink_to(mingw/'mingw-w64-headers/include/_mingw_dxhelper.h')
            cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
            subprocess.run([cargo,'build','--manifest-path',str(ROOT/'cpp/runtime/Cargo.toml'),'--locked','--target','x86_64-apple-darwin'],check=True,cwd=ROOT,timeout=300)
            library=ROOT/'cpp/runtime/target/x86_64-apple-darwin/debug/libalkyhol_cpp_metal.a'
            command=['clang++','-arch','x86_64','-std=c++14','-fms-extensions','-I'+str(ROOT/'cpp/include'),'-I'+str(headers/'include'),'-I'+str(headers/'include/directx'),'-I'+str(overlay),str(source),str(library),'-framework','Metal','-framework','Foundation','-framework','CoreGraphics','-framework','Security','-framework','CoreFoundation','-lobjc','-lresolv','-o',str(executable)]
            compiler=subprocess.check_output(['clang++','--version'],text=True);backend='Rust restricted Metal COM '+label
            inputs += [ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'cpp/runtime/Cargo.toml',ROOT/'cpp/runtime/Cargo.lock',library,*sorted((ROOT/'src').glob('*.rs')),*sorted((ROOT/'cpp/runtime/src').glob('*.rs')),*sorted((ROOT/'cpp/runtime/src').glob('*.metal')),*sorted(p for p in (ROOT/'cpp/include').rglob('*') if p.is_file())]
        if name in ('compute_pipelines','compute_dispatch') and not native:inputs += [ROOT/'shader/Cargo.toml',ROOT/'shader/Cargo.lock',*sorted((ROOT/'shader/src').glob('*.rs'))]
        hashes=lambda:{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}
        before=hashes();compiled=subprocess.run(command,cwd=scratch,capture_output=True,text=True,timeout=120)
        if compiled.returncode:raise RuntimeError('buffer probe compilation failed:\n'+compiled.stdout+compiled.stderr)
        digest=hashlib.sha256(executable.read_bytes()).hexdigest()
        result=subprocess.run([str(executable)],capture_output=True,text=True,cwd=scratch,timeout=30)
        if result.returncode:raise RuntimeError(f'buffer probe execution failed: {result.returncode}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}')
        observation=validate(result.stdout)
        if not native:
            for mode in traps:
                failed=subprocess.run([str(executable),mode],capture_output=True,text=True,cwd=scratch,timeout=30)
                if failed.returncode!=-signal.SIGABRT or 'is not implemented' not in failed.stderr:raise RuntimeError('unsupported void/struct operation returned or failed unexpectedly: '+mode)
        if before!=hashes() or digest!=hashlib.sha256(executable.read_bytes()).hexdigest():raise RuntimeError('buffer probe inputs changed during execution')
        report={'schema_version':1,'backend':backend,'observation':observation,'diagnostics':result.stderr,
                'source_revision':subprocess.check_output(['git','rev-parse','HEAD'],text=True,cwd=ROOT).strip(),
                'tracked_tree_clean':subprocess.run(['git','diff','--quiet','HEAD'],cwd=ROOT).returncode==0,
                'host':platform.platform(),'machine':platform.machine(),'compiler':compiler,'windows_sdk':os.environ.get('WindowsSDKVersion'),
                'build_command':command,'artifacts':before,'executable_sha256':digest}
    output.write_text(json.dumps(report,indent=2)+'\n');return report

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    args=parser.parse_args();run(args.output)
