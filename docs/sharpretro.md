# SharpRetro execution backend: coordination contract

The user now prioritizes a pure-Rust SharpRetro runner for unchanged DAO,
separate from the existing Rosetta path, to compare CPU behavior side by side.
The first [checked native ARM64 runner](../jit32/README.md) executes the existing
owned PE32 ABI contract unchanged. It is not yet a DAO game runner.

On physical M4, both preferred-address and forced-rebase JIT executions complete
100,089 guest instructions and 20,005 real Windows-service transitions over 99
compiled instruction blocks. Their 32 output bytes match the same Rosetta PE
executions and all eight frozen native Windows words. This is finite binary/ABI
evidence, not whole-CPU or game equivalence. The source/guest identity and exact
native-code execution checks are reproduced by tools/check_jit32.py.

The Rosetta DAO baseline is preserved at 614363a. The user's later cutscene crash
is captured as an explicit missing GetLocalTime import. We are leaving that
boundary unchanged while building the second execution backend.

The “Design iterative decompiler workflow” chat confirmed coordination and
separate ownership. This chat owns Alkyhol-side runner changes; that chat keeps
DAODecomp replacement/switch-discovery work separate and will not concurrently
edit Alkyhol or Rust CPU code. Do not edit their active Decompiler files or
DAODecomp sources as part of this runner. Original game inputs/captures remain
local. Coordinate CPU gaps before editing shared SharpRetro code.

Verified local Rust starting points:

- `../SharpRetro/rust/xfusion-recomp`: `disassembler::decode_insn` with
  `decode::XMode::Bits32`, and `lift::lift_one` with `FLAGS_ALL_LIVE`.
- `../SharpRetro/rust/sharpretro-jit`: `interp::InterpretingBuilder`, `GuestMem`
  and `RegState`. `xfusion-recomp/src/bin/isa_diff.rs` demonstrates one-instruction
  interpretation; its current example is Bits64, not a complete Bits32 runner.
- `xfusion-recomp::state::X86State` carries 16 GPRs, EFLAGS, six segment bases,
  XMM values and RIP. Its current state/layout is x64 oriented. FS is segment
  index 4. Explicit x87/control/status/tag and MXCSR state are absent from this
  struct; interpreter native calls/unwired intrinsics still panic. These are
  audit items, not assumed-complete CPU semantics.

Keep the PE loader and Win32/COM/D3D9 service meanings in Alkyhol. Introduce an
execution boundary with bounded step/run-until-exit behavior and explicit exits
for import or COM calls, guest callbacks, CPU faults, unsupported instructions,
resource exhaustion and instruction/time budgets. Each exit retains exact guest
PC/register state. Missing operations, panics or budgets must never become
successful zero-return calls.

Memory and ABI work must precede original-game execution: a checked GuestMem
address space, fetch/read/write permissions and faults, 32-bit register/stack
semantics and an explicit service marshaller. Do not use the interpreter's
HostMem identity-pointer adapter for arbitrary guest addresses. Alkyhol's current
Rosetta services frequently dereference checked low-address mappings directly;
they cannot simply receive emulator addresses without an audited adapter.
Callbacks need saved outer continuations and reentrant guest execution. Alkyhol
continues to own TEB/FS layout, TLS setup/callback order, SEH policy, thread
lifecycle, scheduling and synchronization; the CPU contract must supply accurate
instruction/fault and atomic-operation behavior.

Start with owned Bits32 instruction/import/callback/FS and fault cases compared
to independent native observations, then the unchanged original loader/startup.
A passing finite CPU corpus does not prove DAO, and the existing Rosetta evidence
must stay separately identified. Audit x87/FP environment, precise faults,
callback reentry and Bits32 coverage before promising unchanged DAO execution.

## Implemented initial boundary

jit32/ owns its checked PE materialization/HIGHLOW relocation, sparse guest memory,
per-instruction native code cache, Apple MAP_JIT allocation and service marshaller.
It reuses Alkyhol's portable Image parser and actual kernel last-error/file I/O.
No raw guest address is passed as an ffi.rs host pointer. Code writes and unknown
startup/import requirements fail. A memory guard exits before a denied access;
register/flags state is restored to the instruction snapshot. Only the admitted
single-store, no-read-after-store contract is covered by that rollback policy.

The generated Bits32 CALL template stores a U64 return-PC literal although ESP
moves by four. The generated PUSH Ib-sx def84 instead uses an eight-bit stack
width. Explicit local adapter corrections narrow the former store and sign-extend
the latter immediate to the real 32-bit stack width; tests inspect adjacent bytes,
return addresses, ESP and preserved registers. Stack wraparound is also explicit.
Upstream generator/source files remain untouched; the coordinated chat has the
concrete findings. SharpRetro source/generator/artifact hashes are pinned in
jit32/sharpretro-source.json, separate from its unrelated dirty Decompiler lane.

The runtime currently handles only GetLastError, SetLastError, inherited regular
stdout GetStdHandle and synchronous WriteFile. Other imports, DLL initialization,
TLS/load-config startup, atomics, loops and FP/SIMD are rejected explicitly. The
present Runner::run boundary reports Returned, Fault, Unsupported or Budget with
current PC; register/flag snapshots and checked memory are inspectable. Imports
are handled by the bounded initial dispatcher, not yet a general external gate
protocol. General callback continuations and DAO DLL startup are the next bridge
work, together with accurate extended CPU/fault state. Existing Rosetta services
cannot be transplanted by casting emulator pointers into host pointers.

The actual local DAO preflight currently stops explicitly at unsupported static
TLS initialization, before game instructions execute. Bring up DLL/TLS startup
and callbacks next; do not bypass them to manufacture a game-entry milestone.
