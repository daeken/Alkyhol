#!/usr/bin/env python3
"""Compare reached x87 XLAT table lookup with independent actual x86/Rosetta."""
import json,os,platform,subprocess
from check_jit32_sse import ROOT,sha

def check():
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires physical Apple Silicon and Rosetta')
    output=ROOT/'local/jit32/x87-xlat-reference';output.mkdir(parents=True,exist_ok=True)
    source=ROOT/'jit32/tests/xlat_reference.c';host=output/'x86-observer';data=output/'x86-observations.bin'
    subprocess.run(['clang','-arch','x86_64','-O1','-Wall','-Wextra','-Werror',str(source),'-o',str(host)],check=True)
    with data.open('wb') as stream:subprocess.run(['arch','-x86_64',str(host)],stdout=stream,check=True,timeout=30)
    if data.stat().st_size!=512*16:raise ValueError('incomplete actual x86 observation')
    env=os.environ.copy();env.update(ALKYHOL_XLAT_REFERENCE=str(data),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    subprocess.run([os.path.expanduser('~/.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib','xlat_tests::actual_x86_xlat_observations','--','--ignored','--exact'],env=env,cwd=ROOT,check=True)
    (output/'report.json').write_text(json.dumps({'backend':'physical native ARM JIT vs actual x86/Rosetta assembly','source_sha256':sha(source),'observer_sha256':sha(host),'observation_sha256':sha(data),'rows':512,'host':platform.platform(),'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()},indent=2)+'\n')
    print('512 actual Rosetta/native ARM XLAT rows agree, including preserved arithmetic flags')
if __name__=='__main__':check()
