#!/usr/bin/env python3
"""Exercise an unchanged released Windows pkg-config.exe on Windows or Alkyhol."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile

from fetch_uconfig_binary import PE_SHA256, SOURCE_SHA256, LICENSE_SHA256

ROOT = Path(__file__).resolve().parents[1]
CORPUS = ROOT / 'pe/fixtures/u-config'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def guest_stderr(stderr, rebase):
    prefix = re.match(rb'PE64: 4 sections, 15 named imports, 11 DIR64 fixups\nentering Win64 code; relocated=(true|false)\n', stderr)
    if not prefix or (rebase and prefix.group(1) != b'true'):
        raise ValueError(f'missing checked guest entry/rebase diagnostic: {stderr!r}')
    return stderr[prefix.end():]


def observe(executable, spec, host, rebase=False):
    observations, raw = {}, {}
    with tempfile.TemporaryDirectory(prefix='alkyhol-uconfig-') as temp:
        work = Path(temp)
        shutil.copytree(CORPUS / 'pc', work / spec['environment']['PKG_CONFIG_LIBDIR'])
        environment = {k: v for k, v in os.environ.items() if not k.upper().startswith('PKG_CONFIG_')}
        environment.update(spec['environment'])
        for case in spec['cases']:
            args = [str(executable)]
            if host:
                args += ['--loader-info'] + (['--rebase'] if rebase else []) + ['--']
            args += case['args']
            run = subprocess.run(args, cwd=work, env=environment, capture_output=True, timeout=30)
            stderr = guest_stderr(run.stderr, rebase) if host else run.stderr
            stdout = run.stdout
            raw[case['id']] = {'stdout_hex': stdout.hex(), 'stderr_hex': stderr.hex()}
            if case['stdout_order'] == 'unordered-lines':
                # Directory enumeration order is unspecified. Preserve duplicates
                # and compare complete lines; retain raw bytes separately above.
                stdout = b''.join(sorted(stdout.splitlines(keepends=True)))
            elif case['stdout_order'] != 'exact':
                raise ValueError('unsupported comparison rule')
            record = {'status': run.returncode, 'stdout_hex': stdout.hex(), 'stderr_hex': stderr.hex()}
            expected = {'status': case['status'], 'stdout_hex': case['stdout'].encode().hex(), 'stderr_hex': case['stderr'].encode().hex()}
            if record != expected:
                raise ValueError(f'{case["id"]}: actual {record}, expected {expected}; raw={raw[case["id"]]}')
            observations[case['id']] = record
    return observations, raw


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve()
    provenance = json.loads((binary.parent / 'provenance.json').read_text(encoding='utf-8'))
    if digest(binary) != PE_SHA256 or digest(binary.parent / 'pkg-config.c') != SOURCE_SHA256 or digest(binary.parent / 'UNLICENSE') != LICENSE_SHA256:
        raise ValueError('released binary/source/license identity differs')
    if provenance['pe_sha256'] != PE_SHA256 or provenance['source_sha256'] != SOURCE_SHA256 or provenance['rebuilt'] is not False or provenance['patched'] is not False:
        raise ValueError('invalid release provenance')
    spec = json.loads((CORPUS / 'cases.json').read_text(encoding='utf-8'))
    if platform.system() == 'Darwin':
        from translate_pe import translate, embedded_payload
        executable = translate(binary, ROOT / 'local/pe/u-config/pkg-config-macos')
        executed_hash = hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
        backend, host = 'Alkyhol Win64 ABI / x86-64 Mach-O', True
    elif platform.system() == 'Windows':
        executable, executed_hash = binary, digest(binary)
        backend, host = 'native Windows PE64 / kernel32', False
    else:
        raise RuntimeError('requires native Windows or macOS')
    if executed_hash != PE_SHA256:
        raise ValueError('executed payload differs from upstream released PE')
    normal, raw = observe(executable, spec, host)
    rebased, raw_rebase = observe(executable, spec, host, True) if host else (None, None)
    if host and normal != rebased:
        raise ValueError('forced-rebase guest behavior differs')
    sources = {p.relative_to(ROOT).as_posix(): digest(p) for p in sorted(CORPUS.rglob('*')) if p.is_file()}
    report = {'schema_version': 1, 'backend': backend, 'program': 'u-config 0.35.0 / w64devkit 2.11.0',
              'pe_sha256': PE_SHA256, 'executed_pe_sha256': executed_hash,
              'upstream_source_sha256': SOURCE_SHA256, 'upstream_license_sha256': LICENSE_SHA256,
              'input_sha256': sources, 'observations': normal, 'forced_rebase_observations': rebased,
              'raw_observations': raw, 'raw_forced_rebase_observations': raw_rebase,
              'revision': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
              'tracked_tree_clean': not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + '\n')
    print(f'Passed {len(normal)} real released PE cases: {backend}' + (' (normal + forced rebase)' if host else ''))


if __name__ == '__main__':
    main()
