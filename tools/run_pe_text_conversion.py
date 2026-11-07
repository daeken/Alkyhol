#!/usr/bin/env python3
"""Observe direct UTF conversion calls in the identical owned Windows PE."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import re
import struct
import subprocess
import tempfile
from build_pe_text_conversion import ROOT,SOURCES

REFERENCE=ROOT/'pe/reference/text-conversion-windows-v1.json'
NATIVE,HOST='native Windows PE64 text conversion','Alkyhol PE64 text conversion / Rosetta'
BYTE_LENGTHS=[-1,3,-1,-1,4,1,3,2,3,2,4,0,3,3,3,3,3,4,4,4,4,4,4,3,4,3,3,3]
BYTE_VALID={0,1,2,3,4,14,15,19,20}
WIDE_LENGTHS=[-1,3,-1,-1,2,1,1,3,3,0]
WORD_COUNT=1832


def validate(observation):
    if not isinstance(observation,dict):raise ValueError('missing observation record')
    words=observation['words']
    if type(observation['status']) is not int or observation['status']!=0 or len(words)!=WORD_COUNT or any(type(v) is not int or not 0<=v<=0xffffffff for v in words):raise ValueError('invalid status/word record')
    if observation['stdout_hex']!=struct.pack('<'+str(WORD_COUNT)+'I',*words).hex():raise ValueError('raw output differs from decoded words')
    if words[:3]!=[0x54584655,56,20] or words[-5:]!=[0,122,0,122,0x454e4421]:raise ValueError('missing complete probe/error evidence')
    offset=3
    for api,lengths,strict,seed,maximum in [(1,BYTE_LENGTHS,8,0xa55a,0xffff),(2,WIDE_LENGTHS,128,0xa5,0xff)]:
        for index,length in enumerate(lengths):
            for flags in [0,strict]:
                row=words[offset:offset+24];offset+=24
                if row[:4]!=[api,index,flags,length&0xffffffff]:raise ValueError('changed call identity/order')
                count=row[4]
                invalid=length==0 or (flags==strict and (index not in BYTE_VALID if api==1 else index>=5))
                if (count==0)!=invalid:raise ValueError('valid conversion failed or malformed strict query succeeded')
                if invalid and row[5]!=(87 if length==0 else 1113):raise ValueError('wrong invalid-input error')
                if count>16 or any(v>maximum for v in row[8:]):raise ValueError('invalid conversion count/unit width')
                if count==0:
                    if row[6:8]!=[0xffffffff,0xffffffff] or row[8:]!=[seed]*16:raise ValueError('failed query did not skip the write call')
                elif row[6]!=count or row[8+count:]!=[seed]*(16-count):raise ValueError('query/write counts or output tail differ')
    if offset!=WORD_COUNT-5:raise ValueError('dropped probe records')
    return observation


def decode(run,windows=False,rebase=False):
    if run.returncode!=0 or len(run.stdout)!=WORD_COUNT*4:raise ValueError('probe crashed, failed or returned incomplete bytes')
    if windows:
        if run.stderr:raise ValueError('unexpected native diagnostic')
    else:
        match=re.fullmatch(rb'PE64: 4 sections, 7 named imports, 1 DIR64 fixups\nentering Win64 code; relocated=(true|false)\n',run.stderr)
        if not match or (rebase and match[1]!=b'true'):raise ValueError('missing exact loader/rebase evidence')
    return validate({'status':run.returncode,'words':list(struct.unpack('<'+str(WORD_COUNT)+'I',run.stdout)),'stdout_hex':run.stdout.hex()})


def observe(executable,windows=False,rebase=False):
    command=[str(executable.resolve())]+([] if windows else ['--loader-info'])+(['--rebase'] if rebase else [])
    with tempfile.TemporaryDirectory(prefix='alkyhol-text-conversion-') as directory:
        return decode(subprocess.run(command,cwd=directory,capture_output=True,timeout=30),windows,rebase)


def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--fixture',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
    pe=args.fixture.resolve();manifest=json.loads((pe.parent/'text-conversion-fixture.json').read_text(encoding='utf-8'))
    if type(manifest['schema_version']) is not int or manifest['schema_version']!=1 or set(manifest['sources'])!=set(SOURCES):raise ValueError('invalid source manifest')
    if hashlib.sha256(pe.read_bytes()).hexdigest()!=manifest['pe_sha256']:raise ValueError('changed PE identity')
    for name,digest in manifest['sources'].items():
        if hashlib.sha256((ROOT/name).read_bytes()).hexdigest()!=digest:raise ValueError('changed probe source')
    reference=json.loads(REFERENCE.read_text(encoding='utf-8'))
    for field in ['pe_sha256','sources']:
        if manifest[field]!=reference[field]:raise ValueError('changed frozen text-conversion input')
    windows=platform.system()=='Windows'
    if windows:executable=pe;executed=hashlib.sha256(pe.read_bytes()).hexdigest();backend=NATIVE
    elif platform.system()=='Darwin':
        from translate_pe import translate,embedded_payload
        executable=translate(pe,ROOT/'local/pe/text-conversion/text-conversion-macos');executed=hashlib.sha256(embedded_payload(executable.read_bytes())).hexdigest();backend=HOST
    else:raise RuntimeError('requires Windows or macOS/Rosetta')
    if executed!=manifest['pe_sha256']:raise ValueError('executed payload differs')
    normal=observe(executable,windows);rebased=None if windows else observe(executable,rebase=True)
    if rebased is not None and normal!=rebased:raise ValueError('rebased observations differ')
    report={'schema_version':1,'backend':backend,'pe_sha256':manifest['pe_sha256'],'executed_pe_sha256':executed,'sources':manifest['sources'],'normal':normal,'rebased':rebased,
            'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip()}
    args.output.parent.mkdir(parents=True,exist_ok=True);args.output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Recorded 76 UTF conversion cases and two buffer-error statuses: '+backend)


if __name__=='__main__':main()
