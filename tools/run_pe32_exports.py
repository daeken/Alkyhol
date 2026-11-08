#!/usr/bin/env python3
"""Capture original executable self-export calls in an unchanged owned PE32."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import struct
import subprocess
import tempfile
from build_pe_fixture import ROOT
from build_pe32_exports import SOURCES

NATIVE='native Windows PE32 self exports'
HOST='Alkyhol PE32 self exports / x86-64 Mach-O'
REFERENCE=ROOT/'pe/reference/pe32-exports-windows-v1.json'
EXPECTED_WORDS=[0x45323349,4,1,0x1234abcd,1,0x27182818,1,0x11223344,1,1,
                (7*33)^11^0x31415926,9+0x12345678,1,0x31415926,0,0xa1b2c3d4,
                1,127,1,127,1,182,1,182,1,127]
HOST_DIAGNOSTIC=rb'PE32: 4 sections, 7 imports, 49 HIGHLOW fixups\nchecked i386 mapping; relocated=(true|false)\n'


def verify(pe):
    manifest=json.loads((pe.parent/'pe32-exports-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES):
        raise ValueError('missing exact PE32 export source identity')
    if hashlib.sha256(pe.read_bytes()).hexdigest()!=manifest['pe_sha256']:raise ValueError('PE32 export executable changed')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('PE32 export source changed')
    return manifest


def decode(run,windows=True,forced=False):
    if run.returncode!=0 or len(run.stdout)!=26*4:
        raise ValueError(f'PE32 self-export observer failed: {run.returncode}, {run.stdout.hex()}, {run.stderr!r}')
    if windows:
        if run.stderr:raise ValueError('unexpected native self-export diagnostic')
    else:
        m=re.fullmatch(HOST_DIAGNOSTIC,run.stderr)
        if not m or (forced and m[1]!=b'true'):raise ValueError('missing exact PE32 export loader/rebase evidence')
    return {'status':run.returncode,'stdout_hex':run.stdout.hex(),
            'words':list(struct.unpack('<26I',run.stdout)),'stderr_hex':run.stderr.hex()}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();pe=args.fixture.resolve();manifest=verify(pe)
    reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['pe_sha256','sources']:
        if manifest[field]!=reference[field]:raise ValueError('changed frozen PE32 self-export inputs')
    windows=platform.system()=='Windows'
    if windows:executable,backend,executed=pe,NATIVE,hashlib.sha256(pe.read_bytes()).hexdigest()
    elif platform.system()=='Darwin':
        from translate_pe32 import translate,embedded_payload
        executable=translate(pe,ROOT/'local/pe/pe32-exports/exports-macos');backend=HOST
        executed=hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
    else:raise RuntimeError('requires Windows or macOS')
    if executed!=manifest['pe_sha256']:raise ValueError('executed self-export PE bytes differ')
    def observe(forced=False):
        command=[str(executable)]+([] if windows else ['--loader-info'])+(['--rebase'] if forced else [])
        with tempfile.TemporaryDirectory(prefix='alkyhol-pe32-exports-') as temp:
            result=decode(subprocess.run(command,cwd=temp,capture_output=True,timeout=30),windows,forced)
        if result['words']!=EXPECTED_WORDS:raise ValueError('actual self-export calls differ from source contract')
        return result
    normal=observe();rebased=None if windows else observe(True)
    report={'schema_version':1,'backend':backend,'pe_sha256':manifest['pe_sha256'],
            'executed_pe_sha256':executed,'sources':manifest['sources'],
            'normal':normal,'rebased':rebased,
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual unchanged PE32 self-export calls: '+backend)


if __name__=='__main__':main()
