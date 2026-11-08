#!/usr/bin/env python3
"""Execute the pinned released PE against exact owned PE-file/CLI corpus bytes."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import subprocess
import tempfile
from fetch_peports32_binary import ROOT, PE_SHA256, SOURCE_SHA256, LICENSE_SHA256, URL, ARCHIVE_SHA256, LICENSE_URL

CORPUS=ROOT/'pe/fixtures/peports/cases.json'
REFERENCE=ROOT/'pe/reference/peports32-windows-v1.json'
NATIVE,HOST='native Windows PE32 peports','Alkyhol PE32 peports / Rosetta'
PROGRAM='peports i386 / w64devkit 2.11.0'

PROBE_SHA256='b07f5490efc122124ab9e21037b9dbeab84fb268dce29b7e138fc7b5f106162b'
EXPORT_SHA256='423d6e191a3ebb4f4f5ce11710591edc9ebe163ece79bd35cc6fe5146d13399d'

def fixture_bytes():
    probe=(ROOT/'local/pe/fixture/contract.exe').read_bytes()
    exports=(ROOT/'local/pe/export-fixture/alkyhol_exports.dll').read_bytes()
    if hashlib.sha256(probe).hexdigest()!=PROBE_SHA256 or hashlib.sha256(exports).hexdigest()!=EXPORT_SHA256:raise ValueError('changed frozen owned PE inputs')
    return {'probe.exe':probe,'exports.dll':exports,'tree λ🌸/exports space.dll':exports,'-name.dll':exports,'bad.bin':b'not a PE'}

def corpus_hashes():
    return {'pe/fixtures/peports/cases.json':hashlib.sha256(CORPUS.read_bytes()).hexdigest(),
            'fixture:probe.exe':PROBE_SHA256,'fixture:exports.dll':EXPORT_SHA256,
            'fixture:bad.bin':hashlib.sha256(b'not a PE').hexdigest(),'fixture:truncated-stdin':hashlib.sha256(b'MZ\0').hexdigest()}


def expected():
    spec=json.loads(CORPUS.read_text(encoding='utf-8'))
    if type(spec['schema_version']) is not int or spec['schema_version']!=1:raise ValueError('invalid corpus schema')
    cases=spec['cases']
    if len({c['id'] for c in cases})!=len(cases):raise ValueError('duplicate corpus case')
    return cases,{c['id']:{'status':c['status'],'stdout_hex':c['stdout'].encode().hex(),'stderr_hex':c['stderr'].encode().hex()} for c in cases}


def decode(run,host=False,rebase=False):
    stderr=run.stderr
    if host:
        prefix=re.match(rb'PE32: 6 sections, 10 imports, 239 HIGHLOW fixups\nchecked i386 mapping; relocated=(true|false)\n',stderr)
        if not prefix or (rebase and prefix[1]!=b'true'):raise ValueError('missing exact loader/forced-rebase evidence')
        stderr=stderr[prefix.end():]
        restored=b'host segment selectors restored; owned FS descriptor cleared\n'
        if not stderr.endswith(restored):raise ValueError('missing real restored host segment/FS evidence after ExitProcess')
        stderr=stderr[:-len(restored)]
    return {'status':run.returncode,'stdout_hex':run.stdout.hex(),'stderr_hex':stderr.hex()}


def observe(executable,host,rebase=False):
    cases,wanted=expected();records={}
    with tempfile.TemporaryDirectory(prefix='alkyhol-peports-') as temp:
        work=Path(temp)/'build space λ🌸';work.mkdir();files=fixture_bytes()
        for name,data in files.items():
            path=work/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
        for case in cases:
            command=[str(executable)]+(['--loader-info','--teb-info']+(['--rebase'] if rebase else [])+['--'] if host else [])+case['args']
            stdin=b'' if case['stdin_fixture'] is None else b'MZ\0' if case['stdin_fixture']=='truncated' else files[case['stdin_fixture']]
            run=subprocess.run(command,cwd=work,input=stdin,capture_output=True,timeout=30)
            actual=decode(run,host,rebase)
            if actual!=wanted[case['id']]:raise ValueError(f'{case["id"]}: actual bytes/status differ: {actual}')
            records[case['id']]=actual
    return records


def identity(binary):
    digest=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    if digest(binary)!=PE_SHA256 or digest(binary.parent/'peports.c')!=SOURCE_SHA256 or digest(binary.parent/'UNLICENSE')!=LICENSE_SHA256:
        raise ValueError('changed released PE/source/license')
    provenance=json.loads((binary.parent/'provenance.json').read_text(encoding='utf-8'))
    wanted={'schema_version':1,'program':'peports','version':'w64devkit-2.11.0','distribution':'w64devkit 2.11.0 x86',
            'archive_url':URL,'archive_sha256':ARCHIVE_SHA256,'pe_member':'w64devkit/bin/peports.exe','pe_sha256':PE_SHA256,
            'source_member':'w64devkit/src/peports.c','source_sha256':SOURCE_SHA256,
            'license_url':LICENSE_URL,'license_sha256':LICENSE_SHA256,'rebuilt':False,'patched':False}
    if provenance!=wanted or type(provenance['schema_version']) is not int or provenance['rebuilt'] is not False or provenance['patched'] is not False:raise ValueError('changed release provenance')
    fixture_bytes()
    inputs=corpus_hashes()
    reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    for key,value in [('pe_sha256',PE_SHA256),('upstream_source_sha256',SOURCE_SHA256),('upstream_license_sha256',LICENSE_SHA256),('input_sha256',inputs)]:
        if reference[key]!=value:raise ValueError('changed frozen release/corpus identity: '+key)
    return inputs


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    binary=args.binary.resolve();inputs=identity(binary)
    if platform.system()=='Darwin':
        from translate_pe32 import translate
        from translate_pe import embedded_payload
        executable=translate(binary,ROOT/'local/pe/peports32/peports-macos')
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
    print(f'Passed {len(normal)} unchanged released peports cases: '+report['backend'])


if __name__=='__main__':main()
