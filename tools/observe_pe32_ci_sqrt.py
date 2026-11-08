#!/usr/bin/env python3
"""Frozen native x87 evidence and a separate bounded _CIsqrt runtime witness."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
BASE_SOURCES=['ci-sqrt.c','ci-sqrt.S','ci-sqrt.def','crt-kernel32.def']
FILES=['ci-sqrt.exe','ci-sqrt-domain.exe',*[f'ci-sqrt-bad-{i}.exe' for i in range(1,5)]]
def inputs(d):
    return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in [*BASE_SOURCES,'ci-sqrt-domain.c']},'files':{n:digest(d/n) for n in FILES}}
def frozen():
    p=ROOT/'pe/reference/pe32-ci-sqrt-windows-v1.json'
    if digest(p)!='7b1d29fd99c35c12a318f0a0d70661a943c78b25c181ba4f1065faed10930374':raise ValueError('changed frozen native x87 report')
    r=json.loads(p.read_bytes())
    if r['backend']!='native Windows direct x87 _CIsqrt observer' or r['status'] or r['stderr_hex'] or decode(bytes.fromhex(r['stdout_hex']))!=r['records']:raise ValueError('incomplete frozen native x87 evidence')
    return r
def verify(d):
    r=json.loads((d/'ci-sqrt-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('x87 observer source/PE identity changed')
    original=frozen()['inputs']
    if r['files']['ci-sqrt.exe']!=original['files']['ci-sqrt.exe']:raise ValueError('changed literal frozen x87 PE')
    for name in BASE_SOURCES:
        key=(SOURCE/name).relative_to(ROOT).as_posix()
        if r['sources'][key]!=original['sources'][key]:raise ValueError('changed frozen x87 source')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    literal=ROOT/'pe/reference/pe32-ci-sqrt-v1.exe'
    if digest(literal)!=frozen()['inputs']['files']['ci-sqrt.exe']:raise ValueError('changed literal x87 PE')
    shutil.copyfile(literal,d/'ci-sqrt.exe')
    for name in ['ci-sqrt','crt-kernel32']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    flags=[clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror']
    asm=d/'ci-sqrt-S.obj';subprocess.run([*flags,'-c',str(SOURCE/'ci-sqrt.S'),'-o',str(asm)],check=True)
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('SetLastError',4),('GetLastError',0)]]
    for mode in range(5):
        name='ci-sqrt-domain' if mode==0 else f'ci-sqrt-bad-{mode}';obj=d/(name+'.obj')
        subprocess.run([*flags,f'-DBAD={mode}','-c',str(SOURCE/'ci-sqrt-domain.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(asm),*libs,*aliases,'/out:'+str(d/(name+'.exe'))],check=True)
    (d/'ci-sqrt-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
    print('Selected literal native x87 observer and built the separate bounded profile/restriction PEs')
def row_record(b,index,requested):
    r=b[4:64]
    if struct.unpack_from('<2I',r)!=(index,requested) or any(struct.unpack_from('<I',b,at)[0]!=0x715e7a11 for at in [0,60,64]) or r[18:20]!=b'\xa5\xa5':raise ValueError('x87 observer order/control/guard damaged')
    cw,sw,cwa,swa,swp=struct.unpack_from('<5H',r,8);mx,mxa,error,errno=struct.unpack_from('<4I',r,40)
    return {'input':index,'requested':requested,'cw_before':cw,'sw_before':sw,'cw_after':cwa,'sw_after':swa,'sw_pop':swp,'result80':r[20:30].hex(),'below80':r[30:40].hex(),'mx_before':mx,'mx_after':mxa,'last_error':error,'errno':errno}
def decode(data):
    if len(data)!=24+204*68 or struct.unpack_from('<6I',data)!=(0x53513332,4,60,68,17,12):raise ValueError('incomplete x87 observer transport')
    return [row_record(data[24+i*68:24+(i+1)*68],i%17,0x7f|[0,0x200,0x300][i//68]|(((i//17)%4)<<10)) for i in range(204)]
def decode_domain(data):
    if len(data)!=24+24*68 or struct.unpack_from('<6I',data)!=(0x53513332,4,60,68,12,2):raise ValueError('incomplete bounded x87 transport')
    return [row_record(data[24+i*68:24+(i+1)*68],i%12,[0x27f,0x37f][i//12]) for i in range(24)]
def compare_known(rows):
    baseline={(r['input'],r['requested']):r for r in frozen()['records']}
    for row in rows:
        if row['input']>=10:continue
        expected=baseline[(row['input'],row['requested'])]
        if row!=expected:
            diff={k:[expected[k],row[k]] for k in row if row[k]!=expected[k]}
            raise ValueError(f'x87 known-domain mismatch input={row["input"]} control={row["requested"]:x}: {diff}')
def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[]
    names=['ci-sqrt.exe','ci-sqrt-domain.exe'] if native else FILES[1:]
    path=d/('native-ci-sqrt-profile.json' if native else 'ci-sqrt-host.json')
    for name in names:
        command=[str((d/name).resolve())]
        if not native:command=[str(probe),*command,'--strict']
        r=subprocess.run(command,capture_output=True,timeout=20)
        report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report)
        def save():path.write_text(json.dumps({'backend':'native Windows x87 profile observer' if native else 'Rosetta bounded x87 host assertions','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if 'bad-' in name:
            head=struct.pack('<6I',0x53513332,4,60,68,1,1)
            if r.returncode!=78 or r.stdout!=head or b'outside the measured nonnegative finite x87 profile' not in r.stderr:raise ValueError('unsupported x87 input/control did not stop for its real reason')
        else:
            if r.returncode or native and r.stderr:raise ValueError('x87 observer failed; raw report retained')
            report['records']=decode(r.stdout) if name=='ci-sqrt.exe' else decode_domain(r.stdout)
            if name=='ci-sqrt.exe':
                if report['records']!=frozen()['records']:raise ValueError('native full x87 observations changed')
            else:compare_known(report['records'])
        save()
    if native:
        host=json.loads((d/'ci-sqrt-host.json').read_text())
        if host['inputs']!=identity:raise ValueError('x87 host/native input identities differ')
        h=next(r for r in host['reports'] if r['file']=='ci-sqrt-domain.exe')
        if h['status'] or decode_domain(bytes.fromhex(h['stdout_hex']))!=h['records'] or h['records']!=reports[-1]['records']:raise ValueError('native and translated bounded x87 records differ')
        print('All 24 identical-PE bounded x87 records agree, and all 204 frozen native observations are preserved')
    else:print('20 known native x87 rows agree; four new rounding-boundary rows need native comparison; four restrictions stop honestly')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
