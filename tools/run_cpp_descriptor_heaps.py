#!/usr/bin/env python3
"""Compare descriptor heap allocation/handle invariants, without initialized views."""
import argparse
import json
from pathlib import Path
if __package__:
    from .run_cpp_device_buffers import run_probe
else:
    from run_cpp_device_buffers import run_probe
SOURCE='cpp/tests/descriptor_heaps.cpp'
def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!={'scenario','strides','heaps','parent_lifetime'} or result['scenario']!='descriptor-heaps-v1':raise ValueError('unknown descriptor-heap observation')
    if not isinstance(result['strides'],list) or len(result['strides'])!=4 or any(type(v) is not int or not 0<v<2**32 for v in result['strides']):raise ValueError('four actual positive backend strides required')
    if result['parent_lifetime'] is not True or not isinstance(result['heaps'],list) or len(result['heaps'])!=12:raise ValueError('incomplete descriptor-heap ownership observations')
    for row in result['heaps']:
        if not isinstance(row,dict) or set(row)!={'desc','cpu_present','gpu_present','last_cpu_index','last_gpu_index'}:raise ValueError('invalid descriptor-heap fields')
        if not isinstance(row['desc'],list) or len(row['desc'])!=4 or any(type(v) is not int or not 0<=v<2**32 for v in row['desc']):raise ValueError('invalid returned descriptor-heap words')
        if row['cpu_present'] is not True or type(row['gpu_present']) is not bool:raise ValueError('invalid handle presence')
        for name in ['last_cpu_index','last_gpu_index']:
            if type(row[name]) is not int or not 0<=row[name]<2**32:raise ValueError('invalid descriptor index offsets')
    return result
def semantic_observation(observation):
    # Only opaque driver stride bytes differ. C++ independently computes index
    # offsets using the returned actual stride; descriptors/indices stay exact.
    return {name:value for name,value in observation.items() if name!='strides'}
def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP descriptor heaps','Rust restricted Metal COM descriptor heaps'}:raise ValueError('independent native and Metal descriptor-heap reports required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:raise ValueError('stale or dirty descriptor-heap execution')
        validate_observation(json.dumps(report['observation']))
    if not reports[0]['artifacts'].get(SOURCE) or reports[0]['artifacts'][SOURCE]!=reports[1]['artifacts'].get(SOURCE):raise ValueError('descriptor-heap probe bytes differ')
    if semantic_observation(reports[0]['observation'])!=semantic_observation(reports[1]['observation']):raise ValueError('native and Metal descriptor-heap invariants differ')
def run(output):return run_probe(output,'descriptor_heaps','descriptor heaps',validate_observation)
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
