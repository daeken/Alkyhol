#!/usr/bin/env python3
"""Run unchanged frozen ABI/TLS/heap PEs through the native shared service path.

Does not accept game inputs or rewrite Windows observations. The existing
check_jit32[_startup] comparators separately cover actual Rosetta execution.
"""
import argparse,json,platform,re,subprocess,tempfile
from pathlib import Path
from check_jit32 import ROOT,sha,verify_sharpretro
from build_jit32_shared import build
SUMMARY=re.compile(rb'PE32 diagnostic mode: guarded external imports; every fault stops; no exception recovery\nJIT32 shared runtime: Stats \{ instructions: ([0-9]+), services: ([0-9]+), compiled: ([0-9]+), invocations: ([0-9]+) \}\n')
def decode(status,stdout,stderr,expected):
    match=SUMMARY.fullmatch(stderr)
    if type(status) is not int or status!=0 or stdout!=expected or match is None:
        raise ValueError(f'shared JIT differs: {status}, {stdout.hex()}, {stderr!r}')
    counts=list(map(int,match.groups()))
    if not all(n>0 for n in counts):raise ValueError('missing actual CPU/service evidence')
    return dict(zip(['instructions','services','compiled','invocations'],counts),status=status,stdout_hex=stdout.hex())
def check(output):
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('requires actual Apple Silicon native JIT')
    verify_sharpretro();host=build(output/'startup-arm64');rows={}
    cases=[('abi',ROOT/'local/pe/pe32/pe32.exe',ROOT/'pe/reference/pe32-windows-v1.json'),('tls',ROOT/'local/pe/pe32-tls-data/pe32-tls-data.exe',ROOT/'pe/reference/pe32-tls-data-windows-v1.json'),('heap',ROOT/'pe/reference/pe32-heap-realloc-v1.exe',ROOT/'pe/reference/pe32-heap-realloc-windows-v1.json')]
    for label,pe,reference in cases:
        frozen=json.loads(reference.read_bytes());sources=frozen['inputs']['sources'] if label=='heap' else frozen['sources']
        for source,identity in sources.items():
            if sha(ROOT/source)!=identity:raise ValueError('frozen source identity differs')
        native=frozen['reports'][0] if label=='heap' else frozen['normal'];digest=frozen['inputs']['files']['heap-realloc-0.exe'] if label=='heap' else frozen['pe_sha256']
        expected=bytes.fromhex(native['stdout_hex'])
        if sha(pe)!=digest or native['status'] or len(expected)!=dict(abi=32,tls=96,heap=256)[label]:raise ValueError('native fixture differs')
        before=(sha(pe),sha(reference));modes={}
        for forced in [False,True]:
            with tempfile.TemporaryFile() as stream:
                p=subprocess.run([str(host),str(pe),'--strict','--file-origins']+(['--rebase'] if forced else []),stdout=stream,stderr=subprocess.PIPE,timeout=30)
                stream.seek(0);modes['rebased' if forced else 'normal']=decode(p.returncode,stream.read(),p.stderr,expected)
        if before!=(sha(pe),sha(reference)):raise ValueError('input/reference mutated during execution')
        rows[label]={'pe_sha256':digest,'native_reference_sha256':sha(reference),'modes':modes}
    verify_sharpretro();report={'backend':'native ARM64 SharpRetro with shared Win32 services','host':platform.platform(),'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'tracked_tree_clean':not subprocess.check_output(['git','status','--porcelain','--untracked-files=no'],cwd=ROOT,text=True).strip(),'host_sha256':sha(host),'cases':rows}
    (output/'comparison.json').write_text(json.dumps(report,indent=2)+'\n');print('Shared native ARM services agree with frozen Windows: 8 ABI, 24 TLS and 64 heap words, normal and rebased');return report
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/jit32/shared-comparison');check(p.parse_args().output.resolve())
