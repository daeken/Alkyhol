#!/usr/bin/env python3
"""Observe CBV copy contents: native DXIL vs a bounded Metal record consumer.

This does not exercise general Alkyhol root signatures or D3D12 Dispatch.
"""
import argparse
import json
from pathlib import Path
if __package__:
    from .run_cpp_device_buffers import run_probe
else:
    from run_cpp_device_buffers import run_probe

INPUTS=('cpp/tests/cbv_copies.cpp','cpp/tests/cbv_probe.hlsl',
        'cpp/tests/cbv_probe_dxil.h','tools/build_cpp_cbv_probe.py')

def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!={'scenario','cases','snapshot_independent','gpu_consumed'} or result['scenario']!='cbv-copy-v1':
        raise ValueError('unknown CBV copy observation')
    if result['snapshot_independent'] is not True or result['gpu_consumed'] is not True:
        raise ValueError('GPU consumption and independent copied slots required')
    rows=result['cases']
    if not isinstance(rows,list) or len(rows)!=3:
        raise ValueError('three complete CBV copy readbacks required')
    for row in rows:
        if not isinstance(row,list) or len(row)!=24 or any(type(v) is not int or not 0<=v<2**32 for v in row):
            raise ValueError('24 exact unsigned data/guard words required')
    return result

def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP CBV copies','Rust restricted Metal COM CBV copies'}:
        raise ValueError('independent native and Metal CBV executions required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:
            raise ValueError('stale or dirty CBV execution')
        validate_observation(json.dumps(report['observation']))
    for name in INPUTS:
        if not reports[0]['artifacts'].get(name) or reports[0]['artifacts'][name]!=reports[1]['artifacts'].get(name):
            raise ValueError('CBV observer or shader fixture differs: '+name)
    if reports[0]['observation']!=reports[1]['observation']:
        raise ValueError('native and Metal copied CBV data/guard words differ')

def run(output):
    return run_probe(output,'cbv_copies','CBV copies',validate_observation)

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
