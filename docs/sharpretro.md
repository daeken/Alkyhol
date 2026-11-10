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


## Shared TLS and original DAO dependency startup

The second milestone adds Process::load/run_startup/run_application. It validates
and maps reachable explicitly provided DLLs, relocates them, binds names/ordinals
and code/data exports (including checked forwarding), and rejects missing DLLs,
exports, cycles and unsupported metadata. Image::check_emulated_startup32 admits
validated TLS callbacks only for an emulator which actually invokes them; the
older Rosetta/source preflight profiles retain their previous callback gates.

Static TLS templates are copied after relocation/import binding, with stable
indices, checked alignment and a shared FS-based TEB/vector. Original TLS callbacks
run before each module's original DllMain, dependencies first; EXE TLS follows its
DLL dependencies. Static attach uses a backed non-NULL opaque reserved indicator
per the [DllMain contract](https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain).
The runner verifies stdcall stack restoration and nonvolatile GPRs, retains active
continuations across budgets/import stops and never enters the EXE after rejected
attach. General API callback reentry and detach/failed-load rollback notifications
remain separate work. Nonzero TLS SizeOfZeroFill stays explicitly rejected.

The actual existing TLS-data PE agrees with all 24 frozen Windows words, and the
literal heap-resize PE agrees with all 64. Both execute as native ARM64 JIT code
and through actual Rosetta, at preferred and forced-rebased addresses. These are
finite independent Windows observations; synthetic initializer-order/rollback,
flag and API units remain separately identified host/physical ARM assertions.

DAO now reaches the real msvcr80.dll initializer with 12 static modules mapped and
bound. With the explicit legacy NT 6.2/build 9200 persona and an explicitly empty
guest environment, the captured original path executes 191 guest instructions
and seven real API transitions, stopping at KERNEL32!GetModuleHandleA from
msvcr80.dll+0x2fe3. The original CRT has not finished and EXE entry is not reached.
Input identities, registers/flags, caller and diagnostics remain local. This
replaces the earlier *preflight-only* TLS refusal as the current JIT boundary;
the earlier refusal remains historical evidence. No JIT game frame is claimed.

The checked service state now owns heap lifetimes/resize/quota, recursive critical
sections, dynamic TLS slots, environment buffers, pointer encoding and real
clock/identity outputs. Direct TEB last-error/TLS-slot writes are coherent with the
marshaller. Unsupported flags, executable heaps and unavailable functions fail;
unknown OS calls produce an Import exit rather than a fabricated return value.
Module lookup/loading and exporting the implemented compatibility services are
the next concrete CRT blockers. Do not report missing optional APIs as successful
calls or assume the old low-pointer dispatcher can receive emulated offsets.

The TLS witness exposed another generated-lifter omission: A0-A3 moffs loads/stores
ignored segment overrides (64 A1 2C000000 read 0x2c instead of FS.base+0x2c).
Local adapters correct that, wrap effective addresses before adding segments, and
keep LEA segment independent. All 16 register SETcc predicates have direct boolean
flag-contract tests. Shared SharpRetro tracked sources/pins remain untouched.
