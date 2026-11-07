#!/usr/bin/env python3
"""Build/run portable C/C++ samples; compare only deterministic observations."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = dict.fromkeys(('file_roundtrip', 'eof', 'truncate', 'errors', 'auto_reset', 'manual_reset', 'clock_monotonic'), True)
SAMPLES = {
    'kernel_contract': ('samples/kernel_contract.c', EXPECTED),
    'msdn_file_write': ('samples/msdn/file_write.c', dict.fromkeys(('write_count', 'readback', 'eof'), True)),
    'msdn_event_readers': ('samples/msdn/event_readers.cpp', dict.fromkeys(('all_readers', 'manual_stays_signaled', 'reset'), True)),
    'msdn_semaphore_workers': ('samples/msdn/semaphore_workers.cpp', dict.fromkeys(('permit_capacity', 'all_workers', 'bounded_active', 'previous_count', 'errors'), True)),
    'msdn_thread_local_storage': ('samples/msdn/thread_local_storage.cpp', dict.fromkeys(('initial_null', 'thread_isolation', 'common_payload', 'null_clears_error', 'invalid_index'), True)),
    'msdn_creating_threads': ('samples/msdn/creating_threads.c', dict.fromkeys(('parameters', 'thread_ids', 'pending', 'completion', 'exit_codes', 'persistent_signal'), True)),
}


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f'duplicate observation key: {key}')
        result[key] = value
    return result


def run_observation(command, expected, cwd, timeout=10):
    result = subprocess.run(command, check=True, cwd=cwd, capture_output=True, text=True, timeout=timeout)
    observation = json.loads(result.stdout, object_pairs_hook=unique_object)
    if (not isinstance(observation, dict) or observation != expected
            or any(type(v) is not bool for v in observation.values())):
        raise RuntimeError(f'unexpected observation: {observation}')
    return observation


def compile_command(source, exe):
    cpp = source.suffix == '.cpp'
    if os.name == 'nt':
        return ['cl', '/nologo', '/W4', '/WX', *(['/EHsc', '/std:c++14'] if cpp else []),
                str(source), '/Fe:' + str(exe)]
    library = ROOT / 'target/debug'
    return [os.environ.get('CXX' if cpp else 'CC', 'c++' if cpp else 'cc'),
            '-std=c++11' if cpp else '-std=c11', '-Wall', '-Wextra', '-Werror',
            *(['-pthread'] if cpp else []), '-I' + str(ROOT / 'include'), str(source),
            '-L' + str(library), '-lalkyhol', '-Wl,-rpath,' + str(library), '-o', str(exe)]


def run(output=None):
    observations, compilers, executables = {}, {}, {}
    inputs = [Path(__file__).resolve(), ROOT / 'Cargo.toml', ROOT / 'Cargo.lock']
    for name, (relative_source, expected) in SAMPLES.items():
        source = ROOT / relative_source
        inputs.append(source)
        with tempfile.TemporaryDirectory(prefix=f'alkyhol-{name}-') as scratch:
            exe = Path(scratch) / (name + ('.exe' if os.name == 'nt' else ''))
            command = compile_command(source, exe)
            compiler = subprocess.run([command[0]] if os.name == 'nt' else [command[0], '--version'],
                                      capture_output=True, text=True, timeout=30)
            compilers[name] = (compiler.stdout + compiler.stderr).strip()
            subprocess.run(command, check=True, cwd=scratch, timeout=120)
            observations[name] = run_observation([str(exe)], expected, scratch)
            executables[name] = hashlib.sha256(exe.read_bytes()).hexdigest()
    if os.name != 'nt':
        inputs.extend([ROOT / 'include/alkyhol.h', *sorted((ROOT / 'src').glob('*.rs'))])
        inputs.append(ROOT / 'target/debug' / ('libalkyhol.dylib' if platform.system() == 'Darwin' else 'libalkyhol.so'))
    report = {
        'schema_version': 2, 'host': platform.platform(), 'machine': platform.machine(),
        'backend': 'Windows SDK' if os.name == 'nt' else 'Alkyhol native C ABI',
        'observation': observations['kernel_contract'], 'samples': observations,
        'source_revision': subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip(),
        'tracked_tree_clean': subprocess.run(['git', 'diff', '--quiet', 'HEAD'], cwd=ROOT).returncode == 0,
        'compilers': compilers, 'executables': executables,
        'artifacts': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in inputs},
    }
    if output:
        output.write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))
    return report

if __name__ == '__main__':
    p = argparse.ArgumentParser(description=__doc__); p.add_argument('--output', type=Path)
    run(p.parse_args().output)
