#!/usr/bin/env python3
"""Compare native ARM sign-mask emission with independent actual x86 assembly."""
import argparse
import json
import os
import platform
from pathlib import Path
import subprocess
from check_jit32_sse import ROOT, sha


def compare(data, output):
    if platform.system()!='Darwin' or platform.machine()!='arm64':
        raise RuntimeError('requires physical Apple Silicon JIT')
    if data.stat().st_size!=65556*64:
        raise ValueError('incomplete independent mask observations')
    model=output/'model.json';model.unlink(missing_ok=True)
    env=dict(os.environ,ALKYHOL_MOVEMASK_REFERENCE=str(data),ALKYHOL_MOVEMASK_REPORT=str(model),
             CARGO_PROFILE_DEV_DEBUG='0',CARGO_INCREMENTAL='0')
    command=[str(Path.home()/'.cargo/bin/cargo'),'test','--manifest-path',str(ROOT/'jit32/Cargo.toml'),
             '--features','shared-runtime','--locked','--offline','--lib',
             'movemask_tests::actual_x86_movemask_observations','--','--ignored','--exact']
    result=subprocess.run(command,cwd=ROOT,env=env)
    report=dict(backend='physical native ARM JIT against supplied original x86 observations',
                observation_sha256=sha(data),rows=65556,status=result.returncode,
                model=json.loads(model.read_text()) if model.exists() else None,
                platform=platform.platform(),revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
    (output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
    if result.returncode:
        raise ValueError('actual x86 sign-mask observations differ')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--data',type=Path)
    parser.add_argument('--output',type=Path,default=ROOT/'local/jit32/movemask-rosetta')
    args=parser.parse_args();output=args.output.resolve();output.mkdir(parents=True,exist_ok=True)
    if args.data:
        data=args.data.resolve()
    else:
        source=ROOT/'jit32/tests/movemask_reference.c';observer=output/'x86-observer'
        data=output/'observations.bin'
        subprocess.run(['clang','-arch','x86_64','-O1','-mno-red-zone','-Wall','-Wextra','-Werror',
                        str(source),'-o',str(observer)],check=True)
        with data.open('wb') as stream:
            p=subprocess.run(['arch','-x86_64',str(observer)],stdout=stream,stderr=subprocess.PIPE,timeout=30,check=True)
        (output/'observer.json').write_text(json.dumps(dict(backend='actual x86 assembly under Rosetta',
            source_sha256=sha(source),observer_sha256=sha(observer),observation_sha256=sha(data),
            rows=65556,stderr_hex=p.stderr.hex(),platform=platform.platform()),indent=2)+'\n')
    compare(data,output)
