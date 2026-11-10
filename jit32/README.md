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

The retained initial **standalone** provider capture maps/binds 12 static
modules and stops at GetModuleHandleA in the first CRT initializer. That smaller
provider is separate from the shared service path below, which completes the
supplied DLL graph and enters original DAO engine/resource initialization.
Standalone nonzero TLS zero-fill retains its separately documented native-
evidence mismatch. See docs/sharpretro.md and docs/validation.md for the current
application driver and exact evidence limits.

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
exhaustion is an explicit failure. `ALKYHOL_JIT32_INSTRUCTION_BUDGET` explicitly
selects a decimal count from 1 through 100 billion for longer original startup
and interactive experiments; the chosen limit is printed. Invalid settings fail
before execution, and exhaustion remains a failure. Native callbacks keep separate integer
continuations and shared guest-thread x87 state. See docs/sharpretro.md and
docs/validation.md for precise current game progress and evidence limits.

The reached CPU subset now additionally includes checked aligned atomics,
restartable REP iterations, flag/register-bank operations, normal finite x87
arithmetic and bounded software math, plus legacy XMM transfers/bitwise logic.
Unimplemented arithmetic, exception delivery and uncertified numerical paths
still stop explicitly. Open rustc_apfloat supplies software 80-bit conversions;
there is no proprietary converter or host binary64 substitute for x87 arithmetic.

Native software RaiseException now invokes the original checked SEH chain with
real saved registers. For SIMD bring-up only, `ALKYHOL_JIT32_EXPERIMENTAL_SSE2=1`
selects an explicitly incomplete profile exposing SSE/SSE2 feature checks; it
prints a warning and still stops on unsupported instructions. Leave it unset for
the conservative scalar profile and frozen ABI/TLS/heap comparisons. Six reached
scalar float/integer conversions honor guest MXCSR; reproduce their 512-row
actual Rosetta/native ARM comparison with `python3 tools/check_jit32_sse.py`.
This is finite host evidence, not full SIMD/SEH or game compatibility.

Declared-width immediate arithmetic has a separate original x86 assembly
comparison: `python3 tools/check_jit32_immediates.py`. Its 960 Rosetta rows cover
CMP/SUB/ADD/ADC/SBB result/flags across 8/16/32-bit accumulator, group-register,
checked-memory and eligible sign-extended imm8 encodings on the native ARM JIT.

`python3 tools/check_jit32_peek.py` exercises a native guest-worker window and
callbacks followed by AppKit message polling alongside a real compute worker.
Its 36 finite host assertions cover the CPU lease handoff used by DAO startup;
they are separate from native Windows observations and game progress.

`python3 tools/check_jit32_bits.py` compares 112 signed memory bit-test/update
rows against independent actual x86/Rosetta assembly, including aligned LOCK
forms. The native unit tests separately retain permission/fault, register,
immediate-mask, segment/wrap and excluded-form checks.

`python3 tools/check_jit32_strings.py` compares 432 CMPS/SCAS rows with actual
x86/Rosetta assembly across operand widths, repeat conditions/counts and DF
directions. Memory-fault commitment and zero-count invalid-source tests run in
the default native suite.

`python3 tools/check_jit32_compare_windows.py` checks all 18,432 frozen native
Windows AMD comparison rows. Both native Windows and Rosetta raw tables are
preserved under reference/. Their masks/EFLAGS agree, but 152 NaN/denormal DE
status rows differ. The runner follows the measured Windows AMD profile;
`tools/check_jit32_compare.py` remains a strict Rosetta diagnostic and currently
fails on those differences. Neither table is a universal CPU/game proof.

`python3 tools/check_jit32_examine.py` compares the reached FXAM classification
with 768 independent actual Rosetta assembly rows. This is finite x86/Rosetta
evidence, separate from native Windows and original game progress.

`python3 tools/check_jit32_rotates.py` compares 5,200 actual Rosetta narrow-rotate
rows with both CL and immediate native forms. Only architecturally defined
overflow flags are asserted. These observations are separate from Windows.

`python3 tools/check_jit32_xlat.py` compares all 256 byte-table indices under
two initial flag states against actual Rosetta assembly; native tests separately
exercise guest segment/address wrap and checked memory failures.

`python3 tools/check_jit32_x87_windows.py` checks all 19,200 frozen native Windows
AMD register-arithmetic rows, including masked exceptions and 24/53/64-bit x87
precision. `tools/check_jit32_x87_arithmetic.py` separately runs actual Rosetta
assembly and remains a strict failing diagnostic for its one differing division
result/C1 row. Both complete observations and provenance are preserved.

`python3 tools/check_jit32_double_shifts.py` compares DWORD SHLD/SHRD with
8,320 actual Rosetta rows (both immediate/CL native forms) and four read-only
protection probes. `tools/check_jit32_x87_memory_windows.py` checks the separate
1,440-row native Windows Intel memory-arithmetic fixture, identical to Rosetta.

`python3 tools/check_jit32_rsqrt_windows.py` checks a bounded RSQRT profile
against 16,640 native Windows rows: exact special values and state, plus normal
relative error <=3/8192 using integer inequalities. Normal estimate bit equality
is explicitly not required. Downstream Newton-refined values may differ from a
specific CPU. The strict `tools/check_jit32_rsqrt.py` Rosetta diagnostic retains
its forced-MXCSR-mask failure; both raw observation tables are frozen.

For an explicitly enabled live diagnostic, set `ALKYHOL_JIT32_TRACE_REQUEST` to
an ignored local path. Atomically write a hexadecimal guest PC (at most 16 bytes)
to request 256 instruction snapshots on the first guest thread reaching it.
The request is consumed; no guest state is modified. Malformed requests fail.
`ALKYHOL_PE32_TRACE_INPUT=1` additionally emits bounded changed-state DirectInput
snapshots and readable-output copy comparisons. These are diagnostic evidence,
not an input-success or game-compatibility assertion.


The reached FPATAN instruction uses a bounded mathematical atan2 profile, with
certified software extended-precision rounding and explicit unsupported cases.
It is not a bit-exact CPU polynomial/status oracle. Frozen native Windows and
Rosetta observations can be checked with `python3 tools/check_jit32_atan2.py`
and `--backend rosetta`. Adding `--strict` retains raw 80-bit/status differences
as failed diagnostics. See docs/sharpretro.md for domain and measured counts.


The reached RCPSS and paired RCPPS use deterministic software reciprocal estimates
with exact integer bound checks and explicit tiny-result behavior. Normal CPU
estimate-bit equality is not claimed. `python3 tools/check_jit32_rcp.py` checks the
frozen Rosetta table; `--backend windows-amd64` checks the native Windows Intel
table from CI 42814117586. Both have 67,584 rows. The observer measures MXCSR
immediately before RCP, separately retaining requested-vs-loaded mask differences
(33,792 Rosetta rows; zero on that Windows host). RCP control/flags preservation
passes on both. Older frozen comparisons and strict failures remain unchanged.


`python3 tools/check_jit32_minmax.py` checks all eight legacy scalar/packed
single/double MIN/MAX forms against 65,536 frozen Rosetta rows. Add
`--backend windows-amd64` for the separate native Windows AMD fixture from
CI 42825749976. Both complete raw tables are byte-identical; the ARM model matches
all result bits, complete MXCSR and defined flags. These are masked-exception
observations; unsupported unmasked exceptions stop before state commit.
