#!/usr/bin/env python3
"""Regenerate the shared compute PSO fixture with the pinned open DXC."""
import argparse
from pathlib import Path
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def build(dxc,check=False):
    source=ROOT/'shader/tests/fixtures/compute.hlsl'
    with tempfile.TemporaryDirectory(prefix='alkyhol-compute-pso-') as scratch:
        binary=Path(scratch)/'compute.dxil'
        subprocess.run([str(dxc),'-T','cs_6_0','-E','CSMain','-Ges','-WX','-Fo',str(binary),str(source)],check=True,timeout=60)
        data=binary.read_bytes()
    lines=['// Generated from shader/tests/fixtures/compute.hlsl with pinned open DXC.',
           '// Copyright (c) Alkyhol contributors. MIT licensed.','#pragma once','static const unsigned char compute_probe_dxil[] = {']
    lines += ['    '+','.join(f'0x{byte:02x}' for byte in data[i:i+16])+',' for i in range(0,len(data),16)]
    outputs={ROOT/'cpp/tests/compute_probe.dxil':data,ROOT/'cpp/tests/compute_probe_dxil.h':'\n'.join([*lines,'};','']).encode()}
    for target,content in outputs.items():
        if check:
            if target.read_bytes()!=content:raise ValueError('compute PSO fixture differs from exact open-DXC regeneration: '+target.name)
        else:target.write_bytes(content)
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--dxc',type=Path,default=ROOT/'local/dxc-build/bin/dxc');parser.add_argument('--check',action='store_true')
    args=parser.parse_args();build(args.dxc.resolve(),args.check)
