#!/usr/bin/env python3
"""Match all native Windows AMD SIMD observations, preserving the frozen input."""
import gzip,json,os,platform,subprocess
from pathlib import Path
from check_jit32_sse import ROOT,sha
def check():
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires physical Apple Silicon native ARM JIT')
    reference=ROOT/'jit32/reference/sse-compare-windows-amd64-v1.json';r=json.loads(reference.read_text())
    fixture=ROOT/r['fixture'];source=ROOT/r['source']
    if sha(fixture)!=r['fixture_sha256'] or sha(source)!=r['source_lf_sha256']:raise ValueError('frozen fixture or observer source identity differs')
    raw=gzip.decompress(fixture.read_bytes());output=ROOT/'local/jit32/windows-compare-reference';output.mkdir(parents=True,exist_ok=True);data=output/'native-observations.bin';data.write_bytes(raw)
    if sha(data)!=r['observation_sha256'] or len(raw)!=18432*72:raise ValueError('native observation identity differs')
    env=os.environ.copy();env.update(ALKYHOL_COMPARE_REFERENCE=str(data),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    command=[str(Path.home()/'.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib','sse_compare::tests::actual_x86_comparison_observations','--','--ignored','--exact']
    result=subprocess.run(command,env=env,cwd=ROOT)
    report=dict(backend='physical native ARM JIT vs frozen native Windows AMD64 assembly',native_reference_sha256=sha(reference),observation_sha256=sha(data),rows=18432,model_status=result.returncode,all_rows_match=result.returncode==0,host=platform.platform(),revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    result.check_returncode()
    if sha(fixture)!=r['fixture_sha256']:raise ValueError('frozen fixture changed')
    print('18432 frozen native Windows AMD/native ARM SIMD comparison rows agree; Rosetta differences remain separate')
if __name__=='__main__':check()
