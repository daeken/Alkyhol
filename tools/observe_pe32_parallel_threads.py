#!/usr/bin/env python3
"""Observe simultaneous guest instructions and host-call integrity on an identical PE."""
import argparse
import json
import shutil
import struct
import subprocess
from pathlib import Path

from build_pe_fixture import ROOT, rust_tool
from check_pe32_activation import SOURCE, digest
from observe_pe32_activation_startup import runner


NAME = "parallel-threads"
INPUT = SOURCE / f"{NAME}.c"
SYMBOLS = {
    "CreateThread": 24,
    "WaitForSingleObject": 8,
    "GetExitCodeThread": 8,
    "CloseHandle": 4,
    "GetLastError": 0,
    "SetLastError": 4,
    "InterlockedExchangeAdd": 8,
    "GetProcessHeap": 0,
    "HeapAlloc": 12,
    "HeapFree": 12,
    "GetStdHandle": 4,
    "WriteFile": 20,
}
EXPECTED = (0x4C524150, 3, 6144, 0xABCD3210, 0xABCD3211, 0xABCD3212, 1)


def identity(exe):
    return {"source": digest(INPUT), "pe": digest(exe)}


def build(directory):
    directory.mkdir(parents=True, exist_ok=True)
    linker = rust_tool("rust-lld")
    definitions = directory / f"{NAME}.def"
    library = directory / f"{NAME}.lib"
    obj = directory / f"{NAME}.obj"
    exe = directory / f"{NAME}.exe"
    definitions.write_text("LIBRARY kernel32.dll\nEXPORTS\n" + "\n".join(SYMBOLS) + "\n")
    subprocess.run(
        [linker, "-flavor", "link", "/lib", "/machine:x86", f"/def:{definitions}", f"/out:{library}"],
        check=True,
    )
    subprocess.run(
        [shutil.which("clang"), "--target=i686-pc-windows-msvc", "-O1", "-ffreestanding",
         "-fno-stack-protector", "-Wall", "-Wextra", "-Werror", "-c", str(INPUT), "-o", str(obj)],
        check=True,
    )
    aliases = [f"/alternatename:__imp__{name}@{size}=__imp__{name}" for name, size in SYMBOLS.items()]
    subprocess.run(
        [linker, "-flavor", "link", "/nodefaultlib", "/subsystem:console", "/machine:x86",
         "/dynamicbase", "/fixed:no", "/timestamp:0", "/safeseh:no", "/manifest:no",
         "/entry:entry", str(obj), str(library), *aliases, f"/out:{exe}"],
        check=True,
    )
    (directory / f"{NAME}-inputs.json").write_text(json.dumps(identity(exe), indent=2) + "\n")


def execute(directory, native, host_runner):
    exe = directory / f"{NAME}.exe"
    inputs = identity(exe)
    if inputs != json.loads((directory / f"{NAME}-inputs.json").read_text()):
        raise ValueError("parallel thread witness inputs changed")
    command = [str(exe)]
    if not native:
        command = [str(host_runner or runner(directory)), *command, "--strict", "--file-origins",
                   "--parallel-guest-threads"]
    report = {"backend": "native Windows" if native else "Rosetta", "inputs": inputs}
    path = directory / f"{NAME}-{'native' if native else 'host'}.json"
    try:
        result = subprocess.run(command, capture_output=True, timeout=30)
    except subprocess.TimeoutExpired as error:
        report.update(status="timeout", stdout_hex=(error.stdout or b"").hex(),
                      stderr_hex=(error.stderr or b"").hex())
        path.write_text(json.dumps(report, indent=2) + "\n")
        raise
    report.update(status=result.returncode, stdout_hex=result.stdout.hex(),
                  stderr_hex=result.stderr.hex())
    path.write_text(json.dumps(report, indent=2) + "\n")
    if result.returncode or result.stdout != struct.pack("<7I", *EXPECTED):
        raise ValueError("parallel rendezvous or host-call integrity failed")
    if native:
        peer = json.loads((directory / f"{NAME}-host.json").read_text())
        if peer["inputs"] != inputs or peer["status"] or peer["stdout_hex"] != report["stdout_hex"]:
            raise ValueError("identical PE differs between Windows and Rosetta")
    print("Three guest participants, 6,144 host updates and 384 heap lifecycles agree")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "local/pe/startup-microtests")
    parser.add_argument("--runner", type=Path)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--host", action="store_true")
    mode.add_argument("--native", action="store_true")
    args = parser.parse_args()
    if args.host or args.native:
        execute(args.output.resolve(), args.native, args.runner.resolve() if args.runner else None)
    else:
        build(args.output.resolve())
