#!/usr/bin/env python3
"""Compare byte-granular COM copies on native WARP and actual Rust/Metal GPUs."""
import argparse
import json
from pathlib import Path
if __package__:
    from .run_cpp_device_buffers import run_probe
else:
    from run_cpp_device_buffers import run_probe
SOURCE='cpp/tests/byte_copies.cpp'
RESIDUE_BYTES=35502
def validate_observation(text):
    result=json.loads(text)
    if not isinstance(result,dict) or set(result)!={'scenario','cases','residue_buffer','overlay_buffer','large_bytes_checked','large_checksum','large_windows','deferred'} or result['scenario']!='byte-copies-v1':raise ValueError('unknown byte-copy observation')
    if type(result['cases']) is not int or result['cases']!=272 or type(result['large_bytes_checked']) is not int or result['large_bytes_checked']!=16*1024*1024+97 or result['deferred'] is not True:raise ValueError('incomplete byte-copy workload')
    if type(result['large_checksum']) is not int or not 0<=result['large_checksum']<2**32:raise ValueError('invalid large-copy checksum')
    def hexadecimal(value,count):
        if not isinstance(value,str) or len(value)!=count*2 or any(c not in '0123456789abcdef' for c in value):raise ValueError('invalid or truncated byte-copy readback')
    hexadecimal(result['residue_buffer'],RESIDUE_BYTES);hexadecimal(result['overlay_buffer'],4099)
    if not isinstance(result['large_windows'],list) or len(result['large_windows'])!=3:raise ValueError('all large-copy boundary windows required')
    for value,count in zip(result['large_windows'],[32,64,32]):hexadecimal(value,count)
    return result
def compare_reports(reports,revision):
    if len(reports)!=2 or {r.get('backend') for r in reports}!={'native Windows SDK WARP byte copies','Rust restricted Metal COM byte copies'}:raise ValueError('independent native and Metal byte-copy reports required')
    for report in reports:
        if report.get('schema_version')!=1 or report.get('source_revision')!=revision or report.get('tracked_tree_clean') is not True:raise ValueError('stale or dirty byte-copy execution')
        validate_observation(json.dumps(report['observation']))
    if not reports[0]['artifacts'].get(SOURCE) or reports[0]['artifacts'][SOURCE]!=reports[1]['artifacts'].get(SOURCE):raise ValueError('byte-copy probe source bytes differ')
    if reports[0]['observation']!=reports[1]['observation']:raise ValueError('native and Metal byte-copy observations differ')
def run(output):return run_probe(output,'byte_copies','byte copies',validate_observation)
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--output',type=Path,required=True)
    run(parser.parse_args().output)
