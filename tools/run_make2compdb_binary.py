#!/usr/bin/env python3
"""Execute the pinned released PE against exact owned make/CLI corpus bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import tempfile
from fetch_make2compdb_binary import ROOT, PE_SHA256, SOURCE_SHA256, LICENSE_SHA256, URL, ARCHIVE_SHA256, LICENSE_URL

CORPUS=ROOT/'pe/fixtures/make2compdb/cases.json'
REFERENCE=ROOT/'pe/reference/make2compdb-windows-v1.json'
NATIVE,HOST='native Windows PE64 make2compdb','Alkyhol PE64 make2compdb / Rosetta'
PROGRAM='make2compdb 2026-09-15 / w64devkit 2.11.0'


def expected():
    spec=json.loads(CORPUS.read_text(encoding='utf-8'))
    if type(spec['schema_version']) is not int or spec['schema_version']!=1:raise ValueError('invalid corpus schema')
    cases=spec['cases']
    if len({c['id'] for c in cases})!=len(cases):raise ValueError('duplicate corpus case')
    return cases,{c['id']:{'status':c['status'],'stdout_hex':c['stdout'].encode().hex(),'stderr_hex':c['stderr'].encode().hex()} for c in cases}


def decode(run,host=False,rebase=False):
    stderr=run.stderr
    if host:
        prefix=re.match(rb'PE64: 6 sections, 10 named imports, 95 DIR64 fixups\nentering Win64 code; relocated=(true|false)\n',stderr)
        if not prefix or (rebase and prefix[1]!=b'true'):raise ValueError('missing exact loader/forced-rebase evidence')
        stderr=stderr[prefix.end():]
    return {'status':run.returncode,'stdout_hex':run.stdout.hex(),'stderr_hex':stderr.hex()}


def observe(executable,host,rebase=False):
    cases,wanted=expected();records={}
    with tempfile.TemporaryDirectory(prefix='alkyhol-make2compdb-') as temp:
        work=Path(temp)/'build space λ🌸';work.mkdir()
        for case in cases:
            command=[str(executable)]+(['--loader-info']+(['--rebase'] if rebase else [])+['--'] if host else [])+case['args']
            run=subprocess.run(command,cwd=work,input=case['stdin'].encode(),capture_output=True,timeout=30)
            actual=decode(run,host,rebase)
            if actual!=wanted[case['id']]:raise ValueError(f'{case["id"]}: actual bytes/status differ: {actual}')
            records[case['id']]=actual
    return records


def identity(binary):
    digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    if digest(binary)!=PE_SHA256 or digest(binary.parent/'make2compdb.c')!=SOURCE_SHA256 or digest(binary.parent/'UNLICENSE')!=LICENSE_SHA256:
        raise ValueError('changed released PE/source/license')
    provenance=json.loads((binary.parent/'provenance.json').read_text(encoding='utf-8'))
    wanted={'schema_version':1,'program':'make2compdb','version':'2026-09-15','distribution':'w64devkit 2.11.0 x64',
            'archive_url':URL,'archive_sha256':ARCHIVE_SHA256,'pe_member':'w64devkit/bin/make2compdb.exe','pe_sha256':PE_SHA256,
            'source_member':'w64devkit/src/make2compdb.c','source_sha256':SOURCE_SHA256,
            'license_url':LICENSE_URL,'license_sha256':LICENSE_SHA256,'rebuilt':False,'patched':False}
    if provenance!=wanted or provenance['rebuilt'] is not False or provenance['patched'] is not False:raise ValueError('changed release provenance')
    inputs={'pe/fixtures/make2compdb/cases.json':digest(CORPUS)}
    # Only this new corpus bootstraps its first independently executed Windows record.
    if REFERENCE.exists():
        reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
        if reference['input_sha256']!=inputs:raise ValueError('changed frozen make2compdb corpus')
    return inputs


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    binary=args.binary.resolve();inputs=identity(binary)
    if platform.system()=='Darwin':
        from translate_pe import translate,embedded_payload
        executable=translate(binary,ROOT/'local/pe/make2compdb/make2compdb-macos')
        executed=hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest();host=True
    elif platform.system()=='Windows':executable=binary;executed=hashlib.sha256(binary.read_bytes()).hexdigest();host=False
    else:raise RuntimeError('requires native Windows or macOS/Rosetta')
    if executed!=PE_SHA256:raise ValueError('executed PE differs from released binary')
    normal=observe(executable,host);rebased=observe(executable,True,True) if host else None
    if host and rebased!=normal:raise ValueError('forced-rebase behavior differs')
    report={'schema_version':1,'backend':HOST if host else NATIVE,'program':PROGRAM,'pe_sha256':PE_SHA256,'executed_pe_sha256':executed,
            'upstream_source_sha256':SOURCE_SHA256,'upstream_license_sha256':LICENSE_SHA256,'input_sha256':inputs,
            'observations':normal,'forced_rebase_observations':rebased,
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
            'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print(f'Passed {len(normal)} unchanged released make2compdb cases: '+report['backend'])


if __name__=='__main__':main()
