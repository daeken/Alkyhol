#!/usr/bin/env python3
"""Original PE32 processes measure the mutex acquisition actually reached by DAO."""
import argparse,json,queue,shutil,struct,subprocess,tempfile,threading
from pathlib import Path
from build_pe_fixture import ROOT,rust_tool
from check_pe32_activation import SOURCE,digest
from observe_pe32_activation_startup import runner
from observe_pe32_mutex_create import HOST_BANNER
FILES=[f'mutex-ownership-{i}.exe' for i in range(5)]
SOURCES=['mutex-ownership.c','mutex-ownership.def']
SYMBOLS=[('GetStdHandle',4),('WriteFile',20),('ReadFile',20),('GetLastError',0),('SetLastError',4),('CreateMutexW',12),('CloseHandle',4),('WaitForSingleObjectEx',12),('ReleaseMutex',4)]
E=0xface1234;MAGIC=0x4f574d32
def inputs(d):return {'sources':{(SOURCE/n).relative_to(ROOT).as_posix():digest(SOURCE/n) for n in SOURCES},'files':{n:digest(d/n) for n in FILES}}
def verify(d):
    r=json.loads((d/'mutex-ownership-inputs.json').read_text())
    if r!=inputs(d):raise ValueError('mutex ownership source/PE identity changed')
    if r!=frozen()['inputs']:raise ValueError('mutex ownership identity differs from frozen native evidence')
    return r
def build(d):
    d.mkdir(parents=True,exist_ok=True);clang=shutil.which('clang');lld=rust_tool('rust-lld');lib=d/'mutex-ownership.lib'
    if not clang:raise RuntimeError('open Clang is required')
    subprocess.run([lld,'-flavor','link','/lib','/machine:x86','/def:'+str(SOURCE/'mutex-ownership.def'),'/out:'+str(lib)],check=True)
    for mode,name in enumerate(FILES):
        obj=d/(name+'.obj');subprocess.run([clang,'--target=i686-pc-windows-msvc','-O1','-ffreestanding','-fno-builtin','-fno-stack-protector','-fno-unwind-tables','-fno-asynchronous-unwind-tables','-Wall','-Wextra','-Werror','-Wno-unused-function',f'-DMODE={mode}','-c',str(SOURCE/'mutex-ownership.c'),'-o',str(obj)],check=True)
        subprocess.run([lld,'-flavor','link','/nodefaultlib','/subsystem:console','/machine:x86','/dynamicbase','/fixed:no','/timestamp:0','/safeseh:no','/manifest:no','/entry:entry',str(obj),str(lib),*[f'/alternatename:__imp__{n}@{s}=__imp__{n}' for n,s in SYMBOLS],'/out:'+str(d/name)],check=True)
        literal=ROOT/f'pe/reference/pe32-mutex-ownership-{mode}-v1.exe'
        if digest(literal)!=frozen()['inputs']['files'][name]:raise ValueError('changed literal native ownership PE')
        shutil.copyfile(literal,d/name)
    (d/'mutex-ownership-inputs.json').write_text(json.dumps(inputs(d),indent=2)+'\n');verify(d)
def words(data,expected):
    if len(data)!=len(expected)*4:raise ValueError('incomplete ownership transport')
    got=list(struct.unpack('<'+'I'*len(expected),data))
    if got!=expected:raise ValueError(f'mutex ownership, error or caller-stack result differs: {got}')
    return got
def lifecycle(data):
    expected=[MAGIC,4,1,0,1,183,0,288,0,E,0,E,1,E,1,E,0,288,0,E,1,E,1,E,1,E,1,28]
    return words(data,expected)
def frozen():
    path=ROOT/'pe/reference/pe32-mutex-ownership-windows-v1.json'
    if digest(path)!='0b141673543db2614757811bf53616ca3209468c9ca09d1596f07a3a5fc4dfc4':raise ValueError('changed frozen native ownership report')
    r=json.loads(path.read_bytes())
    if r['backend']!='native Windows mutex ownership' or [x['case'] for x in r['reports']]!=['recursive','handoff','abandoned']:raise ValueError('incomplete native ownership cases')
    for row in r['reports']:
        if row['case']=='recursive':
            if row['status'] or row['stderr_hex'] or lifecycle(bytes.fromhex(row['stdout_hex']))!=row['words']:raise ValueError('failed native recursive observer')
        else:
            abandoned=row['case']=='abandoned'
            if row['client_status'] or row['client_stderr_hex'] or row['holder_stderr_hex'] or (row['holder_status']==0 if abandoned else row['holder_status']!=0):raise ValueError('failed native ownership process state')
            words(bytes.fromhex(row['holder_ready_hex']),[MAGIC,4,1,0,0,E])
            words(bytes.fromhex(row['client_ready_hex']),[MAGIC,4,1,183,258,E])
            words(bytes.fromhex(row['holder_tail_hex']),[] if abandoned else [1,E,1,E])
            if words(bytes.fromhex(row['client_tail_hex']),[0x80 if abandoned else 0,E,0,E,1,E,1,E,1,E])!=row['words']:raise ValueError('invalid native ownership tail')
    return r
def ready(process):
    result=queue.Queue()
    def read():
        try:result.put(process.stdout.read(24))
        except Exception as e:result.put(e)
    threading.Thread(target=read,daemon=True).start()
    try:r=result.get(timeout=20)
    except queue.Empty:raise RuntimeError('guest mutex readiness budget exhausted')
    if isinstance(r,Exception):raise r
    return r
def execute(d,native=False):
    identity=verify(d);probe=None if native else runner(d);reports=[];path=d/('native-mutex-ownership.json' if native else 'mutex-ownership-host.json')
    def save():path.write_text(json.dumps({'backend':'native Windows mutex ownership' if native else 'Rosetta OS-backed mutex ownership','inputs':identity,'reports':reports},indent=2)+'\n')
    stderr_expected=b'' if native else HOST_BANNER
    with tempfile.TemporaryDirectory(prefix='alkyhol-owned-mutex-') as directory:
        def command(mode):
            args=[str((d/FILES[mode]).resolve())]
            return args if native else [str(probe),*args,'--strict','--file-origins','--object-directory',directory]
        r=subprocess.run(command(0),capture_output=True,timeout=20)
        row={'case':'recursive','status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()};reports.append(row);save()
        if r.returncode or r.stderr!=stderr_expected:raise ValueError('recursive ownership observer failed; raw output retained')
        row['words']=lifecycle(r.stdout);save()
        for abandoned in [False,True]:
            holder=client=None;row={'case':'abandoned' if abandoned else 'handoff'};reports.append(row);save()
            try:
                holder=subprocess.Popen(command(1),stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
                raw=ready(holder);row['holder_ready_hex']=raw.hex();save();words(raw,[MAGIC,4,1,0,0,E])
                client=subprocess.Popen(command(2),stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
                raw=ready(client);row['client_ready_hex']=raw.hex();save();words(raw,[MAGIC,4,1,183,258,E])
                if abandoned:holder.kill()
                out,err=holder.communicate(None if abandoned else b'x',timeout=20)
                row.update(holder_status=holder.returncode,holder_tail_hex=out.hex(),holder_stderr_hex=err.hex());save()
                if err!=stderr_expected or (holder.returncode==0 if abandoned else holder.returncode!=0):raise ValueError('unexpected owner termination')
                if abandoned:
                    if out:raise ValueError('abruptly terminated owner emitted a release')
                else:words(out,[1,E,1,E])
                out,err=client.communicate(b'x',timeout=20)
                row.update(client_status=client.returncode,client_tail_hex=out.hex(),client_stderr_hex=err.hex());save()
                if client.returncode or err!=stderr_expected:raise ValueError('waiting guest process failed')
                row['words']=words(out,[0x80 if abandoned else 0,E,0,E,1,E,1,E,1,E]);save()
            finally:
                for p in [client,holder]:
                    if p is not None and p.poll() is None:p.kill();p.communicate(timeout=20)
        if not native:
            for mode in [3,4]:
                r=subprocess.run(command(mode),capture_output=True,timeout=20)
                reports.append({'case':'excluded-'+str(mode),'status':r.returncode,'stdout_hex':r.stdout.hex(),'stderr_hex':r.stderr.hex()});save()
                if r.returncode!=78 or r.stdout or b'mutex waits require zero timeout and alertable FALSE' not in r.stderr:raise ValueError('unsupported wait mode did not stop explicitly')
    if native:
        peer=json.loads((d/'mutex-ownership-host.json').read_text())
        if peer['inputs']!=identity:raise ValueError('native/host ownership producer/PE hashes differ')
        for nr,hr in zip(reports,peer['reports'][:3],strict=True):
            if nr['case']!=hr['case'] or nr['words']!=hr['words']:raise ValueError('native/host ownership semantics differ')
            keys=['stdout_hex'] if nr['case']=='recursive' else ['holder_ready_hex','client_ready_hex','holder_tail_hex','client_tail_hex']
            if any(nr[key]!=hr[key] for key in keys):raise ValueError('native/host raw ownership transcripts differ')
            if nr['case']=='recursive':
                if hr['status'] or bytes.fromhex(hr['stderr_hex'])!=HOST_BANNER:raise ValueError('invalid peer recursive execution')
            elif hr['client_status'] or bytes.fromhex(hr['client_stderr_hex'])!=HOST_BANNER or bytes.fromhex(hr['holder_stderr_hex'])!=HOST_BANNER or (hr['holder_status']==0 if nr['case']=='abandoned' else hr['holder_status']!=0):raise ValueError('invalid peer process lifetime')
    print('Real recursive acquisition/release, contended poll, process handoff and owner-death abandonment observed')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/startup-microtests');m=p.add_mutually_exclusive_group();m.add_argument('--native',action='store_true');m.add_argument('--host',action='store_true');a=p.parse_args();d=a.output.resolve();execute(d,a.native) if a.native or a.host else build(d)
