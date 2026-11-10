#!/usr/bin/env python3
"""Targeted native JIT worker and zero-timeout polling tests for original DAO."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from observe_pe32_thread_family import ROOT,build,decode,FILES,SYMBOLS,inputs,rust_tool
from check_jit32_sse import sha

def check(host):
    output=ROOT/'local/jit32/thread-family';build(output)
    source=ROOT/'jit32/tests/poll_threads.c';obj=output/'poll-threads.obj';pe=output/'poll-threads.exe'
    subprocess.run([shutil.which('clang'),'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-c',str(source),'-o',str(obj)],check=True)
    subprocess.run([rust_tool('rust-lld'),'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(output/'thread-family.lib'),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(pe)],check=True)
    rows=[]
    for name,expected in [(FILES[0],None),('poll-threads.exe',[0x504f4c4c,4]+[1]*9+[12])]:
        data=output/(name+'.stdout')
        with data.open('wb') as stream:r=subprocess.run([str(host),str(output/name),'--strict','--file-origins','--host-machine-info'],stdout=stream,stderr=subprocess.PIPE,timeout=30)
        row={'pe_sha256':sha(output/name),'status':r.returncode,'stdout_hex':data.read_bytes().hex(),'stderr_hex':r.stderr.hex()};rows.append(row)
        if r.returncode or b'JIT32 shared runtime: Stats' not in r.stderr:raise ValueError(f'original native worker PE failed: {row}')
        words=decode(data.read_bytes()) if expected is None else list(struct.unpack('<12I',data.read_bytes()))
        if expected is not None and words!=expected:raise ValueError(f'zero-timeout event/worker semantics differ: {words}')
    (output/'native-arm-threads.json').write_text(json.dumps({'backend':'native ARM shared JIT','host_sha256':sha(host),'thread_inputs':inputs(output),'poll_source_sha256':sha(source),'rows':rows},indent=2)+'\n')
    print('26 original worker/TLS words and 12 zero-timeout poll/worker words pass')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--host',type=Path,default=ROOT/'local/jit32/shared/startup-arm64');check(p.parse_args().host.resolve())
