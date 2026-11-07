#!/usr/bin/env python3
"""Build/run the native D3D12 WARP reference or Alkyhol checkerboard copy probe."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

if __package__:
    from .graphics_contract import validate_observation
else:
    from graphics_contract import validate_observation

ROOT = Path(__file__).resolve().parents[1]


def hashes(paths):
    return {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest() for path in paths}


def execute_probe(command, cwd, backend, timeout=30):
    result = subprocess.run(command, check=True, capture_output=True, text=True, cwd=cwd, timeout=timeout)
    observation = validate_observation(result.stdout)
    adapters = [line for line in result.stderr.splitlines() if line.startswith('adapter: ')]
    if len(adapters) != 1:
        raise RuntimeError('missing or ambiguous executed adapter identity')
    adapter = adapters[0]
    if backend == 'native D3D12 WARP':
        valid = adapter.startswith('adapter: D3D12 WARP ')
    elif backend == 'Alkyhol direct Metal':
        valid = 'backend: Metal' in adapter and 'device_type: Cpu' not in adapter
    else:
        raise ValueError(f'unknown graphics backend: {backend}')
    if not valid:
        raise RuntimeError('probe did not identify the requested backend/adapter class')
    return observation, adapter, result.stderr


def run(output=None):
    with tempfile.TemporaryDirectory(prefix='alkyhol-graphics-reference-') as scratch:
        inputs = [ROOT / 'tools/graphics_contract.py', Path(__file__).resolve()]
        if os.name == 'nt':
            source = ROOT / 'graphics/reference/d3d12_copy_contract.cpp'
            executable = Path(scratch) / 'd3d12_copy_contract.exe'
            build = ['cl', '/nologo', '/W4', '/WX', '/EHsc', '/std:c++17', str(source), '/Fe:' + str(executable),
                     '/link', 'd3d12.lib', 'dxgi.lib', 'd3dcompiler.lib']
            identity = subprocess.run(['cl'], capture_output=True, text=True, timeout=30)
            inputs.append(source)
            backend = 'native D3D12 WARP'
            command = [str(executable)]
        else:
            if platform.system() != 'Darwin':
                raise RuntimeError('Alkyhol now uses direct Metal; this probe requires macOS or the Windows D3D12 reference')
            build = ['cargo', 'build', '--manifest-path', str(ROOT / 'graphics/Cargo.toml'), '--locked', '--example', 'copy_contract']
            identity = subprocess.run(['rustc', '--version', '--verbose'], check=True, capture_output=True, text=True, timeout=30)
            inputs.extend([ROOT / 'Cargo.toml', ROOT / 'Cargo.lock', ROOT / 'graphics/Cargo.toml', ROOT / 'graphics/Cargo.lock',
                           ROOT / 'graphics/src/lib.rs', ROOT / 'graphics/examples/copy_contract.rs', ROOT / 'graphics/examples/checker.metal',
                           *sorted((ROOT / 'src').glob('*.rs'))])
            executable = ROOT / 'graphics/target/debug/examples/copy_contract'
            command = [str(executable)]
            backend = 'Alkyhol direct Metal'
        subprocess.run(build, check=True, cwd=ROOT if os.name != 'nt' else scratch, timeout=300)
        before = hashes(inputs)
        executable_hash = hashlib.sha256(executable.read_bytes()).hexdigest()
        observation, adapter, diagnostics = execute_probe(command, scratch, backend)
        if hashes(inputs) != before or hashlib.sha256(executable.read_bytes()).hexdigest() != executable_hash:
            raise RuntimeError('executed probe inputs changed during measurement')
        report = {
            'schema_version': 1, 'backend': backend, 'adapter': adapter,
            'host': platform.platform(), 'machine': platform.machine(),
            'observation': observation,
            'source_revision': subprocess.run(['git', 'rev-parse', 'HEAD'], check=True, cwd=ROOT, capture_output=True, text=True).stdout.strip(),
            'tracked_tree_clean': subprocess.run(['git', 'diff', '--quiet', 'HEAD'], cwd=ROOT).returncode == 0,
            'compiler': (identity.stdout + identity.stderr).strip(), 'windows_sdk': os.environ.get('WindowsSDKVersion'),
            'build_command': build, 'artifacts': before, 'executable_sha256': executable_hash,
            'diagnostics': diagnostics,
        }
        if output:
            output.write_text(json.dumps(report, indent=2) + '\n')
        print(json.dumps(report, indent=2))
        return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    run(args.output)
