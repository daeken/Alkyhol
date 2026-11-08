#!/usr/bin/env python3
"""Capture Win32 FS/TEB and last-error coherence in an unchanged owned PE32."""
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
from build_pe32_teb import SOURCES

NATIVE='native Windows PE32 FS/TEB'
HOST='Alkyhol PE32 FS/TEB / x86-64 Mach-O'
REFERENCE=ROOT/'pe/reference/pe32-teb-windows-v1.json'
EXPECTED_WORDS=[0x42323349,4,1,1,1,1,0x31415926,0xa1b2c3d4,10000,
                9999^0x12345678,9999^0x12345678,1,1,1,1,0x88776655]
HOST_DIAGNOSTIC=rb'PE32: 4 sections, 4 imports, 23 HIGHLOW fixups\nchecked i386 mapping; relocated=(true|false)\n'
RESTORED=b'host segment selectors restored; owned FS descriptor cleared\n'



def verify(pe):
    manifest=json.loads((pe.parent/'pe32-teb-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES):
        raise ValueError('missing exact PE32 TEB source identity')
    if hashlib.sha256(pe.read_bytes()).hexdigest()!=manifest['pe_sha256']:raise ValueError('PE32 TEB executable changed')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('PE32 TEB source changed')
    return manifest


def decode(run,windows=True,forced=False,repeated=False):
    if run.returncode!=0 or len(run.stdout)!=64*(2 if repeated else 1):
        raise ValueError(f'PE32 TEB observer failed: {run.returncode}, {run.stdout.hex()}, {run.stderr!r}')
    if windows:
        if run.stderr:raise ValueError('unexpected native TEB diagnostic')
    else:
        m=re.fullmatch(HOST_DIAGNOSTIC+re.escape(RESTORED)*(2 if repeated else 1),run.stderr)
        if not m or (forced and m[1]!=b'true'):raise ValueError('missing exact PE32 TEB loader/rebase evidence')
    return {'status':run.returncode,'stdout_hex':run.stdout.hex(),
            'words':list(struct.unpack('<'+str(32 if repeated else 16)+'I',run.stdout)),'stderr_hex':run.stderr.hex()}


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    args=p.parse_args();pe=args.fixture.resolve();manifest=verify(pe)
    reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['pe_sha256','sources']:
        if manifest[field]!=reference[field]:raise ValueError('changed frozen PE32 TEB inputs')
    windows=platform.system()=='Windows'
    if windows:executable,backend,executed=pe,NATIVE,hashlib.sha256(pe.read_bytes()).hexdigest()
    elif platform.system()=='Darwin':
        from translate_pe32 import translate,embedded_payload
        executable=translate(pe,ROOT/'local/pe/pe32-teb/teb-macos');backend=HOST
        executed=hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest()
    else:raise RuntimeError('requires Windows or macOS')
    if executed!=manifest['pe_sha256']:raise ValueError('executed TEB PE bytes differ')
    def observe(forced=False,repeated=False):
        command=[str(executable)]+([] if windows else ['--loader-info','--teb-info'])+(['--rebase'] if forced else [])+(['--repeat'] if repeated else [])
        with tempfile.TemporaryDirectory(prefix='alkyhol-pe32-teb-') as temp:
            result=decode(subprocess.run(command,cwd=temp,capture_output=True,timeout=30),windows,forced,repeated)
        if result['words']!=EXPECTED_WORDS*(2 if repeated else 1):raise ValueError('actual TEB calls differ from source contract')
        return result
    normal=observe();rebased=None if windows else observe(True)
    repeated=None if windows else observe(repeated=True)
    report={'schema_version':1,'backend':backend,'pe_sha256':manifest['pe_sha256'],
            'executed_pe_sha256':executed,'sources':manifest['sources'],
            'normal':normal,'rebased':rebased,'repeated':repeated,
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded actual unchanged PE32 TEB calls: '+backend)


if __name__=='__main__':main()
