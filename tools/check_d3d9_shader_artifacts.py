#!/usr/bin/env python3
"""Regenerate owned legacy artifacts using the pinned open compiler, compare bytes.
No game inputs. Generated work stays under ignored local/.
"""
import argparse,hashlib,json,struct,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def normalize_owned_ps2_output(path):
 """vkd3d 2.1's owned HLSL fixture writes FRC directly to oC0. SM2 only
 permits MOV there. Preserve its last FRC in r0 and append the required MOV.
 This is fixture generation only; runtime game bytecode is never rewritten.
 """
 raw=path.read_bytes();words=list(struct.unpack('<'+'I'*(len(raw)//4),raw))
 if words[0]!=0xffff0200 or words[-4:]!=[0x02000013,0x800f0800,0x80e40000,0xffff]:raise ValueError('unexpected owned SM2 output pattern')
 words[-3]=0x800f0000
 words[-1:]=[0x02000001,0x800f0800,0x80e40000,0xffff]
 path.write_bytes(struct.pack('<'+'I'*len(words),*words))
def check(directory):
 manifest=json.loads((directory/'compiler.json').read_text())
 fixture=ROOT/'d3d9/tests/fixtures';provenance=json.loads((fixture/'legacy-color-provenance.json').read_text())
 if manifest['archive_sha256']!=provenance['archive_sha256'] or manifest['revision']!=provenance['upstream_revision']:raise ValueError('compiler pin differs')
 if sha(ROOT/'d3d9/native/compiler.c')!=provenance['helper_source_sha256'] or manifest['helper_source_sha256']!=provenance['helper_source_sha256']:raise ValueError('native compiler adapter identity differs')
 for name,digest in provenance['files'].items():
  if sha(fixture/name)!=digest:raise ValueError('fixture identity differs: '+name)
 output=ROOT/'local/d3d9/owned-regeneration';output.mkdir(parents=True,exist_ok=True)
 texture_provenance=json.loads((fixture/'legacy-texture-provenance.json').read_text())
 if texture_provenance['helper_source_sha256']!=manifest['helper_source_sha256']:raise ValueError('texture helper identity differs')
 for name,digest in texture_provenance['files'].items():
  if sha(fixture/name)!=digest:raise ValueError('texture fixture identity differs: '+name)
 for name,digest in json.loads((fixture/'legacy-model2-provenance.json').read_text())['files'].items():
  if sha(fixture/name)!=digest:raise ValueError('SM2 fixture identity differs: '+name)
 for prefix in ['legacy-color.ps','legacy-color.vs','legacy-texture.ps','legacy-slots128.ps','legacy-model2.vs']:
  stage=prefix.rsplit('.',1)[1]
  profile=stage+('_2_0' if prefix in ['legacy-slots128.ps','legacy-model2.vs'] else '_3_0')
  subprocess.run([manifest['hlsl_compiler'],'-x','hlsl','-b','d3dbc','-p',profile,str(fixture/(prefix+'.hlsl')),'-o',str(output/(prefix+'.bin'))],check=True)
  if prefix=='legacy-slots128.ps':normalize_owned_ps2_output(output/(prefix+'.bin'))
  following=[str(output/('legacy-slots128.ps.bin' if prefix=='legacy-model2.vs' else 'legacy-color.ps.bin'))] if stage=='vs' else []
  subprocess.run([manifest['helper'],str(output/(prefix+'.bin')),str(output/(prefix+'.spv')),str(output/(prefix+'.json')),*following],check=True)
  for extension in ['bin','spv','json']:
   if (output/(prefix+'.'+extension)).read_bytes()!=(fixture/(prefix+'.'+extension)).read_bytes():raise ValueError('regenerated artifact differs: '+prefix+'.'+extension)
  binary=(fixture/(prefix+'.bin')).read_bytes()
  if prefix=='legacy-slots128.ps':
   words=struct.unpack('<'+'I'*(len(binary)//4),binary);at=1;counts={}
   if words[0]!=0xffff0200:raise ValueError('SM2 header differs')
   while words[at]!=0xffff:
    token=words[at];at+=1;opcode=token&65535
    if opcode==0xfffe:at+=(token>>16)&32767;continue
    counts[opcode]=counts.get(opcode,0)+1;at+=(token>>24)&15
   if counts.get(2)!=64 or counts.get(19)!=64:raise ValueError('owned shader no longer contains 128 ADD/FRC slots')
  include='// Owned MIT legacy bytecode: exact DWORDs from d3d9/tests/fixtures.\n'+',\n'.join('0x%08xu'%n for n in struct.unpack('<'+'I'*(len(binary)//4),binary))+'\n'
  if (ROOT/'pe/fixtures/d3d9'/(prefix+'.inc')).read_text()!=include:raise ValueError('PE embedded bytecode differs')
 print('Owned HLSL, bytecode, linked SPIR-V, reflection and PE bytecode agree exactly')
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--compiler-directory',type=Path,default=ROOT/'local/tools/shader9');a=p.parse_args();check(a.compiler_directory.resolve())
