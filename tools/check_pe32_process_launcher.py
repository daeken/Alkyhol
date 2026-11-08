#!/usr/bin/env python3
"""Exercise public PE32 DLL arguments and prove inspect never calls initializers."""
import argparse
import json
from pathlib import Path
import subprocess
from run_pe32_process import verify,HOST,RESTORED
from compare_pe32_library import record
from translate_pe32 import translate

def check(directory,process_report,output):
    directory=directory.resolve();manifest=verify(directory)
    evidence=json.loads(process_report.read_text(encoding='utf-8'))
    if evidence['backend']!=HOST or any(evidence[k]!=manifest[k] for k in ['files','sources']):raise ValueError('different process probe inputs')
    record(evidence['normal'],105,7,RESTORED+b'false\n')
    executable=translate(directory/'program.exe',directory/'program-macos')
    def command(consumer='consumer.dll'):
        return [str(executable),'--exe-name','program.exe','--dll','consumer.dll='+str(directory/consumer),'--dll','terminate.dll='+str(directory/'terminate.dll'),'--dll','accepted.dll='+str(directory/'accepted.dll')]
    observations={}
    def observe(name,args,status,stdout,contains=None):
        r=subprocess.run(args,capture_output=True,timeout=30)
        if r.returncode!=status or r.stdout!=stdout or (contains is not None and contains not in r.stderr):raise ValueError(f'launcher {name} failed: {r.returncode}, {r.stdout.hex()}, {r.stderr!r}')
        observations[name]={'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()}
    for forced in [False,True]:
        observe('rebased' if forced else 'normal',command()+(['--rebase'] if forced else [])+['--teb-info'],7,bytes.fromhex(evidence['normal']['stdout_hex']),b'host segment selectors restored; owned FS descriptor cleared\n')
    for consumer in ['consumer.dll','failed.dll']:
        observe('inspect-'+consumer,command(consumer)+['--inspect'],0,b'',b'checked PE32 process dependency graph; no guest initialization\n')
    observe('inspect-no-seh-combination',command()+['--inspect','--seh'],78,b'',b'combined PE32 process-graph/SEH execution is unsupported\n')
    for name,args in [('missing-value',['--dll']),('missing-identity',['--dll','a.dll=path']),('malformed-spec',['--exe-name','program.exe','--dll','path'])]:
        observe(name,[str(executable),*args],64,b'',b'Alkyhol PE32 arguments:')
    report={'schema_version':1,'backend':'Alkyhol PE32 launcher / Rosetta','pe_sha256':manifest['files']['program.exe'],'sources':manifest['sources'],'observations':observations}
    output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
    print('Passed eight real PE32 process launcher/inspection checks')

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--directory',type=Path,required=True);p.add_argument('--process-report',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();check(a.directory,a.process_report,a.output)
