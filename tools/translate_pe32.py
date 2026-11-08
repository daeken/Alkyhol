#!/usr/bin/env python3
"""Package unchanged i386 PE32 in a 64-bit Mach-O with low guest address space."""
import argparse
from pathlib import Path
from translate_pe import ROOT,TARGET,embedded_payload,_package


def translate(source,output):
    return _package(source,output,ROOT/'pe/launcher32.rs',
                    'unchanged PE32 in x86-64 Mach-O / Alkyhol i386 loader',
                    ['-pagezero_size','0x1000'])


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('input',type=Path)
    p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    translate(args.input,args.output)
