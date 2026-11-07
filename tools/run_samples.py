#!/usr/bin/env python3
"""Build/run the same C contract probe on the current host, in a fresh directory."""
import argparse
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = dict.fromkeys(('file_roundtrip', 'eof', 'truncate', 'errors', 'auto_reset', 'manual_reset', 'clock_monotonic'), True)

def run(output=None):
    with tempfile.TemporaryDirectory(prefix='alkyhol-sample-') as scratch:
        exe = Path(scratch) / ('kernel_contract.exe' if os.name == 'nt' else 'kernel_contract')
        source = ROOT / 'samples/kernel_contract.c'
        if os.name == 'nt':
            command = ['cl', '/nologo', '/W4', '/WX', str(source), '/Fe:' + str(exe)]
        else:
            library = ROOT / 'target/debug'
            command = [os.environ.get('CC', 'cc'), '-std=c11', '-Wall', '-Wextra', '-Werror', '-I' + str(ROOT / 'include'),
                       str(source), '-L' + str(library), '-lalkyhol', '-Wl,-rpath,' + str(library), '-o', str(exe)]
        subprocess.run(command, check=True, cwd=scratch)
        result = subprocess.run([str(exe)], check=True, cwd=scratch, capture_output=True, text=True)
        observation = json.loads(result.stdout)
        if observation != EXPECTED or any(type(v) is not bool for v in observation.values()):
            raise RuntimeError(f'unexpected observation: {observation}')
        report = {'schema_version': 1, 'host': platform.platform(), 'machine': platform.machine(),
                  'backend': 'Windows SDK' if os.name == 'nt' else 'Alkyhol native C ABI', 'observation': observation}
        if output: output.write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))

if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--output', type=Path)
    run(p.parse_args().output)
