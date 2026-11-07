#!/usr/bin/env python3
"""Compare finite root serialization/COM objects, without GPU binding claims."""
import argparse
import json
from pathlib import Path
if __package__:
    from .run_cpp_device_buffers import run_probe
else:
    from run_cpp_device_buffers import run_probe
SOURCE='cpp/tests/root_signatures.cpp'
def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!={'scenario','layouts','versioned_identical','metadata','parent_lifetime'} or result['scenario']!='root-signatures-v1':raise ValueError('unknown root signature observation')
    if any(result[k] is not True for k in ['versioned_identical','metadata','parent_lifetime']):raise ValueError('root ownership/metadata/versioned serialization checks failed')
    if not isinstance(result['layouts'],list) or len(result['layouts'])!=6:raise ValueError('six complete root containers required')
    for row in result['layouts']:
        if not isinstance(row,list) or not 68<=len(row)<=1048576 or any(type(v) is not int or not 0<=v<256 for v in row):raise ValueError('complete unsigned serialized root bytes required')
    return result
def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP root signatures','Rust restricted Metal COM root signatures'}:raise ValueError('independent native and Rust root observations required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:raise ValueError('stale or dirty root execution')
        validate_observation(json.dumps(report['observation']))
    if not reports[0]['artifacts'].get(SOURCE) or reports[0]['artifacts'][SOURCE]!=reports[1]['artifacts'].get(SOURCE):raise ValueError('root observer source differs')
    if reports[0]['observation']!=reports[1]['observation']:raise ValueError('native and Rust serialized root bytes or COM assertions differ')
def run(output):return run_probe(output,'root_signatures','root signatures',validate_observation)
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
