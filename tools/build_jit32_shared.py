#!/usr/bin/env python3
"""Build the original service runner with the checked native ARM JIT entry path."""
import argparse,os,shutil,subprocess,platform
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def build(output):
    if platform.system()!='Darwin' or platform.machine()!='arm64':raise RuntimeError('shared JIT host requires native Apple Silicon/macOS')
    from check_jit32 import verify_sharpretro
    verify_sharpretro()
    # Keep these large native debug hosts practical; callers can override either.
    os.environ.setdefault('CARGO_PROFILE_DEV_DEBUG','0')
    os.environ.setdefault('CARGO_INCREMENTAL','0')
    cargo=shutil.which('cargo') or str(Path.home()/'.cargo/bin/cargo')
    rustc=shutil.which('rustc') or str(Path.home()/'.cargo/bin/rustc')
    subprocess.run([cargo,'build','--manifest-path',str(ROOT/'jit32/Cargo.toml'),'--features','shared-runtime','--locked','--offline','--lib',*[item for package in ['alkyhol-jit32','alkyhol-pe','xfusion-recomp','sharpretro-jit'] for item in ['--config',f'profile.dev.package.{package}.opt-level=2']]],cwd=ROOT,check=True)
    target=Path(os.environ.get('CARGO_TARGET_DIR',str(ROOT/'jit32/target'))).resolve()/'debug'
    output.parent.mkdir(parents=True,exist_ok=True)
    subprocess.run([rustc,'--edition=2024','--cfg','alkyhol_native_jit32',str(ROOT/'pe/probe_startup32.rs'),'--extern','alkyhol_jit32='+str(target/'libalkyhol_jit32.rlib'),'-L','dependency='+str(target/'deps'),'-C','opt-level=2','-o',str(output)],cwd=ROOT,check=True)
    print('Built native ARM JIT host sharing existing PE32 services: '+str(output))
    return output
if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=ROOT/'local/jit32/shared/startup-arm64');build(p.parse_args().output.resolve())
