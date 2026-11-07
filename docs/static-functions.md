# Registered static AMD64 function tables

The standalone PE launcher now uses a context-borrowing RegisteredImage API. It
accepts validated static `.pdata`/`.xdata` for bounded AMD64 v1 body recipes,
registers the original mapped records before executable entry, and exposes them
through the real Win64 RtlLookupFunctionEntry/RtlVirtualUnwind bridges. This is
module metadata and manual caller-context restoration, not Windows exception
dispatch, throw/catch or control resumption.

```sh
python3 tools/build_pe_static_functions.py
python3 tools/run_pe_static_functions.py --fixture local/pe/static-functions/static-functions.exe --output local/pe/static-functions/observation.json
```

## Rust API and lifetime

`Image::runtime_functions` returns validated original RVAs and 12-byte records
without executing or registering anything. `Image::check_registered_startup`
preflights the registered EXE subset, including all other existing startup gates.
`RegisteredImage::load(bytes, rebase, &process)` maps and registers supported
records before returning; `relocated` and unsafe `enter` retain the ordinary
checked mapping/trusted-code contracts.

The RegisteredImage borrows its ProcessContext, so safe Rust cannot destroy the
scope before the image. Drop removes its static registry entry before unmapping
code/metadata. Static entries cannot be removed through guest RtlDeleteFunctionTable;
dynamic entries retain their separate explicit lifetime. Registration failure
cleans up the mapping and preserves prior entries. The shared registry bounds of
64 tables/65,536 aggregate entries still apply. No Mach-O OS unwind registration
or guest-thread support is implied.

The older `LoadedImage::load`, `LinkedImage` and DLL/library APIs retain their original `.pdata`
startup gate. Static DLL registration/detach integration remains separate. The
standalone launcher selects RegisteredImage automatically for EXEs, including
those without static records; those existing programs retain their exact output
and calling contracts.

## Validated subset

Tables must have DWORD-aligned RVAs and complete 12-byte entries, at most 65,536
per table, sorted nonoverlapping half-open code ranges and direct aligned unwind
RVAs. Table/recipe bytes, including unwind-code alignment padding, must be
initialized readonly non-executable image data. Function ranges must be initialized
RX code. IAT and DIR64 writes are checked for overlap with table/recipe spans,
preventing loader fixups from changing validated metadata before registration.

The same pure recipe decoder is shared with manual restoration: version 1, zero
handler/chaining flags and frame register, supported PUSH_NONVOL, ALLOC_SMALL,
both ALLOC_LARGE forms and bounded integer/vector slot saves. All earlier register,
offset/order/alignment/fixed-frame restrictions remain. Unknown versions/operations,
handlers, frame pointers, TLS/callbacks, CFG/load config, delay/bound imports and
other unsupported startup shapes fail before entry. RtlVirtualUnwind still only
accepts NOP body control points; accepting metadata does not add arbitrary PC,
prologue/epilogue or exception-dispatch semantics.

## Independent Windows client

The owned MIT no-CRT client uses LLVM assembler `.seh_proc`, `.seh_pushreg` and
`.seh_stackalloc` directives to emit a real static PE table. The table/RVAs are
assembler/linker output, not a hand-constructed registration array. Its C helper
is built without additional unwind tables, leaving exactly one recorded target
frame. The program never imports or calls RtlAddFunctionTable.

Before entering that target it resolves its static record/base, checks the record
against the actual PE exception directory, verifies begin/end boundaries and
queries again after a rejected deletion attempt. The active target saves/clobbers
RBX/R12 and allocates 40 bytes. A helper independently reads the real saved stack
values/return address and checks manual caller-context restoration, including all
other seeded CONTEXT bytes; execution then returns through the unchanged stack.

All 20 raw DWORDs are checked exactly, alongside source/executed PE identity and
normal/forced-rebase observations. Native Windows runs the identical PE, relying
on its own OS module-table support. The Native/Alkyhol results and precise tested
revision are recorded in [validation](validation.md). Host tests separately check
metadata shapes/extents/permissions, fixup alias rejection, ordinary-loader/DLL
restrictions, pointer identity, protected static deletion, drop cleanup, table
capacity rollback and the process-guard lifetime compiler constraint. Existing
dynamic/native fixtures remain unchanged. This is finite static-table evidence,
not general compiler-runtime or game compatibility.

Primary contracts: [AMD64 function/unwind records](https://learn.microsoft.com/en-us/cpp/build/exception-handling-x64),
[PE exception directory](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#the-pdata-section),
[RtlLookupFunctionEntry](https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtllookupfunctionentry)
and [RtlVirtualUnwind](https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtlvirtualunwind).

All 20 raw words agree on native Windows, hosted Mac and physical M4 at `9a01d74`.
The actual native record at `pe/reference/static-functions-windows-v1.json` is
frozen and required by default, including exact executed PE/source identity.
This native observer covers one assembler-generated v1 push/small-allocation frame;
other static recipe forms and lifecycle failure boundaries have host assertions.

Strict-reference revision `825549f` passes all three PE and 24 conformance CI
jobs. Downloaded native Windows/hosted Mac reports and fresh clean physical M4
execution at that same revision agree on all 20 words and exact PE/source
identities; normal and forced-rebase host runs agree. See the linked validation
record for the runs and separate host-test evidence. This does not add exception
dispatch or general compiler-runtime coverage.
