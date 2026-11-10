#!/usr/bin/env python3
"""Strict Rosetta comparison; known NaN/DE differences remain failures, not passes."""
import json,os,platform,subprocess
from pathlib import Path
from check_jit32_sse import ROOT,sha
def check():
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires physical Apple Silicon and Rosetta')
    output=ROOT/'local/jit32/compare-reference';output.mkdir(parents=True,exist_ok=True)
    source=ROOT/'jit32/tests/sse_compare_reference.c';host=output/'x86-observer';data=output/'x86-observations.bin'
    subprocess.run(['clang','-arch','x86_64','-O1','-mno-red-zone','-Wall','-Wextra','-Werror',str(source),'-o',str(host)],check=True)
    with data.open('wb') as stream:subprocess.run(['arch','-x86_64',str(host)],stdout=stream,check=True,timeout=30)
    if data.stat().st_size!=18432*72:raise ValueError('incomplete actual x86 observation')
    env=os.environ.copy();env.update(ALKYHOL_COMPARE_REFERENCE=str(data),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    result=subprocess.run([str(Path.home()/'.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib','sse_compare::tests::actual_x86_comparison_observations','--','--ignored','--exact'],env=env,cwd=ROOT)
    (output/'report.json').write_text(json.dumps(dict(backend='physical native ARM JIT vs actual x86/Rosetta assembly',source_sha256=sha(source),observer_sha256=sha(host),observation_sha256=sha(data),rows=18432,model_status=result.returncode,all_rows_match=result.returncode==0,host=platform.platform(),revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()),indent=2)+'\n')
    result.check_returncode()
    print('18432 actual Rosetta/native ARM comparison rows agree, including NaNs, denormals, MXCSR and EFLAGS')
if __name__=='__main__':check()
