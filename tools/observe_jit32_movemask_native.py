#!/usr/bin/env python3
"""Independent native x64 Windows legacy SIMD sign-mask observations; accepts no game inputs."""
import ctypes,hashlib,json,os,platform,shutil,struct,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def sha(p):
    import hashlib
    return hashlib.sha256(p.read_bytes()).hexdigest()
def observe():
    if os.name!='nt':raise RuntimeError('requires native x64 Windows')
    from ctypes import wintypes
    kernel=ctypes.WinDLL('kernel32',use_last_error=True)
    query=kernel.IsWow64Process2;query.argtypes=[wintypes.HANDLE,ctypes.POINTER(wintypes.WORD),ctypes.POINTER(wintypes.WORD)];query.restype=wintypes.BOOL
    process,native=wintypes.WORD(),wintypes.WORD()
    if not query(ctypes.c_void_p(-1),ctypes.byref(process),ctypes.byref(native)):raise ctypes.WinError(ctypes.get_last_error())
    if process.value!=0 or native.value!=0x8664:raise RuntimeError('refusing translated/non-x64 Windows observations')
    output=ROOT/'local/jit32/native-movemask-reference';output.mkdir(parents=True,exist_ok=True)
    source=ROOT/'jit32/tests/movemask_reference.c';host=output/'x64-observer.exe';data=output/'x64-observations.bin'
    subprocess.run([shutil.which('clang') or 'clang','-O1','-mno-red-zone','-Wall','-Wextra','-Werror',str(source),'-o',str(host)],check=True)
    raw=host.read_bytes();at=struct.unpack_from('<I',raw,0x3c)[0]
    if raw[at:at+4]!=b'PE\0\0' or struct.unpack_from('<H',raw,at+4)[0]!=0x8664:raise ValueError('observer is not PE64 AMD64')
    with data.open('wb') as stream:r=subprocess.run([str(host)],stdout=stream,stderr=subprocess.PIPE,timeout=30,check=True)
    if data.stat().st_size!=65556*64:raise ValueError(f'incomplete native observation: {data.stat().st_size} bytes, expected {65556*64}')
    report=dict(backend='native Windows x64 legacy SIMD sign-mask assembly',source_sha256=sha(source),source_lf_sha256=hashlib.sha256(source.read_bytes().replace(b'\r\n',b'\n')).hexdigest(),observer_sha256=sha(host),observation_sha256=sha(data),rows=65556,native_machine=native.value,process_machine=process.value,host=platform.platform(),processor=platform.processor(),stderr_hex=r.stderr.hex(),revision=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip())
    (output/'report.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':observe()
