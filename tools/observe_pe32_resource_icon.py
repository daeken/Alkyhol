#!/usr/bin/env python3
"""Resource-backed LoadIcon at DAO's actual frontier; owned pixels only in CI."""
import argparse,json,shutil,struct,subprocess
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest,resource_record
from observe_pe32_activation_startup import runner
from pe32_resource_icon_data import resources,pixels
FILES=[f'resource-icon-{i}.exe' for i in range(3)]
SOURCES=['resource-icon.c','resource-icon-kernel32.def','resource-icon-user32.def','resource-icon-reference.cpp']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleHandleW',4),('LoadIconW',8)]
def inputs(d):return {'sources':{**{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'tools/pe32_resource_icon_data.py':digest(ROOT/'tools/pe32_resource_icon_data.py')},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'resource-icon-inputs.json').read_text())
    actual=inputs(d)
    if r!=actual:
        changed=[key for kind in r for key in r[kind] if r[kind][key]!=actual[kind].get(key)]
        raise ValueError('resource-icon producer/PE identity changed: '+repr(changed))
    if r!=frozen()['inputs']:raise ValueError('icon identity differs from frozen native evidence')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');libs=[]
    if not clang:raise RuntimeError('open Clang is required')
    res=d/'resource-icon.res';res.write_bytes(resource_record(0,0,0,b'')+b''.join(resource_record(*r) for r in resources()))
    for role in ['kernel32','user32']:
        lib=d/f'resource-icon-{role}.lib';subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/f'resource-icon-{role}.def'),'/out:'+str(lib)],check=True);libs.append(str(lib))
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function',f'-DMODE={mode}','-c',str(SOURCE/'resource-icon.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(res),*libs,*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
        literal=ROOT/f'pe/reference/pe32-resource-icon-{mode}-v1.exe'
        if digest(literal)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native icon PE')
        shutil.copyfile(literal,d/name)
    (d/'resource-icon-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
def decode(data):
    expected=[0x49434f32,4,1,0,1,0,1,1813,1,10]
    if len(data)!=40 or list(struct.unpack('<10I',data))!=expected:raise ValueError('icon load/cache/missing/error/stack observations differ: '+data.hex())
    return expected
def audit(data):
    if len(data)!=4248 or struct.unpack_from('<6I',data)!= (0x4149434f,1,32,32,4096,128):raise ValueError('icon snapshot dimensions/count/extent differ')
    return data[24:4120],data[4120:]
def rgb(data):return bytes(v for i,v in enumerate(data) if i%4!=3)
def frozen():
    path=ROOT/'pe/reference/pe32-resource-icon-windows-v1.json'
    if digest(path)!='3b9a22f85a41334ed546e1f34f0cb0f9687f2a9635e19ef7dd54a257d0c3c6e5':raise ValueError('changed native icon loader report')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows icon loader' or [x['case'] for x in r['reports']]!=[0,'SDK-GDI-readback']:raise ValueError('incomplete native icon observations')
    for row in r['reports']:
        if row['status'] or row['stderr_hex']:raise ValueError('failed native icon/reference execution')
    if decode(bytes.fromhex(r['reports'][0]['stdout_hex']))!=r['reports'][0]['words']:raise ValueError('incomplete native loader words')
    image=ROOT/'pe/reference/pe32-resource-icon-gdi-v1.bin'
    if digest(image)!='41732b8eea027a41ea80cc67d95515a2db2e101d25d38d32954c786c7efa9ba7':raise ValueError('changed native GDI icon readback')
    audit(image.read_bytes());return r
def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-resource-icon.json' if native else 'resource-icon-host.json')
    def readback(save):
        reference=d/'resource-icon-reference.exe';target=d/'resource-icon-native.bin'
        r=subprocess.run([str(reference),str((d/FILES[0]).resolve()),str(target)],capture_output=True,timeout=20)
        reports.append({'case':'SDK-GDI-readback','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()});save()
        if r.returncode or r.stderr:raise ValueError('native SDK icon readback failed')
    for mode in ([0] if native else range(3)):
        command=[str((d/FILES[mode]).resolve())]
        if not native:command=[str(probe),*command,'--strict','--file-origins','--icon-audit',str(d/'resource-icon-host.bin')]
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row)
        def save():path.write_text(json.dumps({'backend':'native Windows icon loader' if native else 'Rosetta resource-backed icons','inputs':identity,'reports':reports},indent=2)+'\n')
        save()
        if mode==0:
            if r.returncode or (native and r.stderr):raise ValueError('icon observer failed; raw result retained')
            if native:readback(save) # Preserve independent pixels even if an initial error-state hypothesis fails.
            row['words']=decode(r.stdout);save()
        elif r.returncode!=78 or r.stdout or b'LoadIconW system icons and string names are unsupported' not in r.stderr:raise ValueError('unsupported icon form did not stop explicitly')
    if native:
        native_pixels,native_mask=audit((d/'resource-icon-native.bin').read_bytes());host_pixels,host_mask=audit((d/'resource-icon-host.bin').read_bytes())
        # BI_RGB's fourth byte is reserved in GetDIBits output. Retain it raw;
        # compare the defined RGB bytes/mask, not an invented alpha guarantee.
        if rgb(native_pixels)!=rgb(host_pixels) or native_mask!=host_mask:raise ValueError('native GDI and decoded resource RGB/mask differ')
        peer=json.loads((d/'resource-icon-host.json').read_text())
        if peer['inputs']!=identity or peer['reports'][0]['status'] or decode(bytes.fromhex(peer['reports'][0]['stdout_hex']))!=reports[0]['words']:raise ValueError('native/host loader identity or observations differ')
    else:
        color,mask=audit((d/'resource-icon-host.bin').read_bytes())
        if color!=pixels(32) or mask!=bytes(128):raise ValueError('decoded source pixels/mask do not match the owned producer')
        nc,nm=audit((ROOT/'pe/reference/pe32-resource-icon-gdi-v1.bin').read_bytes())
        if rgb(nc)!=rgb(color) or nm!=mask:raise ValueError('host pixels/mask differ from frozen native GDI observation')
    print('Actual resource-backed shared icon, missing-resource failure and decoded pixels checked; drawing remains separate')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
