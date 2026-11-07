#!/usr/bin/env python3
"""Regenerate the finite native CBV consumer with pinned open DXC."""
import argparse
from pathlib import Path
import subprocess
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def build(dxc,check=False):
    source=ROOT/'cpp/tests/cbv_probe.hlsl';target=ROOT/'cpp/tests/cbv_probe_dxil.h'
    with tempfile.TemporaryDirectory(prefix='alkyhol-cbv-shader-') as scratch:
        binary=Path(scratch)/'probe.dxil'
        subprocess.run([str(dxc),'-T','cs_6_0','-E','main','-Ges','-WX','-Fo',str(binary),str(source)],check=True,timeout=60)
        data=binary.read_bytes()
    lines=['// Generated from cbv_probe.hlsl by tools/build_cpp_cbv_probe.py using open DXC.',
           '// Copyright (c) Alkyhol contributors. MIT licensed.','#pragma once','static const unsigned char cbv_probe_dxil[] = {']
    lines += ['    '+','.join(f'0x{byte:02x}' for byte in data[i:i+16])+',' for i in range(0,len(data),16)]
    content='\n'.join([*lines,'};',''])
    if check:
        if target.read_bytes()!=content.encode():raise ValueError('CBV probe DXIL does not match exact open-DXC regeneration')
    else:target.write_bytes(content.encode())
if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--dxc',type=Path,default=ROOT/'local/dxc-build/bin/dxc');parser.add_argument('--check',action='store_true')
    args=parser.parse_args();build(args.dxc.resolve(),args.check)
