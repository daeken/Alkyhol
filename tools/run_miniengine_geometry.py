#!/usr/bin/env python3
"""Compare native MSVC H3D ABI/expanded bytes with the Rust reader, not DXR pixels."""
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
from fetch_miniengine import ROOT,fetch
if __name__=='__main__':
    assets=fetch();model=assets/'Sponza/sponza.h3d'
    with tempfile.TemporaryDirectory(prefix='alkyhol-h3d-') as scratch:
        scratch=Path(scratch);output=scratch/'expanded.bin'
        if os.name=='nt':
            executable=scratch/'h3d_geometry.exe'
            subprocess.run(['cl','/nologo','/std:c++17','/EHsc','/W4','/WX',str(ROOT/'miniengine/reference/h3d_geometry.cpp'),'/Fe:'+str(executable)],check=True,cwd=scratch)
            command=[str(executable),str(model),str(output)];backend='native MSVC H3D ABI'
            compiler=subprocess.run(['cl'],capture_output=True,text=True,timeout=30)
        else:
            command=['cargo','run','--manifest-path',str(ROOT/'miniengine/Cargo.toml'),'--locked','--example','geometry_stream','--',str(model),str(output)];backend='Rust H3D reader'
            compiler=subprocess.run(['rustc','--version','--verbose'],capture_output=True,text=True,check=True,timeout=30)
        subprocess.run(command,check=True,cwd=ROOT,timeout=180)
        data=output.read_bytes()
        assert len(data)==34*8+836757*56
        report={'scenario':'sponza-h3d-expanded-v1','backend':backend,'host':platform.platform(),'compiler':compiler.stdout+compiler.stderr,'revision':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'tracked_tree_clean':subprocess.run(['git','diff','--quiet','HEAD'],cwd=ROOT).returncode==0,'asset_sha256':hashlib.sha256(model.read_bytes()).hexdigest(),'expanded_bytes':len(data),'expanded_sha256':hashlib.sha256(data).hexdigest()}
        (ROOT/'miniengine-geometry.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
