#!/usr/bin/env python3
"""Compare native ARM sign-mask emission with independent actual x86 assembly."""
import argparse
import gzip
import json
import os
import platform
from pathlib import Path
import subprocess
from check_jit32_sse import ROOT, sha


def compare(data, output, reference=None):
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
    if reference:
        report['reference']=reference
    (output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n')
    if result.returncode:
        raise ValueError('actual x86 sign-mask observations differ')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    modes=parser.add_mutually_exclusive_group()
    modes.add_argument('--data',type=Path)
    modes.add_argument('--backend',choices=['windows-amd64','rosetta'])
    modes.add_argument('--observe-rosetta',action='store_true')
    parser.add_argument('--output',type=Path)
    args=parser.parse_args();backend=args.backend or 'windows-amd64'
    output=(args.output or ROOT/('local/jit32/movemask-rosetta' if args.observe_rosetta else f'local/jit32/movemask-{backend}-comparison')).resolve();output.mkdir(parents=True,exist_ok=True)
    reference=None
    if args.data:
        data=args.data.resolve()
    elif args.observe_rosetta:
        source=ROOT/'jit32/tests/movemask_reference.c';observer=output/'x86-observer'
        data=output/'observations.bin'
        subprocess.run(['clang','-arch','x86_64','-O1','-mno-red-zone','-Wall','-Wextra','-Werror',
                        str(source),'-o',str(observer)],check=True)
        with data.open('wb') as stream:
            p=subprocess.run(['arch','-x86_64',str(observer)],stdout=stream,stderr=subprocess.PIPE,timeout=30,check=True)
        (output/'observer.json').write_text(json.dumps(dict(backend='actual x86 assembly under Rosetta',
            source_sha256=sha(source),observer_sha256=sha(observer),observation_sha256=sha(data),
            rows=65556,stderr_hex=p.stderr.hex(),platform=platform.platform()),indent=2)+'\n')
    else:
        path=ROOT/f'jit32/reference/movemask-{backend}-v1.json';reference=json.loads(path.read_text())
        fixture=ROOT/reference['fixture'];source=ROOT/reference['source']
        if (sha(fixture)!=reference['fixture_sha256'] or sha(source)!=reference['source_lf_sha256']
                or not reference['exact_results_and_flags_required'] or not reference['strict_requested_mxcsr_load']):
            raise ValueError('frozen sign-mask source/artifact/strict contract differs')
        data=output/'observations.bin';data.write_bytes(gzip.decompress(fixture.read_bytes()))
        if sha(data)!=reference['observation_sha256']:
            raise ValueError('frozen sign-mask observation bytes differ')
    compare(data,output,reference)
