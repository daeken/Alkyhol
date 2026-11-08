#!/usr/bin/env python3
"""Frozen native arctangent evidence and bounded runtime comparisons for DAO."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_ci_sqrt import decode,row_record
BASE=['ci-atan.c','ci-atan.S','ci-atan.def','ci-sqrt.c','ci-sqrt.S','crt-kernel32.def']
FILES=['ci-atan.exe','ci-atan-domain.exe',*[f'ci-atan-bad-{i}.exe' for i in range(1,6)]]
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in [*BASE,'ci-atan-domain.c']},'files':{n:digest(d/n) for n in FILES}}
def frozen():
    p=ROOT/'pe/reference/pe32-ci-atan-windows-v1.json'
    if digest(p)!='ce77f0714d4d0d7e85e415c00888d9288cf7eafd6803d778bb252fbbd28286b2':raise ValueError('changed frozen atan evidence')
    r=json.loads(p.read_bytes())
    if r['backend']!='native Windows direct x87 _CIatan observer' or r['status'] or r['stderr_hex'] or decode(bytes.fromhex(r['stdout_hex']))!=r['records']:raise ValueError('incomplete native atan evidence')
    return r

def verify(d):
    r=json.loads((d/'ci-atan-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('arctangent source/PE identity changed')
    original=frozen()['inputs']
    if r['files']['ci-atan.exe']!=original['files']['ci-atan.exe']:raise ValueError('changed literal native atan PE')
    for name in BASE:
        key=(SOURCE/name).relative_to(ROOT).as_posix()
        if r['sources'][key]!=original['sources'][key]:raise ValueError('changed native atan source')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    literal=ROOT/'pe/reference/pe32-ci-atan-v1.exe'
    if digest(literal)!=frozen()['inputs']['files']['ci-atan.exe']:raise ValueError('changed native atan PE')
    shutil.copyfile(literal,d/'ci-atan.exe')
    for name in ['ci-atan','crt-kernel32']:
        lib=d/(name+'.lib');subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/(name+'.def')),'/out:'+str(lib)],check=True);libs.append(str(lib))
    flags=[clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror']
    asm=d/'ci-atan-S.obj';subprocess.run([*flags,'-c',str(SOURCE/'ci-atan.S'),'-o',str(asm)],check=True)
    aliases=[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in [('GetStdHandle',4),('WriteFile',20),('SetLastError',4),('GetLastError',0)]]
    for mode in range(6):
        name='ci-atan-domain' if mode==0 else f'ci-atan-bad-{mode}';obj=d/(name+'.obj')
        subprocess.run([*flags,f'-DBAD={mode}','-c',str(SOURCE/'ci-atan-domain.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(asm),*libs,*aliases,'/out:'+str(d/(name+'.exe'))],check=True)
    (d/'ci-atan-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d);print('Selected native atan reference and built signed finite profile witnesses')

def decode_domain(data):
    if len(data)!=24+16*68 or struct.unpack_from('<6I',data)!=(0x53513332,4,60,68,16,1):raise ValueError('incomplete bounded atan transport')
    return [row_record(data[24+i*68:24+(i+1)*68],i%16,0x37f) for i in range(16)]

def compare_known(rows):
    reference={(r['input'],r['requested']):r for r in frozen()['records']}
    for r in rows:
        if r['input']>9 and r['input']!=12:continue
        expected=dict(reference[(r['input'],r['requested'])])
        if r!=expected:
            diff={k:[expected[k],r[k]] for k in r if r[k]!=expected[k]}
            raise ValueError(f'atan input={r["input"]} control={r["requested"]:x} differs: {diff}')

def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[]
    path=d/('native-ci-atan-profile.json' if native else 'ci-atan-host.json')
    for name in FILES[:2] if native else FILES[1:]:
        command=[str((d/name).resolve())]
        if not native:command=[str(probe),*command,'--strict']
        r=subprocess.run(command,capture_output=True,timeout=20)
        report={'file':name,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(report)
        def save():path.write_text(json.dumps({'backend':'native Windows atan profile observer' if native else 'Rosetta bounded atan host assertions','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if 'bad-' in name:
            if r.returncode!=78 or r.stdout!=struct.pack('<6I',0x53513332,4,60,68,1,1) or b'outside the measured finite x87 profile' not in r.stderr:raise ValueError('unsupported atan state did not stop for its actual reason')
        else:
            if r.returncode or native and r.stderr:raise ValueError('atan witness failed; raw output retained')
            report['records']=decode(r.stdout) if name=='ci-atan.exe' else decode_domain(r.stdout)
            if name=='ci-atan.exe':
                if report['records']!=frozen()['records']:raise ValueError('native atan baseline changed')
            else:compare_known(report['records'])
        save()
    if native:
        host=json.loads((d/'ci-atan-host.json').read_text());h=next(r for r in host['reports'] if r['file']=='ci-atan-domain.exe')
        if host['inputs']!=identity or h['status'] or decode_domain(bytes.fromhex(h['stdout_hex']))!=h['records'] or h['records']!=reports[-1]['records']:raise ValueError('identical-PE native/host atan records differ')
        print('All 16 bounded atan records agree across identical PEs; full 204 native records remain unchanged')
    else:print('11 known native atan rows agree; five additional signed/boundary rows await native comparison; five restrictions stop')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();(lambda d:execute(d,True) if a.native else execute(d) if a.host else build(d))(a.output.resolve())
