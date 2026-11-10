#!/usr/bin/env python3
"""Frozen native Windows RSQRT architectural bounds and exact special/state checks."""
import gzip,json,os,platform,re,subprocess
from pathlib import Path
from check_jit32_sse import ROOT,sha

def check():
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires physical native Apple Silicon')
    reference=ROOT/'jit32/reference/rsqrt-windows-amd64-v1.json';r=json.loads(reference.read_text());fixture=ROOT/r['fixture'];source=ROOT/r['source']
    if sha(fixture)!=r['fixture_sha256'] or sha(source)!=r['source_lf_sha256']:raise ValueError('frozen source/fixture differs')
    if r['normal_relative_error_limit']!='3/8192' or r['normal_bit_equality_required']:raise ValueError('frozen bounded contract differs')
    output=ROOT/'local/jit32/windows-rsqrt-reference';output.mkdir(parents=True,exist_ok=True);data=output/'native-observations.bin';data.write_bytes(gzip.decompress(fixture.read_bytes()))
    if data.stat().st_size!=16640*72 or sha(data)!=r['observation_sha256']:raise ValueError('native observation identity differs')
    env=os.environ.copy();env.update(ALKYHOL_RSQRT_REFERENCE=str(data),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    command=[str(Path.home()/'.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib','sse_estimate::native_tests::actual_x86_estimate_observations','--','--ignored','--exact','--nocapture']
    result=subprocess.run(command,env=env,cwd=ROOT,capture_output=True,text=True)
    (output/'model.stdout.log').write_text(result.stdout);(output/'model.stderr.log').write_text(result.stderr)
    metrics=re.search(r'(\d+) positive-normal lane results; (\d+) differ in bits',result.stderr)
    report=dict(backend='physical native ARM RSQRT vs frozen native Windows AMD architectural contract',reference_sha256=sha(reference),rows=16640,normal_relative_error_limit='3/8192',normal_bit_equality_required=False,special_values_and_state_exact=True,model_status=result.returncode,all_contract_checks_pass=result.returncode==0,positive_normal_lanes=int(metrics[1]) if metrics else None,different_normal_result_bits=int(metrics[2]) if metrics else None,host=platform.platform(),revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    result.check_returncode()
    if not metrics or int(metrics[1])!=41120:raise ValueError('incomplete bounded result metrics')
    if sha(fixture)!=r['fixture_sha256']:raise ValueError('frozen fixture changed')
    print(f"16640 native Windows RSQRT contract checks pass; {metrics[2]}/{metrics[1]} normal results differ in bits, all within 3/8192 relative error")
if __name__=='__main__':check()
