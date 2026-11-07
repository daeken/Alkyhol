#!/usr/bin/env python3
"""Run unchanged original MiniEngine queue/descriptor objects on restricted Metal.

Requires a complete, current original C++ build. This separate subsystem caller
exercises original allocator/queue/descriptor management and supported COM copies.
The original application's startup gate and unsupported rendering remain intact.
"""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
if __package__:
    from .build_original_miniengine import ROOT,ensure,header_hashes
else:
    from build_original_miniengine import ROOT,ensure,header_hashes

ORIGINALS=['MiniEngine/Core/CommandListManager.cpp','MiniEngine/Core/CommandAllocatorPool.cpp','MiniEngine/Core/DescriptorHeap.cpp']
PIN='68a2b1e97f71642924e74c7757ca21756ad221b5'

def current_objects(upstream,build,report,headers):
    if report.get('complete') is not True or report.get('upstream_revision')!=PIN or report.get('headers_sha256')!=headers:
        raise ValueError('complete current pinned C++ compilation required')
    objects=[]
    for source in ORIGINALS:
        rows=[u for u in report['units'] if u['source']==source]
        if len(rows)!=1:raise ValueError('original queue compilation missing or duplicated')
        row=rows[0];obj=build/row['object']
        if row['exit_code']!=0 or hashlib.sha256((upstream/source).read_bytes()).hexdigest()!=row['sha256'] or hashlib.sha256(obj.read_bytes()).hexdigest()!=row['object_sha256']:
            raise ValueError('original queue source/object changed after compilation')
        objects.append(obj)
    return objects

def validate_observation(text):
    result=json.loads(text)
    expected={'scenario':'original-command-manager-v4','next_values':[9,(2<<56)|9,(3<<56)|9],
              'queues':3,'copy_queues':3,'copy_bytes_per_queue':257,'descriptor_types':4,'visible_heaps':2,'allocator_pool_reused':True,'increments_per_queue':8,'idle':True,'shutdown':True}
    if result!=expected or any(type(result[k]) is not type(v) for k,v in expected.items()) or any(type(v) is not int for v in result['next_values']):
        raise ValueError('original command manager did not complete its full sequence')
    return result

def run(output,build):
    if platform.system()!='Darwin':raise RuntimeError('actual Metal macOS host required')
    upstream=ensure('DirectX-Graphics-Samples');headers=ensure('DirectX-Headers')
    manifest=build/'cpp-build.json';record=json.loads(manifest.read_text())
    objects=current_objects(upstream,build,record,header_hashes())
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'cpp/runtime/Cargo.toml'),'--locked','--target','x86_64-apple-darwin'],cwd=ROOT,check=True,timeout=300)
    library=ROOT/'cpp/runtime/target/x86_64-apple-darwin/debug/libalkyhol_cpp_metal.a'
    source=ROOT/'cpp/tests/original_command_manager.cpp'
    inputs=[source,Path(__file__).resolve(),manifest,library,*objects,*[upstream/p for p in ORIGINALS],
            ROOT/'Cargo.toml',ROOT/'Cargo.lock',ROOT/'cpp/runtime/Cargo.toml',ROOT/'cpp/runtime/Cargo.lock',
            *sorted((ROOT/'src').glob('*.rs')),*sorted((ROOT/'cpp/runtime/src').glob('*.rs')),*sorted((ROOT/'cpp/runtime/src').glob('*.metal'))]
    def hashes():
        return {**header_hashes(),**{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs}}
    before=hashes()
    with tempfile.TemporaryDirectory(prefix='alkyhol-original-queue-') as scratch:
        executable=Path(scratch)/'original_command_manager'
        command=['clang++','-arch','x86_64','-std=c++14','-fms-extensions',
                 '-I'+str(ROOT/'cpp/include'),'-I'+str(headers/'include'),'-I'+str(headers/'include/directx'),
                 '-I'+str(upstream/'MiniEngine/Core'),str(source),*map(str,objects),str(library),
                 '-framework','Metal','-framework','Foundation','-framework','CoreGraphics',
                 '-framework','Security','-framework','CoreFoundation','-lobjc','-lresolv','-o',str(executable)]
        compiled=subprocess.run(command,capture_output=True,text=True,timeout=120)
        if compiled.returncode:raise RuntimeError('original queue caller build failed:\n'+compiled.stdout+compiled.stderr)
        result=subprocess.run([str(executable)],capture_output=True,text=True,timeout=30)
        if result.returncode:raise RuntimeError(f'original queue execution failed: {result.returncode}\n{result.stdout}\n{result.stderr}')
        observation=validate_observation(result.stdout)
        ensure('DirectX-Graphics-Samples')
        if before!=hashes():raise RuntimeError('original queue inputs changed during execution')
        report={'schema_version':1,'backend':'original C++ MiniEngine queue manager on restricted Rust Metal',
                'upstream_revision':PIN,'source_revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                'tracked_tree_clean':subprocess.run(['git','diff','--quiet','HEAD'],cwd=ROOT).returncode==0,
                'host':platform.platform(),'machine':platform.machine(),
                'caller_compiler':subprocess.check_output(['clang++','--version'],text=True),
                'original_units':[u for u in record['units'] if u['source'] in ORIGINALS],
                'build_command':command,'artifacts':before,'executable_sha256':hashlib.sha256(executable.read_bytes()).hexdigest(),
                'observation':observation,'diagnostics':result.stderr}
    output.write_text(json.dumps(report,indent=2)+'\n');return report

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--build-dir',type=Path,default=ROOT/'local/original-miniengine')
    args=parser.parse_args();run(args.output,args.build_dir.resolve())
