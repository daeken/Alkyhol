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

The initial standalone runner has two entry paths. `Runner::load` keeps the initial no-TLS/no-CRT
contract. `Process::load` maps an EXE and explicitly supplied static dependencies,
binds named/ordinal code/data imports and checked forwarders, prepares a shared
TEB/static TLS, then invokes original TLS callbacks and DLL entry points in
dependency order. Startup can pause/resume without repeating completed calls;
failed attach or stdcall/nonvolatile violations block EXE entry. Only reachable
DLLs acquire mappings/TLS/startup state. The static attach reserved argument is
an owned non-NULL opaque indicator, following the documented
[DllMain contract](https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain).
It is not an invented Windows startup structure.

```sh
python3 tools/check_jit32_startup.py
cargo run --manifest-path jit32/Cargo.toml --locked --offline -- \
  local/pe/pe32-tls-data/pe32-tls-data.exe --process > local/jit32-tls.bin
```

`--dll NAME=PATH` supplies original dependencies; `--startup-only` stops before
EXE entry, and `--state` prints current registers/flags/caller at a stop. The
explicit `--legacy-nt62` persona enables NT 6.2/build 9200 version responses.
Without it version calls fail. Guest environment state starts explicitly empty,
is case insensitive, and currently admits ASCII values through both A/W APIs.
It is not silently populated from the host environment.

Initial kernel marshalling now covers owned heaps (including preservation,
zeroed growth and in-place shrink), recursive single-thread critical sections,
dynamic TLS slots, pointer encoding, process/thread IDs, real clocks, environment
copies and the selected version persona, alongside the original four I/O/error
services. Handles/buffers remain checked guest objects; freed heap/environment
storage loses logical access. Unsupported flags and unknown OS functions stop
explicitly. Unknown calls produce an Import exit with PC, DLL and symbol while
preserving the stopped guest continuation. Unprovided original DLLs and missing
exports fail before execution. This is not a complete Win32 provider or dynamic
LoadLibrary/GetProcAddress implementation.

Guest memory uses sparse owned 4-GiB backing with checked logical mappings,
permissions and bounds; it is not identity-mapped to host pointers. Writable
executable mappings are rejected. Every native memory access is guarded. Blocks
contain one guest instruction, with a retained pre-instruction CPU snapshot.
General multiple stores/reads after stores, unaligned atomics, unsupported
FP/SIMD and unwired instructions are explicitly rejected. The expanded reached
subset and shared-service entry are described below. The
bounded code cache admits at most 4096 instructions, each at most 1 MiB of code;
the runner counts import transitions and guest instructions against its run
budget. SharpRetro watch instrumentation is not admitted.

The Bits32 adapter explicitly corrects generated CALL's 64-bit return-PC store,
PUSH imm8's encoded-width versus stack-width mixup, 32-bit address/stack wrapping,
moffs segment overrides and LEA ignoring segment bases. LEAVE and register SETcc
forms are admitted; a flag truth-table test exercises all 16 SETcc predicates.
Targeted tests retain exact instruction bytes, register/flag and memory-footprint
checks. These are adapter corrections, not tracked SharpRetro changes.

The startup comparator checks the unchanged existing TLS-data (24 words) and
heap-resize (64 words) PEs against actual Rosetta runs and frozen native Windows
observations, normal and forced-rebased. Inputs/references stay unchanged. These
comparisons are separate from synthetic callback/order/fault and API-unit tests.

DAO's original first CRT initializer now executes through native ARM JIT code;
its latest capture maps/binds 12 static modules and stops at GetModuleHandleA.
Neither CRT initialization nor EXE entry is complete. Extended x87/MXCSR/exception
state, atomics/loops/SIMD, dynamic module lookup/loading, API callback reentry,
threads, detach/rollback notifications and the larger Win32/COM/D3D9 bridge remain
required before replacing the working Rosetta DAO path. Nonzero TLS zero-fill is
still rejected for its separately documented native-evidence mismatch. See
`docs/sharpretro.md` and `docs/validation.md`.

## Shared Win32/COM/D3D9 service path

For the unchanged original application driver, build the native CPU into the
existing PE32 service launcher:

```sh
python3 tools/build_jit32_shared.py
python3 tools/check_jit32_shared.py
```

This uses `shared-runtime` and the original startup launcher's arguments. The
CPU, service crate and SharpRetro packages build with optimization level 2;
shader quality/geometry are unaffected. Supply the same explicitly configured
DLLs and backing profile used by Rosetta. Native memory backing is biased above
hard PAGEZERO; guest fields keep their original 32-bit layouts. CPU page access
and API live-buffer checks are deliberately distinct.

`ALKYHOL_PE32_TRACE_CALLS=first` limits repeated service diagnostics;
`ALKYHOL_JIT32_TRACE_PROGRESS=1` prints million-instruction progress. The current
shared entry budget is one billion instruction/service transitions per entry;
exhaustion is an explicit failure. Native callbacks keep separate integer
continuations and shared guest-thread x87 state. See docs/sharpretro.md and
docs/validation.md for precise current game progress and evidence limits.

The reached CPU subset now additionally includes checked aligned atomics,
restartable REP iterations, flag/register-bank operations, normal finite x87
arithmetic and bounded software math, plus legacy XMM transfers/bitwise logic.
Unimplemented arithmetic, exception delivery and uncertified numerical paths
still stop explicitly. Open rustc_apfloat supplies software 80-bit conversions;
there is no proprietary converter or host binary64 substitute for x87 arithmetic.
