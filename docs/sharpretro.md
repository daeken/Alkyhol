# SharpRetro execution backend: coordination contract

The sections below record chronological milestones. The latest shared-service
entry and original-game progress are at the end of this document.

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

## Native ARM execution through the existing PE32 services

The opt-in `jit32/shared-runtime` path now builds the existing PE32 process,
Win32, COM and D3D9 implementation for native ARM. It uses the same startup
launcher and supplied DLL graph as Rosetta, with a CPU executor at the guest-call
boundary. `tools/build_jit32_shared.py` builds that launcher;
`tools/check_jit32_shared.py` checks unchanged owned inputs against the frozen
Windows bytes. The older standalone `Runner`/`Process` contracts remain separate.

ARM macOS enforces a four-GiB hard page zero ([XNU mach_loader.c](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/kern/mach_loader.c)).
Guest pointers remain 32-bit. Owned backing uses a checked fixed bias at
`0x8000000000`; no small-PAGEZERO or native i386 execution is required. An internal
syntax adapter translates explicit service pointer casts, retaining numeric
sizes, handles and guest pointer fields. The existing service buffer checks
remain; the CPU additionally sees committed heap arenas, readable image headers
and logical page permissions. This distinguishes CPU-accessible allocation
padding from live API buffer extents. Physical ARM pages stay writable backing;
all guest access/fetch permissions are enforced by the CPU/service checks at
four-KiB guest granularity. Host native code lives separately in MAP_JIT storage.

Native host calls use C ABI; the x86 build retains Win64 ABI. Existing owned x86
exception/math templates are extracted from assembler-produced objects for the
ARM build, not from game files. COM entry addresses register checked guest gates.
Reentrant API callbacks receive separate CPU continuations and share the guest
thread's floating-point state. FPU state keys include process generation and
host-owned thread identity, so address reuse or a guest TEB write cannot silently
select another floating-point environment. Native x86 signal continuation is not
installed on ARM; unimplemented exception delivery still stops explicitly.

CPU additions are driven by actual original DAO startup: integer divide faults,
widening multiplication, aligned atomics, restartable REP iterations, flag-stack
widths, AH binding for SAHF/LAHF, complete admitted PUSHA/POPA frames, CPUID and a
real host-clock TSC. The virtual family-6 CPU advertises one logical processor
and no SIMD arithmetic facilities. Out-of-range CPUID inputs follow the
[documented highest-basic-leaf rule](https://cdrdv2-public.intel.com/858441/252046-080-sdm-change-document.pdf).
RDTSC and register-bank encodings absent from the pinned decoder have explicit
local adapters. SharpRetro sources and its audited pin remain unchanged.

Guest x87 environments, transfers, comparisons and normal finite arithmetic use
pinned open [rustc_apfloat](https://github.com/rust-lang/rustc_apfloat)
(`0.2.3+llvm-462a31f5a5ab`, Apache-2.0 WITH LLVM-exception). Directed bounds and an
exact wider midpoint prevent reduced-precision double rounding; square root uses
an integer radicand. Bounded atan/trig use outward-rounded intervals and explicit
series-tail bounds. Uncertified rounding, unsupported argument domains, unmasked
data exceptions and exceptional arithmetic stop. Trig implements certified
mathematical rounding; it does **not** claim Intel's proprietary approximation is
bit-identical. SIMD transfers/bitwise operations preserve raw bits and check
aligned accesses; general SIMD arithmetic remains unimplemented.

ARM `_CIsqrt`/`_CIatan` service adapters consume real guest x87 operands under the
same bounded contracts as the existing Rosetta providers. They preserve the
caller integer continuation and return actual floating-point results, without
executing the owned x86 wrapper or invoking Rosetta. Twenty sqrt and eleven atan
accepted frozen Windows rows agree on result80, control, status and the retained
lower stack operand. This is component evidence, not execution of the complete
numeric PE observer or a full CPU equivalence claim.

The native ARM process has completed every original DAO DLL initializer and
entered the EXE's own CRT/engine startup. A local original-input run executes
332,460,348 guest instructions and 798,587 services before its next explicit
FSINCOS stop; after trig bring-up it reaches a generated memory-SAR read-after-
store limitation at 337,870,384 instructions and 821,784 services. No native
window, shaded JIT game frame or cutscene parity is claimed at this milestone.
Input identities, exact PCs/bytes and captures remain ignored under local/jit32;
original game/dependency files are hash-checked before and after the run.

The DLL inputs retain the existing explicitly configured staged/unpacked
dependencies; their exact identities remain in the local capture. The original
DAOrigins.exe file is unchanged.

The next CPU milestone forwards a generated post-store operand reread only when
its SSA address/width match and the store dominates that read's branch scope.
The forwarded value is narrowed like a real memory reload. Other addresses,
widths, mutually exclusive branches and multiple stores still reject execution.
This admits the original memory-shift path without adding a fallible access after
a committed write. 120 shift/input/count cases check results and defined flags,
with alias/dominance rejection, signed displacement and denied-write rollback.
All 41 JIT tests pass on M4. The original EXE now creates/writes its real startup
log and reaches its thread-naming software exception at 338,719,466 instructions
and 825,382 services. The next stop is the owned x86 exception wrapper's snapshot
instruction; native ARM needs a CPU-aware service entry, not a fabricated handler
return. Original inputs remain hash-identical.

### Native software exception entry

The shared native executor supplies RaiseException's actual integer registers and
flags directly to the existing checked first-pass SEH dispatcher. Original guest
registration records and original search/continuation handlers execute through
normal reentrant JIT callbacks; there is no special-case successful return for
thread naming. The native entry preserves the modeled x87/SIMD caller state as
the owned x86 wrapper does, without pretending to implement general guest
FXSAVE/FXRSTOR or inventing an extended context. The advertised context remains
control/integer/segments only. Native guest selectors now use an explicit virtual
flat ring-3 profile (CS 0x1b, data 0x23, FS 0x1017), separate from host registers.
Nested exceptions, changed contexts and unhandled/noncontinuable exceptions still
fail explicitly. A biased-host comparison in registration-chain bounds now uses
guest stack addresses.

`tools/check_jit32_software_exception.py` builds the unchanged existing owned
software-exception PEs and executes their two original chained handlers on
physical ARM. Their 15-word contract passes; unhandled, noncontinuable and
context-mutation modes each stop with the expected unsupported reason. This is
finite native ARM host evidence, not a new Windows measurement or full SEH/unwind
support. The existing Rosetta exception observer remains separate.

The original game now completes its actual thread-naming software notifications,
starts its original worker procedures, registers its Win32 class and creates a
real Metal-backed D3D9 factory. Its platform validation then rejects the scalar
virtual CPU's absent SSE/SSE2 bits; an instruction trace confirms execution of
both original CPUID checks. Shutdown reaches an existing explicit last-owned-
mutex-close exclusion. No native JIT window or frame is claimed. This is further
than the prior owned FXSAVE wrapper stop, but remains short of the preserved
Rosetta cutscene boundary. All original input hashes agree before/after.

Optional `ALKYHOL_JIT32_TRACE_AFTER=<hex guest PC>` records the next 256 original
instructions/registers per entry for local diagnostics. It does not change guest
control flow. Keep game-derived traces local. Shared-host builds now default to
no debug info/incremental objects for practical build time; external Cargo
profile overrides remain respected.

### Opt-in SIMD bring-up and reached scalar conversions

`ALKYHOL_JIT32_EXPERIMENTAL_SSE2=1` selects an explicit **incomplete development
CPU profile** exposing SSE/SSE2 to original feature checks. It is off by default,
prints a warning and leaves every unimplemented opcode/control/exception path
fatal. This is not a claim of full SSE/SSE2 or game support. The Win32 processor
feature provider and CPUID agree on this selected profile; original game bytes
and conditional branches are unchanged.

This immediately reaches original FMOD CVTTSD2SI during DLL initialization.
Six scalar integer/float conversion forms now use software APFloat with actual
guest MXCSR rounding, sticky IE/PE status, DAZ inputs, masked integer-indefinite
results and checked operand widths. Unmasked exceptions stop before committing
the destination; no #XM handler delivery is invented. Other registers, scalar
upper XMM lanes and EFLAGS are preserved. The behavior follows the conversion
exception rules in [Intel SDM Volume 1, §11.5](https://cdrdv2-public.intel.com/868137/325462-089-sdm-vol-1-2abcd-3abcd-4.pdf).

`tools/check_jit32_sse.py` independently assembles an owned x86 observer, executes
it under actual Rosetta on this M4, then compares the same encoded conversions
on native ARM. All 512 rows agree including result bits/MXCSR across all four
rounding modes, DAZ settings, finite values, boundary integers, infinities,
quiet/signaling NaNs and tiny inputs. Two additional ARM tests check denied
reads, unmasked precision failures and preserved state. The complete default
suite passes 43 tests with one explicit external-observer test ignored; that
external test passes separately through the checker. These are finite Rosetta
and native ARM observations, not new Windows or physical Intel observations.

### Reached x87 constants and native code packing

The next original-input run completes resource indexing and reaches FLDPI.
The remaining x87 ROM constant-load family now supplies full 64-significand
values according to guest RC, ignoring PC and preserving sticky exceptions,
without inventing a precision exception. The rules follow
[Intel SDM Volume 1, §8.3.4](https://cdrdv2-public.intel.com/789574/253665-sdm-vol-1.pdf).
`tools/check_jit32_constants.py` independently executes seven owned original x86
constant instructions under actual Rosetta, then compares their native ARM
result80/status across four rounding modes and three precision settings: all
84 rows agree. This is finite Rosetta/native ARM evidence, not a new native
Windows measurement or general x87 equivalence.

The native code allocator now packs immutable generated blocks at disjoint
16-byte offsets in shared 64-KiB MAP_JIT arenas instead of allocating a separate
host page for every tiny guest instruction. Pages remain owned until their last
block is dropped; large blocks get sufficient dedicated capacity. Thread-local
weak arena selection prevents retaining dead pages, and no emitted live block
is rewritten/reused. Tests execute 64 distinct packed blocks, remove their peers,
execute a surviving block, verify final page destruction and execute a larger
allocation. This changes native storage only; guest instruction, permission,
fault and rendering checks remain intact.

On physical M4, 45 default JIT tests pass; the two separately invoked external
observers pass their 512 and 84 rows. Runner Clippy and unchanged shared frozen
Windows ABI/TLS/heap comparisons (96 words, both bases) also pass after packing.
The next original-input replay explicitly supplies the same open D3D9 shader
compiler as the Rosetta renderer; earlier diagnostic native startup omitted that
launch setting. Game shader inputs/artifacts remain ignored local data.

### Native nonblocking poll handoff

DAO's original message worker repeatedly invokes zero-timeout waits while the
loader indexes archives. The native provider now polls the real kernel object
without releasing/reacquiring the full guest CPU/TEB lease for each nonblocking
call. The existing dispatch scheduler still yields every 64 services or 2 ms
when workers are active; positive/infinite waits retain their real blocking
release/wait/reacquire behavior. The Rosetta implementation is unchanged.

`tools/check_jit32_threads.py` executes the unchanged 26-word owned original
worker/TLS/FLS witness plus a new 12-word targeted original PE for unsignaled/
signaled/consumed auto-reset polling, progress of a polling worker, join/status
and real handle closure. Both pass on native ARM before and after this handoff
change. These are finite host assertions, not a new Windows timing measurement
or a measured game loading-speed result. It remains a semantic poll: no sleep,
forged completion, event suppression or skipped guest instructions is added.
