#!/usr/bin/env python3
"""Strict actual Rosetta x87 diagnostic; retained differences are failures."""
import json,os,platform,subprocess
from pathlib import Path
from check_jit32_sse import ROOT,sha

def check():
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires physical Apple Silicon and Rosetta')
    output=ROOT/'local/jit32/x87-arithmetic-reference';output.mkdir(parents=True,exist_ok=True)
    source=ROOT/'jit32/tests/x87_arithmetic_reference.c';host=output/'x86-observer';data=output/'x86-observations.bin'
    subprocess.run(['clang','-arch','x86_64','-O1','-Wall','-Wextra','-Werror',str(source),'-o',str(host)],check=True)
    with data.open('wb') as stream:subprocess.run(['arch','-x86_64',str(host)],stdout=stream,check=True,timeout=30)
    if data.stat().st_size!=19200*40:raise ValueError('incomplete actual x86 observation')
    env=os.environ.copy();env.update(ALKYHOL_X87_ARITHMETIC_REFERENCE=str(data),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    command=[str(Path.home()/'.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib','fpu_math::arithmetic_reference::actual_x86_arithmetic_observations','--','--ignored','--exact']
    result=subprocess.run(command,env=env,cwd=ROOT)
    report=dict(backend='physical native ARM arithmetic vs actual x86/Rosetta assembly',source_sha256=sha(source),observer_sha256=sha(host),observation_sha256=sha(data),rows=19200,model_status=result.returncode,all_rows_match=result.returncode==0,host=platform.platform(),revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    result.check_returncode()
if __name__=='__main__':check()
