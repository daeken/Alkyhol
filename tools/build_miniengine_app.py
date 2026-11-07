#!/usr/bin/env python3
"""Build the local MiniEngine Sponza geometry diagnostic app."""
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
from fetch_miniengine import ROOT,fetch
if __name__=='__main__':
    fetch();cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'miniengine/Cargo.toml'),'--locked','--release'],check=True,cwd=ROOT)
    bundle=ROOT/'local/miniengine/MiniEngine Sponza.app';exe=bundle/'Contents/MacOS/MiniEngine Sponza';exe.parent.mkdir(parents=True,exist_ok=True)
    temp=exe.with_suffix('.new');shutil.copy2(ROOT/'miniengine/target/release/alkyhol-miniengine',temp);os.replace(temp,exe)
    (bundle/'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleExecutable':'MiniEngine Sponza','CFBundleIdentifier':'org.alkyhol.miniengine','CFBundleName':'MiniEngine Sponza','CFBundlePackageType':'APPL','NSHighResolutionCapable':True}))
    print(bundle)
