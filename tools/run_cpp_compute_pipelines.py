#!/usr/bin/env python3
"""Observe real compute PSO construction/COM ownership, not D3D12 Dispatch."""
import argparse
import hashlib
import json
from pathlib import Path
if __package__:
    from .run_cpp_device_buffers import ROOT,run_probe
else:
    from run_cpp_device_buffers import ROOT,run_probe
INPUTS=('cpp/tests/compute_pipelines.cpp','cpp/tests/compute_probe.dxil','cpp/tests/compute_probe_dxil.h',
        'tools/build_cpp_compute_probe.py','shader/tests/fixtures/compute.hlsl')
DXIL_SHA256='e3084e25dfa23ef53e98eb1dd6eac386c3695718ac03d6631904b6711cc11d4e'
def validate_observation(text):
    result=json.loads(text)
    expected={'scenario':'compute-pso-v1','created':2,'capability_checked':True,'canonical_identity':True,'pageable':True,'metadata':True,'parent_lifetime':True}
    if result!=expected or any(type(result[k]) is not type(v) for k,v in expected.items()):raise ValueError('compute PSO did not complete construction/COM checks')
    return result
def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP compute pipelines','Rust restricted Metal COM compute pipelines'}:raise ValueError('independent native/Rust compute PSO reports required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:raise ValueError('stale or dirty compute PSO execution')
        validate_observation(json.dumps(report['observation']))
        if report.get('shader_dxil_sha256')!=DXIL_SHA256:raise ValueError('unexpected compute PSO shader bytes')
    for name in INPUTS:
        if not reports[0]['artifacts'].get(name) or reports[0]['artifacts'][name]!=reports[1]['artifacts'].get(name):raise ValueError('compute PSO observer/fixture identity differs')
    if reports[0]['observation']!=reports[1]['observation']:raise ValueError('native and Rust compute PSO observations differ')
def run(output):
    report=run_probe(output,'compute_pipelines','compute pipelines',validate_observation)
    report['shader_dxil_sha256']=hashlib.sha256((ROOT/'cpp/tests/compute_probe.dxil').read_bytes()).hexdigest()
    if report['shader_dxil_sha256']!=DXIL_SHA256:raise ValueError('PSO fixture differs from the established compute DXIL')
    output.write_text(json.dumps(report,indent=2)+'\n');return report
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
