#!/usr/bin/env python3
"""Build pinned upstream DXC locally; never install or alter system tools."""
import argparse
import os
from pathlib import Path
import platform
import subprocess

ROOT = Path(__file__).resolve().parents[1]
REVISION = "b106a961d09221b3c5bdb37be45b679257da08b8"  # v1.8.2505.1
URL = "https://github.com/microsoft/DirectXShaderCompiler.git"


def run(*args):
    subprocess.run([str(a) for a in args], check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jobs", type=int, default=min(os.cpu_count() or 2, 6))
    args = parser.parse_args()
    if args.jobs < 1:
        parser.error("--jobs must be positive")
    source = ROOT / "local/upstream/DirectXShaderCompiler"
    build = ROOT / "local/dxc-build"
    source.parent.mkdir(parents=True, exist_ok=True)
    if not source.exists():
        run("git", "clone", "--depth", "1", "--branch", "v1.8.2505.1",
            "--recurse-submodules", "--shallow-submodules", URL, source)
    revision = subprocess.check_output(["git", "-C", str(source), "rev-parse", "HEAD"], text=True).strip()
    clean = subprocess.check_output(["git", "-C", str(source), "status", "--porcelain"], text=True)
    if revision != REVISION or clean:
        raise SystemExit("DXC source must be the clean pinned revision; existing checkout left untouched")
    command = ["cmake", "-S", source, "-B", build, "-G", "Ninja",
               "-C", source / "cmake/caches/PredefinedParams.cmake", "-DCMAKE_BUILD_TYPE=Release"]
    if platform.system() == "Darwin":
        # Older LLVM StringRef specializations trigger a new Apple Clang diagnostic.
        # Keep upstream bytes intact; disable only that diagnostic for this dependency.
        command += ["-DCMAKE_CXX_FLAGS=-Wno-invalid-specialization"]
    run(*command)
    run("cmake", "--build", build, "--target", "dxc", "dxcompiler", "-j", args.jobs)
    executable = build / "bin" / ("dxc.exe" if os.name == "nt" else "dxc")
    run(executable, "--version")
    print(f"DXC={executable}")


if __name__ == "__main__":
    main()
