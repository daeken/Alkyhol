#!/usr/bin/env python3
"""Extract our owned i386 helper templates for an ARM-hosted CPU executor.

Only assembler-produced Mach-O objects in OUT_DIR are read. No original game
inputs are involved. Local branches are assembled; service-address slots remain
patched by the existing runtime's checked template installer.
"""
import argparse,struct,subprocess
from pathlib import Path

def extract(data):
    def take(off,n):
        if off<0 or n<0 or off+n>len(data):raise ValueError('truncated owned object')
        return data[off:off+n]
    def u32(off):return struct.unpack('<I',take(off,4))[0]
    if u32(0)!=0xfeedfacf or u32(4)!=0x1000007:raise ValueError('expected owned x86-64 assembler object')
    offset=32;text=None;symbols=None
    for _ in range(u32(16)):
        cmd,size=u32(offset),u32(offset+4)
        if size<8:raise ValueError('invalid load command')
        take(offset,size)
        if cmd==0x19:
            for i in range(u32(offset+64)):
                section=offset+72+i*80
                if take(section,16).split(b'\0')[0]==b'__text':
                    address,length=struct.unpack('<QQ',take(section+32,16));text=(address,take(u32(section+48),length))
        if cmd==2:symbols=struct.unpack('<IIII',take(offset+8,16))
        offset+=size
    if text is None or symbols is None:raise ValueError('missing text/symbols')
    first,body=text;symoff,count,stroff,strlen=symbols;strings=take(stroff,strlen);labels={}
    for i in range(count):
        string,kind,section,desc,value=struct.unpack('<IBBHQ',take(symoff+i*16,16))
        if not kind&0xe0 and section:
            if string>=len(strings):raise ValueError('symbol string exceeds table')
            name=strings[string:].split(b'\0')[0].decode('ascii')
            labels[name]=value-first
    return body,labels

def generate(output):
    lines=['// Owned guest templates extracted from the unchanged assembly sources.']
    for source,prefix in [('ci_sqrt32','SQRT'),('ci_atan32','ATAN'),('raise32','RAISE')]:
        obj=output/(source+'.guest.o')
        subprocess.run(['xcrun','clang','-arch','x86_64','-c',f'src/{source}.S','-o',str(obj)],check=True)
        body,labels=extract(obj.read_bytes())
        stem='_alkyhol_'+source.replace('ci_','ci_',1)
        start=labels[stem+'_start'];end=labels[stem+'_end'];slot=labels[stem+('_service' if source=='raise32' else '_failure')]
        if not 0<=start<end<=len(body) or not start<=slot<=end-4:raise ValueError('invalid owned helper span/patch')
        lines.append(f'pub(super) static {prefix}: &[u8] = &'+str(list(body[start:end]))+';')
        lines.append(f'pub(super) const {prefix}_PATCH: usize = {slot-start};')
    (output/'guest_templates.rs').write_text('\n'.join(lines)+'\n')
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('output',type=Path);generate(p.parse_args().output)
