#!/usr/bin/env python3
"""Observe finite CPU/fence/Metal queue-completion behavior against native WARP."""
import argparse
import json
from pathlib import Path
if __package__:
    from .run_cpp_device_buffers import ROOT,run_probe
else:
    from run_cpp_device_buffers import ROOT,run_probe

SOURCE='cpp/tests/queue_fence.cpp'

def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!={'scenario','desc','values','waits','retained_submission','parent_lifetime'} or result['scenario']!='queue-fence-v1':
        raise ValueError('unknown synchronization observation')
    for name,count,limit in [('desc',4,2**32),('values',8,2**64),('waits',10,2**32)]:
        if not isinstance(result[name],list) or len(result[name])!=count or any(type(v) is not int or not 0<=v<limit for v in result[name]):
            raise ValueError('incomplete or invalid synchronization words')
    if result['retained_submission'] is not True or result['parent_lifetime'] is not True:
        raise ValueError('synchronization lifetime checks failed')
    # Values come from independently executed APIs. Do not synthesize descriptors
    # or native results here; shared C++ assertions and exact comparison gate them.
    return result

def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP queue/fence','Rust restricted Metal COM queue/fence'}:
        raise ValueError('one independent native and one Metal synchronization report required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:
            raise ValueError('stale or dirty synchronization execution')
        validate_observation(json.dumps(report['observation']))
    if not reports[0]['artifacts'].get(SOURCE) or reports[0]['artifacts'][SOURCE]!=reports[1]['artifacts'].get(SOURCE):
        raise ValueError('synchronization probe source bytes differ')
    if reports[0]['observation']!=reports[1]['observation']:
        raise ValueError('native and Metal synchronization observations differ')
    frozen=json.loads((ROOT/'cpp/reference/queue-fence-windows-v1.json').read_text())
    if reports[0]['observation']!=frozen['observation']:
        raise ValueError('live synchronization differs from frozen native contract')

def run(output):
    return run_probe(output,'queue_fence','queue/fence',validate_observation,
                     ['trap-execute','trap-marker','trap-begin','trap-end','trap-tiles','trap-copy-tiles'])

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
