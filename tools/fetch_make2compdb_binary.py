#!/usr/bin/env python3
"""Fetch unchanged public-domain make2compdb from the pinned official release."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import urllib.request
from fetch_uconfig_binary import ROOT, URL, ARCHIVE_SHA256, verified

PE_SHA256 = '6f23be1906edbe2f9a9009918bc3c2800580f18dc2df04490e56b787d94fbafd'
SOURCE_SHA256 = '7b7740c0458cfe8f6520c65f814b5738290f2076b1468bbceeeea354a5e3701d'
LICENSE_SHA256 = '7e12e5df4bae12cb21581ba157ced20e1986a0508dd10d0e8a4ab9a4cf94e85c'
LICENSE_URL = 'https://raw.githubusercontent.com/skeeto/w64devkit/v2.11.0/UNLICENSE'


def fetch(output):
    if platform.system() != 'Darwin': raise RuntimeError('uses macOS libarchive; Windows receives the identical release files')
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    archive=ROOT/'local/pe/off-the-shelf/w64devkit-x64-2.11.0.7z.exe'
    archive.parent.mkdir(parents=True,exist_ok=True)
    if not archive.exists(): archive.write_bytes(urllib.request.urlopen(URL,timeout=90).read())
    verified(archive,ARCHIVE_SHA256)
    for member,digest in [('bin/make2compdb.exe',PE_SHA256),('src/make2compdb.c',SOURCE_SHA256)]:
        path=output/Path(member).name
        if not path.exists():path.write_bytes(subprocess.check_output(['/usr/bin/tar','-xOf',str(archive),'w64devkit/'+member]))
        verified(path,digest)
    license_path=output/'UNLICENSE'
    if not license_path.exists():license_path.write_bytes(urllib.request.urlopen(LICENSE_URL,timeout=30).read())
    verified(license_path,LICENSE_SHA256)
    manifest={'schema_version':1,'program':'make2compdb','version':'2026-09-15',
              'distribution':'w64devkit 2.11.0 x64','archive_url':URL,'archive_sha256':ARCHIVE_SHA256,
              'pe_member':'w64devkit/bin/make2compdb.exe','pe_sha256':PE_SHA256,
              'source_member':'w64devkit/src/make2compdb.c','source_sha256':SOURCE_SHA256,
              'license_url':LICENSE_URL,'license_sha256':LICENSE_SHA256,'rebuilt':False,'patched':False}
    (output/'provenance.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Verified unchanged released Windows make2compdb 2026-09-15')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/make2compdb')
    fetch(p.parse_args().output)
