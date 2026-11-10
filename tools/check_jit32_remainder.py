#!/usr/bin/env python3
"""Strict frozen complete finite FPREM/FPREM1 comparison; Rosetta differences remain failures."""
import argparse,gzip,json,os,platform,subprocess
from pathlib import Path
from check_jit32_sse import ROOT,sha

def check():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--backend',choices=['windows-amd64','rosetta'],default='windows-amd64');args=p.parse_args()
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires native Apple Silicon JIT')
    reference=ROOT/f'jit32/reference/x87-remainder-{args.backend}-v1.json';m=json.loads(reference.read_text());fixture=ROOT/m['fixture'];source=ROOT/m['source']
    if sha(fixture)!=m['fixture_sha256'] or sha(source)!=m['source_lf_sha256']:raise ValueError('frozen input identity differs')
    if not all(m.get(k) is True for k in ['complete_finite_reductions_only','masked_exception_observations','initial_environment_observed','strict_equality_required']):raise ValueError('frozen contract differs')
    output=ROOT/f'local/jit32/remainder-{args.backend}-comparison';output.mkdir(parents=True,exist_ok=True);data=output/'observations.bin';raw=gzip.decompress(fixture.read_bytes());data.write_bytes(raw)
    if len(raw)!=24576*80 or sha(data)!=m['observation_sha256']:raise ValueError('frozen observation differs')
    model=output/'model.json';model.unlink(missing_ok=True)
    env=dict(os.environ,ALKYHOL_REMAINDER_REFERENCE=str(data),ALKYHOL_REMAINDER_REPORT=str(model),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    command=[str(Path.home()/'.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib','fpu_remainder::native_tests::actual_x86_complete_remainder_observations','--','--ignored','--exact','--nocapture']
    result=subprocess.run(command,cwd=ROOT,env=env);report=dict(backend='native ARM complete finite x87 remainder vs '+m['backend'],reference_sha256=sha(reference),observation_sha256=sha(data),model_status=result.returncode,comparison_passed=result.returncode==0,model=json.loads(model.read_text()) if model.exists() else None,revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    if sha(fixture)!=m['fixture_sha256']:raise ValueError('frozen input mutated during comparison')
    if report['model'] is None or report['model']['rows']!=24576:raise ValueError('complete model report missing')
    result.check_returncode()
if __name__=='__main__':check()
