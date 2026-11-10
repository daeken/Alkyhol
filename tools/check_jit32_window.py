#!/usr/bin/env python3
"""Run the unchanged owned window/callback PE on the shared native ARM runner."""
import argparse,json,subprocess
from pathlib import Path
from observe_pe32_window_family import ROOT,build,decode,FILES,inputs
from check_jit32_sse import sha

def check(host):
    output=ROOT/'local/jit32/window-family';build(output);before=inputs(output)
    data=output/'stdout.bin'
    with data.open('wb') as stream:r=subprocess.run([str(host),str(output/FILES[0]),'--strict','--file-origins'],stdout=stream,stderr=subprocess.PIPE,timeout=30)
    report={'backend':'physical native ARM JIT with AppKit window','inputs':before,'host_sha256':sha(host),'status':r.returncode,'stdout_hex':data.read_bytes().hex(),'stderr_hex':r.stderr.hex()}
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or b'JIT32 shared runtime: Stats' not in r.stderr:raise ValueError('native window PE failed; raw output retained')
    report['words']=decode(data.read_bytes())
    if inputs(output)!=before:raise ValueError('owned window input changed during execution')
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print('32 unchanged owned window/callback/coordinate/lifetime words pass on native ARM')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host',type=Path,default=ROOT/'local/jit32/shared/startup-arm64');check(p.parse_args().host.resolve())
