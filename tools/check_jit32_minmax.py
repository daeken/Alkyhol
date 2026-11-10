#!/usr/bin/env python3
"""Frozen exact legacy SIMD minimum/maximum result and state observations."""
import argparse,gzip,json,os,platform,subprocess
from pathlib import Path
from check_jit32_sse import ROOT,sha

def check():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--backend',choices=['windows-amd64','rosetta'],default='rosetta')
    args=parser.parse_args()
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires native Apple Silicon JIT')
    reference=ROOT/f'jit32/reference/minmax-{args.backend}-v1.json';metadata=json.loads(reference.read_text())
    fixture=ROOT/metadata['fixture'];source=ROOT/metadata['source']
    if sha(fixture)!=metadata['fixture_sha256'] or sha(source)!=metadata['source_lf_sha256']:raise ValueError('frozen input identity differs')
    if not metadata['result_bit_equality_required'] or not metadata['masked_exception_observations']:raise ValueError('frozen exact masked contract differs')
    raw=gzip.decompress(fixture.read_bytes());output=ROOT/f'local/jit32/minmax-{args.backend}-comparison';output.mkdir(parents=True,exist_ok=True)
    data=output/'observations.bin';data.write_bytes(raw)
    if len(raw)!=65536*80 or sha(data)!=metadata['observation_sha256']:raise ValueError('frozen observation bytes differ')
    model=output/'model.json';model.unlink(missing_ok=True)
    env=os.environ.copy();env.update(ALKYHOL_MINMAX_REFERENCE=str(data),ALKYHOL_MINMAX_REPORT=str(model),CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    command=[str(Path.home()/'.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib','sse_minmax::native_tests::actual_x86_minmax_observations','--','--ignored','--exact','--nocapture']
    result=subprocess.run(command,env=env,cwd=ROOT)
    report=dict(backend='native ARM exact SIMD min/max vs '+metadata['backend'],reference_sha256=sha(reference),observation_sha256=sha(data),model_status=result.returncode,comparison_passed=result.returncode==0,model=json.loads(model.read_text()) if model.exists() else None,revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    if sha(fixture)!=metadata['fixture_sha256']:raise ValueError('frozen input changed during comparison')
    result.check_returncode()
    if report['model'] is None or report['model']['rows']!=65536:raise ValueError('complete model report missing')
if __name__=='__main__':check()
