#!/usr/bin/env python3
"""Record native loading/cookie behavior for frozen owned directory/record length pairs."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import tempfile
from build_pe_fixture import ROOT
from build_pe32_load_config_geometry import SOURCES,BASE_REFERENCE,variants
from run_pe32_load_config import capture

NATIVE='native Windows PE32 load-config lengths'
HOST='Alkyhol PE32 load-config lengths / x86-64 Mach-O'
REFERENCE=ROOT/'pe/reference/pe32-load-config-geometry-windows-v1.json'


def verify(directory):
    manifest=json.loads((directory/'load-config-geometry-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES):raise ValueError('changed geometry schema/source identities')
    reference=json.loads(BASE_REFERENCE.read_text(encoding='utf-8'));bases={}
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed geometry source')
    for name,digest in reference['sources'].items():
        if manifest['sources'][name]!=digest:raise ValueError('changed frozen canonical source')
    for size in [64,72]:
        data=(directory/f'base{size}.exe').read_bytes()
        expected=next(row for row in reference['cases'] if row['case']==f'legacy{size}_default')['pe_sha256']
        if hashlib.sha256(data).hexdigest()!=expected:raise ValueError('changed frozen canonical PE')
        bases[size]=data
    items=variants(bases)
    if manifest['cases']!=[row for row,_ in items]:raise ValueError('changed length-pair identity/order')
    for row,data in items:
        if (directory/(row['case']+'.exe')).read_bytes()!=data:raise ValueError('more than directory length changed')
    return manifest,items


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    manifest,items=verify(args.directory.resolve())
    if platform.system()!='Windows':raise RuntimeError('initial geometry collector requires Windows')
    observations=[]
    with tempfile.TemporaryDirectory(prefix='alkyhol-load-config-lengths-') as temp:
        for row,data in items:
            path=Path(temp)/(row['case']+'.exe');path.write_bytes(data)
            observations.append({'case':row['case'],**capture([str(path)],temp)})
    baseline=json.loads(BASE_REFERENCE.read_text(encoding='utf-8'))['normal'][2]
    if {k:v for k,v in observations[0].items() if k!='case'}!={k:v for k,v in baseline.items() if k!='case'}:raise ValueError('canonical geometry baseline differs')
    report=dict(schema_version=1,backend=NATIVE,sources=manifest['sources'],cases=manifest['cases'],normal=observations,rebased=None,
                revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
                tracked_tree_clean=not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip())
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded native directory/record length pairs; host policy comparison is separate')


if __name__=='__main__':main()
