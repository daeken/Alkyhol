#!/usr/bin/env python3
"""Compare DEFAULT-buffer GPU copies, transitions and decay against native WARP."""
import argparse
import json
from pathlib import Path
if __package__:
    from .run_cpp_device_buffers import ROOT,run_probe
else:
    from run_cpp_device_buffers import ROOT,run_probe
SOURCE='cpp/tests/default_buffers.cpp'
def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!={'scenario','map_hr','heap','heap_flags','desc','fences','buffers'} or result['scenario']!='default-buffer-transitions-v1':
        raise ValueError('unknown DEFAULT-buffer observation')
    for name in ['map_hr','heap_flags']:
        if type(result[name]) is not int or not 0<=result[name]<2**32:raise ValueError('invalid returned DEFAULT-buffer value')
    for name,count in [('heap',5),('desc',11),('fences',3)]:
        if not isinstance(result[name],list) or len(result[name])!=count or any(type(v) is not int or not 0<=v<2**64 for v in result[name]):raise ValueError('invalid or incomplete DEFAULT-buffer words')
    if not isinstance(result['buffers'],list) or len(result['buffers'])!=4:raise ValueError('four complete DEFAULT-buffer readbacks required')
    for buffer in result['buffers']:
        if not isinstance(buffer,str) or len(buffer)!=2048 or any(c not in '0123456789abcdef' for c in buffer):raise ValueError('invalid DEFAULT-buffer readback bytes')
    return result
def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP DEFAULT buffers','Rust restricted Metal COM DEFAULT buffers'}:raise ValueError('independent native and Metal DEFAULT-buffer reports required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:raise ValueError('stale or dirty DEFAULT-buffer execution')
        validate_observation(json.dumps(report['observation']))
    if not reports[0]['artifacts'].get(SOURCE) or reports[0]['artifacts'][SOURCE]!=reports[1]['artifacts'].get(SOURCE):raise ValueError('DEFAULT-buffer probe source bytes differ')
    if reports[0]['observation']!=reports[1]['observation']:raise ValueError('native and Metal DEFAULT-buffer observations differ')
    frozen=json.loads((ROOT/'cpp/reference/default-buffer-transitions-windows-v1.json').read_text())
    if reports[0]['observation']!=frozen['observation']:raise ValueError('live DEFAULT-buffer behavior differs from frozen native contract')
def run(output):
    return run_probe(output,'default_buffers','DEFAULT buffers',validate_observation,['trap-default-unmap'])
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
