#!/usr/bin/env python3
"""Build only the pinned open vkd3d shader compiler and Alkyhol's native adapter.

No Wine/D3D runtime is built or used. Outputs and source caches stay under local/.
macOS build tools: clang, GNU make >=4.3, bison >=3 and flex (e.g. Homebrew).
The Vulkan SONAME configure input is unused metadata for the unbuilt runtime;
this compiler target neither links nor calls a Vulkan implementation.
"""
import argparse,hashlib,json,os,platform,re,shlex,shutil,subprocess,tarfile,urllib.request
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
VERSION='2.1'
URL='https://dl.winehq.org/vkd3d/source/vkd3d-2.1.tar.xz'
SHA='7510146aff2adfb4ae07ab890701a607e5ff7c66e57100cfc9f630ec92eeda6a'
REPOS=[('spirv','https://github.com/KhronosGroup/SPIRV-Headers.git','01e0577914a75a2569c846778c2f93aa8e6feddd'),('vulkan','https://github.com/KhronosGroup/Vulkan-Headers.git','19725e4d48082fe78e26622b15d3080ccd54112b')]
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def run(command,cwd,log,env=None):
 log.write(('\n$ '+shlex.join([str(v) for v in command])+'\n').encode());log.flush()
 try:subprocess.run([str(v) for v in command],cwd=cwd,stdout=log,stderr=subprocess.STDOUT,check=True,env=env)
 except subprocess.CalledProcessError:
  log.flush();print(Path(log.name).read_text(errors='replace')[-12000:]);raise
def checkout(url,revision,path,log):
 if not (path/'.git').is_dir():
  if path.exists() and any(path.iterdir()):raise ValueError('refusing nonempty unrecognized source directory: '+str(path))
  path.mkdir(parents=True,exist_ok=True);run(['git','init',path],ROOT,log);run(['git','remote','add','origin',url],path,log)
 head=subprocess.run(['git','rev-parse','HEAD'],cwd=path,capture_output=True,text=True)
 if head.returncode or head.stdout.strip()!=revision:
  run(['git','fetch','--depth','1','origin',revision],path,log);run(['git','checkout','--detach',revision],path,log)
 if subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=path,text=True).strip():raise ValueError('pinned header source has local changes')
def version(tool,minimum):
 result=subprocess.check_output([tool,'--version'],text=True);m=re.search(r'(\d+)\.(\d+)',result)
 if not m or tuple(map(int,m.groups()))<minimum:raise ValueError(f'{tool} version {minimum} or newer is required')
 return result.splitlines()[0]
def build(output,jobs):
 if platform.system() not in ['Darwin','Linux']:raise ValueError('compiler bootstrap currently supports macOS/Linux hosts')
 output=output.resolve();output.mkdir(parents=True,exist_ok=True)
 make=shutil.which('gmake') or shutil.which('make');bison=shutil.which('bison');flex=shutil.which('flex');cc=shutil.which('clang') or shutil.which('cc')
 if platform.system()=='Darwin':
  for name in ['bison','flex']:
   prefix=subprocess.check_output(['brew','--prefix',name],text=True).strip() if shutil.which('brew') else '/opt/homebrew/opt/'+name
   candidate=Path(prefix)/'bin'/name
   if candidate.is_file():
    if name=='bison':bison=str(candidate)
    else:flex=str(candidate)
 if not all([make,bison,flex,cc]):raise ValueError('clang/cc, GNU make, bison and flex are required')
 tools={'make':version(make,(4,3)),'bison':version(bison,(3,0)),'cc':subprocess.check_output([cc,'--version'],text=True).splitlines()[0]}
 archive=output/'vkd3d-2.1.tar.xz'
 if not archive.exists():
  partial=output/'download.partial'
  with urllib.request.urlopen(URL,timeout=60)as response,partial.open('wb')as out:shutil.copyfileobj(response,out)
  if digest(partial)!=SHA:raise ValueError('pinned vkd3d archive digest mismatch')
  partial.replace(archive)
 if digest(archive)!=SHA:raise ValueError('cached vkd3d archive digest mismatch')
 source=output/'source'
 if not source.exists():
  unpack=output/'unpack';unpack.mkdir()
  with tarfile.open(archive)as tar:tar.extractall(unpack,filter='data')
  (unpack/'vkd3d-2.1').rename(source);unpack.rmdir()
 # Verify every cached upstream file, so a modified local source cannot be
 # reported as the pinned release merely because its archive still matches.
 with tarfile.open(archive)as tar:
  for member in tar.getmembers():
   if member.isfile():
    relative=Path(member.name).relative_to('vkd3d-2.1')
    original=tar.extractfile(member).read()
    if (source/relative).read_bytes()!=original:raise ValueError('cached upstream source differs: '+str(relative))
 builddir=output/'build';builddir.mkdir(exist_ok=True)
 with (output/'build.log').open('ab')as log:
  for name,url,revision in REPOS:checkout(url,revision,output/name,log)
  flags=shlex.join(['-I'+str(output/'spirv/include'),'-I'+str(output/'vulkan/include')])
  # configure and header generators require the real Perl JSON module.
  # Some macOS images only provide JSON::PP. Use pinned pure-Perl JSON 4.10
  # from CPAN when necessary, without altering the host Perl installation.
  perl=shutil.which('perl')
  if not perl:raise ValueError('Perl is required by the upstream build')
  env=dict(os.environ,CC=cc,OBJC=cc,CPPFLAGS=flags,BISON=bison,FLEX=flex,PERL=perl)
  if subprocess.run([perl,'-MJSON','-e','1'],capture_output=True).returncode:
   json_archive=output/'JSON-4.10.tar.gz'
   json_sha='df8b5143d9a7de99c47b55f1a170bd1f69f711935c186a6dc0ab56dd05758e35'
   if not json_archive.exists():
    with urllib.request.urlopen('https://cpan.metacpan.org/authors/id/I/IS/ISHIGAKI/JSON-4.10.tar.gz',timeout=60)as response,json_archive.open('wb')as out:shutil.copyfileobj(response,out)
   if digest(json_archive)!=json_sha:raise ValueError('pinned Perl JSON archive differs')
   with tarfile.open(json_archive)as tar:tar.extractall(output,filter='data')
   env['PERL5LIB']=str(output/'JSON-4.10/lib')+(os.pathsep+os.environ['PERL5LIB'] if os.environ.get('PERL5LIB') else '')
   run([perl,'-MJSON','-e','print $JSON::VERSION'],ROOT,log,env)
  configure=[source/'configure','--prefix='+str(output/'install'),'--disable-tests','--disable-demos','--without-xcb','--without-ncurses','--without-opengl','--without-spirv-tools','SONAME_LIBVULKAN='+('libvulkan.dylib'if platform.system()=='Darwin'else'libvulkan.so.1')]
  configuration={'argv':[str(v)for v in configure],'flags':flags,'tools':tools,'architecture':platform.machine(),'archive':SHA}
  marker=output/'configuration.json'
  if not marker.exists() or json.loads(marker.read_text())!=configuration:
   run(configure,builddir,log,env);marker.write_text(json.dumps(configuration,indent=2)+'\n')
  run([make,'-j'+str(jobs),'BISON='+bison,'FLEX='+flex,'include/private/vkd3d_version.h','vkd3d-compiler'],builddir,log,env)
  libdir=builddir/'.libs';helper=output/'alkyhol-shader9';library=libdir/('libvkd3d-shader.1.dylib'if platform.system()=='Darwin'else'libvkd3d-shader.so.1')
  command=[cc,'-std=c11','-Wall','-Wextra','-Werror','-I',source/'include',ROOT/'d3d9/native/compiler.c','-L',libdir,'-lvkd3d-shader','-Xlinker','-rpath','-Xlinker',libdir,'-o',helper]
  if platform.system()=='Darwin':command+=['-Xlinker','-headerpad_max_install_names']
  run(command,ROOT,log)
  if platform.system()=='Darwin':
   old=subprocess.check_output(['otool','-D',library],text=True).splitlines()[-1].strip()
   run(['install_name_tool','-change',old,library,helper],ROOT,log)
 manifest={'upstream':'https://gitlab.winehq.org/wine/vkd3d','revision':'fb7f9ff3c1f8d972ee6f1b8c6a92caf6c8ba1e10','version':VERSION,'archive_url':URL,'archive_sha256':SHA,'license':'LGPL-2.1-or-later','headers':REPOS,'host':platform.platform(),'machine':platform.machine(),'tools':tools,'helper':str(helper),'helper_source_sha256':digest(ROOT/'d3d9/native/compiler.c'),'helper_sha256':digest(helper),'library':str(library),'library_sha256':digest(library),'hlsl_compiler':str(builddir/'vkd3d-compiler')}
 (output/'compiler.json').write_text(json.dumps(manifest,indent=2)+'\n');print(json.dumps(manifest,indent=2))
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/tools/shader9');p.add_argument('--jobs',type=int,default=4);a=p.parse_args()
 if not 1<=a.jobs<=64:p.error('jobs outside 1..64')
 build(a.output,a.jobs)
