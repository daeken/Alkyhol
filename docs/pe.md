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
view. No Windows PEB/TEB is installed. The first [released off-the-shelf program,
u-config](u-config.md), runs unchanged with real package queries.

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

Imports are resolved by exact symbol and case-insensitive **kernel32.dll** identity
only. No arbitrary `dlsym` of native exports, name-only DLL matching, fallback
libraries or successful unknown stubs occur. The compiler-generated
`extern "win64"` wrappers preserve the Windows argument/return and nonvolatile
register contract while invoking Alkyhol's native exports. `CreateFileW` receives
16-bit UTF-16, including a surrogate pair; source compatibility's 32-bit wchar_t
adapters are not used. Current mapped guest addresses are actual native virtual
addresses in this deliberately shared process. This is not a pointer conversion
scheme for another address space and is not a memory safety sandbox.

The original PE fixture exercises these 24 bridges; the [released u-config
program](u-config.md) adds ten process-service names (34 explicit names total):

| Area | Imports |
| --- | --- |
| Error / lifetime | GetLastError, SetLastError, CloseHandle, ExitProcess |
| Synchronous files | CreateFileW, ReadFile, WriteFile, GetFileSizeEx, SetFilePointerEx, SetEndOfFile, FlushFileBuffers |
| Events | CreateEventW, SetEvent, ResetEvent, WaitForSingleObject |
| Semaphores | CreateSemaphoreW, ReleaseSemaphore |
| Timing | QueryPerformanceCounter, QueryPerformanceFrequency, Sleep |
| Dynamic TLS slots | TlsAlloc, TlsFree, TlsGetValue, TlsSetValue |

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

Static TLS/callbacks, unwind/SEH, CFG/load config, delay/bound/ordinal imports,
DLL startup/loading, CLR, dynamic lookup, guest threads/callbacks, COM vtable ABI
bridges, general Windows process structures and Windows CRT startup are not implemented in this
binary path. Unsupported metadata or imports fail before entry; unrecognized
instructions are **not** statically detected. Trusted code must not issue Windows
syscalls or access GS/TEB/PEB, throw exceptions or retain pointers after unmapping.
The source-ABI COM/Metal runtime cannot be passed directly to guest Win64 COM
callers; its vtables also need explicit binary ABI bridges.

Game executables, assets and their inventories stay ignored/local. A static game
import inventory can prioritize API families but cannot establish dynamic lookup,
transitive dependencies, COM method usage, actual execution or game compatibility.
No game launch or rendering claim follows from this fixture. For the decompilation
pipeline, unchanged PE execution supplies an independent behavioral check before
recompilation through Alkyhol. Next useful coverage is DLL/import lifecycle and
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

This is metadata support only. `LoadedImage::load` still accepts EXEs only, and
`check_startup` explicitly rejects DLL lifecycle even for a DLL without exports.
No DllMain, dependent-library initialization, LoadLibrary/GetProcAddress/FreeLibrary,
reference-counting, unload or Windows exception behavior is provided yet. The
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
guessing that Windows necessarily returns NULL. Native execution is pending the
initial workflow run; local parser/protocol units are not native evidence.
`tools/compare_pe_exports.py` requires distinct native and metadata-only backends,
identical DLL/source/revision identities, clean tracked trees and equal common
records. It also rejects invented host execution or missing native call results.
This prepares a DLL lifecycle reference; Rust DLL execution remains unsupported.
