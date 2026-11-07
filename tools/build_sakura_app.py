#!/usr/bin/env python3
"""Build a local macOS app bundle; assets retain pinned hashes in ignored local/."""
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
from fetch_sakura import ROOT, fetch

if __name__ == '__main__':
    fetch()
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'sakura/Cargo.toml'),'--locked','--release'],check=True,cwd=ROOT)
    bundle=ROOT/'local/sakura/Sakura Forest.app'
    executable=bundle/'Contents/MacOS/Sakura Forest'
    executable.parent.mkdir(parents=True,exist_ok=True)
    temporary=executable.with_suffix('.new')
    shutil.copy2(ROOT/'sakura/target/release/alkyhol-sakura',temporary)
    os.replace(temporary,executable)
    (bundle/'Contents/Info.plist').write_bytes(plistlib.dumps({
        'CFBundleExecutable':'Sakura Forest', 'CFBundleIdentifier':'org.alkyhol.sakura',
        'CFBundleName':'Sakura Forest','CFBundlePackageType':'APPL','NSHighResolutionCapable':True,
    }))
    print(bundle)
