#!/usr/bin/env python3
"""Exercise DAO's reached worker/AppKit polling contention without game inputs."""
import argparse
import json
import shutil
import struct
import subprocess
import time
from pathlib import Path
from observe_pe32_worker_window import ROOT, SOURCE, SYMBOLS, decode
from build_pe_fixture import rust_tool
from check_jit32_sse import sha

def check(host):
    output=ROOT/'local/jit32/peek-workers';output.mkdir(parents=True,exist_ok=True)
    source=ROOT/'jit32/tests/poll_window.c';lld=rust_tool('rust-lld');libs=[]
    for name,extra in [('worker-window',''),('window-family-user','\nPeekMessageW\n')]:
        definition=output/(name+'.def');definition.write_text((SOURCE/(name+'.def')).read_text()+extra)
        lib=output/(name+'.lib');libs.append(str(lib))
        subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(definition),'/out:'+str(lib)],check=True)
    obj=output/'poll.obj';pe=output/'poll.exe'
    subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(source),'-o',str(obj)],check=True)
    subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS+[('PeekMessageW',20)]],'/out:'+str(pe)],check=True)
    identities={str(p.relative_to(ROOT)):sha(p) for p in [source,SOURCE/'window-family.c',pe]}
    started=time.monotonic()
    with (output/'stdout.bin').open('wb+') as stream:
        r=subprocess.run([str(host),str(pe),'--strict','--file-origins'],stdout=stream,stderr=subprocess.PIPE,timeout=30)
        stream.seek(0);data=stream.read()
    report=dict(backend='physical native ARM worker window/PeekMessage/compute',inputs=identities,host_sha256=sha(host),status=r.returncode,elapsed=time.monotonic()-started,stdout_hex=data.hex(),stderr_hex=r.stderr.hex())
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    if r.returncode or len(data)!=144 or b'JIT32 shared runtime: Stats' not in r.stderr:raise ValueError('owned polling/window witness failed; report retained')
    decode(data[:128])
    if struct.unpack('<4I',data[128:])!=(0x5045454b,1,1,1):raise ValueError('worker progress or lifetime differs')
    if identities!={str(p.relative_to(ROOT)):sha(p) for p in [source,SOURCE/'window-family.c',pe]}:raise ValueError('owned input changed')
    print('36 owned window/callback/worker-progress words pass on native ARM; seconds',report['elapsed'])

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host',type=Path,default=ROOT/'local/jit32/shared/startup-arm64');check(p.parse_args().host.resolve())
