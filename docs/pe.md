# Windows binary execution

Alkyhol now has a first **binary execution** path alongside source compatibility:
unchanged AMD64 Windows PE machine code runs in a checked image mapping inside an
x86-64 Mach-O host, through explicit Win64-to-native Rust bridges. On Apple Silicon
that host executes under Rosetta. The output links Alkyhol's real kernel functions;
it does not recompile the guest source as macOS code. This first PE-to-Mach-O
artifact is a standalone wrapper with an embedded PE loader, not static rewriting
of all instructions or a general Windows executable implementation.

## Reproduce

macOS with Xcode command line tools, Rust and its `x86_64-apple-darwin` target:

```sh
rustup target add x86_64-apple-darwin
python3 tools/build_pe_fixture.py
python3 tools/run_pe_probe.py --fixture local/pe/fixture/contract.exe --output local/pe/alkyhol.json
cargo test --manifest-path pe/Cargo.toml --target x86_64-apple-darwin --locked --offline
```

The fixture is compiled once to real Windows COFF/PE using open Clang and Rust's
bundled LLVM linker. Its import library is generated from our explicit kernel32
`.def`; no Microsoft runtime/SDK binary or proprietary converter is linked.
`pe/fixtures/contract.c` and `abi_canary.S` are original MIT-licensed test code,
not Microsoft samples or game-derived code. Native Windows runs the **identical
PE file**, without recompiling it, against the OS's real kernel32. The separate
[PE workflow](../.github/workflows/pe-binaries.yml) transfers that artifact and
compares the PE SHA-256, fixture sources, revision and actual results. Existing
Windows SDK, original-source, shader and Metal comparisons remain separate.

To package another trusted input:

```sh
python3 tools/translate_pe.py /path/to/program.exe --output local/pe/program-macos
local/pe/program-macos --inspect
local/pe/program-macos
local/pe/program-macos --rebase
```

Packaging embeds an unchanged snapshot in read-only `__ALKYHOL,__pefile`, extracts
that section from the finished Mach-O to verify its exact SHA-256 against the
input snapshot, and writes an adjacent identity report. Packaging alone is not validation or execution.
`--inspect` checks startup/import support and creates/destroys a checked mapping
without entering guest code. Missing support exits 78. Guest arguments follow
`--`; command line, environment and inherited streams now have a bounded process
view. No Windows PEB/TEB is installed. The released [u-config](u-config.md) and [make2compdb](make2compdb.md) programs
run unchanged with real package queries and make-to-JSON work.

## Image and ABI contract

The parser accepts bounded conventional page-aligned PE32+ AMD64 GUI/console EXEs.
It checks raw and virtual extents, disjoint section pages, initialized code entry,
file-backed directories, bounded import descriptor/name/thunk termination and
unique DIR64 relocation intervals. Headers are read-only; final section pages use
their declared read/write/execute permissions; writable executable sections fail.
Gaps are inaccessible. Anonymous mapping never uses MAP_FIXED or replaces host
mappings. A missing preferred base needs valid DIR64 relocation targets. The
runtime currently requires 4K pages in the x86 process. Memory starts zero-filled,
including uninitialized section bytes; imports/fixups are applied before final
permissions. Import slots cannot overlap relocation fixups.

The ordinary executable and isolated DLL constructors resolve imports by exact
symbol from the explicit **kernel32.dll**, **shell32.dll**, **d3dcompiler_47.dll** and **d3d12.dll** whitelists,
with case-insensitive DLL identity. The dependency APIs below also accept named/
ordinal imports from caller-supplied DLL instances. `LinkedImage::load` extends
EXEs with those explicit borrowed dependencies without changing the ordinary
`LoadedImage::load` or single-file launcher contract.
No arbitrary `dlsym` of native exports, name-only DLL matching, fallback
libraries or successful unknown stubs occur. The compiler-generated
`extern "win64"` wrappers preserve the Windows argument/return and nonvolatile
register contract while invoking Alkyhol's native exports. `CreateFileW` receives
16-bit UTF-16, including a surrogate pair; source compatibility's 32-bit wchar_t
adapters are not used. Current mapped guest addresses are actual native virtual
addresses in this deliberately shared process. This is not a pointer conversion
scheme for another address space and is not a memory safety sandbox.

The original PE fixture exercises these 24 bridges; the [released u-config
program](u-config.md) adds ten process-service names (34 kernel32 names total):

| Area | Imports |
| --- | --- |
| Error / lifetime | GetLastError, SetLastError, CloseHandle, ExitProcess |
| Synchronous files | CreateFileW, ReadFile, WriteFile, GetFileSizeEx, SetFilePointerEx, SetEndOfFile, FlushFileBuffers |
| Events | CreateEventW, SetEvent, ResetEvent, WaitForSingleObject |
| Semaphores | CreateSemaphoreW, ReleaseSemaphore |
| Timing | QueryPerformanceCounter, QueryPerformanceFrequency, Sleep |
| Dynamic TLS slots | TlsAlloc, TlsFree, TlsGetValue, TlsSetValue |

The separate binary-COM milestone below adds d3dcompiler_47!D3DCreateBlob and
five explicit Win64 ID3DBlob virtual slots. The serializer milestone adds
d3d12!D3D12SerializeRootSignature and D3D12SerializeVersionedRootSignature.
The scoped guest lookup milestone adds GetModuleHandleW/GetProcAddress (36
kernel32 names total); these require an explicit ModuleContext for supported
DLL lookups and do not create fake builtin/main-image module handles.
VirtualProtect, VirtualFree and FlushInstructionCache add bounded owned-allocation
protection, release and generated-code execution (39 kernel32 names total).
Explicit dynamic function-table registration/lookup/removal adds three Rtl names
(42 kernel32 names total). Bounded manual RtlVirtualUnwind adds the 43rd name;
its supported frame/context contract is described below. The released
[make2compdb](make2compdb.md) adds GetCurrentDirectoryW/LocalFree (45 kernel32
names total) and a separate bounded shell32!CommandLineToArgvW bridge.

The existing [restricted kernel contracts](contracts.md) still apply: for example,
file sharing must be FILE_SHARE_READ|WRITE|DELETE, security/template/named objects
and overlapped I/O are unsupported. This milestone adds binary calling convention
bridges and explicit no-effect current-process/thread pseudo-handle CloseHandle
semantics. Other native kernel restrictions remain. Unix status supports 0..255;
a larger Windows ExitProcess/entry-return DWORD reports the unsupported status
rather than silently claiming its low byte is equivalent.

## Measured coverage and limits

The no-CRT executable checks all eight nonvolatile Win64 integer registers and
XMM6..15 across real imported calls, plus five/seven-argument stack calls,
UTF-16/surrogate filenames, a relocated absolute pointer, zero-filled storage,
10,000 last-error iterations, 9,999 TLS round trips and last-error clearing,
manual/auto-reset events, semaphore consumption/release, monotonic-counter
predicates, file seek/EOF/truncation/flush/readback and a failed null handle. Separate current-process/thread pseudo-handle closes
record both return values and unchanged last-error sentinels.
It writes 19 deterministic predicate/data words and a separate 12-byte Unicode-
named file. Raw handles, pointers, clocks and TLS indices are deliberately not
compared as if their implementation-specific values ought to match.

Alkyhol executes the payload normally and at a forced different base, checking the
same observations and actual relocation diagnostic. Host tests check every
truncated prefix of an independent PE layout, malformed imports/directories,
section overlap, W+X rejection and relocation kinds/overlap. Isolated child
processes verify real protection faults for header/code/read-only-data writes and
writable-data execution; allowed data writes and code execution pass. These host
assertions and ABI observations are distinct from native Windows observations.
Native Windows, hosted Mac and physical M4 executions agree on the identical
fixture at `77631c7` (PE workflow 42382929856). Strict-reference revision
`14c009d` then passes all three PE jobs and all 24 existing conformance jobs, with
a fresh matching physical M4 run. The full native report is frozen in
[`pe/reference/windows-v1.json`](../pe/reference/windows-v1.json); future pairs
must match its PE/source identities and observations. This deliberately detects
joint drift as well as mismatched hosts. Changing this fixture requires a new,
explicitly reviewed native evidence milestone, not silently updating expected
output. See [validation](validation.md) for executed revisions and CI results.

Static TLS/callbacks, general unwind/SEH, CFG/load config, delay/bound imports, system-DLL ordinal resolution,
Filesystem DLL discovery, process-global module loading, CLR, dynamic lookup, guest threads/callbacks, general COM vtable ABI
bridges, general Windows process structures and Windows CRT startup are not implemented in this
binary path. Unsupported metadata or imports fail before entry; unrecognized
instructions are **not** statically detected. Trusted code must not issue Windows
syscalls or access GS/TEB/PEB, throw exceptions or retain pointers after unmapping.
Source-ABI COM/Metal pointers require explicit Win64 adapters. The ID3DBlob
adapter below covers that interface; general binary graphics COM remains separate.

Game executables, assets and their inventories stay ignored/local. A static game
import inventory can prioritize API families but cannot establish dynamic lookup,
transitive dependencies, COM method usage, actual execution or game compatibility.
No game launch or rendering claim follows from this fixture. For the decompilation
pipeline, unchanged PE execution supplies an independent behavioral check before
recompilation through Alkyhol. Next useful coverage is automatic module lifecycle and
TEB/static TLS/callbacks, then CRT startup and Windows unwinding, followed by real
binary COM/graphics callers and redistributable sample/benchmark executables.

## Primary specifications

The implementation follows Microsoft's [PE/COFF image and relocation format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format)
, the [documented pseudo-handle close behavior](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getcurrentprocess)
and [x64 calling convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention).
Apple documents the native-host architecture in [Rosetta's translation environment](https://developer.apple.com/documentation/apple-silicon/about-the-rosetta-translation-environment).
Finite tests above establish only the operations actually exercised; these links
and the presence of a CI definition are not execution evidence.

## DLL export metadata foundation

`Image::parse_dll` can now inspect conventional AMD64 DLL layouts separately from
`Image::parse`'s executable contract. DLL entry zero is allowed for inspection;
a nonzero entry still needs initialized code. `Image::exports` reads the bounded
export directory, biased ordinals, sparse address holes, multiple public aliases,
ordinal-only entries, mapped code/data RVAs (including zero-filled data) and raw
ASCII forwarder strings. It follows Microsoft's
[export directory/address/name/ordinal tables](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#the-edata-section-image-only).
The independent synthetic fixtures are owned test layouts, not native Windows
observations or Microsoft sample binaries.

At most 65536 address slots/names and 4096-byte strings are accepted. Directory,
array, ordinal, name order/uniqueness, arithmetic and mapped-target bounds are
checked. A forwarder's terminating NUL must be inside the directory's declared
range, not merely somewhere later in the file. Forwarder text remains verbatim;
no dependency resolution, symbol grammar guessing or callable-function promise
is implied. Address exports can point to data, not just functions. Named missing
slots, unsupported absolute/out-of-image targets and malformed metadata fail.

This metadata API does not execute code. `LoadedImage::load` still accepts EXEs
only and normal `check_startup` rejects DLLs. An explicit isolated runtime API is
now available below; broader dependency support and scoped guest export lookup
are documented separately. Guest LoadLibrary/FreeLibrary, global module caching/
reference counts and Windows exceptions remain unavailable. The
existing EXE startup gates and both frozen native binary observers remain intact.
Six new host parser units exercise these rules, every truncated prefix, aliases,
ordinal-only/empty tables, zero-entry DLLs and malformed forwarder/table bounds.

## Owned DLL export reference probe

`tools/build_pe_export_fixture.py` builds the owned MIT-licensed
`pe/fixtures/exports/library.c` / `library.def` once to `alkyhol_exports.dll` with
open Clang/LLVM and no CRT. `inspect_exports` is a Rust read-only metadata CLI;
it never maps or enters the DLL. The expanded PE workflow transfers the identical
DLL and manifest to native Windows, where `tools/run_pe_exports.py` independently
queries the source-declared named/ordinal/data exports with actual GetProcAddress.
Forwarding is tested against the real kernel32 function address, not a successful
unknown placeholder. The paired common record compares actual target RVAs with
the Rust metadata, plus the known source-declared forwarder and ordinal/name map.

```sh
python3 tools/build_pe_export_fixture.py
python3 tools/run_pe_exports.py --dll local/pe/export-fixture/alkyhol_exports.dll --output local/pe/export-fixture/metadata.json
```

Windows additionally observes DllMain's BSS initialization, real exported-function
returns through Win64 ABI, initialized data, the result of a sparse-ordinal lookup
(without dereferencing it), and successful FreeLibrary. Those fields are labeled
`native_only`; Rust must report null there. The sparse ordinal is recorded without
guessing that Windows necessarily returns NULL. Native CI 42388553558 on
`c82d780` measured NULL for hole 19, successful attach/calls and forwarding;
Rust still performs metadata inspection only. Six common entries agree on
native Windows, hosted Rust and physical M4 metadata inspection. The native
report is frozen at `pe/reference/exports-windows-v1.json`; local parser/protocol
units remain separate evidence.
`tools/compare_pe_exports.py` requires distinct native and metadata-only backends,
identical DLL/source/revision identities, clean tracked trees and equal common
records. It also rejects invented host execution or missing native call results.
This establishes the native reference used by the separate isolated DLL runtime
below. The metadata observer itself continues to report no Rust DLL execution.

## Isolated trusted DLL runtime

On x86-64 macOS, `unsafe LoadedLibrary::load` now maps one trusted PE64 DLL,
applies DIR64 relocations and supported kernel32 IAT bindings, preflights all
forwarders, sets final W^X protections and calls the original DLL entry with
DLL_PROCESS_ATTACH and NULL reserved (dynamic-load semantics). Its case-sensitive
name and biased-ordinal lookups return raw code/data addresses owned by that
instance. Drop calls DLL_PROCESS_DETACH before releasing the mapping. Failed
ATTACH returns an error and receives exactly one DETACH; zero entry calls neither.
Images now reserve/trim owned VM space to a 64-KiB-aligned base without MAP_FIXED;
cleanup ownership is updated after each trim so a failure cannot unmap a reused
gap. The ordinary executable loader retains its startup gates.

This object represents an **isolated library instance**, not Windows' global
module manager. There is no same-file cache/reference count, DLL dependency graph,
loader-lock/reentrancy support, guest LoadLibrary/GetProcAddress/FreeLibrary export,
static TLS, SEH/unwind support or thread notification. Imports and forwarders may
only target the explicit builtin bridge whitelists. Unsupported dependencies,
ordinal forwarders and startup metadata fail before DllMain. Any required
ProcessContext must be installed and kept alive by the trusted embedding caller;
all exported pointer lifetimes end with their library/context. Guest instructions
remain trusted native code, not sandboxed or statically safety-verified.

The new `probe_library` binary executes the **unchanged frozen DLL** from the
native export reference, including its original C DllMain, relocated data pointer,
exported code/data and forwarded Win64 API. `tools/run_pe_library.py` verifies
exact DLL/source identity against that reference before entering it. Actual
lookup RVAs, initialized/BSS data and function results match the frozen native
Windows record on physical M4, normally and after forced rebase. A forwarder also
returns a real last-error sentinel through the Win64 bridge. The existing
metadata-only observer remains separate and unchanged.

```sh
python3 tools/run_pe_library.py --dll local/pe/export-fixture/alkyhol_exports.dll --output local/pe/library-runtime.json
```

Five independent host units use a tiny owned DLL entry that writes Win64 reason,
module/reserved arguments and call count into an external UnsafeCell. They check
ATTACH/DETACH order, failed-attach cleanup, zero entry, lookup/error cases and
pre-entry rejection. An isolated child uses actual Mach VM region queries before
and after Drop to verify the mapping is gone after DETACH. Dereferencing freed
pointers or expecting Linux mincore error semantics is not the cleanup oracle.
These host lifecycle assertions are distinct from the frozen Windows successful
load/call/free observations; full Windows unload side-effect parity is unmeasured.
The expanded PE workflow now compares actual Alkyhol DLL execution separately
from static metadata. Native Windows, hosted Mac and clean physical M4 actual
runtime records agree at `cf5be5e` (PE run 42391811360); all 27 jobs pass there.
General module loading and unmeasured lifecycle effects remain separate.

A [PE32 feasibility note](pe32.md) records the user's 32-bit question, primary
WoW64/Rosetta sources and observed current 4-GiB page-zero reservation. It is
research only; no Alkyhol 32-bit mode switch or runtime support is claimed.

## Failed DLL initialization probe

`pe/fixtures/failed-attach/library.c` is a separate owned no-CRT DLL that calls
actual imported GetStdHandle, WriteFile and SetLastError from DllMain. Its original
ATTACH returns FALSE. ATTACH and cleanup DETACH each write a 16-byte record to an
inherited pipe: reason, incremented zero-initialized counter, NULL-reserved check
and fixed magic. A volatile absolute counter pointer retains a DIR64 relocation.
Exact 32-byte output requires one ATTACH followed by one DETACH, with persistent
state and successful Win64 IAT calls; crashes, missing/reordered/duplicate records
and wrong bytes fail. This is additional negative-path evidence, not a change to
the frozen successful DLL or an implementation of guest LoadLibraryW.

```sh
python3 tools/build_pe_failed_attach.py
python3 tools/run_pe_failed_attach.py --dll local/pe/failed-attach/alkyhol_failed_attach.dll --output local/pe/failed-attach-alkyhol.json
```

Native Windows, hosted Mac and physical M4 observe the identical trace at
`2f88f1b` ([PE run 42393459700](https://github.com/daeken/Alkyhol/actions/runs/42393459700));
host normal and forced-rebase executions agree. The actual Windows report is
frozen at `pe/reference/failed-attach-windows-v1.json`. Future observers verify
exact DLL/source identity before entry and comparisons require that immutable
native trace/error reference, including protection against joint drift.
Last-error fields are deliberately backend-specific:
the Rust unsafe constructor returns an error string and retains the DLL's actual
SetLastError sentinel `0x1234`; native Windows LoadLibraryW returns NULL with
loader error `1114` (ERROR_DLL_INIT_FAILED).
Those interfaces have different contracts. No Win32 loader-error parity is
claimed by comparing the shared lifecycle trace. General module-manager,
dependencies, threads, TLS and SEH restrictions above still apply.

## Executables with explicit borrowed DLL dependencies

`LinkedImage::load(exe_bytes, rebase, dependencies)` maps an EXE against up to 64
already attached `LoadedLibrary` instances. It binds named and biased ordinal
code/data imports, including exports forwarded through those DLLs to other
already resolved targets. DLL names are restricted case-insensitive basenames;
symbol names remain case-sensitive. Builtin identities are reserved. Missing,
zero/sparse/system ordinals and unsupported startup still fail before EXE entry.
Loading never enters the EXE or initializes any DLL. The caller controls initial
attachment; errors do not claim to precede attachment of those borrowed DLLs.

`image()` borrows the checked mapping for the existing unsafe `enter()` API.
An explicit destructor unmaps the EXE while Rust still requires every dependency
owner to remain alive, including transitive owners. Paired compiler checks accept
correct teardown and reject prematurely dropping a borrowed DLL (E0505). No
filesystem discovery, global cache, guest LoadLibrary/GetProcAddress, static TLS,
SEH, automatic EXE graph loader or single-file multi-module packaging is added.

The new owned EXE imports Direct/Alias, Bss/Data, ordinal-21 data, NONAME ordinal-23
code and the kernel32-forwarded GetLastError from the **unchanged frozen** export
DLL. Actual PE thunks are independently checked. Its real instructions record
DLL-initialized BSS, function/ordinal results, actual named/ordinal data aliasing,
cross-DLL writes affecting later function results and restored data, then return
normally to the host. Integer addresses pass through volatile storage so a C
compiler cannot fold distinct extern object names as unequal instead of observing
the actual import targets. Both binary files are copied as checked snapshots into
a temporary directory for execution. Windows loads the actual DLL beside the
actual EXE; the Mac observer uses the explicit Rust linking API and drops the EXE
before detaching the DLL. This is binary execution of both files, separate from
source compilation or static instruction rewriting.

```sh
python3 tools/build_pe_linked_image.py
python3 tools/run_pe_linked_image.py --directory local/pe/linked-image --output local/pe/linked-image-alkyhol.json
```

Physical M4/Rosetta executes all 14 raw words normally and with both images
actually rebased. All three PE jobs pass on `c24f203`
([run](https://github.com/daeken/Alkyhol/actions/runs/42406635787)); downloaded clean
native/hosted reports and clean physical M4 execution at that same revision agree
on the identical EXE/DLL/sources, actual import thunks and all raw results.
`pe/reference/linked-image-windows-v1.json` freezes the complete native record;
drivers/comparisons require its exact identities/results by default.
Strict-reference revision `36e91cc` passes all 27 CI jobs; clean native/hosted
records and fresh physical M4 execution agree at that same revision, with both
host images actually rebased in the forced run. Exact CI evidence is recorded
in [validation](validation.md).
This finite owned EXE/DLL test does not establish game compatibility.

## Explicit borrowed DLL dependencies

`unsafe LinkedLibrary::load(bytes, rebase, dependencies)` extends the isolated
loader with a finite set of **already attached** DLL instances. Named IAT imports
and named export forwarders resolve to those actual mapped code/data addresses;
kernel32 still uses only its explicit Win64 ABI whitelist. All imports/forwarders
are preflighted before the new DLL's DllMain. Missing modules or symbols never
receive successful placeholders. The ordinary EXE and isolated constructors keep
rejecting foreign dependencies.

Dependencies have caller-assigned DLL basenames (at most 255 bytes), starting with
an ASCII letter/digit, containing only ASCII alphanumeric/`._-` and ending in
`.dll`. Module names match case-insensitively; symbols stay case-sensitive.
There are at most 64 names, with
no duplicate case aliases, path components or kernel32 override. Module-definition
LIBRARY text is not treated as a filesystem identity, consistent with Microsoft's
[LoadLibrary contract](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibraryw).
The owner borrows each LoadedLibrary for its lifetime and drops its own DLL before
releasing those borrows. `library()` exposes lookups or a borrow for another
explicit dependent DLL, retaining transitive lifetimes. The embedding caller loads
the acyclic dependency order. No filesystem search, automatic graph discovery,
global cache/refcount, guest LoadLibrary/GetProcAddress, system-DLL ordinal resolution,
loader-lock/reentrancy, TLS, thread or SEH support is implied.

The new owned `pe/fixtures/dependent` consumer is linked once against the **unchanged
frozen** `alkyhol_exports.dll`. Its original C DllMain and Evaluate function import
code and data from that second DLL; ReadError calls an imported forwarded API.
Two exports additionally forward directly and transitively into that dependency.
Each ATTACH/DETACH writes a 24-byte trace containing reason, persistent counter,
NULL-reserved flag, dependency function result and initialized/BSS data. The
dependency must remain initialized throughout consumer DETACH. Actual call
results, forwarded target identities and all 48 trace bytes agree between normal
and forced-rebase physical M4 executions; both DLLs report relocation in the
forced case. Three synthetic host units separately cover IAT/forwarder addresses,
transitive lifetimes, pre-entry failures and failed-parent cleanup.

```sh
python3 tools/build_pe_dependent.py
python3 tools/run_pe_dependent.py --directory local/pe/dependent --output local/pe/dependent-alkyhol.json
```

The existing PE workflow now transfers the identical two DLLs to Windows, where
actual LoadLibraryExW performs its own dependency resolution and initialization.
All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42395051128) pass
on `1ab6e23`: native Windows and hosted Mac agree on actual cross-DLL calls,
forwarded target equality and the exact lifecycle trace. The native report is
frozen at `pe/reference/dependent-windows-v1.json`; future drivers verify both
DLL/source identities before entry and comparisons require its immutable results.
The driver uses DLL_LOAD_DIR|DEFAULT_DIRS solely for
the explicit native fixture directory; Alkyhol's API adds no Windows search policy.

A subsequent compile-only regression caught that reference-containing container
fields alone allowed Rust to end a dependency borrow before implicit consumer
destruction. LinkedLibrary now has an explicit destructor that detaches/unmaps
its owned DLL before releasing dependencies. Paired compiler checks accept correct
teardown and reject early dependency destruction with E0505, even when the
consumer has no further explicit use. These are lifetime checks, separate from
executed DLL behavior; see [validation](validation.md) for exact revisions.
The final `9b766e5` implementation passes all 27 CI jobs, with a clean physical
M4 run matching the same-revision Windows/hosted records and frozen reference.

## Explicit ordinal imports and forwarders

`Import.symbol` now uses `ImportSymbol::Name(String)` or `Ordinal(u16)`, retaining
the PE32+ high-bit distinction instead of encoding ordinals as guessed names.
The parser checks reserved thunk bits, low-16-bit ordinals, backed named hints,
ordered/disjoint IAT slots and normal termination. Literal named `#23` remains a
name. Zero ordinals are retained as metadata; runtime resolution rejects them.
The implementation follows Microsoft's [import lookup table and export forwarder
format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#the-idata-section).

LinkedLibrary resolves ordinals 1..65535 only from explicitly supplied attached
DLLs, including exported data and NONAME code. Forwarders such as `module.#23`
use checked decimal syntax and the same biased ordinal lookup. Empty, signed,
hexadecimal, overflowing or zero ordinal forwarders fail. Missing/sparse slots
fail before consumer DllMain. That policy is not a claim of full Windows sparse-
ordinal error behavior. System DLL ordinals remain unsupported: no per-Windows-
build kernel32 ordinal map is invented. Isolated/default executable loading,
ownership, startup gates and named-kernel32 whitelist remain as documented above.

The owned `pe/fixtures/ordinal` consumer imports named Direct plus ordinal 21 data
and ordinal 23 NONAME code from the **unchanged frozen** export DLL. Its DllMain
uses ordinal code to initialize state, Evaluate mixes named/code/data imports,
and three exports forward by ordinal to code/data. Independent Python inventory
and the Rust parser both check the actual LLVM-produced mixed thunks. Physical
M4 normal/forced-rebase runs agree on all seven actual call/data/target fields;
both DLLs relocate in the forced case. Native Windows, hosted Mac and clean
physical M4 records agree at `150ca60` ([PE run 42397230715](https://github.com/daeken/Alkyhol/actions/runs/42397230715)).
The full native report is frozen at `pe/reference/ordinal-windows-v1.json`;
drivers require the exact frozen DLL/source/import identities before entry and
comparisons require its native execution results, rejecting joint drift too.
See [validation](validation.md) for exact executed revisions.
The strict `b083926` revision passes all 27 CI jobs, with a clean physical M4
record matching the same-revision native Windows/hosted records and frozen input.

```sh
python3 tools/build_pe_ordinal.py
python3 tools/run_pe_ordinal.py --directory local/pe/ordinal --output local/pe/ordinal-alkyhol.json
```

No general module manager, TEB/static TLS, unwind/SEH, binary COM or game
compatibility follows from these finite owned-DLL observations. The previous
frozen native executable, DLL and released-program comparisons remain intact.

## Closed DLL bundles

`unsafe LibrarySet::load(inputs, rebase)` accepts 1..64 explicitly supplied DLL
byte snapshots in any order. It checks the complete import/forwarder graph,
symbol existence, startup metadata, relocations and IAT/fixup separation before
the first initializer. Acyclic dependencies load first, shared DLLs load once per
normalized supplied basename, and consumers detach before their dependencies.
`get(name)` borrows an attached DLL without allowing individual unload. Mapping
or ATTACH failure rolls back attached DLLs in reverse order; external initializer
side effects are not undone. Invalid/cyclic/self graphs, missing modules/symbols,
unsupported builtins and known metadata failures never enter any initializer.

All supplied modules participate, including otherwise unreferenced DLLs. The
existing restricted filename, named/ordinal resolution and trusted-code/context
contracts apply. No filesystem search, process-global cache/refcounts, guest
LoadLibrary/GetProcAddress, static TLS, SEH, threads or reentrant loader callbacks
are added. This is an automatic closed-bundle lifecycle API, not a general Windows
module manager or executable-process startup implementation.

The new observer supplies the two **unchanged frozen** named/ordinal consumer DLLs
before their shared, unchanged export DLL. The loader orders them and owns their
lifetimes. Physical M4 normal/forced-rebase runs preserve both earlier call/data/
forwarder records, observe actual equality of both consumers' leaf targets and
retain the exact original 48-byte C ATTACH/DETACH trace. All three DLLs relocate
in the forced run. Four independent synthetic runtime units additionally check
a shared-leaf diamond, pre-entry errors, failed-attach rollback and a real mapping
collision that rolls back an attached dependency without replacing its mapping.
Paired compiler checks retain consumer borrows of DLLs owned by a set.

```sh
python3 tools/build_pe_library_set.py
python3 tools/run_pe_library_set.py --directory local/pe/library-set --output local/pe/library-set-alkyhol.json
```

The expanded PE workflow transfers the identical three DLLs to Windows. Its
observer keeps two real LoadLibraryEx handles, reuses the frozen native call
observers against those cached instances, compares shared targets, then frees
consumers in reverse order. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42399759919)
pass at `1e028b8`: clean native Windows, hosted Mac and physical M4 records agree
on the unchanged three DLLs, both earlier result sets, shared target identity and
all 48 C notification bytes. The full native report is frozen at
`pe/reference/library-set-windows-v1.json`; drivers and comparators require its
input identities and actual results. See [validation](validation.md).
Native shared-leaf notification counts are not
measured by these DLLs; the once-only diamond count is host-unit evidence.
The strict `08aaf02` revision passes all 27 CI jobs, with a clean physical M4 run
matching the same-revision native Windows/hosted records and frozen reference.

## Binary COM: ID3DBlob

The d3dcompiler_47.dll provider exposes exactly D3DCreateBlob. It creates a real
owned Rust COM blob and wraps its native interface in a separate five-slot
**Win64** vtable: QueryInterface, AddRef, Release, GetBufferPointer and GetBufferSize.
Native source-ABI interface pointers never escape through a guest QueryInterface;
successful queries retain the guest identity and acquire the underlying reference.
The final Release frees the native object/data and its exclusive wrapper once.
Allocation, data access and atomic reference semantics reuse `src/com.rs`; source
C++ vtables and callers retain their existing native ABI.

The builtin DLL identities are reserved across isolated/dependent/bundle
loading. Unknown compiler exports, wrong DLL versions/names and system ordinals
fail preflight. Shader compilation retains its existing open toolchain. macOS
does not load a Windows compiler DLL to implement this memory interface.

An owned no-CRT PE64 fixture imports D3DCreateBlob and performs actual virtual
calls through every slot, queries IUnknown/ID3DBlob aliases, checks canonical
IUnknown identity and shared data, records unsupported-IID HRESULT/output clearing,
balances reference transitions through final release and verifies 37/16384-byte
buffer contents before/after alias release. It writes every byte before reading;
native zero initialization is not assumed. The exact executable is transferred
to Windows, where the OS's d3dcompiler_47 implementation is the independent peer.
All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42402557855) pass
on `ab08287`: clean native Windows, hosted Mac and physical M4 records agree on
the identical executable/source and all 23 output words. The complete native
record is frozen at `pe/reference/binary-com-windows-v1.json`; drivers verify its
PE/source identities before execution, and comparisons require its actual results.
See [validation](validation.md) for exact executed revisions.
Strict revision `1e47b28` passes all 27 CI jobs, with a clean physical M4 record
matching the same-revision native Windows/hosted reports and frozen reference.

```sh
python3 tools/build_pe_binary_com.py
python3 tools/run_pe_binary_com.py --fixture local/pe/binary-com/binary-com.exe --output local/pe/binary-com-alkyhol.json
```

Physical M4 executes all 23 output words normally and after forced rebase; the
Mach-O's extracted PE payload hash equals the original file. Separate host units
cover null-output/null-IID, zero-size and impossible-allocation behavior. Those
edge cases and stronger atomic ownership are host contracts, not new native
Windows observations. General COM activation/marshaling, guest threads, binary
D3D12 devices and game compatibility remain separate milestones.

Primary contracts: [D3DCreateBlob](https://learn.microsoft.com/en-us/windows/win32/api/d3dcompiler/nf-d3dcompiler-d3dcreateblob),
[ID3D10Blob/ID3DBlob inheritance](https://learn.microsoft.com/en-us/windows/win32/api/d3dcommon/nn-d3dcommon-id3d10blob)
and [IUnknown::QueryInterface](https://learn.microsoft.com/en-us/windows/win32/api/unknwn/nf-unknwn-iunknown-queryinterface%28refiid_void%29).

## Binary D3D12 root serialization

The d3d12.dll provider exposes exactly D3D12SerializeRootSignature and
D3D12SerializeVersionedRootSignature. Both take real Windows x64 structure layouts,
including nested pointer arrays, and call the existing open Rust version-1.0
[serializer](../cpp/root-signatures.md). Success and diagnostic blobs each acquire
an exclusive Win64 wrapper; no native-ABI vtable is published. Optional error
outputs, failure clearing and reference ownership remain explicit. Version 1.1/1.2,
device creation, deserialization and other d3d12 exports remain unsupported.

The separate owned no-CRT fixture calls both exports from unchanged PE machine
code for the six existing frozen SDK layouts: empty, 64 constants, seven root
descriptors, offset/APPEND tables, static samplers and stage-disjoint bindings.
It reads and compares both returned containers, queries their COM identity and
balances actual virtual releases. Its 1180-byte common prefix includes all 980
existing native SDK container bytes and actual COM/HRESULT predicates. A separate
36-byte profile preserves a known difference: Windows successfully serializes
65 constants, while the existing bounded Rust serializer rejects them with an
owned NUL-terminated diagnostic and cleared success output. Each distinct profile
is checked exactly; this case is not claimed equivalent. Diagnostic spelling is
backend-specific and is not compared. Neither result is a GPU validity test.
No GPU/device or Windows compiler binary is used on macOS.

```sh
python3 tools/build_pe_root_signatures.py
python3 tools/run_pe_root_signatures.py --fixture local/pe/root-signatures/root-signatures.exe --output local/pe/root-signatures-alkyhol.json
```

Physical M4/Rosetta normal/forced-rebase execution passes, including the exact
input/extracted PE SHA-256 check. The PE workflow transfers that identical file
to native Windows and compares actual observations. All three PE jobs pass on
`b882623` ([run](https://github.com/daeken/Alkyhol/actions/runs/42405009703)); clean
Windows/hosted records and clean physical M4 execution agree on the supported
prefix and each distinct restriction profile. The complete native record is
frozen at `pe/reference/root-signatures-windows-v1.json` and required by default,
including its exact PE/source identities. Strict-reference revision `16c8d50`
passes all 27 CI jobs; clean native/hosted records and fresh physical M4 execution
agree at that same revision, including both restriction profiles. See [validation](validation.md)
for exact native execution evidence and the known restriction; source-ABI SDK
bytes alone do not establish native execution of this PE. This CPU milestone does not
add binary D3D12 device execution or game compatibility.

Primary contracts: [D3D12SerializeRootSignature](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-d3d12serializerootsignature)
and [D3D12SerializeVersionedRootSignature](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-d3d12serializeversionedrootsignature).

## Scoped guest module and export lookup

`ModuleContext::install(&[(name, &dll), ...])` exposes 1..64 already attached
explicit DLLs to real Win64 GetModuleHandleW/GetProcAddress bridges on this thread.
It snapshots their resolved export metadata and retains dependency-owner borrows
through teardown. The thread-bound guard rejects nested installation, duplicate
case-insensitive names, duplicate actual image bases and builtin replacement.
Installing/dropping a view neither initializes/detaches DLLs nor increments a
module reference. Paired compiler checks reject dropping an owner before its view.

GetModuleHandleW accepts case-insensitive ASCII DLL basenames, including omitted
`.dll` and trailing-period handling. It returns the **actual full-width PE image
base**, not a kernel handle token. GetProcAddress accepts that exact registered
base and a bounded case-sensitive ASCII name or low-word ordinal. Named aliases,
NONAME exports, data and already resolved forwarders preserve their actual target
addresses. Missing modules return NULL/ERROR_MOD_NOT_FOUND; missing symbols return
NULL/ERROR_PROC_NOT_FOUND. Calls without a view, main-image NULL lookup, builtin
module-image lookup, paths and Unicode module identities fail honestly with
ERROR_NOT_SUPPORTED. Host invalid-handle/string/ordinal edge contracts are tested
separately and are not inferred to match every native Windows edge behavior.

This is an explicit borrowed lookup view, not a guest LoadLibrary/FreeLibrary
implementation, process-global loader/cache, filesystem search or general PEB/TEB.
DLL attach/detach occur outside this view; lookup from those callbacks is not
provided. Guest threads/reentrancy, ANSI lookup and implicit main/builtin module
images remain unsupported. Already returned raw pointers may not outlive their
module owners or the trusted runtime contract.

The owned EXE retains one static Direct import to force native DLL attachment,
then invokes the real guest lookup APIs. It checks an actual MZ header at the
module base, case/extension handling, name/ordinal/alias targets, initialized BSS,
shared writable data, forwarded calls and missing-name/module error results.
Both the unchanged EXE and the earlier frozen DLL run as checked snapshots.

```sh
python3 tools/build_pe_dynamic_lookup.py
python3 tools/run_pe_dynamic_lookup.py --directory local/pe/dynamic-lookup --output local/pe/dynamic-lookup-alkyhol.json
```

Physical M4/Rosetta passes all 21 raw words normally and with both images rebased.
All three PE jobs pass on `99bce97`
([run](https://github.com/daeken/Alkyhol/actions/runs/42408269870)); clean native/
hosted reports and clean physical M4 execution at that same revision agree on
the identical EXE/DLL/source/import identities and all raw results. The complete
native record is frozen at `pe/reference/dynamic-lookup-windows-v1.json` and is
required by default, including its exact identities. Strict-reference revision
`1374b0b` passes all 27 CI jobs; downloaded clean native/hosted records and fresh
physical M4 execution agree at that same revision, including both actual rebases.
Exact CI evidence is tracked in [validation](validation.md). These are finite owned
binary tests, not general Windows loader or game compatibility claims.

Primary contracts: [GetModuleHandleW](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulehandlew)
and [GetProcAddress](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getprocaddress).

## Owned memory protection and guest-generated code

VirtualAlloc still accepts NULL-address MEM_RESERVE|MEM_COMMIT/PAGE_READWRITE,
bounded to 512 MiB, page-rounded and 64-KiB aligned in a checked 4-KiB x86 process.
Each reservation now tracks its per-page protection. Its temporary mapping owns
the remaining reservation after each successful alignment trim, so failed trims
cannot leak ownership or blindly unmap a region already released.

VirtualProtect changes real Darwin page permissions for a nonempty byte range
entirely inside one owned allocation, rounding both boundaries to affected pages.
It supports PAGE_NOACCESS, READONLY, READWRITE and EXECUTE_READ; old protection is
that of the first affected page. Cross-reservation/unowned ranges, RWX, guard and
other modifiers fail honestly. This does not yet protect arbitrary PE image or
host heap regions. VirtualFree supports exact-base, size-zero MEM_RELEASE and
consumes ownership only after successful unmapping. Decommit/reserve-only and
partial release are not implemented; unreleased mappings still die with context.

FlushInstructionCache accepts the current-process pseudo-handle and an explicit
nonempty readable range in one owned allocation, invoking Darwin's public
sys_icache_invalidate operation. Whole-process NULL flushes, other processes and
unowned/no-access ranges fail explicitly. The platform documents coherent-cache
behavior on physical x86. Executing rewritten code after protection/cache calls
establishes the combined result; it does not isolate which operation invalidated
Rosetta's translation cache or claim that a standalone flush caused the change.

An owned unchanged no-CRT PE emits the actual Win64 bytes
`b8 44 33 22 11 31 c8 01 d0 c3` (MOV immediate, XOR ECX, ADD EDX, RET), changes its
page to executable/readable, flushes and calls it with real Windows arguments.
It then returns the page to writable, rewrites the immediate to `0xfedcba98`,
restores execute/read, flushes and observes a different actual result. It also
checks neighboring data-page independence, a two-byte range straddling pages,
first-page old protection, no-access/restoration and release. No original sample,
shader, geometry or game binary is modified.

```sh
python3 tools/build_pe_generated_code.py
python3 tools/run_pe_generated_code.py --fixture local/pe/generated-code/generated-code.exe --output local/pe/generated-code-alkyhol.json
```

Physical M4/Rosetta normally and forcibly rebased executes all 29 raw words.
All three PE jobs pass on `2444dd5`
([run](https://github.com/daeken/Alkyhol/actions/runs/42410651433)); clean native/
hosted reports and fresh clean physical M4 execution at that same revision agree
on the identical executed PE/source identity and all raw observations. The full
native record is frozen at `pe/reference/generated-code-windows-v1.json` and
required by default, including its exact input identity. Strict-reference
revision `7c6bf36` passes all 27 CI jobs; downloaded clean Windows/hosted records
and fresh physical M4 execution agree at that same revision on all 29 raw words.
Host-only child tests independently observe faults for write-to-RX, execute-from-
RW, read-from-NOACCESS and write-to-READONLY, and query the actual Mach VM map to
verify release without dereferencing freed storage. Native results and exact
revisions are recorded in [validation](validation.md). This is finite generated-
code/memory API evidence, not general JIT, SEH or game compatibility.

Primary contracts: [VirtualProtect](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualprotect),
[VirtualFree](https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualfree)
and [FlushInstructionCache](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-flushinstructioncache).

## Explicit dynamic function-table metadata

RtlAddFunctionTable, RtlLookupFunctionEntry and RtlDeleteFunctionTable now maintain
an explicit dynamic AMD64 table list in the current ProcessContext. The ABI uses
the real one-byte BOOLEAN returns and 12-byte, DWORD-aligned RUNTIME_FUNCTION
records. Lookup returns pointers to the **original caller-owned records** and the
full 64-bit base. Functions use half-open address ranges; gaps/end boundaries miss.
Deletion removes only the exact registered array, leaves other tables available
and permits re-registration. Context teardown clears snapshots before its owned
code allocations are unmapped; it never frees a caller's record array.

The finite subset accepts 1..65536 sorted nonoverlapping direct entries per call,
at most 65536 entries/64 tables in total and disjoint table address envelopes.
Unwind RVAs must be nonzero/DWORD-aligned; indirect entries and overlapping table
envelopes fail explicitly. Records and code/unwind storage must stay valid and
immutable until deletion. Registration snapshots metadata; it does not interpret
or validate unwind recipe bytes, register Mach-O stack frames, unwind registers
or dispatch Windows exceptions. Manual RtlVirtualUnwind support is described below;
RtlUnwindEx, callback/growable tables and static `.pdata` startup remain unavailable. Existing loader startup
gates are preserved. Guest threads/reentrant callbacks remain unsupported.

Lookup currently requires a nonnull image-base output and NULL history pointer.
Unsupported lookup context/history/output forms terminate with an explicit status
78 diagnostic rather than being reported as a successful leaf miss. BOOLEAN APIs
fail honestly for unsupported registration contexts/shapes. Host invalid-input
errors and image-base output on misses are separate from native evidence.

The owned unchanged PE emits three actual Win64 leaf functions in RX memory,
provides a minimal version-1 unwind header, registers two arrays and queries their
exact entries, base, code/end/gap boundaries and independent lifetimes. It calls
all three functions through their lookup-resolved base/RVA targets, then deletes,
re-registers and releases code. No exception is raised and no frame is unwound.

```sh
python3 tools/build_pe_function_tables.py
python3 tools/run_pe_function_tables.py --fixture local/pe/function-tables/function-tables.exe --output local/pe/function-tables-alkyhol.json
```

Physical M4/Rosetta normal/forced-rebase execution agrees on all 37 raw words.
All three PE jobs pass on `d86fd18`
([run](https://github.com/daeken/Alkyhol/actions/runs/42412363341)); downloaded clean
native/hosted records and clean physical M4 execution at that same revision agree
on the identical PE/source identity and all results. The full native record is
frozen at `pe/reference/function-tables-windows-v1.json` and required by default,
including its exact input identity. Strict-reference revision `253685a` passes
all 27 CI jobs; downloaded clean native/hosted reports and fresh physical M4
execution agree at that same revision on all 37 raw words. No live unwinding
was exercised.
Native evidence and executed revisions are recorded in [validation](validation.md).
This is finite function-table API evidence, not exception or game compatibility.

Primary contracts: [RtlAddFunctionTable](https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtladdfunctiontable),
[RtlLookupFunctionEntry](https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtllookupfunctionentry),
[RtlDeleteFunctionTable](https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtldeletefunctiontable)
and [AMD64 function/unwind record layout](https://learn.microsoft.com/en-us/cpp/build/exception-handling-x64).


## Bounded manual AMD64 frame unwinding

RtlVirtualUnwind now restores caller fields in a supplied AMD64 CONTEXT for an
explicitly registered dynamic entry. This is a manual metadata/stack operation;
it does not resume execution, dispatch exceptions or implement throw/catch.
The Win64 bridge accepts all eight real arguments, including four stack arguments.
The finite contract requires HandlerType zero, NULL ContextPointers, a 16-byte
aligned CONTEXT, valid caller-owned stack/recipe storage, and matching registered
entry/base/control address. Only a NOP control point in the completed function
body is accepted, so unsupported prologue/epilogue semantics fail explicitly.

Version-1 flags-zero, frame-register-zero recipes support PUSH_NONVOL for the
eight AMD64 nonvolatile integer registers, ALLOC_SMALL and both ALLOC_LARGE
operand forms. Code offsets must descend within the prologue, allocations must
precede register pops during restoration, and the complete stack frame is bounded
to 512 MiB. Unknown operations, handlers, chaining, frame pointers, context-pointer
outputs and other control points terminate with diagnostic/status 78. No missing
operation returns a fake successful no-handler result. Explicit integer/vector
slot-save support is described below. Static `.pdata`, RtlUnwindEx, SEH dispatch
and actual stack resumption remain separate work.

The implementation reads only required fields, plans restorations before changing
CONTEXT, restores saved GPRs/RSP/RIP, and writes the original body RSP as the
no-frame-register establisher frame. HandlerData is left untouched; its no-handler
value has no native parity claim. Caller pointers are trusted native addresses,
not a protected guest address space.

```sh
python3 tools/build_pe_virtual_unwind.py
python3 tools/run_pe_virtual_unwind.py --fixture local/pe/virtual-unwind/virtual-unwind.exe --output local/pe/virtual-unwind-alkyhol.json
```

The owned unchanged probe manually captures required fields of three actual active
Win64 assembly frames: RBX/R12 pushes plus a 40-byte allocation; RBP plus a 768-byte
scaled large allocation; and a leaf. It independently reads saved stack values
and the real return address, verifies caller-context restoration and preservation
of unrelated seeded bytes, then returns normally through the original stack.
This is not a full RtlCaptureContext implementation. The unscaled ALLOC_LARGE form
has host tests only. Native Windows SDK layout assertions compile separately;
Windows executes the identical PE rather than rebuilding it. Native results and
frozen input identities are recorded in [validation](validation.md). All 34 raw
words agree on native Windows, hosted Mac and physical M4 at `5d92635`; the native
record at `pe/reference/virtual-unwind-windows-v1.json` is required by default. Strict-reference
revision `fb8fe7a` passes all 27 CI jobs; downloaded clean native/hosted reports
and clean physical M4 execution at that same revision agree on all 34 words.

Primary contracts: [RtlVirtualUnwind](https://learn.microsoft.com/en-us/windows/win32/api/winnt/nf-winnt-rtlvirtualunwind),
[AMD64 CONTEXT](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-context)
and [AMD64 unwind operations](https://learn.microsoft.com/en-us/cpp/build/exception-handling-x64).


## AMD64 integer and vector save slots

The same bounded manual RtlVirtualUnwind now accepts SAVE_NONVOL/SAVE_NONVOL_FAR
and SAVE_XMM128/SAVE_XMM128_FAR. The offset scale is eight bytes for near integer
saves, sixteen for near vectors, and unscaled bytes for both far forms. Restores
copy all saved bits into the actual AMD64 CONTEXT GPR/XMM slots. Saves do not
advance RSP; fixed allocations and pushes continue to restore it independently.

This subset accepts the eight nonvolatile integer registers and XMM6..15, with
aligned nonoverlapping slots wholly inside the fixed allocation. All slot saves
must precede allocation/pop actions in the unwind sequence; duplicate save targets
and other ordering fail honestly. Caller home-space saves outside the fixed
allocation remain unsupported. Version/flags/frame-register/NOP-body/registration
restrictions remain. Stack extents and alignment are checked before each read,
and context writes are committed only after the entire recipe has been decoded
and its required stack data read successfully. These checks do not protect against
arbitrary invalid native pointers supplied by trusted input code.

```sh
python3 tools/build_pe_unwind_saves.py
python3 tools/run_pe_unwind_saves.py --fixture local/pe/unwind-saves/unwind-saves.exe --output local/pe/unwind-saves-alkyhol.json
```

A separate owned PE leaves all older frozen fixtures unchanged. Three real active
frames save/clobber/restore RBX/R12, XMM6/XMM15, and both groups with far offsets.
The far frame has a 1,048,696-byte allocation, integer saves at 524,288/524,304,
and vector saves at 1,048,576/1,048,608; it exercises unscaled ALLOC_LARGE as well.
It probes stack pages sequentially and reserves a four-MiB Windows stack. Context
checks independently read real saved stack bytes and the return address, verify
all restored/unrelated bytes, and return normally through the actual stack.
No exception is dispatched or resumed; x87/AVX state and full capture remain absent.
The separate Windows SDK probe asserts all ten XMM slot offsets. Native execution
and finite host coverage are recorded in [validation](validation.md).

Native Windows, hosted Mac and physical M4 agree on all 37 raw words at `8b67924`.
The actual native record at `pe/reference/unwind-saves-windows-v1.json` is frozen
and required by default; native cases exercise RBX/R12 and XMM6/XMM15. The remaining
supported register numbers have host tests, not independent native observations.

Strict-reference revision `9a4bc97` passes all 27 CI jobs. Downloaded clean native/
hosted records and fresh clean physical M4 execution at that same revision agree
on all 37 words and the frozen executed PE/source identity. See the executed
[validation record](validation.md) for CI links, counts and precise coverage.
