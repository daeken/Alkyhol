#!/usr/bin/env python3
"""Execute C++ ABI probes against Rust, or independently against the Windows SDK."""
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


def run(output,compiler_tests=False):
    windows=platform.system()=='Windows'
    build=ROOT/'local/cpp-tests';build.mkdir(parents=True,exist_ok=True)
    shared=['com_blob','input_formats']
    names=shared if windows else [*shared,'platform_headers','crt','runtime']
    if not windows:
        headers=ensure('DirectX-Headers');mingw=ensure('mingw-w64')
        overlay=build/'dependency-include';overlay.mkdir(exist_ok=True)
        for name in ['dinput.h','xinput.h','_mingw_dxhelper.h','dxgi.h','dxgi1_2.h','dxgi1_3.h','dxgi1_4.h','dxgi1_5.h','dxgi1_6.h','dxgicommon.h','dxgiformat.h','dxgitype.h']:
            dest=overlay/name
            if not dest.exists():dest.symlink_to(mingw/'mingw-w64-headers/include'/name)
        cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
        subprocess.run([cargo,'build','--offline','--target','x86_64-apple-darwin'],cwd=ROOT,check=True)
        library=ROOT/'target/x86_64-apple-darwin/debug/libalkyhol.a'
        base=['clang++','-arch','x86_64','-std=c++14','-fms-extensions','-I'+str(ROOT/'cpp/include'),'-I'+str(headers/'include'),'-I'+str(headers/'include/directx'),'-I'+str(overlay)]
    observations={};records=[]
    for name in names:
        source=ROOT/'cpp/tests'/(name+'.cpp');executable=build/(name+('.exe' if windows else ''))
        command=['cl','/nologo','/std:c++14','/EHsc',str(source),'/Fe:'+str(executable),'/Fo:'+str(build/(name+'.obj')),'d3dcompiler.lib','dinput8.lib','dxguid.lib'] if windows else [*base,str(source),str(library),'-framework','Security','-framework','CoreFoundation','-lresolv','-o',str(executable)]
        compiled=subprocess.run(command,cwd=build,capture_output=True,text=True)
        (build/(name+'.build.log')).write_text(compiled.stdout+compiled.stderr)
        if compiled.returncode:raise RuntimeError(f'{name} compilation failed: {compiled.stderr}\n{compiled.stdout}')
        result=subprocess.run([str(executable)],cwd=build,capture_output=True,text=True)
        (build/(name+'.run.log')).write_text(result.stdout+result.stderr)
        if result.returncode:raise RuntimeError(f'{name} execution failed: {result.returncode}\n{result.stderr}')
        records.append({'source':str(source.relative_to(ROOT)),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'command':command,'executable_sha256':hashlib.sha256(executable.read_bytes()).hexdigest()})
        if name in shared:observations[name]=result.stdout
        print(f'{name}: passed')
    if compiler_tests:
        if windows:raise ValueError('patched Clang tests run separately on macOS')
        compiler=ROOT/'local/clang-compat/bin/clang++'
        sdk=subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip()
        base=[str(compiler),'-arch','x86_64','-std=c++14','-isysroot',sdk]
        for name in ['msvc_record_temporary','msvc_reference_temporary','msvc_legacy_user']:
            sources=[str(ROOT/'cpp/tests'/(name+'.cpp'))]
            if name=='msvc_legacy_user':sources.append(str(ROOT/'cpp/tests/msvc_legacy_provider.cpp'))
            executable=build/name
            command=[*base,'-fms-extensions',*sources,'-o',str(executable)]
            subprocess.run(command,cwd=build,check=True)
            subprocess.run([str(executable)],cwd=build,check=True)
            # Standard mode must retain diagnostics, not silently gain extensions.
            rejected=subprocess.run([*base,'-fsyntax-only',sources[0]],cwd=build,capture_output=True,text=True)
            if rejected.returncode==0:raise RuntimeError('standard-mode rejection lost: '+name)
            print(name+': passed, including standard-mode rejection')
        missing=subprocess.run([*base,'-fms-extensions',str(ROOT/'cpp/tests/msvc_legacy_user.cpp'),'-o',str(build/'missing-provider')],capture_output=True,text=True)
        if missing.returncode==0 or 'legacy_transform' not in missing.stderr:raise RuntimeError('missing real template provider did not fail at link')
    revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    clean=subprocess.run(['git','diff','--quiet','HEAD'],cwd=ROOT).returncode==0
    report={'schema_version':1,'backend':'native Windows SDK' if windows else 'Rust native-host COM/Win32','revision':revision,'tracked_tree_clean':clean,'observations':observations,'executions':records}
    output.write_text(json.dumps(report,indent=2)+'\n')
    return report

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,default=Path('cpp-observation.json'))
    parser.add_argument('--compiler-tests',action='store_true')
    args=parser.parse_args();run(args.output,args.compiler_tests)
