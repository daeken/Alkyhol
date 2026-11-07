#!/usr/bin/env python3
"""Execute identical DXIL on Metal or Windows SDK WARP and record measured words."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
if not __package__:
    sys.path.insert(0, str(ROOT))

if __package__:
    from .shader_compute_contract import validate_observation
else:
    from tools.shader_compute_contract import validate_observation

def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def execute_probe(command, cwd, backend, timeout=30):
    if backend not in ('translated Metal', 'native D3D12 WARP'):
        raise ValueError('unknown compute backend')
    result = subprocess.run(command, cwd=cwd, check=True, capture_output=True, text=True, timeout=timeout)
    observed = validate_observation(result.stdout)
    identities = [line for line in result.stderr.splitlines() if line.startswith('adapter: ')]
    if len(identities) != 1 or not identities[0].startswith('adapter: ' + backend + ' '):
        raise RuntimeError('missing/ambiguous/mismatched executed adapter identity')
    if backend == 'translated Metal' and 'backend: Metal' not in identities[0]:
        raise RuntimeError('translated probe did not report Metal')
    return observed, identities[0], result.stderr


def run(dxil, dxc=None, output=None):
    dxil = dxil.resolve()
    fixture = ROOT / 'shader/tests/fixtures/compute.hlsl'
    with tempfile.TemporaryDirectory(prefix='alkyhol-dxil-compute-') as scratch:
        inputs = [Path(__file__).resolve(), ROOT / 'tools/shader_compute_contract.py',
                  ROOT / 'tools/graphics_contract.py', fixture]
        frontend = None
        if os.name == 'nt':
            source = ROOT / 'shader/reference/dxil_compute.cpp'
            inputs.append(source)
            exe = Path(scratch) / 'dxil_compute.exe'
            build = ['cl', '/nologo', '/W4', '/WX', '/EHsc', '/std:c++17', str(source), '/Fe:' + str(exe),
                     '/link', 'd3d12.lib', 'dxgi.lib']
            identity = subprocess.run(['cl'], capture_output=True, text=True, timeout=30)
            backend = 'native D3D12 WARP'
            build_cwd = scratch
        else:
            if platform.system() != 'Darwin' or not dxc:
                raise ValueError('Metal measurement requires macOS and an explicit --dxc')
            dxc = dxc.resolve()
            frontend = {'sha256': sha256(dxc), 'version': subprocess.check_output([str(dxc), '--version'], text=True).strip()}
            dxil.parent.mkdir(parents=True, exist_ok=True)
            compile_command = [str(dxc), '-T', 'cs_6_0', '-E', 'CSMain', '-Ges', '-WX', '-Fo', str(dxil), str(fixture)]
            subprocess.run(compile_command, check=True, cwd=ROOT, timeout=60)
            frontend['command'] = compile_command
            build = ['cargo', 'build', '--manifest-path', str(ROOT / 'shader/Cargo.toml'), '--locked', '--example', 'compute_contract']
            inputs.extend([ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'graphics/Cargo.toml',
                           ROOT / 'graphics/Cargo.lock', ROOT / 'shader/Cargo.toml', ROOT / 'shader/Cargo.lock',
                           ROOT / 'shader/examples/compute_contract.rs',
                           *sorted((ROOT / 'shader/src').glob('*.rs')), *sorted((ROOT / 'graphics/src').glob('*.rs'))])
            exe = ROOT / 'shader/target/debug/examples/compute_contract'
            identity = subprocess.run(['rustc', '--version', '--verbose'], check=True, capture_output=True, text=True, timeout=30)
            backend = 'translated Metal'
            build_cwd = ROOT
        subprocess.run(build, cwd=build_cwd, check=True, timeout=300)
        before = {str(p.relative_to(ROOT)): sha256(p) for p in inputs}
        binary = sha256(exe)
        shader = sha256(dxil)
        observation, adapter, diagnostics = execute_probe([str(exe), str(dxil)], scratch, backend)
        if ({str(p.relative_to(ROOT)): sha256(p) for p in inputs} != before
                or sha256(exe) != binary or sha256(dxil) != shader):
            raise RuntimeError('measured compute inputs changed during execution')
        report = {'schema_version': 1, 'backend': backend, 'adapter': adapter,
                  'host': platform.platform(), 'machine': platform.machine(),
                  'source_revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  'tracked_tree_clean': subprocess.run(['git', 'diff', '--quiet', 'HEAD'], cwd=ROOT).returncode == 0,
                  'shader_dxil_sha256': shader, 'observation': observation, 'artifacts': before,
                  'executable_sha256': binary, 'compiler': (identity.stdout + identity.stderr).strip(),
                  'windows_sdk': os.environ.get('WindowsSDKVersion'), 'frontend': frontend,
                  'build_command': build, 'diagnostics': diagnostics}
        if output:
            output.write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))
        return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--dxil', required=True, type=Path)
    parser.add_argument('--dxc', type=Path)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    run(args.dxil, args.dxc, args.output)
