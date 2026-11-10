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

### Reached SIMD environment and math group

Original DAO next reaches STMXCSR, then its original SIMD math routine. The
adapter now reads/writes actual guest MXCSR through checked four-byte operands;
reserved bits fail explicitly. It does not use the pinned lifter's empty
LDMXCSR or constant-default STMXCSR templates. The reached group includes normal
finite ADD/SUB/MUL/DIV scalar and packed single/double forms with software guest
rounding/sticky precision status, exact packed i32-to-double conversion, word
insert/extract, per-lane shifts/add/subtract, low-double moves and fixed-width
GPR/XMM moves with required upper-lane clearing. Unmodeled arithmetic exceptions,
subnormals and nonfinite inputs still stop explicitly in this milestone.

Native ARM matches 1,024 additional independent actual Rosetta arithmetic rows,
including result bits/MXCSR across all forms, RC and DAZ/FTZ settings on finite
normal inputs. The earlier 512 conversion rows remain intact. The complete
native suite passes 54 tests with three separately invoked external observers;
52 intermediate tests exposed and corrected a cross-half word insertion issue
before the final fixed-width aliases were admitted. Vector element insertion/
extraction uses explicit 64-bit halves instead of relying on the pinned wide
shift-at-zero lowering. Neither SharpRetro nor any frozen reference was edited.

The native memory validator also tests already-owned TEB/stack spans before its
unchanged image-region union, preserving all bounds and permissions. Together
with the native poll quantum, the subsequent original replay reaches the SIMD
math stop in 91.6 seconds versus 577.6 seconds for the prior MXCSR stop. Host
load was uncontrolled and stop points differ slightly; this is an observed
loading improvement, not an FPS benchmark or a claim of game equivalence.
A following replay reaches a real denormal operand, the next numerical boundary.
Original binary hashes agree before/after. No JIT window/frame is claimed yet.

### Masked SIMD edge semantics and original native splash

The actual denormal-operand stop now runs through software binary32/64
arithmetic, including DAZ, masked overflow/divide/invalid results, NaN sign/
payload priority and FTZ. Tininess/overflow classification uses the destination
significand with a wider exponent range, as specified in
[Intel SDM §4.9.1.5 / §11.5](https://cdrdv2-public.intel.com/868137/325462-089-sdm-vol-1-2abcd-3abcd-4.pdf),
while final values round directly to the real destination format. This avoids
confusing the rounded normal/subnormal boundary with the unbounded-exponent
rounding step. Unmasked #XM delivery remains explicit unsupported.

A fixed independent x86 assembly matrix adds 6,144 actual Rosetta/native ARM
result/status comparisons over denormals, DAZ/FTZ settings, signed zeros,
NaN priority, infinities and overflow/underflow boundaries. All rows agree,
alongside the unchanged 512 conversion / 1,024 normal arithmetic rows. The native
suite passes 55 tests with four separately invoked external observation tests.
Clippy and all unchanged shared frozen Windows ABI/TLS/heap comparisons pass.
These are finite host observations, not a new native Windows/Intel measurement.

Original DAO then reached GraphicsSystems initialization but window registration
failed: ui32's executable-procedure check compared a guest DWORD to biased host
image bounds. Using the existing host_code_address normalization retains all
permission/range checks and fixes the native path; Rosetta's identity address
behavior is unchanged. The unchanged owned 32-word window/callback witness now
passes on native ARM (13 actual original callback invocations); the old host's
zero exit status had not passed its output contract. Reproduce with
`tools/check_jit32_window.py`.

The following original-input replay creates and shows DAO's actual splash
window, executes its original bitmap/DIB drawing and message procedures, and
continues resource loading. A window-specific physical screen capture confirms
the original Dragon Age splash artwork. Captures remain local. This establishes
an original native-JIT splash, not a D3D9 game frame, menu, cutscene or parity with
the preserved Rosetta Orzammar boundary. The original binary files remain local
and unchanged; shader quality/assets/source profiles are preserved.

### Declared-width immediate carry checks

Original graphics enumeration reached the real M4/Metal D3D9 factory, but DAO's
adapter callback rejected its equal shader-version comparison. A bounded original
instruction trace showed CF and ZF both set: the decoder's sign-extended immediate
occupied a 64-bit host container while the DWORD loaded from caps was canonical
32-bit. Tier0 literal materialization did not mask to the declared IL width.
The Alkyhol checked builder now canonicalizes integer literals at construction;
the original application and SharpRetro source pin remain unchanged. No adapter
capabilities were added or original checks bypassed.

An owned regression reproduces the equal-value carry failure before the fix.
`python3 tools/check_jit32_immediates.py` compares 960 independent actual Rosetta
assembly rows (CMP/SUB/ADD/ADC/SBB, 8/16/32-bit, both carry inputs) against native
ARM accumulator, register-group, memory-group and eligible sign-extended imm8
forms. All result bits and six arithmetic flags agree. The 56 default CPU tests,
runner Clippy, existing 512/1,024/6,144 SIMD rows and unchanged frozen Windows
shared-service comparisons pass; five external observers run separately.
These are finite host CPU observations. The original-input replay with this fix
is in progress; no native JIT game frame or Rosetta cutscene parity is established.

The corrected original replay passes DAO's actual acceptance callback, enumerates
depth formats, creates a real windowed D3D9 device and initializes DirectInput.
It then stops at BSR in the original Umbra DLL, after 498,296,856 guest instructions
on that thread (193.9s wall time for the process, uncontrolled host load). Original
input hashes remain unchanged. The existing SharpRetro BSF/BSR templates are now
admitted after targeted native tests across every nonzero source bit, 16/32-bit
register/memory operands, partial destination writes, zero-source ZF and precise
bad-memory failure. Undefined arithmetic flags/zero-source destination are not
asserted as x86 guarantees. All 57 default CPU tests pass; the next original
replay is pending. Device creation is not evidence of an original game Present,
menu, cutscene, native shaded-frame parity or general game compatibility.

The subsequent original replay passes Umbra bit scans, opens the full game
window and creates original vertex/pixel shaders and vertex/index buffers. Its
next explicit stop is UNPCKHPD in the original EXE (816,434,379 instructions on
the loading thread, 633.8s uncontrolled wall time, original inputs unchanged).
No original Present was observed. The four legacy floating unpack forms now
use the pinned interleave templates with checked aligned 128-bit memory operands.
Native tests cover low/high 32/64-bit lanes, source aliasing, preserved GPR/
EFLAGS/MXCSR/source bits, alignment and complete-source memory faults.

For continued original startup and interactive execution, the shared runner now
accepts an explicit finite `ALKYHOL_JIT32_INSTRUCTION_BUDGET` (1..100 billion),
retaining the default billion and explicit exhaustion failure. Pure parser tests
reject malformed/overflow/out-of-range settings. On the physical native host,
the unchanged owned ABI PE stops at budget 1 without output, passes all eight
frozen Windows words at budget 1,000,000, and rejects zero/over-limit settings
before guest execution. All 59 default CPU tests and runner Clippy pass; five
independent external observers remain separate. The next original replay uses
an explicit ten-billion budget. No frame/menu/cutscene parity is claimed yet.

That original replay passes the unpack stop and proceeds through further
textures/viewport/buffer setup before SHUFPS in original matrix code, after
825,096,084 loading-thread instructions (457.1s uncontrolled process wall time).
Original file hashes remain unchanged and no Present was observed. SHUFPS/SHUFPD
now use the pinned shuffle templates. Native tests cover all 256 selectors for
both forms, distinct/aliased registers and aligned memory, exact bit placement,
preserved source/GPR/EFLAGS/MXCSR and precise bad-memory/alignment failure.

A physical startup sample also showed the splash-message worker waiting for
AppKit while holding the guest CPU lease, blocking the loading worker. Native
PeekMessage now retains the host-only App through Arc, drops its UI RefCell
borrow, and releases the CPU lease for the host pump. Returned events are
processed after reacquiring the lease; output ownership is checked again at
write. The Rosetta branch retains its existing pump/ownership path. The owned
`tools/check_jit32_peek.py` witness passes the same 36 window/callback/worker-
progress words before/after. Existing 32-word window and 26+12 worker/poll
witnesses pass afterward, as do all 60 default CPU tests and runner Clippy.
The owned witness's whole-process times were effectively unchanged (~0.835s);
this is no measured game-loading/FPS improvement. Original replay is next.

The next original run clears matrix shuffling and reaches the variable-count
PSRLQ form, at 829,550,784 loading-thread instructions. All original input hashes
remain unchanged. It takes 138.7s versus 457.1s for the preceding nearby SHUFPS
stop; host load is uncontrolled, stop points differ, and no FPS result follows.
There is still no original Present. Eight XMM variable-count packed shift forms
now use pinned templates, checked across 16/32/64-bit lanes, logical/arithmetic
directions, register/memory/aliased counts and oversized full low-QWORD counts
(upper count QWORD ignored). Source bits, GPR/EFLAGS/MXCSR and memory/fault
boundaries are checked; MMX forms remain explicitly excluded. Mandatory 66 is
verified from instruction prefixes because definition selection consumes that
prefix in the decoder. All 61 default native CPU tests and runner Clippy pass;
the next original replay retains the explicit ten-billion entry budget.

Original execution next clears variable shifts and reaches BTS in the EXE's
runtime code at 829,889,181 loading-thread instructions (179.2s uncontrolled wall
time, inputs unchanged). No Present has yet been observed. BT/BTS/BTR/BTC now
have an explicit Bits32 adapter for register/imm8 forms and 32-bit memory bit
strings: register indices are signed at operand width for neighboring-word
addressing, with explicit sext before floor division, address wrap before segment
translation, and CF from the old selected bit. Aligned locked modifying forms
use actual guarded atomic OR/AND/XOR operations. LOCK register forms and 16-bit
memory addressing remain explicit exclusions; generic unaligned atomic access
retains the existing exclusion.

The independent `tools/check_jit32_bits.py` assembly observer agrees on 112
actual Rosetta/native ARM memory/CF rows, including negative indices, both
operand widths and aligned locked forms. Native units additionally cover
register/imm8 masking, partial writes, source alias/boundary behavior, FS/wrap
and read-only/truncated memory with no partial flag/write commitment. All 65
default CPU tests and runner Clippy pass; six external observers run separately.
This is finite CPU evidence, not new Windows or original game-frame evidence.

Original execution clears BTS and advances through further initialization and
the application's AddIns.xml settings load. At 1,415,045,498 loading-thread
instructions it explicitly stops at repeated CMPSB's unwired intrinsic (217.5s
uncontrolled wall time, original inputs unchanged). No Present was observed.
CMPS/SCAS byte/word/DWORD forms now compare checked sources, set all six
arithmetic flags and advance the correct 32-bit pointers in guest DF order.
REPE/REPNE retain the existing one-iteration boundary, count-zero skip and
last-completed-iteration state on a subsequent fault. Segmented/16-bit-address
strings remain explicit exclusions.

`tools/check_jit32_strings.py` agrees with independent actual Rosetta assembly
on all 432 rows: both operations, three widths, forward/backward direction,
plain/REPE/REPNE, six initial counts and two accumulator values. Result flags,
pointer endpoints, count, accumulator and unchanged source bytes agree. Native
units also verify a fault on the second source of a later iteration and invalid
sources skipped by zero repeat count. All 67 default CPU tests and runner Clippy
pass; seven external observers remain separately invoked. This is finite host
CPU evidence, not an original game frame or parity with Rosetta's cutscene.

The subsequent original run reaches CMPPD in runtime math at 1,617,727,162
loading-thread instructions, after further settings setup (227.5s uncontrolled
wall time, original inputs unchanged). No Present was observed. The eight legacy
scalar/packed predicate and COMI/UCOMI comparison forms now use software
binary32/64 classification/order, guest DAZ/MXCSR and atomic result/flags commit.
Bounds, alignment, aliasing, ignored high immediate bits and explicit unmasked
#XM exclusion have native unit coverage.

The independent matrix exposed a real evidence difference. Native x64 Windows
Server on AuthenticAMD in [CI 42765765607](https://github.com/daeken/Alkyhol/actions/runs/42765765607)
and actual Rosetta agree on all 18,432 comparison masks and EFLAGS results, but
differ on MXCSR DE in 152 ORD/UNORD NaN/denormal rows. Both complete raw tables
and provenance are frozen under jit32/reference; neither was rewritten. The
runner follows that measured native Windows AMD profile. All 18,432 Windows
rows agree via `tools/check_jit32_compare_windows.py`; the separate strict
`tools/check_jit32_compare.py` Rosetta comparison remains failed and records its
failure. This is not physical Intel evidence or universal CPU-model parity.
Windows CRLF versus macOS LF observer source identities are recorded explicitly;
the Windows stdout transport uses binary mode. All 71 default native CPU tests
and runner Clippy pass, with eight external observations invoked separately.
The next original replay retains the explicit ten-billion budget.

The next original replay passes CMPPD and reaches FXAM at 1,618,306,012
loading-thread instructions (218.3s uncontrolled wall time, original input hashes
unchanged). No Present was observed. FXAM now classifies raw physical ST(0),
including empty-slot sign, without calling the stack-reading helper or changing
exception flags, tags or register contents. Its C0/C1/C2/C3 behavior follows the
[Intel FXAM contract](https://cdrdv2-public.intel.com/812383/253666-sdm-vol-2a.pdf);
raw pseudo-denormals and unsupported extended encodings have explicit tests.
`tools/check_jit32_examine.py` compares all 768 owned actual Rosetta/native ARM
rows across twelve raw classes, both signs, empty/full slots, all TOP values
and clear/sticky status. All agree, including preserved data/control and empty
signs. This adds no native Windows or original game-frame evidence. The default
native suite passes 72 tests, with nine external observations separately invoked;
runner Clippy passes. Original replay follows with the same explicit budget.

FXAM clears the original stop; execution next reaches byte ROL in the same runtime
routine, at 1,618,333,452 loading-thread instructions. The pinned native backend
rejects rotate widths other than 32/64. Alkyhol's checked builder now lowers
8/16-bit rotates using unsigned widened temporaries, masked narrow counts and
final truncation; upstream files remain unchanged. Existing lifted rotate flag
semantics are retained. All 5,200 independent actual Rosetta rows agree for both
CL and immediate forms (10,400 native cases), covering byte/high-byte/word
registers, memory, masked/wrapped counts and two initial flag states. Undefined
OF for multi-step rotates is not asserted. Native tests also check preserved
partial registers and no write/flag commitment on read-only memory faults. The
73 default tests and runner Clippy pass; ten external observations are separate.
Original inputs are unchanged, no Present has been observed, and cutscene parity
remains unestablished. Next replay retains the explicit ten-billion budget.

The rotate replay clears ROL and reaches the adjacent original XLAT at
1,618,333,706 loading-thread instructions. The pinned decoder has no XLAT
entry. The checked adapter now decodes this one-byte operation with bounded
legacy prefixes, reads only the selected checked byte at segment+(EBX+zero-
extended AL modulo 2^32), and preserves upper EAX, other registers and flags.
Sixteen-bit addressing and LOCK remain explicit exclusions. All 512 owned
actual Rosetta rows agree; native tests additionally cover operand-prefix
invariance, FS/wrap, single-byte boundaries, precise invalid-memory failure and
the 15-byte length limit. All 74 default native tests and runner Clippy pass;
eleven external observations remain separately invoked. Original input hashes
remain unchanged and no Present has yet been observed. The next replay uses the
new host without overwriting the preceding running executable.

The XLAT replay clears that routine and advances to 1,685,086,081 loading-thread
instructions (226.8s uncontrolled wall time, original hashes unchanged). It stops
at FDIV m64 because the x87 arithmetic helper excludes nonfinite/denormal inputs.
The helper now handles masked arithmetic special values, x87 NaN priority and
indefinite results, sticky exceptions, directed overflow saturation and extended
underflow. Reduced precision uses software 64-bit directed bounds with wider
exponents and an exact software midpoint, avoiding host-f64 approximation and
double rounding. Unmasked x87 data exceptions remain explicitly unsupported.
CPU-fault diagnostics now retain the raw modeled x87 environment locally.

Independent owned register-arithmetic assembly on native Windows AMD in
[CI 42770518331](https://github.com/daeken/Alkyhol/actions/runs/42770518331)
provides 19,200 rows across four operations, three precisions, four rounding
modes and twenty operand classes. All result bits and defined exception/C1 bits
agree through `tools/check_jit32_x87_windows.py`. Complete raw Windows and Rosetta
tables/provenance are frozen separately under jit32/reference. Exactly one
PC64 nearest division row differs by one result ULP and C1 on Rosetta; the model
follows native Windows AMD, and the strict Rosetta comparator remains failed.
Neither oracle nor acceptance criteria were rewritten to hide that difference.
This is finite register arithmetic evidence, not a full memory-operand exception
priority matrix, physical Intel evidence, game-frame or cutscene equivalence.
All 76 default native tests and runner Clippy pass; twelve external observations
are invoked separately. Original replay follows with the existing explicit budget.

A focused follow-up checks the actual reached m64 arithmetic form separately.
Memory widening previously quieted sNaNs and set DE before arithmetic resolved
exception priority. Binary32/64 arithmetic operands now retain signaling NaNs
and defer source-denormal status until that resolution. All 1,440 independent
actual Rosetta m64 rows agree across the four operations, precisions/rounding
and normal/zero/denormal/NaN/infinity combinations. A native Windows memory
observation job is added; its result is pending. Separate native units cover
binary32/64 quiet-versus-signaling priority and sticky flags. The unchanged
19,200-row native Windows register matrix still passes. The running original
host is retained while a separate output path holds the new build.

Native Windows memory CI 42771552614 succeeds on hosted GenuineIntel x64 Windows
Server. All 1,440 m64 rows are byte-identical to Rosetta and agree with native ARM
through `tools/check_jit32_x87_memory_windows.py`; the complete owned table and
provenance are frozen in jit32/reference/x87-memory-windows-x64-v1.*. This does
not turn the separate one-row AMD/Rosetta register-division difference into a
pass, or provide physical Intel/game equivalence evidence.

The next original replay clears the division and enters the original Bink video
decoder, reaching SHRD in binkw32.dll after 1,825,258,068 loading-thread
instructions and 27 entry invocations. Original input hashes remain unchanged;
there is still no observed Present. SHLD/SHRD DWORD forms now concatenate checked
operands in software IR, mask count to five bits, and set defined CF/SF/ZF/PF and
single-step OF. Count zero preserves flags; undefined AF/multi-step OF are not
claimed. Sixteen-bit forms remain explicitly excluded. All 8,320 actual Rosetta
rows agree for immediate and CL forms (16,640 native cases), including register
and memory destinations. Four separate actual Rosetta protection probes confirm
read-only memory faults for counts zero and one in both directions. Native tests
cover source/destination/count aliasing and no partial state on write faults.
All 78 default native tests and runner Clippy pass; fourteen external observations
are invoked separately. The next original replay includes the memory-priority
follow-up and the new shifts; no video/menu/cutscene parity is yet claimed.

The double-shift replay reaches the first original native-JIT DrawPrimitive and
SwapChain Present calls. A physical M4 capture verifies the original animated
intro artwork actually reaches the native window; this is video/graphics evidence,
not just an observed API call. The capture, process identity, host/input hashes
and raw logs stay ignored under local/jit32/shared-game-prefetch-before. It later
stops at PREFETCHNTA in the original Bink decoder, after 2,857,138,100 loading-
thread instructions, 27 entries and 368.9s uncontrolled wall time. Input hashes
remain unchanged. An initial brief Escape input post did not establish a skip;
a later held-input attempt found the process already stopped and sent nothing.
No menu or Orzammar cutscene parity has been established.

The NTA/T0/T1/T2 prefetch hints are now explicitly admitted without an
architectural memory read, as permitted by the [Intel PREFETCHh contract](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2b-manual.pdf).
This is an intentional cache-policy choice, not an omitted required result or
successful API stub. Native tests verify full register/FPU/flag preservation,
next PC and inaccessible/FS/wrapped addresses, while rejecting invalid/truncated
encodings and LOCK. All 79 default CPU tests and runner Clippy pass; the existing
fourteen external observers remain separate. Original replay follows.

The prefetch replay again presents original video and next reaches MOVNTPS in
Bink's frame copy at 2,205,415,328 loading-thread instructions (324.0s uncontrolled
wall time). A foreground-verified held Escape was posted through process-targeted
Quartz input; a menu/skip result was not established before the explicit stop.
Original input hashes remain unchanged. MOVNTPS/MOVNTPD/MOVNTDQ now use the pinned
bit-preserving vector store lifters with the existing full-range write guard and
16-byte alignment check. The host uses ordinary coherent stores as its cache
policy; it does not emulate x86 write-combining cache placement. SFENCE uses the
pinned native DMB ISH emitter, not an empty operation.

Native tests cover every XMM source register, arbitrary NaN/denormal bit patterns,
sentinel bytes around the complete 16-byte write, unchanged registers/flags/FPU,
actual fence execution, precise misalignment/read-only/truncated-range failure,
and invalid encodings. All 81 default native tests and runner Clippy pass; the
fourteen external observations remain separate. These tests do not prove a full
concurrent x86 memory model. Another original replay follows toward the menu and
preserved Rosetta cutscene progress.

The streaming-store replay progresses through original video playback and reaches
RSQRTSS in the original D3DX9 DLL at 2,356,949,458 loading-thread instructions
(438.2s uncontrolled wall time, input hashes unchanged). No original menu/3D
scene has yet been observed. The checked RSQRTSS/RSQRTPS implementation computes
a deterministic software estimate, preserving scalar upper lanes, source/GPR/
EFLAGS and all MXCSR bits. Signed zero/denormal/infinity/NaN behavior is explicit;
rounding control, DAZ/FTZ and masks do not change the instruction's results.
The [Intel architectural contract](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2b-manual.pdf)
allows normal-result relative error up to 3/8192; it does not specify one CPU's
result bits. Bounds are checked with exact integer products rather than a
floating-point reference tolerance. More than 65,536 deterministic normal input
checks and native scalar/packed/aliased/memory/fault/preservation units pass.

[Native Windows AMD CI 42775871328](https://github.com/daeken/Alkyhol/actions/runs/42775871328)
provides 16,640 owned assembly rows. Exact special/state checks and all normal
error bounds pass via tools/check_jit32_rsqrt_windows.py. Of 41,120 active
positive-normal lanes, 40,800 model estimate bit patterns differ from that CPU;
this is a bounded architectural estimate profile, not bitwise CPU emulation.
Downstream Newton-refined D3DX/game float bits are not claimed identical.
Both complete raw Windows and Rosetta tables/provenance are frozen separately.
Rosetta differs from Windows in 22,480 allowed estimate lanes, and forces MXCSR
mask bits on in 8,320 initially unmasked rows. The strict Rosetta diagnostic
retains the latter failure; it is not dropped or relabeled as a pass. All 84
default native tests and runner Clippy pass; fifteen external observations are
separate. The next original replay retains all assets, rendering and the explicit
instruction budget, and continues toward menu/Orzammar progress parity.

Original execution at ae16f00 now reaches the complete main menu, with animated
background, logo, text and New Game/other controls visible in physical M4 window
captures. This is original video/UI rendering; it is not yet original in-engine
3D or Orzammar evidence. The initial menu run was explicitly terminated for input
instrumentation (not a game crash), with original input hashes unchanged. A
second input-traced run confirms actual native keyboard/mouse acquisition,
key/button delivery and explicit relative motion, but no New Game activation
has yet been observed. Foreground/focus were checked through actual native
window state; a live debugger attach did not complete or change trace settings.
The menu/control captures and raw input evidence remain ignored and local.

Optional input diagnostics now record at most 128 changed DirectInput state
snapshots per device, with HRESULT, nonzero bytes and a readback comparison when
the output storage is readable. They do not alter API results or require an
architectural read of write-only output. An opt-in ALKYHOL_JIT32_TRACE_REQUEST
file accepts one hexadecimal guest PC, at most 16 bytes, and requests 256
instruction snapshots on the first guest thread reaching it. Requests are
consumed once; malformed requests fail explicitly. No guest PC/register/data is
modified. These bounded logs are for identifying the next actual DAO blocker.

All 86 default native JIT tests and runner Clippy pass, including the same
production input-deduplication helper and trace-request parser. A direct owned
frozen-ABI run consumes a request and emits exactly 256 trace lines while keeping
all eight Windows output words unchanged. The existing complete shared 8 ABI,
24 TLS and 64 heap-word comparisons also pass normal/rebased modes. A direct
native PE crate unit-test build remains blocked by pre-existing missing legacy
x87 test symbols; no native full-PE-unit result is claimed, and no oracle/fixture
was changed. The input helper is tested from its production source file through
the native JIT test target. Another original input/caller-traced run follows;
New Game, Quick Play and Orzammar progress parity remain unestablished.


The next physical input experiment encountered an independent fullscreen host
condition. Our original window extends beyond the actual 1024x663 desktop;
CoreGraphics reports the display captured, active and awake, while the console
is unlocked. The Alkyhol host never calls CGDisplayCapture. A mode change fails
with CoreGraphics error 1004; successful accessibility focus requests do not
change the actual foreground owner, the independent Wine playtest. The helper's
foreground guard therefore posts no key or click. Only our own window was moved
and restored; only our own native process was suspended to preserve its remaining
explicit instruction budget. The independent process stays running and untouched.

This is an observed host test condition, not evidence of a JIT input/CPU failure
or of New Game activation. The original DirectInput caller trace correctly
rejects its NOTACQUIRED result. A coordinated windowed foreground run remains
necessary to inspect successful state reads and advance the original menu. Local
window/input diagnostics and capture metadata remain ignored; no game code/data,
frozen comparison, API acquisition rule or rendering quality was changed.


After the independent playtest hid its window asynchronously, CoreGraphics
reports display capture released and normal activation succeeds. The original
native process acquires keyboard/mouse, copies successful DirectInput state with
matching readback, and its original caller accepts S_OK with the expected flags.
A foreground-verified, unobstructed desktop click at New Game delivers real
motion and down/up bytes. Later captures are black/inactive while other guest
thread activity begins. Because focus changes can also clear the background
frame, this does not establish New Game activation. Character creation and
original in-engine 3D have not yet been observed.

That old run then reaches its explicit 10-billion instruction/service loop limit,
not a new instruction or API failure; every original input hash remains unchanged.
It is archived with PID/source/host-bound capture metadata. A clean replay now
uses the already-built same native host with an explicit 100-billion limit and
a held actual desktop mode that fits the complete window. No original code,
assets, backbuffer size, API acquisition rule or shader/rendering quality is
changed. The native UI condition has been isolated; loading/Orzammar parity is
still the next execution goal.


The clean replay confirms matching native client/screen mouse coordinates once
our local helper refreshes window bounds after activation/repositioning. It
visibly highlights New Game and then reaches a new explicit FPATAN stop in
original game math: 7,363,291,599 native guest instructions, 40 entries, status 78,
and unchanged original input hashes. Character creation, original in-engine 3D
and Orzammar have still not been captured. The independent Wine playtest and its
results remain separate.

FPATAN now has an explicit mathematical profile: masked exceptions, nearest
rounding and canonical finite normal/zero operands; when both are nonzero,
the exponent separation is at most 1024. Outward software Quad intervals certify mathematical 64-significand
rounding; an exact integer quotient residual plus alternating-series bounds
handles tiny corrections. The instruction consumes ST(0), replaces ST(1), pops
once, preserves unrelated registers and EFLAGS/MXCSR, and ignores the three valid precision
settings. Other controls, stack underflow, nonfinite/subnormal/noncanonical data,
out-of-range ratios and uncertifiable rounding fail without state commit.
This follows the precision scope in [Intel SDM volume 1, sections 8.1.5.2 and 8.3.10](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-1-manual.pdf);
it does not reproduce one CPU's polynomial or claim native C1/PE bit parity.

[Native Windows AMD observer CI 42794807414](https://github.com/daeken/Alkyhol/actions/runs/42794807414)
passes all three observer jobs at 858e2cf. Its 768 owned FPATAN rows and a separate
768-row Rosetta table are frozen unchanged with source/artifact/host provenance.
All binary64 results agree; each CPU result is within one extended-precision ulp
of the mathematically rounded model. Windows has 6 raw 80-bit result differences
and 12 defined-status differences; Rosetta has 12 and 309 respectively. The strict
comparison fails on both and retains those failures via
`tools/check_jit32_atan2.py --strict` (optional `--backend rosetta`). The default
checker tests the stated math/stack/environment profile and reports differences;
it never calls that strict CPU parity. No game-derived fixture is used in CI.

All 90 default native JIT tests and runner Clippy pass. The two new external math
comparisons pass separately; the two strict diagnostics fail as described, and
the other 15 external observers remain separate. The next original replay uses
the new FPATAN host; full application progress parity remains unestablished.


The built 6f78b1d native host also preserves the frozen native Windows outputs: 8 ABI words, 24 TLS words and
64 heap words in normal/rebased runs. A fresh unchanged original process
has completed CRT startup and created its owned splash window under the recorded
full-window desktop. It is retained, suspended, for the independent playtest's
foreground restore. Continue from the ignored `local/jit32/shared-game/run.json`,
`startup-slot.json` and `startup-extension.json` rather than starting a second
copy. The next foreground replay must confirm the post-FPATAN scene; original
character/3D/Orzammar progress parity remains unobserved.


The retained FPATAN-host replay renders another identifiable original Bink intro
frame on the physical M4 and remains alive without a new CPU stop. A bounded
coordinated presentation slot confirms the display capture is released, but the
busy original decoder does not service activation in that interval. The local
input guard refuses to post while actual foreground remains the independent
playtest. No click/key is delivered in that slot. The original is suspended with
its state intact during the other playtest's subsequent input slot. This is an
intro checkpoint, not character creation, in-engine 3D or Orzammar parity.

A separate bridge fix removes startup-cached desktop dimensions from App/Window.
Mouse MSG screen points, primary metrics, window/client rectangles and window
placement now query the current primary logical desktop. Missing/invalid geometry
fails explicitly. This addresses the observed mode-change error where a physical
Y=900 point was reported as Y=320 because startup height 663 was reused after the
desktop became 1243 high. The retained original process still uses its existing
host; this fix has not been injected into it or claimed as a game replay result.

On the physical native ARM M4, all ten tests of the actual window module pass,
including the new regression, signed/offscreen rectangle consistency, invalid
geometry rejection and a read-only current CoreGraphics mode/bounds comparison.
The existing bitmap, mouse/capture and main-thread closure tests also pass. These
tests do not open a window, activate another app, change modes or post input.
The native-runtime32 ARM and x86_64 PE libraries both check successfully, and a
separate native shared JIT host builds with the fix. The retained original host
is not overwritten. A live post-change window
and input replay remains separate from these assertions. Reproduce the isolated
window-module tests without the unrelated legacy x86 transition link objects:

```sh
xcrun --sdk macosx clang -arch arm64 -mmacosx-version-min=14.0 -fobjc-arc \
  -Wall -Wextra -Werror -c pe/src/screen_capture32.m -o local/window-capture.o
rustc --edition=2024 --test pe/probe_window_host.rs \
  -C link-arg=local/window-capture.o -l framework=ScreenCaptureKit \
  -l framework=Foundation -o local/window-host-tests
local/window-host-tests --test-threads=1
```


The unchanged original native replay now displays its actual foreground menu,
receives the New Game click with matching native client/screen and DirectInput
button states, and executes past the exact former FPATAN stop. A bounded 256-step
read-only trace records that instruction and its next original PCs. It then
stops explicitly at the newly reached RCPSS; status 78, original EXE/provided DLL
hashes unchanged. The run and capture/trace provenance are archived in ignored
`local/jit32/shared-game-rcpss-before`. Character creation, original in-engine
3D and Orzammar progress parity remain unobserved. The independent replacement
playtest is separate evidence and its now-expired original-PID control is revoked.

RCPSS/RCPPS now share a deterministic software reciprocal profile. The admitted
estimate bound, signed special values, quiet-NaN payloads and unconditional tiny
flush follow the [Intel RCP entries](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2b-manual.pdf).
Finite normal estimate bits and flush choices near the tiny boundary need not
match a CPU's table. Scalar upper lanes, flags and MXCSR are preserved. Packed
alignment and complete source read rights/extent are checked before commit.

All 94 default native ARM JIT tests and runner Clippy pass; 18 external observers
are ignored by default. The new owned 67,584-row Rosetta RCP comparison runs
separately: 167,200 normal lanes meet exact integer bounds, 164,000 differ in
normal result bits and 320 differ in permitted flush choice. RCP itself preserves
all measured MXCSR and defined flags. The observer additionally records the
actual MXCSR immediately after LDMXCSR: 33,792 rows differ from the requested
mask state under Rosetta. That separate load behavior remains visible; none of
the older frozen RSQRT or other strict comparisons is changed. This is finite
instruction evidence, not application or raw CPU-bit parity.

The owned C source/observer/data identities and compressed Rosetta table are
frozen. `python3 tools/check_jit32_rcp.py` replays that table; a native Windows
observer passes with all three jobs in [CI 42814117586](https://github.com/daeken/Alkyhol/actions/runs/42814117586) at 94ee263.
Its host reports GenuineIntel, native AMD64 machine/process checks, and Windows
Server build 26100. The frozen 67,584-row table also passes on physical ARM:
167,200 normal lanes meet the bound, 164,000 estimate-bit differences and 320
permitted flush-choice differences are retained, with zero requested-load,
instruction-MXCSR or defined-flag differences. Reproduce with
`python3 tools/check_jit32_rcp.py --backend windows-amd64`.
No original bytes, API results, geometry or rendering quality are substituted.
A fresh native host replay is required to test the reciprocal fix in DAO.


The built clean 94ee263 shared host passes the frozen native Windows 8-word ABI,
24-word TLS and 64-word heap witnesses, each in normal/rebased mode. It starts a
fresh unchanged original replay with reciprocal support and the dynamic desktop
geometry fix. The original run is loading in the background under a newly
coordinated process identity; consult ignored `local/jit32/shared-game/run.json`
for its exact host/source/input hashes. A bounded trace is armed at the newly
reached reciprocal instruction. This restart is not evidence of passing that
instruction or reaching character/3D/Orzammar until actually observed.


The fresh unchanged original replay now executes past the former RCPSS stop:
a bounded trace records that original instruction and its successors. It stops
explicitly at newly reached MINPS, status 78, with original EXE/provided DLL
hashes unchanged. The stopping thread reports 12,551,112,518 instructions.
Foreground captures still show a loading transition, not character creation or
in-engine 3D. The scoped UI controller independently detects exit and restores
the prior desktop before the coordinated deadline. Raw evidence remains in
ignored `local/jit32/shared-game-minps-before`; no live original PID remains.

A preserved replacement-generated character/save input was copied into Alkyhol's
own ignored profile after independent source-hash verification. The source and
live replacement files are untouched. Load Game remained disabled after this
live copy, so no original save parse/load or save incompatibility is established.
The input is now present for fresh-start discovery; its provenance is a shared
test input, never original reference behavior or recovered-body acceptance.

All eight legacy SIMD minimum/maximum forms are now implemented through the
existing checked arithmetic adapter. The selection follows the [Intel entries](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2b-manual.pdf).
All register routes, scalar upper lanes, aliases, read extents/rights and packed
alignment have targeted native ARM tests. Unsupported unmasked exceptions fail
without destination/environment commit; general #XM delivery remains separate.
The compile-stop diagnostic now prints the emulated GPR/XMM/x87/MXCSR state from
owned Rust arrays, without native Win32 thread-context sampling or guest mutation.

All 98 default native ARM JIT tests and runner Clippy pass, with 19 external
observers ignored by default. The separate new Rosetta observer has 65,536 rows:
all result bits, complete MXCSR and defined flags agree exactly, including the
special/denormal pair matrix and deterministic raw-bit pairs across all forms
and control combinations. The C source, compiled-observer identity and table are
frozen; `python3 tools/check_jit32_minmax.py` replays them. The existing SIMD
workflow also observes this unit on native x64 Windows. All four jobs pass in
[CI 42825749976](https://github.com/daeken/Alkyhol/actions/runs/42825749976) at 80912a8.
Its hosted native Windows AMD machine reports Family 26/Model 2 and build 26100.
All 65,536 rows match the physical ARM model exactly. The entire 5,242,880-byte
Windows observation table is also byte-identical to the separate Rosetta table;
source/artifact/data identities and the native fixture are preserved. Reproduce
with `python3 tools/check_jit32_minmax.py --backend windows-amd64`.
The result checkers additionally require a complete model report so an empty
Rust test selection cannot become success. Older frozen observations/strict
failures are retained. A fresh DAO run is still required for the MINPS fix;
Orzammar progress parity and native full-frame equality remain unestablished.


A clean 80912a8 native shared host also preserves the frozen native Windows ABI,
TLS and heap words in both normal/rebased witnesses. The preserved shared scene
input is already in the owned profile for the next fresh original launch. The
host now includes min/max and read-only compile-stop operand diagnostics, but its
actual application progress beyond MINPS still requires a fresh observed replay.
No original executable, DLL, save source or rendering geometry is patched.

The fresh unchanged native replay now discovers the pre-staged shared scene
input at startup: actual original menu captures show Resume and Load Game
enabled. Its own DirectInput state readback also confirms the foreground Escape
down/up events; the later intro-to-menu transition does not by itself prove
which event caused it. Save parsing/loading, execution past MINPS, original
in-engine 3D and Rosetta's Orzammar progress remain unobserved in this replay.
The process is retained paused/hidden for the next coordinated input slot.
This is original menu discovery evidence, not reference behavior for the
replacement-generated save. Raw process/capture provenance stays ignored/local.

Native profiling of the original Bink replay identifies repeated immutable
executable-section scans on instruction-cache hits. The shared executor now
remembers one admitted static section per execution context. Zero length, rights
and guest-extent validation still precede lookup; complete instruction bytes are
still compared on every cache hit. Dynamic executable allocations use their
existing live check and are never remembered by this cache. Separate contexts
cannot inherit a previous image's bounds; adjacent sections are not merged.

Three UI-free physical ARM tests cover equivalence to the previous full scan,
unsorted/overlapping sections, gaps, page rounding, context isolation, biased
host addresses and overflow. Reproduce with `rustc --edition 2024 --test -D warnings
pe/probe_exec_ranges.rs -o local/jit32/executable-range-tests` and run that binary.
The newly built native shared host still agrees with the frozen native Windows
8 ABI, 24 TLS and 64 heap words in normal/rebased modes; the x86_64 PE library
also passes its compile check. The full PE native-runtime library-test target
fails on four absent CI-sqrt/atan symbol references on both the unchanged parent
and changed tree; that pre-existing target failure is not counted as a pass.

An isolated physical M4 benchmark of 128 fixed sections uses six alternating
trials of ten million checks per path. Full scans take 790–912 ms, versus
10.5–13.6 ms for repeated cached-range checks, with identical admitted counts.
These are bounds-lookup timings, not an end-to-end DAO speedup or FPS result.
The retained live original process still uses its earlier recorded host binary;
this optimization applies to the next host build/replay. Raw reports stay in
ignored `local/jit32/executable-range-*` and `exec-ranges-shared-check`.

The unchanged original native replay executes past the former MINPS stop; its
bounded emulated-array trace records that instruction and successors. It then
stops explicitly at newly reached SQRTSD while loading, status 78, with original
EXE/provided DLL hashes unchanged. The stopping thread reports 6,365,664,291
instructions. Foreground/background captures show black loading, not an original
in-engine frame. The menu click appears to select New Game after the retained
window resized; no savegame.das load was observed. The prior shared-save menu
discovery remains separate evidence. Raw evidence is archived in ignored
`local/jit32/shared-game-sqrtsd-before`; its process identity is revoked.

This replay also exposes a host GUI limitation: while its main thread performs
an original blocking wait, host handoff tasks run but AppKit events are not
pumped. Normal hide requests therefore remain unprocessed during loading.
Independent native ARM stack samples and WindowServer observations distinguish
this from a stopped loading worker. The scoped desktop handback explicitly
accepted the visible background loading window; no successful-hide claim is
made. Native-event pumping during such waits needs a separate checked change.

SQRTSS/SQRTSD/SQRTPS/SQRTPD now use an exact integer radicand and remainder to
round once at destination precision under guest MXCSR. DAZ, signed zero,
infinities, NaN payload quieting, invalid/denormal/precision flags and unmasked
failure follow the [Intel square-root entries](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2b-manual.pdf).
Positive finite binary32/64 roots are normal and finite; FTZ has no effect.
All 102 default physical ARM JIT tests and runner Clippy pass; 20 external
observers are ignored by default. Targeted tests cover all register routes,
aliases, scalar upper lanes, memory extents/rights/alignment, unrelated state
and non-commit on unsupported unmasked exceptions.

The new owned Rosetta observer records 32,768 rows over all four forms, all
RC/DAZ/FTZ combinations, sticky flags, a 32-value special-source set and
deterministic raw bits. Every result bit, complete MXCSR and defined EFLAGS
agrees with physical ARM execution; requested and actually loaded MXCSR also
agree in this masked corpus. Source/observer/data identities and the compressed
table are frozen; `python3 tools/check_jit32_sqrt.py` replays it. All five jobs in the native observer workflow pass at 925b667 in
[CI 42836887248](https://github.com/daeken/Alkyhol/actions/runs/42836887248).
The square-root host reports native AMD64 Windows Server build 26100 and
AuthenticAMD Family 25/Model 17; native/process machine checks are retained.
Its entire 2,621,440-byte table is byte-identical to the separate Rosetta table,
and all 32,768 rows match physical ARM result bits, MXCSR and defined flags.
The native source/artifact/table identities and compressed fixture are frozen;
reproduce with `python3 tools/check_jit32_sqrt.py --backend windows-amd64`.
The clean 925b667 shared host also retains the frozen Windows ABI/TLS/heap words
in normal/rebased cases. No older frozen comparison is weakened. A fresh original replay is
required to verify this square-root implementation in DAO, and Rosetta's
Orzammar progress parity remains unestablished.

The next unchanged original replay opens the real Load Game path and stops
explicitly at FPREM, status 78, before save selection or an observed savegame.das
load. The stopping thread reports 8,391,647,650 instructions; original EXE and
provided DLL hashes remain unchanged. Its state is archived in ignored
`local/jit32/shared-game-fprem-before`. No original in-engine frame is established,
and the previous SQRTSD loading route has not yet been exercised past its stop.

FPREM and FPREM1 now implement exact finite complete reductions with exponent
difference below 64, using integer significands and exact remainders. Rounding
and precision controls do not round this result; the observed quotient-magnitude
bits update C0/C3/C1 and C2 clears. Partial reductions, exceptional/noncanonical
operands, zero divisors, empty stack slots, reserved precision and unmasked
exception states fail explicitly without committing destination/environment.
The partial-reduction algorithm remains separate; see the [Intel entries](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-2a-manual.pdf).
All 106 default physical ARM JIT tests and runner Clippy pass; 21 external
observers are ignored by default. Targeted generated-code tests cover every
stack rotation, legal precision/rounding/infinity controls, signed ties/zeros,
exact denormals, large complete quotients, unrelated GPR/XMM/environment state
and non-commit on unsupported inputs.

All four observation jobs pass in [CI 42847987636](https://github.com/daeken/Alkyhol/actions/runs/42847987636)
at source b5d9d95. The native AMD64 Windows host reports AuthenticAMD Family
26/Model 2 and Server build 26100. Its 24,576 complete finite FPREM/FPREM1 rows
match physical ARM result bits, full x87 status/tag/control, MXCSR and defined
EFLAGS exactly. Source/observer/table identities and the compressed fixture are
frozen. `python3 tools/check_jit32_remainder.py` checks that native table.

The new Rosetta table is also frozen, with its strict failures preserved:
96 FPREM1 result/status rows differ, covering four large-quotient input pairs
across 24 controls. Exact rational arithmetic agrees with the Windows/model
remainders; the observed Rosetta magnitudes for those pairs exceed half the
divisor. Ordinary FPREM rows agree. This is a measured standalone sequence
difference, not a live-game or universal-CPU diagnosis.
`python3 tools/check_jit32_remainder.py --backend rosetta` deliberately fails
strict equality; no old comparison is weakened. The first diagnostic also
exposed an assumed initial full-tag state: the observer now measures the actual
before environment, rather than guessing tags from raw values. Adding that
observation changed none of the prior result/status/output fields. Both initial
and corrected-input strict failures remain ignored/local. Full after-state
comparison is retained. A fresh DAO replay is still required for this fix.

The GetLocalTime import reached by the preserved Rosetta cutscene now has a
checked PE32 stdcall implementation. It writes the real current wall clock as
eight SYSTEMTIME words under the existing explicitly selected UTC guest profile;
no host timezone/DST mapping is inferred. Missing timezone configuration, odd,
null and non-writable output pointers fail explicitly before the write. The
contract follows [Microsoft GetLocalTime](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getlocaltime)
and [SYSTEMTIME](https://learn.microsoft.com/en-us/windows/win32/api/minwinbase/ns-minwinbase-systemtime).

The same four owned PE32 microtests run on physical M4 native ARM JIT and
Rosetta, at normal and forced placement. Each backend passes 48 UTC clock samples
bracketed by actual FILETIME calls, complete field/weekday checks, word-aligned
output guards and preserved last-error state, plus eight explicit unsupported
profile/pointer failures. These are executing-host clock/ABI assertions, not
frozen native Windows wall-clock equality. Reproduce with
`python3 tools/check_pe32_current_local_time.py` (Rosetta), or `--host` pointing
to the shared native ARM host. The existing PE workflow includes this check.
The rebuilt native host also retains all frozen Windows ABI/TLS/heap words in
normal/rebased modes. Reports remain ignored/local under
`local/pe/current-local-time-rosetta` and `local/jit32/current-local-time-check`.
The retained game replay still uses its earlier source-pinned host; neither
this application call nor Rosetta's cutscene progress parity has been verified
in a fresh game replay.

The clean 352148e original replay passes the reached FPREM instruction and its
successors in a bounded trace, then renders the actual Load Game selection UI.
The frozen first Cousland quicksave is selected and the original CreateFile path
for that exact staged savegame.das is observed. The staged hash remains unchanged;
this log alone does not verify a successful save read or a restored world. The
run then stops at checked guest read/fetch 0x12f000+1 (status 78), with
7,724,405,480 instructions and original EXE/provided DLL hashes unchanged. No
native original in-engine frame is established. The guard/call identity needs
better native fault diagnostics; it is not yet diagnosed as corrupt game state.
Evidence is preserved in ignored `local/jit32/shared-game-fetch-before`; its
process identity is revoked. The SQRTSD loading route is still unverified.

Native denied fetch/read reports now resolve inaccessible import traps through
the owned diagnostic registry, converting guest offsets to host addresses before
that metadata lookup. Denied fetches include current flags/registers and up to
32 bytes of independently checked stack; denied data reads name their target.
No Win32 thread-context sampling, signal injection or permission relaxation is
used. Four actual owned deferred-function/data PE cases (normal/rebased) retain
status 78 and their prior output marker while reporting the correct import;
all six frozen Windows ABI/TLS/heap shared-host checks also pass. Raw assertions
are in ignored `local/jit32/guard-import-diagnostics.json`.

A separate local metadata-only inspector exits before the first requested guest
instruction. With the original run's launch profile/environment, its inaccessible
import range starts at 0x124000 and 0x12f000 labels CopyFileW; the earlier table
prefix is unchanged by the GetLocalTime addition. This points to a deliberately
blocked copy import as the loading stop, rather than establishing corrupt game
state. The stopped run did not itself print that label or its argument paths;
a fresh replay must confirm the call. Private mapping evidence stays alongside
`local/jit32/shared-game-fetch-before`. No additional game execution or frame
compatibility is inferred from mapping-only inspection.

CopyFileW now performs a real streamed copy into the existing explicitly
configured Documents write capability. It checks UTF-16/ordinary paths, source
identity and readable sharing, destination identity, overwrite/create semantics,
readonly state and all file I/O before reporting completion. Exact contents,
truncation, access/write times and the ordinary readonly attribute are retained.
Missing source, existing exclusive destination, readonly destination and denied
source read sharing return actual modeled Win32 failures. Same-file/directory
copies, live destination handles, live source writers, symbolic links/aliases,
paths outside the write capability and unsupported metadata/I/O conditions fail
explicitly. This is a bounded ordinary-file contract, not NTFS security, EFS,
alternate-stream or complete filesystem compatibility. See
[Microsoft CopyFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-copyfilew).

Twenty-two owned PE32 cases pass independently on physical M4 native ARM JIT and
Rosetta at normal/forced placement. Assertions check every binary content byte,
overwrite truncation, write timestamp, readonly state, exact supported failure
codes, unchanged source/old failed target, nonzero BOOL and explicit exclusions;
no game input is used. Runner Clippy and the six frozen native Windows
ABI/TLS/heap shared-host checks also pass. The same PE files are carried by the
existing PE workflow for a native Windows observation; that new result is still
pending. Dynamic file/clock assertions are not a new frozen Windows comparison.
Reproduce with `python3 tools/check_pe32_copy_file.py`, or `--host` pointing to the
shared native ARM host. Reports remain ignored/local under `local/pe/copy-file`
and `local/jit32/copy-file-check`. Fresh original execution must still confirm
the identified copy call and actual save restoration; native 3D/Rosetta cutscene
parity is not established by these microtests.
