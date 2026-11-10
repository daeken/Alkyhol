#!/usr/bin/env python3
"""Compare reached x87 SHLD/SHRD with independent actual x86/Rosetta."""
import json,os,platform,struct,subprocess
from check_jit32_sse import ROOT,sha

def check():
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires physical Apple Silicon and Rosetta')
    output=ROOT/'local/jit32/x87-double-shift-reference';output.mkdir(parents=True,exist_ok=True)
    source=ROOT/'jit32/tests/double_shift_reference.c';host=output/'x86-observer';data=output/'x86-observations.bin'
    subprocess.run(['clang','-arch','x86_64','-O1','-Wall','-Wextra','-Werror',str(source),'-o',str(host)],check=True)
    with data.open('wb') as stream:subprocess.run(['arch','-x86_64',str(host)],stdout=stream,check=True,timeout=30)
    if data.stat().st_size!=8320*32:raise ValueError('incomplete actual x86 observation')
    write_data=output/'x86-readonly-observations.bin';probe_env=os.environ.copy();probe_env['ALKYHOL_DOUBLE_SHIFT_WRITE_CHECK']='1'
    with write_data.open('wb') as stream:subprocess.run(['arch','-x86_64',str(host)],stdout=stream,env=probe_env,check=True,timeout=30)
    rows=list(struct.iter_unpack('<III',write_data.read_bytes()))
    if len(rows)!=4 or any(op!=i//2 or count!=i%2 or os.waitstatus_to_exitcode(status) not in (74,75) for i,(op,count,status) in enumerate(rows)):raise ValueError('readonly shift observation differs')
    env=os.environ.copy();env.update(ALKYHOL_DOUBLE_SHIFT_REFERENCE=str(data),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    subprocess.run([os.path.expanduser('~/.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib','double_shift_tests::actual_x86_double_shift_observations','--','--ignored','--exact'],env=env,cwd=ROOT,check=True)
    (output/'report.json').write_text(json.dumps({'backend':'physical native ARM JIT vs actual x86/Rosetta assembly','source_sha256':sha(source),'observer_sha256':sha(host),'observation_sha256':sha(data),'rows':8320,'readonly_observation_sha256':sha(write_data),'readonly_rows':rows,'host':platform.platform(),'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()},indent=2)+'\n')
    print('8320 actual Rosetta/native ARM SHLD/SHRD rows agree for immediate and CL forms; undefined AF/OF not asserted')
if __name__=='__main__':check()
