#!/usr/bin/env python3
"""Compile the unmodified pinned MiniEngine projects using Alkyhol's host headers."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
import shutil
from pathlib import Path
import subprocess
import xml.etree.ElementTree as ET
if __package__:
    from .fetch_cpp_deps import ROOT,ensure
else:
    from fetch_cpp_deps import ROOT,ensure
NS={'m':'http://schemas.microsoft.com/developer/msbuild/2003'}
PROJECTS={'Core':'MiniEngine/Core/Core.vcxproj','Model':'MiniEngine/Model/Model.vcxproj',
          'Sample':'Samples/Desktop/D3D12Raytracing/src/D3D12RaytracingMiniEngineSample/ModelViewer_VS16.vcxproj'}

def project_items(path,kind):
    return [(path.parent/element.attrib['Include'].replace('\\','/'),
             {child.tag.split('}')[-1]:(child.text or '').strip() for child in element if not child.get('Condition')})
            for element in ET.parse(path).findall(f'.//m:{kind}[@Include]',NS)]

def shader_command(source,metadata,header,dxc):
    profile={'Compute':'cs','Pixel':'ps','Vertex':'vs','Geometry':'gs','Library':'lib'}[metadata.get('ShaderType','Compute')]
    version=metadata.get('ShaderModel','6.2')
    options=metadata.get('AdditionalOptions','-HV 2021').split()
    command=[str(dxc),'-T',profile+'_'+version.replace('.','_'),'-Fh',str(header),'-Vn','g_p'+source.stem,*options]
    if profile!='lib':command+=['-E',metadata.get('EntryPointName','main')]
    return command+['-D_GAMING_DESKTOP=1',str(source)]

def build_shaders(upstream,output,dxc,jobs=3):
    work=[]
    for project,relative in PROJECTS.items():
        for source,metadata in project_items(upstream/relative,'FxCompile'):
            header=output/project/'CompiledShaders'/(source.stem+'.h');header.parent.mkdir(parents=True,exist_ok=True)
            cmd=shader_command(source,metadata,header,dxc)
            work.append((source,header,cmd))
    def compile(item):
        source,header,cmd=item
        result=subprocess.run(cmd,capture_output=True,text=True)
        header.with_suffix('.log').write_text(result.stdout+result.stderr)
        if result.returncode:raise RuntimeError(f'shader compilation failed: {source}\n{result.stderr}')
        return {'source':str(source.relative_to(upstream)),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
                'header':str(header.relative_to(output)),'header_sha256':hashlib.sha256(header.read_bytes()).hexdigest(),'command':cmd}
    with ThreadPoolExecutor(max_workers=jobs) as pool: report=list(pool.map(compile,work))
    ensure('DirectX-Graphics-Samples')
    (output/'shader-build.json').write_text(json.dumps(report,indent=2)+'\n')
    print(f'Compiled all {len(report)} original shader entries')
    return report


def build_cpp(upstream,output,compiler,jobs=3):
    headers=ensure('DirectX-Headers');math=ensure('DirectXMath');mingw=ensure('mingw-w64')
    tex=ensure('DirectXTex');mesh=ensure('DirectXMesh')
    overlay=output/'dependency-include';overlay.mkdir(parents=True,exist_ok=True)
    names=('dxgi','d3d11','xinput','dinput')
    for src in (mingw/'mingw-w64-headers/include').glob('*.h'):
        if src.name.lower().startswith(names) or src.name=='_mingw_dxhelper.h':
            dest=overlay/src.name
            if not dest.exists():dest.symlink_to(src)
    sdk=subprocess.check_output(['xcrun','--show-sdk-path'],text=True).strip()
    base=[str(compiler),'-arch','x86_64','-std=c++14','-fms-extensions','-msse4.1','-Wno-address-of-temporary',
          '-DRELEASE','-DNDEBUG','-DUNICODE','-D_UNICODE','-D_GAMING_DESKTOP=1','-isysroot',sdk,
          '-include',str(ROOT/'cpp/include/Windows.h')]
    common=[ROOT/'cpp/include',headers/'include',headers/'include/directx',math/'Inc',
            upstream/'MiniEngine/Core',upstream/'MiniEngine/Core/Math',upstream/'MiniEngine/Model',
            overlay,tex/'DirectXTex',mesh/'DirectXMesh']
    compiler_hash=hashlib.sha256(compiler.read_bytes()).hexdigest()
    headers_hash=header_hashes()
    units=[]
    for project,relative in PROJECTS.items():
        include=[output/project,output/'Model',output/'Core',*common]
        for source,_ in project_items(upstream/relative,'ClCompile'):
            obj=output/'objects'/project/(source.stem+'.o');obj.parent.mkdir(parents=True,exist_ok=True)
            command=[*base,*[f'-I{p}' for p in include],*(['-include',str(ROOT/'cpp/bridge/modern_crt.h')] if source.name=='EngineTuning.cpp' else []),'-MMD','-MF',str(obj.with_suffix('.d')),'-c',str(source),'-o',str(obj)]
            units.append((source,obj,command))
    def compile(item):
        source,obj,command=item
        result=subprocess.run(command,capture_output=True,text=True)
        log=obj.with_suffix('.log');log.write_text(result.stdout+result.stderr)
        return {'source':str(source.relative_to(upstream)),'sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
                'exit_code':result.returncode,'object':str(obj.relative_to(output)),
                'object_sha256':hashlib.sha256(obj.read_bytes()).hexdigest() if result.returncode==0 else None,'log':str(log.relative_to(output)),'command':command}
    with ThreadPoolExecutor(max_workers=jobs) as pool:report=list(pool.map(compile,units))
    ensure('DirectX-Graphics-Samples')
    if compiler_hash!=hashlib.sha256(compiler.read_bytes()).hexdigest() or headers_hash!=header_hashes():
        raise RuntimeError('compiler or compatibility headers changed during compilation')
    (output/'cpp-build.json').write_text(json.dumps({'complete':all(r['exit_code']==0 for r in report),'compiler_sha256':compiler_hash,'headers_sha256':headers_hash,'upstream_revision':'68a2b1e97f71642924e74c7757ca21756ad221b5','units':report},indent=2)+'\n')
    passed=sum(r['exit_code']==0 for r in report);print(f'Compiled {passed}/{len(report)} original C++ translation units')
    for row in report:
        if row['exit_code']:
            lines=(output/row['log']).read_text().splitlines()
            errors=[line for line in lines if 'error:' in line]
            print(row['source'],':',errors[0] if errors else 'compiler failure')
    if passed!=len(report):raise SystemExit(1)
    return report

def header_hashes():
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()
            for p in sorted([*(ROOT/'cpp/include').rglob('*'),ROOT/'cpp/bridge/modern_crt.h']) if p.is_file()}

def link_original(upstream,output):
    report=json.loads((output/'cpp-build.json').read_text())
    if not report['complete'] or report['headers_sha256']!=header_hashes():
        raise RuntimeError('complete current C++ compilation is required before linking')
    for unit in report['units']:
        if hashlib.sha256((upstream/unit['source']).read_bytes()).hexdigest()!=unit['sha256'] or hashlib.sha256((output/unit['object']).read_bytes()).hexdigest()!=unit['object_sha256']:
            raise RuntimeError('source or object changed after compilation: '+unit['source'])
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    subprocess.run([cargo,'build','--offline','--target','x86_64-apple-darwin'],cwd=ROOT,check=True)
    library=ROOT/'target/x86_64-apple-darwin/debug/libalkyhol.a'
    driver=output/'main.o';headers=ensure('DirectX-Headers')
    compiler=['clang++','-arch','x86_64','-std=c++14','-fms-extensions',
              '-I'+str(ROOT/'cpp/include'),'-I'+str(headers/'include'),'-I'+str(headers/'include/directx')]
    subprocess.run([*compiler,'-c',str(ROOT/'cpp/bridge/main.cpp'),'-o',str(driver)],check=True)
    archives=[]
    for project in ['Core','Model']:
        archive=output/(project+'.a')
        if archive.exists():archive.unlink()
        objects=[str(output/u['object']) for u in report['units'] if Path(u['object']).parts[1]==project]
        subprocess.run(['ar','rcs',str(archive),*objects],check=True);archives.append(archive)
    samples=[output/u['object'] for u in report['units'] if Path(u['object']).parts[1]=='Sample']
    executable=output/'MiniEngine-Sponza'
    command=[*compiler,str(driver),*map(str,samples),*map(str,archives),str(library),
             '-framework','Security','-framework','CoreFoundation','-lresolv','-lz','-o',str(executable)]
    result=subprocess.run(command,capture_output=True,text=True)
    (output/'link.log').write_text(result.stdout+result.stderr)
    record={'complete':result.returncode==0,'command':command,'rust_library_sha256':hashlib.sha256(library.read_bytes()).hexdigest(),
            'entry_bridge_sha256':hashlib.sha256((ROOT/'cpp/bridge/main.cpp').read_bytes()).hexdigest(),
            'cpp_report_sha256':hashlib.sha256((output/'cpp-build.json').read_bytes()).hexdigest()}
    if result.returncode==0:record['executable_sha256']=hashlib.sha256(executable.read_bytes()).hexdigest()
    (output/'link-build.json').write_text(json.dumps(record,indent=2)+'\n')
    if result.returncode:raise RuntimeError('original application link failed; see '+str(output/'link.log'))
    ensure('DirectX-Graphics-Samples')
    print('Linked complete original MiniEngine application against Rust: '+str(executable))
    return executable

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--shaders-only',action='store_true')
    parser.add_argument('--cpp-only',action='store_true')
    parser.add_argument('--link-only',action='store_true')
    parser.add_argument('--dxc',type=Path,default=ROOT/'local/dxc-build/bin/dxc')
    parser.add_argument('--compiler',type=Path,default=ROOT/'local/clang-compat/bin/clang++')
    parser.add_argument('--jobs',type=int,default=3)
    args=parser.parse_args()
    if sum([args.shaders_only,args.cpp_only,args.link_only])>1:parser.error('choose at most one partial build phase')
    output=ROOT/'local/original-miniengine';output.mkdir(parents=True,exist_ok=True)
    upstream=ensure('DirectX-Graphics-Samples')
    if not (args.cpp_only or args.link_only):build_shaders(upstream,output,args.dxc.resolve(),args.jobs)
    if not (args.shaders_only or args.link_only):build_cpp(upstream,output,args.compiler.resolve(),args.jobs)
    if args.link_only or not (args.shaders_only or args.cpp_only):link_original(upstream,output)
