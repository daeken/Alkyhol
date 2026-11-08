#!/usr/bin/env python3
"""Capture compiler-generated static TLS data in an unchanged owned PE32."""
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
from build_pe32_tls_data import SOURCES

NATIVE='native Windows PE32 static TLS data'
HOST='Alkyhol PE32 static TLS data / x86-64 Mach-O'
REFERENCE=ROOT/'pe/reference/pe32-tls-data-windows-v1.json'
EXPECTED_WORDS=[0x54323349,4,1,1,1,1,1,0x31415926,0,1,0x13579bdf,0x31415926,
                0xa1b2c3d4,0x88776655,0x2468ace0,1,1,1,1,1,1,0x11223344,0x31415926,0x99aabbcc]
HOST_DIAGNOSTIC=rb'PE32: 5 sections, 4 imports, 44 HIGHLOW fixups\nchecked i386 mapping; relocated=(true|false)\n'
RESTORED=b'host segment selectors restored; owned FS descriptor cleared\n'
REPEAT_ERROR=b'Alkyhol PE32 unsupported: PE32 static TLS images support one entry\n'



def verify(pe):
    manifest=json.loads((pe.parent/'pe32-tls-data-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES):
        raise ValueError('missing exact PE32 TLS data source identity')
    if hashlib.sha256(pe.read_bytes()).hexdigest()!=manifest['pe_sha256']:raise ValueError('PE32 TLS data executable changed')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('PE32 TLS data source changed')
    return manifest


def decode(run,windows=True,forced=False,repeat=False):
    if run.returncode!=(78 if repeat else 0) or len(run.stdout)!=96:
        raise ValueError(f'PE32 TLS data observer failed: {run.returncode}, {run.stdout.hex()}, {run.stderr!r}')
    if windows:
        if run.stderr:raise ValueError('unexpected native TLS data diagnostic')
    else:
        m=re.fullmatch(HOST_DIAGNOSTIC+re.escape(RESTORED)+(re.escape(REPEAT_ERROR) if repeat else b''),run.stderr)
        if not m or (forced and m[1]!=b'true'):raise ValueError('missing exact PE32 TLS data loader/rebase evidence')
    return {'status':run.returncode,'stdout_hex':run.stdout.hex(),
            'words':list(struct.unpack('<24I',run.stdout)),'stderr_hex':run.stderr.hex()}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();pe=args.fixture.resolve();manifest=verify(pe)
    reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['pe_sha256','sources']:
        if manifest[field]!=reference[field]:raise ValueError('changed frozen PE32 TLS data inputs')
    windows=platform.system()=='Windows'
    if windows:executable,backend,executed=pe,NATIVE,hashlib.sha256(pe.read_bytes()).hexdigest()
    elif platform.system()=='Darwin':
        from translate_pe32 import translate,embedded_payload
        executable=translate(pe,ROOT/'local/pe/pe32-tls-data/tls-macos');backend=HOST
        executed=hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
    else:raise RuntimeError('requires Windows or macOS')
    if executed!=manifest['pe_sha256']:raise ValueError('executed TLS data PE bytes differ')
    def observe(forced=False,repeat=False):
        command=[str(executable)]+([] if windows else ['--loader-info','--teb-info'])+(['--rebase'] if forced else [])+(['--repeat'] if repeat else [])
        with tempfile.TemporaryDirectory(prefix='alkyhol-pe32-tls-data-') as temp:
            result=decode(subprocess.run(command,cwd=temp,capture_output=True,timeout=30),windows,forced,repeat)
        if result['words']!=EXPECTED_WORDS:raise ValueError('actual TLS data calls differ from source contract')
        return result
    normal=observe();rebased=None if windows else observe(True)
    repeat_rejected=None if windows else observe(repeat=True)
    report={'schema_version':1,'backend':backend,'pe_sha256':manifest['pe_sha256'],
            'executed_pe_sha256':executed,'sources':manifest['sources'],
            'normal':normal,'rebased':rebased,'repeat_rejected':repeat_rejected,
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual unchanged PE32 TLS data calls: '+backend)


if __name__=='__main__':main()
