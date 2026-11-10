#!/usr/bin/env python3
"""Frozen FPATAN math-profile comparison, or an explicitly strict CPU diagnostic."""
import argparse,gzip,json,os,platform,subprocess
from pathlib import Path
from check_jit32_sse import ROOT,sha

def check():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--backend',choices=['windows-amd64','rosetta'],default='windows-amd64');parser.add_argument('--strict',action='store_true');args=parser.parse_args()
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires native Apple Silicon JIT')
    reference=ROOT/f'jit32/reference/x87-atan2-{args.backend}-v1.json';metadata=json.loads(reference.read_text())
    fixture=ROOT/metadata['fixture'];source=ROOT/metadata['source']
    if sha(fixture)!=metadata['fixture_sha256'] or sha(source)!=metadata['source_lf_sha256']:raise ValueError('frozen input identity differs')
    raw=gzip.decompress(fixture.read_bytes());output=ROOT/f"local/jit32/atan2-{args.backend}-{'strict' if args.strict else 'math'}"
    output.mkdir(parents=True,exist_ok=True);data=output/'observations.bin';data.write_bytes(raw)
    if len(raw)!=768*64 or sha(data)!=metadata['observation_sha256']:raise ValueError('frozen observation bytes differ')
    model=output/'model.json';model.unlink(missing_ok=True)
    env=os.environ.copy();env.update(ALKYHOL_ATAN2_REFERENCE=str(data),ALKYHOL_ATAN2_REPORT=str(model),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    test='actual_x86_fpatan_strict_observations' if args.strict else 'actual_x86_fpatan_math_profile_observations'
    command=[str(Path.home()/'.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib',f'atan2_tests::{test}','--','--ignored','--exact','--nocapture']
    result=subprocess.run(command,env=env,cwd=ROOT)
    report=dict(backend='native ARM mathematical FPATAN vs '+metadata['backend'],reference_sha256=sha(reference),observation_sha256=sha(data),strict_comparison=args.strict,model_status=result.returncode,comparison_passed=result.returncode==0,model=json.loads(model.read_text()) if model.exists() else None,revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    if sha(fixture)!=metadata['fixture_sha256']:raise ValueError('frozen input changed during comparison')
    result.check_returncode()
if __name__=='__main__':check()
