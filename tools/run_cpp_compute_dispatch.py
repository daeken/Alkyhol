#!/usr/bin/env python3
"""Compare actual COM compute dispatch/readback on native SDK and Rust Metal."""
import argparse
import hashlib
import json
from pathlib import Path
if __package__:
    from .run_cpp_compute_pipelines import DXIL_SHA256
    from .run_cpp_device_buffers import ROOT,run_probe
else:
    from run_cpp_compute_pipelines import DXIL_SHA256
    from run_cpp_device_buffers import ROOT,run_probe
INPUTS=('cpp/tests/compute_dispatch.cpp','cpp/tests/compute_probe.dxil','cpp/tests/compute_probe_dxil.h','tools/build_cpp_compute_probe.py','shader/tests/fixtures/compute.hlsl')
def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!={'scenario','cases','queues','root_offsets','reset','zero_dispatch','binding_snapshots','same_root_preserved'} or result['scenario']!='compute-dispatch-v1':raise ValueError('unknown compute dispatch observation')
    if type(result['queues']) is not int or result['queues']!=2 or result['root_offsets']!=[256,16,8,16] or any(type(v) is not int for v in result['root_offsets']):raise ValueError('missing queues/root offsets')
    if any(result[k] is not True for k in ['reset','zero_dispatch','binding_snapshots','same_root_preserved']):raise ValueError('compute state sequence did not complete')
    if not isinstance(result['cases'],list) or len(result['cases'])!=6 or any(not isinstance(row,list) or len(row)!=104 or any(type(v) is not int or not 0<=v<2**32 for v in row) for row in result['cases']):raise ValueError('six complete 104-word readbacks required')
    return result
def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP compute dispatch','Rust restricted Metal COM compute dispatch'}:raise ValueError('independent native and Rust dispatch reports required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:raise ValueError('stale or dirty dispatch execution')
        validate_observation(json.dumps(report['observation']))
        if report.get('shader_dxil_sha256')!=DXIL_SHA256:raise ValueError('different DXIL dispatch fixture')
    for name in INPUTS:
        if not reports[0]['artifacts'].get(name) or reports[0]['artifacts'][name]!=reports[1]['artifacts'].get(name):raise ValueError('dispatch observer/fixture identity differs')
    if reports[0]['observation']!=reports[1]['observation']:raise ValueError('native and Rust compute data/guards/state sequence differ')
def run(output):
    report=run_probe(output,'compute_dispatch','compute dispatch',validate_observation)
    report['shader_dxil_sha256']=hashlib.sha256((ROOT/'cpp/tests/compute_probe.dxil').read_bytes()).hexdigest()
    if report['shader_dxil_sha256']!=DXIL_SHA256:raise ValueError('unexpected dispatch fixture')
    output.write_text(json.dumps(report,indent=2)+'\n');return report
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
