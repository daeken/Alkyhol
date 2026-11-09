# SharpRetro execution backend: coordination contract

The user requests a future runner for the unchanged DAO PE32 binary using
SharpRetro instead of Rosetta compatibility-mode execution, both to run the
application and to validate SharpRetro's x86 implementation. Pure Rust is the
target. This document records integration planning; no SharpRetro game runner
has been delivered or validated by this milestone.

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
