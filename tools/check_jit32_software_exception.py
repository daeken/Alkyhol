#!/usr/bin/env python3
"""Exercise unchanged owned software-SEH PEs on the shared native ARM executor.

This is a finite host execution test, not a new native Windows observation.
"""
import argparse
import json
import subprocess
from pathlib import Path
from build_jit32_shared import ROOT, build as build_host
from observe_pe32_software_exception import build, decode, inputs, FILES


def check(output):
    output.mkdir(parents=True, exist_ok=True)
    build(output)
    identity = inputs(output)
    host = build_host(output / 'startup-arm64')
    rows = []
    for mode, name in enumerate(FILES):
        result = subprocess.run([str(host), str(output / name), '--strict', '--file-origins'],
                                capture_output=True, timeout=30)
        row = {'mode': mode, 'status': result.returncode,
               'stdout_hex': result.stdout.hex(), 'stderr_hex': result.stderr.hex()}
        rows.append(row)
        if mode == 0:
            if result.returncode:
                raise ValueError(f'native software exception failed: {row}')
            row['words'] = decode(result.stdout)
            if b'invocations: 3' not in result.stderr:
                raise ValueError('missing execution of both original handlers')
        else:
            reason = [None, b'unhandled software exception',
                      b'noncontinuable or oversized software exception',
                      b'mutated context/record/chain'][mode]
            if result.returncode != 78 or result.stdout or reason not in result.stderr:
                raise ValueError(f'unsupported software exception did not fail explicitly: {row}')
    if inputs(output) != identity:
        raise ValueError('owned source or PE changed during execution')
    (output / 'native-arm-software-exception.json').write_text(json.dumps(
        {'backend': 'physical native ARM shared JIT', 'inputs': identity, 'rows': rows}, indent=2) + '\n')
    print('Native ARM executes both original SEH handlers and rejects all three excluded continuations')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=ROOT / 'local/jit32/software-exception')
    check(parser.parse_args().output.resolve())
