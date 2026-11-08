#!/usr/bin/env python3
"""Reached DAO CreateDirectoryW: real scoped mutations, errors and stdcall ABI."""
import argparse,json,shutil,struct,subprocess,tempfile
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
FILES=[f'create-directory-{i}.exe' for i in range(7)]
SOURCES=['create-directory.c','create-directory.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('GetLastError',0),('SetLastError',4),('GetModuleFileNameW',12),('CreateDirectoryW',8)]
FIXTURE=b'owned regular-file collision, never replaced\n'
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES},'fixture_hex':FIXTURE.hex()}
def verify(d):
    r=json.loads((d/'create-directory-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('directory source/PE identity changed')
    if r!=frozen()['inputs']:raise ValueError('directory identity differs from literal native observation')
    return r

def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'create-directory.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'create-directory.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function',f'-DMODE={mode}','-c',str(SOURCE/'create-directory.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
        literal=ROOT/f'pe/reference/pe32-create-directory-{mode}-v1.exe'
        if digest(literal)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native directory-constructor PE')
        shutil.copyfile(literal,d/name)
    (d/'create-directory-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n')

def decode(data):
    if len(data)!=96:raise ValueError('incomplete directory creation report')
    words=list(struct.unpack('<24I',data))
    if words[:2]!=[0x4d4b4432,4] or words[-2:]!=[1,24]:raise ValueError('directory creation ABI/stack mismatch')
    expected=[(1,None),(0,183),(1,None),(0,3),(0,183),(0,183),(0,3),(0,183),(1,None),(0,183)]
    contract=[];success_errors=[]
    for i,(success,error) in enumerate(expected):
        result,last=words[2+i*2:4+i*2]
        if result!=success or error is not None and last!=error:raise ValueError('directory creation result/error differs at case '+str(i))
        contract.append([result,error])
        if success:success_errors.append(last)
    return {'words':words,'contract':contract,'success_last_errors_observed_only':success_errors,'stack_balanced':words[-2]}

def frozen():
    path=ROOT/'pe/reference/pe32-create-directory-windows-v1.json'
    if digest(path)!='c151a3b75b6f8fb3db741a35dfa567309891418f0fd993e5e107689b9ad1ebab':raise ValueError('changed native directory creation record')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows owned directory creation' or len(r['reports'])!=1:raise ValueError('incomplete native directory creation report')
    row=r['reports'][0]
    if row['case'] or row['status'] or row['stderr_hex'] or decode(bytes.fromhex(row['stdout_hex']))!=row['decoded']:raise ValueError('failed native directory creation observation')
    if row['filesystem']!={'first':True,'child':True,'trailing':True,'missing_parent_created':False,'file_hex':FIXTURE.hex()}:raise ValueError('failed native directory effects')
    return r

def execute(d,native=False):
    identity=verify(d);reports=[];probe=None if native else runner(d)
    path=d/('native-create-directory.json' if native else 'create-directory-host.json')
    def save():path.write_text(json.dumps({'backend':'native Windows owned directory creation' if native else 'Rosetta Documents-scoped directory creation','inputs':identity,'reports':reports},indent=2)+'\n')
    for mode in ([0] if native else range(8)):
        # Every observation owns a fresh directory; no cleanup touches an
        # existing install, unknown user files or a previous capture.
        run=Path(tempfile.mkdtemp(prefix='mkdir-',dir=d));root=run/'root';root.mkdir();outside=run/'root-outside';outside.mkdir();(root/'file').write_bytes(FIXTURE)
        if mode==4 and not native:(root/'linked').symlink_to(outside,target_is_directory=True)
        exe=run/FILES[0 if mode==7 else mode];shutil.copyfile(d/exe.name,exe)
        command=[str(exe.resolve())]
        if not native:
            command=[str(probe),*command,'--strict','--file-origins']
            if mode!=7:command+=['--documents-directory',str(root)]
        r=subprocess.run(command,capture_output=True,timeout=20);row={'case':mode,'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex(),'owned_root':str(root)};reports.append(row);save()
        if mode==0:
            if r.returncode or r.stderr!=(b'' if native else HOST_BANNER):raise ValueError('directory observer failed; raw output retained')
            row['decoded']=decode(r.stdout)
            # Exact finite native observation, including its recorded successful
            # LastError values; this is not a general API guarantee.
            if r.stdout!=bytes.fromhex(frozen()['reports'][0]['stdout_hex']):raise ValueError('directory output differs from complete native capture')
            row['filesystem']={'first':(root/'first').is_dir(),'child':(root/'first'/'child').is_dir(),'trailing':(root/'trailing').is_dir(),'missing_parent_created':(root/'missing').exists(),'file_hex':(root/'file').read_bytes().hex()}
            if row['filesystem']!={'first':True,'child':True,'trailing':True,'missing_parent_created':False,'file_hex':FIXTURE.hex()}:raise ValueError('real directory effects differ from returned results')
            save()
        else:
            reasons={1:b'directory creation security attributes are unsupported',2:b'directory creation is restricted to the explicit Documents backing',3:b'PE32 find supports exact ordinary paths',4:b'PE32 find symbolic links/reparse points are unsupported',5:b'directory creation requires measured ASCII descendant names',6:b'directory creation is restricted to the explicit Documents backing',7:b'directory creation requires an explicit Documents backing'}
            if r.returncode!=78 or r.stdout or reasons[mode] not in r.stderr:raise ValueError('directory exclusion failed for a different reason: '+str(mode))
            if (root/'first').exists():raise ValueError('rejected directory call mutated the profile')
        if list(outside.iterdir()) or (run/'escape').exists():raise ValueError('directory creation escaped its owned profile')
        save()
    if native:
        peer=json.loads((d/'create-directory-host.json').read_text());row=peer['reports'][0]
        if peer['inputs']!=identity or row['status'] or decode(bytes.fromhex(row['stdout_hex']))!=row['decoded'] or row['decoded']['contract']!=reports[0]['decoded']['contract'] or row['filesystem']!=reports[0]['filesystem']:raise ValueError('native/host directory identity, state or documented result/error contract differs')
    print('Real directory creation, nested state, duplicates, file collisions, parent failures and stack checks passed')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
