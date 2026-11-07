# Bounded partial-prologue restoration

RtlVirtualUnwind now accepts function entry and the recorded operation-end
boundaries strictly inside a supported AMD64 v1 prologue. It validates the entire
recipe, then skips operations whose CodeOffset exceeds the control offset.
Multi-slot operands are skipped as a unit and cannot become control boundaries.
This follows Microsoft's [prologue unwind rules](https://learn.microsoft.com/en-us/cpp/build/exception-handling-x64).

Completed-body control points still require a NOP. An offset equal to the full
prologue length uses that existing body gate. Unknown interior offsets, frame
pointers, handlers/chains, epilogues and unsupported opcodes/orderings still fail.
There is no instruction decoder, exception dispatcher, throw/catch or execution
resumption. The trusted caller supplies a context consistent with the selected
control point. Full-recipe validation precedes stack reads and context/output
writes, including when the unsupported operation has not yet executed.

```sh
python3 tools/build_pe_unwind_prologue.py
python3 tools/run_pe_unwind_prologue.py --fixture local/pe/unwind-prologue/unwind-prologue.exe --output local/pe/unwind-prologue/observation.json
```

The new owned MIT observer is built once as a real Windows PE. LLVM SEH assembler
directives emit its static function/unwind records for two pushes, a 168-byte
large allocation, an R13 save and a full XMM6 save. No manual registration or
replacement Windows implementation is linked. A real fully executed frame saves
its caller's values and calls a C helper; that helper constructs six curated
CONTEXT inputs at offsets 0, 1, 3, 10, 15 and 21 using those saved stack bytes.
These partial-prologue inputs are constructed states, not CPU snapshots of
interrupted prologue execution. The fully executed assembly frame restores its
registers and returns normally through its original stack.

Each manual unwind records control offset, handler absence, caller RIP, RSP
delta, establisher, RBX/R12/R13 restoration, equality of all 16 XMM6 bytes, flags
and a predicate checking every other seeded context byte. All 68 DWORDs are compared exactly with the
identical PE on native Windows, alongside executed PE/source hashes, clean
revision and normal/forced-rebase host observations. Native results are pending
the first independent CI execution; local physical M4/Rosetta development runs
pass. See [validation](validation.md) for the actual tested revisions and results.

Host tests separately cover every accepted boundary and rejection of other
offsets, each multi-slot operand form, invalid skipped opcodes/slots/save extents,
duplicate/overlapping saves and out-of-order recipes. Actual context restoration
tests check only completed register saves change, precise stack/establisher
results, preserved unrelated bytes and transactional rejection before an unusable
stack pointer can be read. Broader recipe forms remain host assertions. The
existing frozen active-body/static-table observers remain unchanged. This finite
manual restoration contract does not establish Windows exception or game support.
