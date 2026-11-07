#!/usr/bin/env python3
"""Compare actual COM command-list GPU copies on native WARP and restricted Metal."""
import argparse
import json
from pathlib import Path
if __package__:
    from .run_cpp_device_buffers import ROOT,run_probe
else:
    from run_cpp_device_buffers import ROOT,run_probe

SOURCE='cpp/tests/command_copy.cpp'
def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!={'scenario','results','buffers','deferred','parent_lifetime'} or result['scenario']!='command-copy-v1':
        raise ValueError('unknown command-copy observation')
    rows=result['results']
    if not isinstance(rows,list) or len(rows)!=11 or any(type(v) is not int or not 0<=v<2**32 for v in rows):
        raise ValueError('incomplete command-list state observations')
    buffers=result['buffers']
    if not isinstance(buffers,list) or len(buffers)!=2:
        raise ValueError('two complete GPU buffer observations required')
    for buffer in buffers:
        if not isinstance(buffer,str) or len(buffer)!=2048 or any(c not in '0123456789abcdef' for c in buffer):
            raise ValueError('invalid or truncated copied buffer bytes')
    if result['deferred'] is not True or result['parent_lifetime'] is not True:
        raise ValueError('command-list execution/lifetime checks failed')
    return result

def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP command copies','Rust restricted Metal COM command copies'}:
        raise ValueError('one independent native and one Metal GPU-copy report required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:
            raise ValueError('stale or dirty command-copy execution')
        validate_observation(json.dumps(report['observation']))
    if not reports[0]['artifacts'].get(SOURCE) or reports[0]['artifacts'][SOURCE]!=reports[1]['artifacts'].get(SOURCE):
        raise ValueError('command-copy probe source bytes differ')
    if reports[0]['observation']!=reports[1]['observation']:
        raise ValueError('native and Metal command-list GPU-copy observations differ')
    frozen=json.loads((ROOT/'cpp/reference/command-copy-windows-v1.json').read_text())
    if reports[0]['observation']!=frozen['observation']:
        raise ValueError('live command-copy behavior differs from frozen native contract')

def run(output):
    return run_probe(output,'command_copy','command copies',validate_observation,
                     ['trap-draw','trap-dispatch','trap-barrier','trap-clear','trap-root','trap-indirect'])

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
