# Checked SharpRetro PE32 runner

This experimental runner executes an unchanged owned PE32 through SharpRetro's
Bits32 decoder/lifter and Tier0-generated ARM64 code on Apple Silicon. Execution
is pure Rust plus libSystem JIT/memory primitives. There is no Rosetta or
interpreter fallback in this runner. The separate comparator also executes the
same PE through the existing Rosetta runner.

Prerequisites: macOS/Apple Silicon, Rust, and the adjacent open-source
[SharpRetro](https://github.com/daeken/SharpRetro) checkout at
`1ab08109ac75cfe65d2393828706bac11f01cc91` (Apache-2.0). Its generated Rust sources
must match `sharpretro-source.json`. In a fresh adjacent checkout regenerate them
with `bash rust/xfusion-recomp/regen.sh` from the SharpRetro root; that generation
step uses open .NET, not a runtime dependency of the runner. This does not require
editing SharpRetro's tracked sources. Cargo.lock pins the Rust dependencies; the
source manifest additionally pins the local CPU/generator inputs and generated
artifacts. The comparison refuses different versions or hashes.

From the Alkyhol root:

```sh
python3 tools/build_pe32_fixture.py
cargo test --manifest-path jit32/Cargo.toml --locked --offline --lib
python3 tools/check_jit32.py
```

The last command builds both hosts, checks the PE against the unchanged frozen
Windows input identity, runs normal and forced-rebase cases and writes
`local/jit32/comparison.json`. It requires cached Cargo dependencies for the
explicit offline builds; populate those with `cargo fetch --manifest-path
jit32/Cargo.toml --locked` if necessary. Direct invocation requires regular-file
stdout, matching the initial shared kernel admission:

```sh
cargo run --manifest-path jit32/Cargo.toml --locked --offline -- \
  local/pe/pe32/pe32.exe > local/jit32-output.bin
```

The current executable contract is scalar, no CRT/DLL initialization or static
TLS, with four real marshalled kernel services: GetLastError, SetLastError,
GetStdHandle (inherited regular-file stdout) and synchronous WriteFile. Guest
handles are tokens; buffers/output counts are checked and copied across the
boundary. The loader rejects other imports/startup requirements before execution.

Guest memory uses sparse owned 4-GiB backing with checked logical mappings,
permissions and bounds; it is not identity-mapped to host pointers. Writable
executable mappings are rejected. Every native memory access is guarded. Blocks
contain one guest instruction, with a retained pre-instruction CPU snapshot.
Multiple stores, reads after committed stores, atomics, string loops, FP/SIMD,
16-bit stack operations and unwired instructions are explicitly rejected. The
bounded code cache admits at most 4096 instructions, each at most 1 MiB of code;
the runner counts import transitions and guest instructions against its run
budget. SharpRetro watch instrumentation is not admitted.

The Bits32 adapter explicitly corrects generated CALL's 64-bit return-PC store,
PUSH imm8's encoded-width versus stack-width mixup, and 32-bit stack wrapping.
Targeted tests retain exact instruction bytes, register/flag and memory-footprint
checks. These are adapter corrections, not tracked SharpRetro changes.

This milestone compares one finite original owned PE, not DAO. DLL initialization,
TEB/FS/TLS, callback/reentrant execution, x87/MXCSR/exception state, threads and the
large Win32/COM/D3D9 marshalling surface remain required before replacing the
working Rosetta DAO path. See `docs/sharpretro.md` and `docs/validation.md`.
