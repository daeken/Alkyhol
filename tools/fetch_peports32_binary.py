#!/usr/bin/env python3
"""Fetch unchanged public-domain i386 peports from the pinned official release."""
import argparse
import hashlib
import json
from pathlib import Path
import platform
import subprocess
import urllib.request
from fetch_uconfig_binary import ROOT, verified

URL = 'https://github.com/skeeto/w64devkit/releases/download/v2.11.0/w64devkit-x86-2.11.0.7z.exe'
ARCHIVE_SHA256 = 'f7e8c79eacc4cbdd718a4cbb6519f2f370311017dc08f7750abedb4340d1b65d'

PE_SHA256 = '387d317019c36e86a26fa35818353d67dd47e9558e0b10819a2bb27a67fdeab9'
SOURCE_SHA256 = '8635550fa472b8393122174427bae61de0630a938ecd09c2718440978a72d670'
LICENSE_SHA256 = '7e12e5df4bae12cb21581ba157ced20e1986a0508dd10d0e8a4ab9a4cf94e85c'
LICENSE_URL = 'https://raw.githubusercontent.com/skeeto/w64devkit/v2.11.0/UNLICENSE'


def fetch(output):
    if platform.system() != 'Darwin': raise RuntimeError('uses macOS libarchive; Windows receives the identical release files')
    output=output.resolve();output.mkdir(parents=True,exist_ok=True)
    archive=ROOT/'local/pe/off-the-shelf/w64devkit-x86-2.11.0.7z.exe'
    archive.parent.mkdir(parents=True,exist_ok=True)
    if not archive.exists(): archive.write_bytes(urllib.request.urlopen(URL,timeout=90).read())
    verified(archive,ARCHIVE_SHA256)
    for member,digest in [('bin/peports.exe',PE_SHA256),('src/peports.c',SOURCE_SHA256)]:
        path=output/Path(member).name
        if not path.exists():path.write_bytes(subprocess.check_output(['/usr/bin/tar','-xOf',str(archive),'w64devkit/'+member]))
        verified(path,digest)
    license_path=output/'UNLICENSE'
    if not license_path.exists():license_path.write_bytes(urllib.request.urlopen(LICENSE_URL,timeout=30).read())
    verified(license_path,LICENSE_SHA256)
    manifest={'schema_version':1,'program':'peports','version':'w64devkit-2.11.0',
              'distribution':'w64devkit 2.11.0 x86','archive_url':URL,'archive_sha256':ARCHIVE_SHA256,
              'pe_member':'w64devkit/bin/peports.exe','pe_sha256':PE_SHA256,
              'source_member':'w64devkit/src/peports.c','source_sha256':SOURCE_SHA256,
              'license_url':LICENSE_URL,'license_sha256':LICENSE_SHA256,'rebuilt':False,'patched':False}
    (output/'provenance.json').write_text(json.dumps(manifest,indent=2)+'\n',encoding='utf-8')
    print('Verified unchanged released Windows peports w64devkit-2.11.0')


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/pe/peports32')
    fetch(p.parse_args().output)
