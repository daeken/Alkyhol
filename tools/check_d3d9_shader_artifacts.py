#!/usr/bin/env python3
"""Regenerate owned legacy artifacts using the pinned open compiler, compare bytes.
No game inputs. Generated work stays under ignored local/.
"""
import argparse,hashlib,json,struct,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def check(directory):
 manifest=json.loads((directory/'compiler.json').read_text())
 fixture=ROOT/'d3d9/tests/fixtures';provenance=json.loads((fixture/'legacy-color-provenance.json').read_text())
 if manifest['archive_sha256']!=provenance['archive_sha256'] or manifest['revision']!=provenance['upstream_revision']:raise ValueError('compiler pin differs')
 if sha(ROOT/'d3d9/native/compiler.c')!=provenance['helper_source_sha256'] or manifest['helper_source_sha256']!=provenance['helper_source_sha256']:raise ValueError('native compiler adapter identity differs')
 for name,digest in provenance['files'].items():
  if sha(fixture/name)!=digest:raise ValueError('fixture identity differs: '+name)
 output=ROOT/'local/d3d9/owned-regeneration';output.mkdir(parents=True,exist_ok=True)
 for stage in ['ps','vs']:
  prefix='legacy-color.'+stage
  subprocess.run([manifest['hlsl_compiler'],'-x','hlsl','-b','d3dbc','-p',stage+'_3_0',str(fixture/(prefix+'.hlsl')),'-o',str(output/(prefix+'.bin'))],check=True)
  following=[str(output/'legacy-color.ps.bin')] if stage=='vs' else []
  subprocess.run([manifest['helper'],str(output/(prefix+'.bin')),str(output/(prefix+'.spv')),str(output/(prefix+'.json')),*following],check=True)
  for extension in ['bin','spv','json']:
   if (output/(prefix+'.'+extension)).read_bytes()!=(fixture/(prefix+'.'+extension)).read_bytes():raise ValueError('regenerated artifact differs: '+prefix+'.'+extension)
  binary=(fixture/(prefix+'.bin')).read_bytes()
  include='// Owned MIT legacy bytecode: exact DWORDs from d3d9/tests/fixtures.\n'+',\n'.join('0x%08xu'%n for n in struct.unpack('<'+'I'*(len(binary)//4),binary))+'\n'
  if (ROOT/'pe/fixtures/d3d9'/(prefix+'.inc')).read_text()!=include:raise ValueError('PE embedded bytecode differs')
 print('Owned HLSL, bytecode, linked SPIR-V, reflection and PE bytecode agree exactly')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--compiler-directory',type=Path,default=ROOT/'local/tools/shader9');a=p.parse_args();check(a.compiler_directory.resolve())
