# Executed validation

Current graphics development uses direct Metal from Rust. The historical records
below describe their exact revisions, including the retired wgpu/Vulkan backend.
See the latest direct-Metal record for current implementation evidence.

## PE32 FS/TEB: native observation pending

A new original MIT no-CRT PE32 probe reads FS-relative self/stack fields and
checks direct FS last-error reads/writes against real GetLastError/SetLastError
calls, including 10,000 transitions. It emits 16 raw DWORDs; no game code/data is
involved. Its initial native Windows execution is pending. This capture revision
does not change Alkyhol's CPU gates, FS state, startup gates or TLS behavior.
Collector tests separately reject byte-length/status/stderr failures and input/
source drift. A supported FS/TEB subset is a prerequisite for later static TLS,
not evidence that TLS callbacks, CRT or the game already work.

## PE32 executable self exports: native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `7b7dcff`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42435424200) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42435424012).
Downloaded clean native Windows/hosted Mac reports and fresh clean physical M4
execution at that same revision agree on all 26 raw DWORDs and frozen source/
executed PE identities. Both strict paired comparisons pass, including actual
normal/forced-rebase host execution. Existing frozen PE32/PE64, released binaries,
DLL/source/SDK/COM, shader and Metal comparisons remain green. This is self-export
API/ABI evidence; no game code executes.

Native Windows at `272fb60` executes the original owned PE32 self-export observer
and returns all 26 raw DWORDs. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42434858915) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42434858901) pass
on that capture revision, before the new Alkyhol self-export API comparison.
The complete downloaded native report is frozen unchanged at
`pe/reference/pe32-exports-windows-v1.json`. Exact C/.def source and executed binary
hashes are required by default. The existing PE32 and other frozen corpora stay
unchanged; no game source/data is included.

Actual GetModuleHandleA/W(NULL) calls return the current executable image base
without changing the probe's previous last error. GetProcAddress returns matching
named/ordinal/alias pointers; returned stdcall function pointers execute the
expected arithmetic, and returned initialized/BSS pointers permit real reads and
a BSS write. An ordinal-only function is not visible by its private name. Missing/
case-mismatched names yield NULL/error 127, while the tested below/above-range
ordinals yield NULL/error 182. Successful queried lookups preserve last error.
These are observed finite native results, not an exhaustive Win32 API claim.

Alkyhol now supplies those three additional PE32 import gates for current-executable
lookup and validated direct self exports. Bound ASCII names are read only through
checked readable guest storage. Metadata must be initialized readonly non-executable
section data without loader writes; sparse/forwarded tables and every other
unsupported startup feature remain rejected. Named module lookup, non-self module
handles and export queries on absent tables fail honestly. AMD64 EXE startup gates
and the original owned PE32 bytes remain unchanged.

Physical M4 development execution agrees on all 26 output words in normal and
forced-rebase runs. Strict same-clean-revision CI/physical comparison passes at
`7b7dcff` as recorded above.
The comparator requires the frozen native profile, exact inputs/sources/revision,
raw bytes and decoded words, actual relocation and separate diagnostic contracts;
crashes, changed results, missing references and joint input drift cannot pass.
All 102 PE host tests/12 compiler checks, 35 portable tests, 274 Python tests and
Clippy pass locally. Host/portable assertions additionally enforce stable metadata,
loader-write overlap, range/string bounds, ordinal bias and remaining startup gates.
The unchanged local game clears its export preflight and reaches a later unsupported
startup feature before entry; no game code executed. See [PE32](pe32.md).

## PE32 certificate metadata loading: native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `70ef3d4`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42434197943) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42434197923).
Downloaded clean native Windows/hosted Mac reports and fresh clean physical M4
execution at that same revision agree on all eight exact inputs/sources and every
output byte, including normal/forced-rebase host execution. Both strict paired
comparisons pass against the frozen native record. All existing binary/source,
application/DLL, SDK/COM, shader and Metal comparisons remain green. The unchanged
local game clears its next file-range gate but still stops before entry.

Native Windows execution at `8a9e0fb` accepts all eight owned certificate-directory
variants and returns the unchanged observer's eight DWORDs in every case. All
three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42433365332) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42433365353) pass
on that initial observation revision, before the paired Alkyhol comparator.
The complete downloaded native report is frozen unchanged at
`pe/reference/pe32-certificates-windows-v1.json`. The generator changes only the
non-mapped directory pair and two explicit eight-byte appends, accepts only the
frozen owned base and rejects force-integrity inputs. No game input enters CI.

PE32 ordinary loading now retains the raw certificate metadata without reading or
mapping it, matching these finite native observations. `certificate_directory()`
returns untrusted file-offset/size fields, not a signature/trust result. Explicit
FORCE_INTEGRITY requests fail preflight as unsupported. Partial pairs, the AMD64
certificate file-backing gate and every other mapped-directory/section bound
remain checked. Signature verification APIs remain unsupported. Successful image
loading does not establish a valid signature or successful trust verification.

Physical M4 development runs agree on all eight exact outputs, normally and with
forced rebase. The paired collector/comparator enforces same clean revision, exact
frozen source/PE/case identity, raw output and actual forced relocation; crashes,
unknown OS failures and observer failures cannot count as compatible rejections.
Strict same-revision CI/physical comparison passes at `70ef3d4` as recorded
above. All 99 PE host tests/
12 compiler checks, 34 portable tests, 268 Python tests and Clippy pass locally.
Synthetic tests additionally retain raw fields/input bytes, reject integrity
requests and partial pairs, preserve unrelated bounds, and reject corrupted
outputs, false relocation evidence, missing references and joint identity drift.

The unchanged local game clears this next file-range gate and stops at unsupported
startup features before entry. No game code executed; its files and derived
inventories stay local. This is finite owned PE loading/Win32 integer ABI evidence,
not game compatibility. See [PE32](pe32.md).

## PE32 SizeOfImage geometry: native Windows and physical M4 development

All 27 CI jobs pass at strict-reference revision `d08bcc4`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42432430209) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42432430273).
Downloaded clean native Windows/hosted Mac reports and fresh clean physical M4
execution at that same revision agree on exact inputs/sources, five successful
executions with every output byte, and three launch/preflight rejections. Host
normal/forced-rebase results agree; raw native errors and host status/diagnostics
remain separate. All older frozen PE64/PE32, application/DLL, SDK/source/COM,
shader and Metal comparisons pass. This clears the local game's previous image-
size gate, without entering its code or establishing game compatibility.

Native Windows's actual collector at `7c8fd27` executes five adequate owned PE32
variants, including four unaligned image-size fields, with the original eight raw
output words. Three insufficient extents fail CreateProcess with winerror 193 /
errno 8 before a child runs. Only four SizeOfImage bytes differ from the unchanged
frozen owned PE32 fixture. The full actual native report is frozen at
`pe/reference/pe32-geometry-windows-v1.json`, including exact case/source hashes and
all native process/error observations. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42431518982) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42431518949) pass
on that observation revision; those jobs predate the Alkyhol geometry comparison.

The candidate now rounds the allocation extent for partial PE32 image pages with
4-KiB section alignment and retains the raw header value through declared_size().
Section/32-bit address/one-GiB/W^X bounds remain. Physical M4 development runs
reproduce five executions and three preflight rejections in normal/forced-rebase
runs. Native/raw OS errors are retained separately from host status 78; crashes,
unknown OS failures and failed observer code cannot become compatible rejections.
Strict same-revision paired comparison passes at `d08bcc4` as recorded above. This
measures finite loader acceptance and the existing integer/stdcall observer, not
native allocation/query layout or game execution. The unchanged local game now
passes the old image-size gate and stops at file-range validation, before entry;
its files and all derived inventories remain local. See [PE32](pe32.md).
All 97 PE host tests/12 compiler checks, 32 portable tests, 261 Python tests and
Clippy pass locally. New host assertions retain declared header bytes, enforce
rounded extent/address bounds, keep other alignment/AMD64 gates, and reject
crashes, changed payload bytes, dropped cases, bad rebase evidence and joint drift.

## First unchanged PE32/i386 execution: native Windows and physical M4

All 27 CI jobs pass at hardened strict-reference revision `796e395`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42430262565) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42430262653).
Downloaded clean native Windows/hosted Mac records and fresh clean physical M4
execution at that same revision agree on all eight raw words, exact executed
PE/source identities and the frozen native record. Normal/forced-rebase host
results agree. All 96 PE host tests/12 compiler checks, 31 portable tests,
252 Python tests and Clippy pass. Previous frozen PE64/application/DLL, native
SDK/source/COM, shader and Metal comparisons remain green. This establishes the
finite owned integer/stdcall observer, not DAOrigins startup or game compatibility.

The new original MIT no-CRT i386 PE executes unchanged in a 64-bit Rosetta host
on physical M4. Normal and forced-rebase development runs agree on all eight raw
DWORDs: pointer width, 10,000 last-error round trips, preserved Win32 nonvolatile
registers/ESP, distinct 32-bit INC behavior, relocated pointer identity and exact
WriteFile data. Four real stdcall import gates transition to Rust on the saved
host stack; five-argument WriteFile uses explicit 32-bit standard-handle proxies.
Downloaded clean native Windows/hosted Mac records and clean physical M4
execution at `63b6f53` agree on all eight raw words and exact executed PE/source
identities. The owned PE SHA-256 is
`da101b3b451c8d511dd02e6316dc3101ef5c1b13cd56add33c029f9a476b191c`.
The actual native record is frozen at `pe/reference/pe32-windows-v1.json` and
required by default. These are integer/API/loader observations, not CRT, FS/TEB,
game startup or graphics.
All 27 CI jobs pass at `63b6f53`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42429463492) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42429463527).
Subsequent boundary hardening snapshots Win32 argument values before host calls
and rejects unaligned stack/count pointers. It does not add new native alias or
unaligned-pointer observations; the original frozen PE/source inputs stay intact.
All 96 PE host tests/12 compiler checks, 31 portable tests, 252 Python tests
and Clippy pass.
See [PE32](pe32.md) for address-space, metadata, API and trust restrictions.

## Registered static DLL lifecycle: native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `3fb563a`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42428220473) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42428220368).
Downloaded clean Windows/hosted Mac reports and fresh clean physical M4 execution
at that same revision agree on both DLL/source identities, all 40 raw trace words
and actual exported result/alias and attach success/rejection. The 92 PE host
tests/11 compiler checks, 28 portable tests, 246 Python tests and Clippy pass.
Older frozen binary and graphics comparisons remain green. This is isolated DLL
static-record lifecycle evidence, without a general guest loader or SEH dispatch.

Two new owned unchanged DLLs expose original static function-table pointers during
ATTACH/DETACH, reject guest deletion and retain lookup. Physical M4/Rosetta normal
and forced-rebase development runs agree on all 40 raw DWORDs, actual exported
Probe result/name-ordinal alias, successful dynamic lifecycle and rejected-attach
cleanup. The new context-borrowing RegisteredLibrary registers before attach,
keeps metadata through cleanup and removes entries before unmapping. Independent
native Windows/hosted Mac records and clean physical M4 execution at `331beeb`
agree on both DLL/source identities, all 40 raw DWORDs and actual exported
result/name-ordinal alias and attach success/rejection. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42427644748) pass. The
actual native record is frozen at `pe/reference/registered-dll-windows-v1.json`
and required by default; a missing-reference/joint-input-drift guard brings the
Python suite to 246.
All 92 PE host tests/11 compiler checks, 28 portable tests, 246 Python tests and
Clippy pass locally.
Existing loader APIs retain their old static-table gate. Host tests separately
check failure/capacity rollback before initialization, visibility after drop,
ordinary/no-entry DLLs, export views and scope lifetime. No guest module loader,
TLS, exception dispatch or game support is added. See
[registered DLL lifecycle](registered-dll.md) for exact contracts and provenance.

## Partial-prologue manual restoration: native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `b2ab1e3`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42426538166) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42426538129).
Downloaded clean native Windows/hosted Mac reports and fresh clean physical M4
execution at that same revision agree on all 68 raw DWORDs, exact executed
PE/source identities and the frozen native record. Normal and forced-rebase host
results agree. All 89 PE host tests/nine compiler checks, 27 portable tests,
240 Python tests and Clippy pass; previous frozen binary and graphics comparisons
remain green. This is finite manual restoration of six curated contexts, not
exception dispatch or CPU snapshots of interrupted prologues.

The new owned unchanged PE observer constructs six partial-prologue/body CONTEXT
inputs from a real fully executed frame's saved stack words. Physical M4/Rosetta
normal and forced-rebase runs pass all 68 raw DWORDs, including two pushes, a
168-byte allocation, R13/XMM6 saves, precise stack results and preservation of
all unrelated seeded bytes. These are curated manual-unwind contexts, not CPU
snapshots of interrupted prologues or Windows exception dispatch. The original
static/body fixtures remain unchanged. Downloaded clean native Windows/hosted Mac
records and clean physical M4 execution at `67343b6` agree on all 68 raw DWORDs
and exact executed PE/source identities. Native Windows runs the identical PE
with SHA-256 `2661b67008de5120b699c6bbb00e09e92992d12628aea335338b8ef2492d4a9a`.
The actual native record is frozen at `pe/reference/unwind-prologue-windows-v1.json`
and required by default. A missing-reference/joint-input-drift guard brings the
Python suite to 240. Native coverage is these six curated contexts for one
assembler-generated function; broader recipe shapes remain host assertions.
All 27 CI jobs pass at `67343b6`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42426111979) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42426111974).
Older frozen binary, SDK/source/COM, shader and Metal comparisons remain green.

The runtime validates the full supported v1 recipe before skipping unexecuted
operations at function entry/recorded operation boundaries. Completed body PCs
still require NOP and unknown prologue PCs/epilogues/handlers remain unsupported.
All 89 PE host tests/nine compiler checks, 27 portable tests, 240 Python tests and
Clippy pass. See [partial prologue restoration](unwind-prologue.md) for contracts,
source provenance and the difference between constructed and captured states.

## Registered static PE function tables: native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `825549f`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42425038558) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42425038579).
Downloaded clean native Windows/hosted Mac records and fresh clean physical
M4/Rosetta execution at that same revision agree on all 20 raw DWORDs and exact
executed PE/source identities, including normal and forced-rebase host results.
The 86 PE host tests/nine lifetime compiler checks, 25 portable tests, 234 Python
tests and Clippy pass. All older frozen binary, SDK/source/COM, shader and Metal
comparisons remain green. Native observations cover one static frame's lookup
and manual caller restoration; other recipes/lifetime failures remain host
assertions, and Windows exception dispatch remains unsupported.

The owned unchanged PE SHA-256 is
`70071d28d28a2d6167450e4fe7c8a8096adbb132ec6f1e82a6ab4cca16539e48`.
LLVM SEH assembler/linker directives emit its actual `.pdata`/`.xdata`; the C
client never imports/calls RtlAddFunctionTable. Physical M4/Rosetta development
execution matches all 20 raw words in normal and forced-rebase runs. It resolves
original static record/base/ranges, observes rejected guest deletion and retained
lookup, then manually restores a real active RBX/R12/small-allocation frame and
checks preserved seeded context bytes. Actual execution returns normally; no
Windows exception is raised or dispatched. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42424556455)
pass at `9a01d74`. Downloaded clean native Windows/hosted Mac records and clean
physical M4 execution at that same revision agree on all 20 words and exact
executed PE/source identity. Native Windows uses its actual OS module tables;
its static deletion returns false and lookup remains available. The complete
native record is frozen at `pe/reference/static-functions-windows-v1.json` and
required by default. A missing-reference/joint-input-drift guard brings the Python
suite to 234. Native coverage is one generated push/small-allocation frame; other
supported recipe forms and lifecycle failures remain host assertions.

RegisteredImage borrows ProcessContext and unregisters before unmapping. The
launcher now selects this EXE path; ordinary LoadedImage/DLL/library startup
contracts still reject static metadata. Pure inspection/preflight APIs validate
read-only table/recipe extents, RX ranges, supported v1 body recipes and loader-
write disjointness. Guest deletion cannot remove static entries. Capacity failure
rolls back without consuming other registry entries. Existing TLS/CFG/handler/
frame/prologue/epilogue/dispatch restrictions remain; this is not full SEH support.

All 86 PE host tests/nine lifetime compiler checks, 25 portable tests, Clippy and
233 Python tests pass. Additional host checks cover unsupported startup shapes,
code/metadata boundaries, mutable data, IAT/relocation aliasing, exact original
record identity, delete/drop/capacity behavior and scope lifetime. The unchanged
released peports corpus and old manual-unwind PE still pass on the new launcher.
See [static function tables](static-functions.md) for exact APIs and constraints.

## Direct UTF conversion boundaries: native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `c008de5`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42422663982) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42422663943).
Downloaded clean native Windows/hosted Mac records and fresh clean physical
M4/Rosetta execution at that same revision agree on exact executed PE/source
identities and all 1,832 raw DWORDs. Normal and forced-rebase host runs agree.
The 81 PE host tests/eight lifetime compiler checks, 22 portable tests, 228 Python
tests and Clippy pass. Previous frozen binary, SDK/source/COM, shader and Metal
comparisons remain green. This validates the finite query/successful-write/error-
status contract below; failed-write buffer contents and other code pages remain
outside these native observations.

A new owned MIT no-CRT PE SHA-256
`2563ffb1a26e0202e47bf72e4718e02b6234230c20653e66eec564246ffe3a7f`
records 76 UTF-8/UTF-16 input/flag query cases, 50 exact-size writes and two short-
buffer status/error calls. Physical M4/Rosetta normal and forced-rebase runs agree
on all 1,832 raw DWORDs. The runtime now performs default U+FFFD replacement for
malformed input; strict flags still fail with error 1113. Cases cover supplementary/
maximum scalars, embedded/terminated NULs, illegal/truncated/surrogate/overlong/
out-of-range UTF-8, lone/repeated UTF-16 surrogates and invalid zero input lengths.
All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42422253798)
pass at corrected/expanded revision `c9a974c`. Downloaded clean Windows/hosted
records and clean physical M4 execution at that same revision agree on all 1,832
raw words and exact executed PE/source identities. The actual native record is
frozen at `pe/reference/text-conversion-windows-v1.json` and required by default;
a missing-reference/joint-input-drift guard brings the Python suite to 228.
Corrected/expanded `c9a974c` also passes all 27 jobs, including all 24 conformance
jobs in [42422253741](https://github.com/daeken/Alkyhol/actions/runs/42422253741).

The first independent [PE run](https://github.com/daeken/Alkyhol/actions/runs/42421647507)
on `3aa1eca` correctly failed comparison: native Windows groups `ED A0 80` as two
U+FFFD units and `F4 90 80 80` as three; Rust's standard lossy decoder produced
three/four. Six DWORDs differed (query/write counts and resulting sentinel tails).
The runtime now consumes the rejected leading pair explicitly. A recovered copy
of the **unchanged original** PE (SHA-256
`e2fcefadce6eb2cf582ea427cd5b2559ccf8c6a3d93e5291d91c4de5f101e739`)
then agrees with all 1,064 original actual Windows words under the corrected
runtime, in normal and forced-rebase development execution. No acceptance was
relaxed. The new, not-yet-frozen observer was expanded with lower/upper boundary
neighbors and invalid continuations to test the generalized prefix handling;
independent native execution of that expanded matrix at `c9a974c` agrees on all
1,832 words, including both lower and upper rejected-prefix boundaries, valid
neighbors, invalid continuations and retained ASCII suffixes.

All successful write bytes/units and destination tails are recorded exactly.
Strict-invalid and zero-length inputs only make queries, with no failed-query
write calls. Short-buffer cases compare return/last-error only; failed-write
buffer contents are not recorded, normalized or claimed equal. Other code pages,
flags/default-character machinery and remaining invalid-pointer/buffer boundaries
stay unsupported or host-only. See [text conversion](text-conversion.md) for the
complete finite contract. All 81 PE host tests/eight lifetime compiler checks,
22 portable tests, Clippy and 227 Python tests pass. Older frozen binaries,
source inputs, application corpora and graphics remain intact.

## Third released PE64 application: peports native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `04fb895`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42420310102) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42420310162).
Downloaded clean native Windows/hosted Mac records and fresh clean physical
M4/Rosetta execution at that same revision match the unchanged released PE,
source/license/corpus/original-input identities and every status/stdout/stderr
byte in all 16 cases. Normal and forced-rebase host results agree. The 80 PE host
tests/eight lifetime compiler checks, 22 portable tests, 222 Python tests and
Clippy pass. All older frozen binary, SDK/source/COM, shader and Metal comparisons
pass; native conversion observations remain limited to this application's used
valid flag-zero paths.

The unchanged released w64devkit 2.11.0 x64 peports PE SHA-256 is
`1a57a02c27cbdd69e1c49ee59a2e84e46684163750700b56d03fad4108c71c3d`.
Its original bundled source agrees byte-for-byte with tagged upstream. Physical
M4/Rosetta passes all 16 owned cases in normal and forced-rebase execution, with
exact raw status/stdout/stderr. The original program parses the unchanged frozen
owned EXE/DLL, reads named/ordinal/forwarded export text and imports, handles
Unicode/spaced paths and stdin, and recovers from a missing file to process the
next valid PE. Malformed/truncated inputs execute its actual GCC builtin longjmp
error paths; this is not Windows SEH support. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42419886890)
pass on `64f37ae`. Downloaded clean native Windows/hosted Mac records and clean
physical M4 execution at that same revision match release/source/license/corpus/
input-PE identities and every status/stdout/stderr byte in all 16 cases.
The actual native record is frozen at `pe/reference/peports-windows-v1.json`;
future drivers/comparisons require its exact identities/results. A missing-reference/
joint-corpus-drift guard brings the Python suite to 222. Initial `64f37ae` also
passes all 27 jobs, including all 24 conformance jobs in
[42419886852](https://github.com/daeken/Alkyhol/actions/runs/42419886852). No binary/source patch, replacement parser or output/path
normalization is used.

Two real 16-bit UTF-16 Win64 conversion imports support valid CP_UTF8 input,
explicit NUL/positive lengths and unit/byte sizing. Strict malformed Unicode,
unsupported code pages/flags, unbounded scans and buffer/default-pointer forms
fail honestly. Native application cases exercise valid flag-zero inputs; remaining
boundaries have host assertions. The kernel32 whitelist is now 47 names.

The original 512-MiB static arena makes SizeOfImage `0x20008000`, beyond the old
512-MiB bound. The image policy is explicitly expanded to one GiB of sparse
address space. A portable exact-boundary/extent/overflow test and a physical
sparse mapping test check zero-filled/writable end pages and real RX execution;
VirtualAlloc remains capped at 512 MiB. Startup/section/W^X gates stay enforced.
Initial code passes 80 PE host tests/eight compiler checks, 22 portable tests,
Clippy and 221 Python tests; strict-reference hardening adds the 222nd Python test. Older frozen PE/source fixtures and graphics are untouched. See
[peports](peports.md) for exact provenance, bounded conversions and coverage.

## Second released PE64 application: make2compdb native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `3899551`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42418468977) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42418469012).
Downloaded clean native Windows/hosted Mac records and fresh clean physical
M4/Rosetta execution at that same revision agree on the unchanged released PE,
source/license/corpus identities and every status/stdout/stderr byte in all 16
cases. Normal and forced-rebase host observations agree. The 76 PE host tests,
eight lifetime compiler checks, 21 portable tests, 216 Python tests and Clippy pass.
All previous frozen binary, SDK/source/COM, shader and Metal comparisons pass.

The official unchanged w64devkit 2.11.0 x64 make2compdb 2026-09-15 PE SHA-256 is
`6f23be1906edbe2f9a9009918bc3c2800580f18dc2df04490e56b787d94fbafd`.
Its bundled source agrees byte-for-byte with the tagged upstream source; original
source, binary, archive and license identities are pinned. Physical M4/Rosetta
passes all 16 owned cases in normal and forced-rebase execution, with every
status/stdout/stderr byte checked. Cases exercise actual make-to-JSON work,
multiple/nested compilation entries, Unicode/quoted paths/options, 22,629-byte
stdin crossing the original 16-KiB read chunk, and unknown quoted CLI arguments.
There is no executable rebuild, source-port replacement, binary patch or normalized
JSON/path/output acceptance. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42418104279)
pass on `761fc2e`. Downloaded clean native Windows/hosted Mac records and clean
physical M4 execution at that same revision agree on PE/source/license/corpus
identity and every status/stdout/stderr byte in all 16 cases. The complete actual
native record is frozen at `pe/reference/make2compdb-windows-v1.json`; future
runs/comparisons require it and detect joint corpus drift or missing references.
That protocol guard brings the Python suite to 216. Initial `761fc2e` also passes
all 27 jobs, including all 24 conformance jobs in
[42418104268](https://github.com/daeken/Alkyhol/actions/runs/42418104268).

GetCurrentDirectoryW returns the real host path with UTF-16 sizing/termination;
shell32!CommandLineToArgvW decodes the exact canonical quoting generated by our
command encoder into a real contiguous owned UTF-16 argv block. Other valid
Shell32 command spellings, including empty command lines, fail explicitly for now.
LocalFree releases these blocks by full native address, with ownership/lifetime
checks. LocalFree boundaries are host-only evidence: make2compdb does not free
its argv block before process exit. Native observations cover only paths actually
used by the released application. These add two kernel32 names (45 total) and one
shell32 name. Initial code passed 76 PE host tests/eight lifetime compiler checks,
21 portable tests, Clippy and 215 Python tests; the strict-reference guard adds
the 216th Python test. Prior frozen fixtures and startup/image
bounds remain intact. See [make2compdb](make2compdb.md) for provenance and contracts.

The preceding docs-only `002b8d7` conformance run
[42417224973](https://github.com/daeken/Alkyhol/actions/runs/42417224973) initially
failed during an asset download with WinError 10054 before the H3D observer ran.
Rerunning its failed/dependent jobs passes all 24 jobs at that unchanged revision;
no acceptance or asset hashes were changed.

## Integer/vector stack-slot unwinding: native Windows, hosted Mac and physical M4

All 27 CI jobs pass at strict-reference revision `9a4bc97`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42416821128) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42416821230).
Downloaded clean Windows/hosted records and fresh clean physical M4/Rosetta
execution at that same revision match the identical executed PE/source identity
and all 37 raw DWORDs. Both normal and forced-rebase host executions pass. The
71 PE host tests/eight lifetime compiler checks, 19 portable tests, 209 Python
tests and Clippy pass. All older frozen binary and native SDK/source/COM/shader/
Metal comparisons remain intact; these are finite manual-restoration observations,
not exception dispatch, full CPU-state capture or game compatibility.

The new owned unchanged PE SHA-256 is
`5b3835f6a641536355a7f6b68af588a53630aa95017612e0c6b646930ec3e8b2`.
Physical M4/Rosetta development executions match all 37 raw words in normal and
forced-rebase runs. Actual active assembly frames cover near GPR saves, near
128-bit XMM saves, and far GPR/vector saves in a 1,048,696-byte fixed allocation.
The far frame supplies the first independent probe for unscaled ALLOC_LARGE.
Context checks read expected saved bytes/return addresses directly from the real
stack and verify restored RIP/RSP/GPRs/XMM6/XMM15, establisher, exact flags and every
other seeded byte. This is manual context restoration, with normal actual returns;
no full capture, exception dispatch/resumption or game compatibility is claimed.
All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42416493858)
pass on `8b67924`, including actual native Windows execution and SDK/MSVC assertions
for all ten XMM slot offsets. Downloaded clean native/hosted records and clean
physical M4 execution at that same revision agree on executed PE/source identity
and all 37 words. This independently covers RBX/R12 and XMM6/XMM15 near/far saves
and unscaled ALLOC_LARGE; other supported registers have host assertions only.
The complete native record is frozen at `pe/reference/unwind-saves-windows-v1.json`;
drivers and comparisons now require its exact inputs/results by default. A
missing-reference/joint-drift protocol guard brings the Python suite to 209. Initial
`8b67924` also passes all 27 jobs, including all 24 conformance jobs in
[42416493864](https://github.com/daeken/Alkyhol/actions/runs/42416493864).

SAVE_NONVOL, SAVE_NONVOL_FAR, SAVE_XMM128 and SAVE_XMM128_FAR now extend the bounded
NOP-body, flags-zero, frame-register-zero manual unwinder. All eight nonvolatile
GPRs and ten nonvolatile XMM registers have host assertions, including operand
truncation, scaling/alignment, frame extents, duplicate/overlapping saves, ordering,
address overflow and preservation of unrelated bytes. Saves must precede all
allocation/pop actions and lie inside the fixed allocation; unsupported forms
fail explicitly. The old frozen 34-word body-unwind probe still passes unchanged
on the expanded implementation. All 71 PE host tests/eight compiler checks, Clippy
and 208 Python tests pass. Other native fixtures and graphics sources/assets are
unchanged. Runtime and native SDK evidence remain separate from general unwind,
static `.pdata`, exception dispatch and game compatibility.

## Manual AMD64 body unwinding: native Windows, hosted Mac and physical M4

All 27 CI jobs pass at strict-reference revision `fb8fe7a`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42415391964) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42415391802).
Downloaded clean Windows/hosted reports and fresh clean physical M4/Rosetta
execution at that same revision agree on the identical executed PE/source identity
and all 34 raw DWORDs. Both normal and forced-rebase host executions pass. The
69 PE host tests, eight lifetime compiler checks, 19 portable tests, 203 Python
tests and Clippy pass; prior frozen native binary, SDK/source/COM, shader and Metal
comparisons remain intact. This establishes the finite manual frame-restoration
contract below; exception dispatch/resumption and general unwind remain separate.

The new owned MIT PE SHA-256 is
`01526ce4bd95eac7b9c868459b5ebe7177849e4f1c6688e9b0bf2ad7e3bd182e`.
Physical M4/Rosetta development execution agrees on all 34 raw words in normal
and forced-rebase runs. Three real active assembly frames supply captured fields
and real saved stack values: small allocation/RBX/R12, scaled large allocation/RBP,
and leaf. The test verifies restored caller RIP/RSP/registers, establisher frame,
exact context flags and every other seeded CONTEXT byte. Actual execution returns
normally; no exception is thrown, dispatched or resumed. Native Windows executes the identical PE and passes all 34 words; its real SDK
CONTEXT/RUNTIME_FUNCTION layout assertions also pass under MSVC. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42415039999) pass on
`5d92635`. Downloaded clean native/hosted records and clean physical execution
at that same revision agree on every word and PE/source identity. The complete
actual native record is frozen at `pe/reference/virtual-unwind-windows-v1.json`;
drivers/comparisons now require its exact inputs and results by default. Initial
`5d92635` also passes all 27 jobs, including all 24 conformance jobs in
[42415039878](https://github.com/daeken/Alkyhol/actions/runs/42415039878).

The bounded RtlVirtualUnwind implementation supports v1 flags-zero/frame-zero
recipes at NOP body control points, with PUSH_NONVOL, ALLOC_SMALL and both large
allocation forms. Unscaled large allocations and all eight nonvolatile register
forms have host assertions, not yet independent native coverage. Unsupported
handlers, chaining, frame registers, XMM saves, prologue/epilogue control points,
context-pointer outputs and unregistered entries fail explicitly. General unwind,
RtlUnwindEx, static `.pdata` startup and exception dispatch remain unimplemented.
No full CPU-context capture or game compatibility is claimed.

Initial `5d92635` passes 68 PE host tests plus eight lifetime compiler checks
under Rosetta, Clippy, 19 portable tests and 202 Python tests. Strict-reference
hardening adds original-entry identity/base/half-open-range selection coverage
and a required-reference/joint-drift guard (69 PE host and 203 Python tests). New host assertions
cover opcode operands/bounds/overflow, preserved context bytes, real eight-argument
Win64 calling convention and child-process failure status 78. The older function-
table protocol's synthetic positive host header was corrected from seven to its
actual ten imports, and is now positively decoded before comparison; production
acceptance and its frozen inputs/native results are unchanged.

## Dynamic AMD64 function-table metadata: native Windows, hosted Mac and physical M4

All 27 jobs pass at strict-reference revision `253685a`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42412746658) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42412746646).
Downloaded clean Windows/hosted reports and fresh clean physical M4 execution at
that same revision match the frozen executed PE/source identity and all 37 raw
DWORDs. Normal/forced host observations agree, including calls to all three
lookup-resolved generated functions. All prior frozen binary, SDK/source/COM,
shader and Metal comparisons remain intact. This is metadata/lookup evidence;
actual unwinding and exception dispatch remain unimplemented.

All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42412363341)
pass on `d86fd18`. Downloaded clean native Windows/hosted Mac records and clean
physical M4 execution at that same revision agree on the identical executed
PE/source identity and all 37 raw DWORDs. Normal and forced-rebase host results
agree, including all three generated functions reached through lookup results.
The complete native record is frozen at `pe/reference/function-tables-windows-v1.json`;
drivers and comparisons require its exact inputs/results by default. A missing-
reference/joint-drift guard brings the Python suite to 197. Initial `d86fd18`
also passes all 27 jobs: that PE run and all 24 conformance jobs
([42412363277](https://github.com/daeken/Alkyhol/actions/runs/42412363277)).
This does not establish live unwinding.

RtlAddFunctionTable/RtlLookupFunctionEntry/RtlDeleteFunctionTable now maintain a
bounded explicit dynamic metadata list inside ProcessContext. Real Win64 wrappers
use one-byte BOOLEAN and original 12-byte record pointers with full-width bases.
Lookup, half-open boundaries, gaps, independent table deletion and re-registration
are implemented. This does not interpret recipes, register Mach-O unwind frames,
restore registers or dispatch exceptions; RtlVirtualUnwind/RtlUnwindEx/callback
tables remain unresolved and `.pdata` startup is still rejected. Unsupported
lookup history/context/output shapes fail loudly rather than returning a leaf miss.

The owned unchanged PE SHA-256 is
`f0a287c6795475eb1d33e86badd9813239f21ad1e57b2180c9a34e677f6f5d49`.
Physical M4/Rosetta normal and forced-rebase executions agree on all 37 raw words:
original record/base/field identities, exact entry/end/gap lookups, two independent
table lifecycles and actual calls to three emitted functions through resolved
base/RVA addresses. A minimal valid leaf unwind header is supplied; no exception
or unwind is executed. The extracted Mach-O PE payload equals the input. Native
Windows receives the identical PE, and that independent comparison agrees above.

All 64 PE host tests, eight lifetime compiler checks, 19 portable arm64 tests,
197 Python tests and PE Clippy pass locally. Four host units cover real Win64
lookup/delete/base identity, metadata caps/shapes/overflow, scoped teardown and
host-thread isolation, plus child-process status-78 rejection of unsupported
lookup forms. Abstract metadata bound tests and host invalid-input/miss-output
contracts are separate from native observations and do not establish live unwind
support. The older generated-code fixture still passes locally. No earlier frozen
fixture/reference, sample/shader/asset or game data changes. No game compatibility
claim; this is a metadata milestone toward future exception support.

## Guest-generated code and memory permissions: native Windows, hosted Mac and physical M4

All 27 jobs pass at strict-reference revision `7c6bf36`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42410932748) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42410932707).
Downloaded clean Windows/hosted records and fresh clean physical M4 execution at
that same revision agree with the frozen PE/source identity and all 29 raw
DWORDs, including both actual emitted/rewritten-code results. Normal and forced
host results agree. All prior frozen binary, SDK/source/COM, shader and Metal
comparisons remain intact. This is finite code-emission/memory evidence, not
general JIT/exception handling or game compatibility.

All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42410651433)
pass on `2444dd5`. Downloaded clean native Windows/hosted Mac records and clean
physical M4 execution at that same revision agree on the exact PE/source/input
identity and all 29 raw DWORDs, including both actual generated-code results.
Normal and forced-rebase host observations agree. The complete native report is
frozen at `pe/reference/generated-code-windows-v1.json`; drivers and comparisons
require its exact input/results by default. A missing-reference/joint-drift
negative brings the Python suite to 191. Initial `2444dd5` also passes all 27
jobs: that PE run and all 24 conformance jobs
([42410651366](https://github.com/daeken/Alkyhol/actions/runs/42410651366)).

VirtualProtect/VirtualFree/FlushInstructionCache now perform real protection,
release and platform cache operations for the process context's bounded owned
allocations. VirtualAlloc keeps per-page state and correct temporary ownership
during 64-KiB alignment trims. Arbitrary image/heap protection, RWX, guard pages,
decommit/reserve-only, foreign/global cache flushes and unowned ranges remain
unsupported. No new exceptions/unwind startup acceptance or successful stubs.

The owned unchanged PE SHA-256 is
`a138926f31fad3b21b07eb8cdb7bfca097ebdfcc8745055397e543054ee1d0b5`.
Physical M4/Rosetta normal and forced-rebase runs agree on all 29 raw DWORDs,
including both actual results from emitted/rewritten Win64 instructions,
neighbor-page data, cross-page protection rounding/old values, no-access recovery
and successful release. The Mach-O's extracted PE payload equals the exact input.
The identical PE is transferred to Windows for independent OS API execution,
and that native comparison agrees as recorded above.

All 60 PE host tests, eight lifetime compiler checks, 19 portable arm64 tests,
191 Python tests and PE Clippy pass locally. Three new host units exercise
actual generated Win64 code, mixed per-page changes and release; preserve outputs/
ownership on rejected inputs; and use isolated children to measure write-RX,
execute-RW, read-NOACCESS and write-READONLY faults. A child queries the actual
Mach VM map before/after release, without reading freed memory. Host failure/error
contracts and permission-fault behavior are separate from native observations.
The released u-config binary still passes all 11 cases normally/forcibly rebased.
All earlier frozen fixtures/references and sample/asset/shader quality remain
intact. Combined protect/cache/rewrite execution is measured; the individual
cause of Rosetta cache invalidation is not isolated. No game compatibility claim.

## Guest module/export lookup: native Windows, hosted Mac and physical M4

All 27 jobs pass at strict-reference revision `1374b0b`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42408581516) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42408581621).
Downloaded clean Windows/hosted records and fresh clean physical M4 execution at
that same revision match the frozen EXE/DLL/source/import identities and all 21
raw DWORDs. Normal/forced host results agree with both images actually rebased.
All earlier frozen binary, SDK/source/COM, shader and Metal comparisons remain
intact. This is finite scoped guest lookup evidence, not a general guest module
loader or game compatibility result.

All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42408269870)
pass on `99bce97`. Downloaded clean native Windows/hosted Mac records and a clean
physical M4 record at that same revision agree on the identical EXE/DLL/source/
import identities and all 21 raw DWORDs. Both host images relocate in the forced
run. The complete native report is frozen at
`pe/reference/dynamic-lookup-windows-v1.json`; drivers/comparisons require its
exact binary/source/import/result record by default. Missing-reference and joint
identity drift guards bring the Python suite to 185. Initial `99bce97` also
passes all 27 jobs: that PE run and all 24 conformance jobs
([42408269823](https://github.com/daeken/Alkyhol/actions/runs/42408269823)).

The new ModuleContext retains explicit attached DLL owner borrows and exposes
real Win64 GetModuleHandleW/GetProcAddress bridges with actual full-width image
bases and resolved named/ordinal/code/data/forwarded targets. It does not add a
guest loader, fake module handles, implicit main/builtin images or DLL callback
lookup. Unsupported contexts and shapes fail honestly; view teardown neither
detaches nor reinitializes DLLs. The old binary fixtures/references are intact.

The owned EXE SHA-256 is
`369d6e7ffcbc6a098ff6e46281e3c18e2bdb8f170202cf906ff606d08f8b0540`,
with unchanged dependency DLL
`423d6e191a3ebb4f4f5ce11710591edc9ebe163ece79bd35cc6fe5146d13399d`.
Physical M4/Rosetta normal and forced-rebase runs agree on all 21 raw DWORDs:
actual PE-header/module identity, case/extension lookup, Direct/Alias/ordinal
target identity, NONAME calls, named/ordinal data sharing and writes, initialized
BSS, forwarded LastError calls and missing-symbol/module failures. Both images
actually rebase in the forced run. The same two binaries are transferred to
native Windows, and that independent comparison agrees as recorded above.

All 57 PE host tests, eight lifetime compiler checks, 19 portable arm64 tests,
185 Python tests and PE Clippy pass locally. Host units execute real lookup
targets and forwarders, reject duplicate/nested/invalid registries, preserve
64-bit module identity instead of truncating kernel-handle bits, check scoped
teardown and thread isolation, and exercise unsupported/null/string/ordinal
failures. Those edge/error cases and host-thread isolation are separate from
native observations and do not enable guest threads. No sample/shader/asset or
game data changes; no game compatibility claim.

## EXEs linked to borrowed DLLs: native Windows, hosted Mac and physical M4

All 27 jobs pass at strict-reference revision `36e91cc`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42406899110) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42406899070).
Downloaded clean Windows/hosted reports and fresh clean physical M4 execution at
that same revision agree with the frozen EXE/DLL/source/import identities and all
14 raw words. Both host images actually relocate in the forced run. All earlier
frozen binary, SDK/source/COM, shader and Metal comparisons remain intact.

All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42406635787)
pass on `c24f203`. Downloaded clean native Windows/hosted Mac records and a clean
physical M4 record at that same revision agree on the identical EXE/DLL/source
identities, exact mixed import thunks and all 14 raw DWORDs. Host normal/forced
results agree, with both images actually relocated in the forced run. The
complete native report is frozen at `pe/reference/linked-image-windows-v1.json`;
drivers and comparisons require that exact binary/source/import record by
default. Missing-reference and joint-drift guards bring the Python suite to 179.
Initial `c24f203` also passes all 27 jobs: the PE run above and all 24 conformance
jobs ([42406635857](https://github.com/daeken/Alkyhol/actions/runs/42406635857)).

`LinkedImage` maps a checked EXE against explicitly supplied attached DLLs while
retaining their borrows through unmapping. It shares the existing named/ordinal
resolver with `LinkedLibrary`; ordinary `LoadedImage::load` remains builtin-only.
The owned EXE SHA-256 is
`9d0d6eac3b3f2f79ff07e42845a7174d25218d85a30c9c20dfaa18812585ecb5`;
the dependency remains the previously frozen DLL
`423d6e191a3ebb4f4f5ce11710591edc9ebe163ece79bd35cc6fe5146d13399d`.
Physical M4/Rosetta normal and forced-rebase runs agree on all 14 raw words:
actual initialized BSS, named/ordinal/alias calls, named/ordinal data target
identity, kernel forwarder calls, shared data writes affecting later DLL code
and restored values. Both EXE and DLL relocate in the forced run. Native Windows
execution of these identical two binary snapshots agrees in the PE workflow above.
The Mac observer is a Rust Mach-O host reading unchanged PE snapshots; the old
single-file packager/launcher is unchanged and does not yet bundle DLL payloads.

All 54 PE host tests, six lifetime compiler checks, 19 portable arm64 tests,
179 Python tests and PE Clippy pass locally. New host units execute named/ordinal
and forwarded IAT targets, verify no DLL reinitialization or early detach, and
reject missing/duplicate/reserved dependencies, bad symbols/ordinals, static TLS
and IAT/relocation overlap before EXE entry. A synthetic EXE initially inherited
its helper DLL's export directory and was correctly rejected; the test now clears
that directory rather than loosening executable startup rules. Compiler tests
reject early dependency destruction. No earlier frozen binary/source/reference,
graphics sample/shader/asset or game data changes. This is finite owned binary
execution evidence, not a guest dynamic loader or game compatibility result.

## Binary D3D12 serializers: supported layouts and a measured native difference

All 27 jobs pass at strict-reference revision `16c8d50`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42405296187) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42405296216).
Downloaded clean Windows/hosted records and a fresh clean physical M4 execution
at that same revision match the frozen PE/source identity, full supported prefix
and each distinct restriction profile. Normal and forced-rebase host results
agree. Every earlier frozen binary, SDK/source/COM, shader and Metal comparison
still passes. This is CPU serialization/COM evidence, not binary device or game
compatibility.

All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42405009703)
pass on `b882623`. Downloaded clean native Windows/hosted Mac records and a clean
physical M4 record at that same revision have the identical PE/source identities.
All 1180 supported-prefix bytes agree, including the full 980 serialized bytes
and actual Win64 COM observations. The separate 36-byte native/host profiles
confirm the known 65-constant difference below; neither is called equivalent.
The complete native record is frozen at `pe/reference/root-signatures-windows-v1.json`.
Drivers/comparisons now require that exact PE/source/native record by default;
missing-reference and joint-drift negatives bring the Python suite to 173 tests.
All 24 conformance jobs also pass at `b882623`
([42405009639](https://github.com/daeken/Alkyhol/actions/runs/42405009639)), so that
pre-freeze revision passes all 27 jobs too. Original implementation `bad2ead`
passes all 24 conformance jobs ([42404379585](https://github.com/daeken/Alkyhol/actions/runs/42404379585));
its new PE comparison initially failed as described below.

The new binary d3d12 provider bridges both root serializers to the existing open
Rust version-1.0 implementation and wraps both success/error COM blobs in Win64
vtables. The owned unchanged PE SHA-256 is
`8d24b6875941525ce0dd5af5ee8a7263437fc1b558222a9d9ce0f4f60d9e0d6c`.
Physical M4/Rosetta normal and forced-rebase executions agree on all 1216 output
bytes. The common 1180-byte prefix contains all 980 bytes of the six previously
frozen SDK root containers plus actual legacy/versioned success, canonical COM
identity and virtual reference releases. The extracted Mach-O payload equals the input.
The fixture uses no CRT/SDK headers or device and is transferred unchanged for
native Windows d3d12 execution. This final fixture now passes the native and
hosted comparisons above; it preserves the observed restriction separately.

All 52 PE host tests, four lifetime compiler checks, 19 portable arm64 tests,
173 Python tests and PE Clippy pass locally. New host units cover both actual
Win64 exports, ABI layout/deep ownership, returned diagnostic virtual calls,
optional outputs and unsupported versions; closed bundles bind all three COM
builtin exports and reject replacement of the d3d12 namespace. Native failure
diagnostic text, host null/unsupported-version edge cases and allocation failure
are not claimed equivalent. Existing source interfaces, frozen PE inputs,
graphics samples/shaders/assets and game data are unchanged.

Initial PE CI `42404379361` on `bad2ead` passes the Mac job but native Windows
exits 28: the 65-DWORD case did not return the assumed diagnostic blob. Its PE
hash was `1a8e67d25912746b273acc8308245694d5999c3fdca6c109623ffa05fe4112d9`.
Instrumented revision `db121a3`, PE `f3b4199923e719567edfd93ad4afb0ded29ed5b22880afbb060ca8f84e467fea`,
in native run `42404657986` emits 1192 bytes before the same failure. The entire
1180-byte supported prefix agrees; its last three words are HRESULT 0,
success-is-null 0, error-is-present 0. Thus Windows serializes 65 constants
successfully while our existing bounded serializer rejects them. This is an
explicit support difference, not equivalent error behavior or a GPU validity test.
The final unfrozen fixture retains that case, reads/releases whichever actual
blobs are returned and records a separate nine-word backend profile. Comparisons
require the exact supported prefix and each distinct restriction profile; they
cannot silently erase the difference or treat either profile as parity. The
implementation's original 64-DWORD bound is unchanged. A fifth protocol negative
checks that substituting one backend's restriction result for the other fails.
All earlier frozen PE fixtures and acceptance criteria are intact.

## Win64 virtual COM: native Windows, hosted Mac and physical M4

All 27 jobs pass at strict-reference revision `1e47b28`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42403105379) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42403105337).
Downloaded clean Windows/hosted reports and a fresh clean physical M4 run at
that same revision agree with the frozen PE/source identities and all 23 raw
output words, including both client-written buffer checksums and canonical COM
identity/reference observations. Normal/forced-rebase host executions agree.
Initial `ab08287` also passes all 27 jobs (PE 42402557855; conformance 42402557869).
Existing SDK/source/COM/Metal/shader and all older frozen binary comparisons pass.

The binary d3dcompiler_47 provider now exposes D3DCreateBlob through a Win64
factory and a separate five-slot Win64 ID3DBlob vtable over the existing native
Rust blob. QueryInterface translates native identity back to the guest wrapper;
all five methods enter actual ABI adapters and the existing allocation/data/
reference implementation. Native source-ABI COM interfaces are unchanged. The
builtin namespace is reserved consistently across executable/DLL/bundle loading.

The owned no-CRT PE SHA-256 is
`ec197196d90cc8987169cbb6308034bd1bda026f7d84165d07fb5b917ab386cd`.
Physical M4/Rosetta normal and forced-rebase runs produce all 23 expected DWORDs:
canonical IUnknown/shared buffer predicates, unsupported-IID `0x80004002` with
cleared output, reference transitions 5→4→3→2→1→0, buffer lengths 37/16384,
and client-written checksums `850568607`/`2478659192`. Data survives alias releases
while the original reference remains live. The extracted Mach-O PE bytes match
the exact input hash. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42402557855)
pass on `ab08287`. Downloaded clean Windows/hosted reports and a clean physical
M4 run at that same revision agree on the unchanged PE/source identities and all
23 raw DWORDs against the actual Windows d3dcompiler_47 implementation. The full
native record is frozen at `pe/reference/binary-com-windows-v1.json`; future
drivers require its exact input before execution and comparisons require its
observed words/raw output. This is binary virtual-call evidence, separate from
the earlier source-C++ COM probe.
[Contract/reproduction](pe.md#binary-com-id3dblob).

All 50 PE tests, four lifetime compiler checks, 19 portable arm64 tests, 166 Python
tests and PE formatting/Clippy pass. Two host blob units cover real Win64 virtual
identity/data/ownership plus explicit failure contracts; a third new unit binds
the builtin through closed-bundle metadata without supplying a system DLL.
Every previous frozen executable, released u-config case and DLL observer passes
again locally, including forced rebase. Null-output/null-IID, zero-size and huge
allocation behavior is host-only here, and no native zero-fill or concurrent-
reference parity is inferred. Prior docs-only `3411d84` passes all 27 jobs
(PE 42401067461; conformance 42401067373). No original source/asset/shader, older
frozen reference, native shader-tool requirement or game data changes.
Freezing the native record adds a fifth protocol unit (167 Python tests),
including missing-reference and joint-identity-drift rejection.

## Closed DLL bundle lifecycle: native Windows, hosted Mac and physical M4

All 27 jobs pass at strict-reference revision `08aaf02`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42400565239) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42400565232).
Downloaded clean native Windows/hosted Mac records and a fresh clean physical M4
record at that same revision agree with the frozen three-DLL/source identities,
both prior result sets, actual shared target equality and all 48 C notification
bytes. All three host DLLs actually rebase in the forced run. Initial code
`1e028b8` also passes all 27 jobs (PE 42399759919; conformance 42399759282).
All previous frozen binary, SDK/source/COM/Metal/shader comparisons remain intact.

LibrarySet now owns an explicitly supplied acyclic DLL graph, preflights all
imports/forwarders/metadata before any initializer, orders dependencies before
consumers, shares instances within the set and detaches in reverse order. It
rolls back attached DLLs on mapping/ATTACH failure. There is no filesystem search,
global module cache/refcount, guest module API, static TLS or SEH support.

The fixture uses the **unchanged three already frozen DLLs**: exports `423d6e1…`,
named consumer `915abbc…` and ordinal consumer `49ce61a…`. No C/.def fixture bytes
change. Physical M4/Rosetta accepts consumers before their dependency in the input
list, preserves all earlier named/ordinal call/data/forwarder observations, checks
that both consumers target the same actual leaf code address, and retains the
exact 48-byte original C notification trace. Normal and forced-rebase results
agree; all three DLLs actually rebase in the forced run. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42399759919) pass at
`1e028b8`. Downloaded clean native Windows/hosted Mac records and a clean physical
M4 record at that same revision agree on the unchanged DLLs, both earlier call/
data/forwarder records, shared code target equality and all 48 notification bytes.
The full native report is frozen at `pe/reference/library-set-windows-v1.json`;
future drivers require its exact input identities before entry, and comparisons
require its observed results. [Contract/reproduction](pe.md#closed-dll-bundles).

Four new host runtime units check once-only shared-leaf diamond initialization,
lookup/IAT targets, complete pre-entry graph/symbol/startup rejection, failed-
attach rollback, and placement-failure rollback after a real dependency ATTACH.
Two new paired compiler checks accept correct teardown and reject dropping a set
while an implicitly dropped consumer still borrows its DLL (E0505). These host
counts/lifetime assertions are separate from native observations; the frozen
DLLs do not directly measure the native shared leaf's notification count.
All 47 PE host tests, four lifetime compiler checks, 19 portable arm64 tests,
161 Python tests and PE formatting/Clippy pass. Every prior frozen executable,
released u-config case and successful/failed/dependent/ordinal DLL observer passes
again locally, including forced rebase. No original sample, shader, asset or game
data is changed, and existing native comparison fixtures remain intact.
Prior docs-only `6af2878` passes all 27 jobs (PE 42398043239; conformance 42398043244).
Freezing the shared native reference adds a sixth protocol unit and brings Python
coverage to 162 passing tests. No fixture bytes or older native expectations change.

## Mixed ordinal DLL imports/forwarders: native Windows, hosted Mac and physical M4

All 27 jobs pass at the strict-reference revision `b083926`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42397548501) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42397548540).
Downloaded clean native Windows/hosted Mac records and a fresh clean physical M4
run at that same revision match the frozen DLL/source/import identities and all
seven execution observations. Both host DLLs actually relocate in the forced run.
The earlier `150ca60` milestone also passes all 27 jobs (PE 42397230715;
conformance 42397230794). Existing frozen binary, SDK/source/COM/Metal/shader
comparisons remain intact; the native record and old fixture bytes are unchanged.

The PE parser now retains named and ordinal imports in distinct ImportSymbol
variants. Two new portable units check mixed order, 0/65535 metadata boundaries,
literal named `#23`, ILT fallback, reserved bits and backed named hints. Two
runtime units check real borrowed ordinal IAT/export targets and pre-entry
rejection of malformed/missing/zero/system ordinals. No kernel32 ordinal table is
guessed and no sparse native-error parity is assumed.

The new owned consumer DLL (SHA-256
`49ce61a1f2ecf91a4ed14672370cd6db579c350d297d041c31e76ad093f09fdc`)
keeps the original frozen dependency unchanged. Its actual LLVM import table has
ordinal 21 data, named Direct and ordinal 23 NONAME code. Three forwarders target
ordinals 17, 21 and 23. Physical M4/Rosetta normal and forced-rebase runs agree on
Evaluate `324508621`, initializer/forwarded call `305419905`, forwarded data
`826366246` and three actual resolved pointer-equality predicates. Both DLLs
actually relocate in the forced run. This is actual trusted DLL execution, not
static inventory alone. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42397230715)
pass at `150ca60`. Downloaded clean native Windows/hosted Mac reports and a clean
physical M4 run at that same revision agree on both DLL/source identities,
the independently inventoried mixed thunks and all seven execution observations.
The complete actual native report is frozen at `pe/reference/ordinal-windows-v1.json`.
Future drivers require its DLL/source/import identity before entry, and the
comparator requires its native execution observations. [Contract/reproduction](pe.md#explicit-ordinal-imports-and-forwarders).

All 43 PE tests plus two lifetime compiler checks pass under Rosetta, 19 portable
arm64 tests and 155 Python tests pass, and formatting/PE Clippy pass. The frozen
original executable, all eleven released u-config cases and successful/failed/
dependent DLL observers pass again locally, including forced rebase. Launcher
diagnostics retain the exact existing named-only format for old binary observers.
Four initial synthetic protocol units reject identity/call/data/target/rebase drift;
those host assertions are separate from native Windows evidence. All 27 CI jobs
passed on prior docs-only `cd993fd` (PE 42396358867; conformance 42396358846).
The default frozen-reference regression adds a fifth protocol unit and brings
Python coverage to 156 passing tests; frozen original observers are unchanged.

## Explicit DLL-to-DLL linking: native Windows, hosted Mac and physical M4

All 27 jobs pass on the final implementation `9b766e5`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42395797481) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42395797350).
Downloaded clean native Windows/hosted Mac records and a fresh clean physical M4
run at that same revision agree with the frozen two-DLL identities, cross-DLL
calls/forwarded targets and all 48 lifecycle bytes. Both host DLLs actually rebase
in the forced run. Hosted PE tests include all 39 units, both lifetime compiler
checks and the dedicated-address real VM-unmap check. Existing executable,
source/COM/Metal/shader and native reference comparisons continue to pass.

The new unsafe LinkedLibrary API binds named code/data IAT imports and named
forwarders to explicitly supplied, already attached DLLs. Rust borrows retain
their lifetimes through consumer DETACH; nested consumers retain transitive
borrows. It does not discover/load a graph, search paths, expose guest module
APIs, add ordinals/TLS/SEH or change the isolated loader's rejection rules.

An owned no-CRT consumer DLL has SHA-256
`915abbc81ac8138e0320fb56a0b6f3e735bcd359039fb35d9a03596626e17628`;
it imports actual code/data from the unchanged frozen DLL with SHA-256
`423d6e191a3ebb4f4f5ce11710591edc9ebe163ece79bd35cc6fe5146d13399d`.
Normal and forced-rebase physical M4/Rosetta runs agree on Evaluate `826366381`,
forwarded Direct `571916821`, forwarded last-error sentinels `0x3456`/`0x4567`,
actual resolved target equality and all 48 ordered ATTACH/DETACH trace bytes.
The trace observes the dependency's initialized data/BSS during both consumer
notifications. Both DLLs actually relocate in the forced run. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42395051128) pass at
`1ab6e23`. Downloaded clean native Windows/hosted Mac reports agree on both
identical DLLs, all cross-DLL calls, forwarded target identities and all 48 trace
bytes. The full native report is frozen at `pe/reference/dependent-windows-v1.json`;
future drivers require both DLL/source identities before entry and the comparator
requires its frozen observations. [Exact contract and reproduction](pe.md#explicit-borrowed-dll-dependencies).

All 39 PE Rust tests pass under Rosetta (three new dependency units), all 17
portable arm64 tests and initial 150 Python tests pass, and PE formatting/Clippy pass.
The unchanged frozen executable, successful/failed DLL observers and all eleven
released u-config cases pass again on physical M4, including forced rebasing.
These host/regression assertions are separate from the native dependency
comparison. No original sample, shader, asset, old frozen fixture or game data changes.

A follow-up compiler regression exposed early dependency destruction permitted by
reference-container fields alone. An explicit LinkedLibrary destructor now
detaches/unmaps the consumer while its borrowed dependencies remain required by
Rust's drop checker. Two paired compile checks accept correct destruction order
and reject an early dependency drop with actual E0505; all 39 PE host tests and
the physical full-DLL observer still pass. Freezing the native baseline adds a
sixth protocol unit and brings Python coverage to 151 tests. No DLL bytes or
observed native expectations were changed to make this lifetime fix pass.

All 27 jobs pass on `1ab6e23` (PE 42395051128; conformance 42395051127).
The follow-up `ec5d673` PE run 42395383558 fails the pre-existing automatic-address
VM-unmap child assertion before the compiler/native observers run. That snapshot
cannot distinguish address reuse from a retained mapping; the cause remains
unconfirmed. The unmap unit now requests and asserts a dedicated 6-GiB preferred
address, then retains the same real Mach VM presence/absence and DETACH checks.
It never replaces existing mappings. All 100 isolated repetitions pass on physical
M4. A 128-GiB trial hint was not honored and was rejected by the placement
assertion. This changes test placement, not runtime unmapping or frozen evidence;
the final full hosted run passes as recorded above.

## Failed DLL initialization: native Windows, hosted Mac and physical M4

A separate owned no-CRT DLL (SHA-256
`207e9f12aec86c44d23f6b0f256cc6c0201c078b0560fd36a5e808f0d151a433`)
now exercises actual imported GetStdHandle/WriteFile/SetLastError from its C
DllMain, failed ATTACH and ordered cleanup DETACH. On physical M4 under Rosetta,
normal and forced-rebase runs both emit exactly two 16-byte trace records:
reasons `[1,0]`, counter `[1,2]`, NULL-reserved flags `[1,1]` and magic
`0xa17ac0de`. The Rust unsafe constructor rejects attachment and retains actual
guest last-error `0x1234`. No product runtime change or successful error stub is
introduced. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42393459700)
pass at `2f88f1b`; downloaded clean native/hosted reports and a fresh clean physical
M4 run at that same revision agree on the identical DLL/source and all 32 ordered
trace bytes. Windows LoadLibraryW returns NULL with actual last-error `1114`;
that loader error is recorded separately from the Rust constructor's sentinel.
The full native report is frozen at `pe/reference/failed-attach-windows-v1.json`.
Subsequent observers check exact frozen DLL/source identity before entry; the
comparator requires the frozen native trace and loader-error observations.
All 36 PE Rust tests under Rosetta, 17 portable arm64 tests, PE formatting and
Clippy passed locally. Freezing the native reference brings Python tests to 145
passing tests. Six new protocol units reject crashed,
missing/duplicate/reordered traces, invented success, identity/rebase drift and
changed frozen native evidence; these synthetic negatives are host assertions.
[Probe contract](pe.md#failed-dll-initialization-probe).

All 27 jobs pass at `2f88f1b`: the three PE jobs above and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42393459612).
The strict frozen-reference revision `1f442b6` also passes all three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42393693649); downloaded
clean native/hosted reports and a fresh clean physical M4 run at that revision
agree with the frozen DLL/source/trace/error evidence. Its separate 24-job
[conformance run](https://github.com/daeken/Alkyhol/actions/runs/42393693625)
also passes, as do all 27 jobs on docs-only `8d851f6` (PE 42394130461,
conformance 42394130444). Existing frozen native executable/
DLL comparisons and all source/COM/Metal/shader implementations remain unchanged.

## Isolated PE64 DLL execution: physical M4 milestone

All 27 jobs now pass on `cf5be5e`: the three expanded
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42391811360) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42391811409).
Downloaded clean native Windows and hosted Mac DLL execution reports, plus a
fresh clean physical M4 run at that same revision, agree on the unchanged DLL,
all six lookup records and all five frozen initialization/call/hole observations.
Normal and forced-rebase Alkyhol executions agree. All previous executable,
metadata-only, SDK/source/COM/Metal/shader comparisons remain intact.

The explicit unsafe LoadedLibrary API now executes the unchanged owned DLL
already measured/frozen on native Windows (SHA-256
`423d6e191a3ebb4f4f5ce11710591edc9ebe163ece79bd35cc6fe5146d13399d`).
Physical M4/x86-64 executes original C DllMain, rebases the absolute data pointer,
resolves name/ordinal/data/forwarded exports and calls original exported code.
All six actual lookup targets and all five initialization/call/hole observations
match the frozen Windows record, in normal and forced-rebase runs. Forwarded
GetLastError additionally returns the actual Rust thread-local sentinel `0x1234`.
The original DLL bytes, source and frozen native observer are unchanged. This is
real DLL code execution, separate from the earlier metadata-only observer.

Five new independent host lifecycle units pass (36 PE tests total), including
actual Win64 DllMain argument capture, ATTACH/DETACH order, failed-attach cleanup,
zero entry, unsupported-feature rejection before any initializer, lookup cases
and actual Mach VM region disappearance after Drop. Darwin mincore is a residency
query and freed-pointer reads are not the unmap oracle. Image mappings now use
64-KiB alignment with failure-safe trimming. All 17 portable arm64 tests, 139
Python tests, PE formatting/Clippy and both unchanged executable observers pass.

The live same-revision runtime comparison now passes as recorded above. This
establishes finite DLL success-path evidence; general Windows
module caching/refcounts, dependent DLLs, guest LoadLibrary/GetProcAddress APIs,
threads, static TLS, SEH and full unload side-effect parity remain unimplemented
or unmeasured. [Exact isolated-runtime contract](pe.md#isolated-trusted-dll-runtime).
No graphics implementation, original sample/shader/asset or game data is changed.

## DLL export reference probe: native Windows observations

The native Windows and Rust metadata comparison now passes in
[42388553558](https://github.com/daeken/Alkyhol/actions/runs/42388553558) on `c82d780`;
all three PE jobs pass, retaining both previous frozen executable comparisons.
Downloaded clean Windows/hosted reports and a clean physical M4 **metadata-only**
run agree on all six export entries and DLL SHA-256
`423d6e191a3ebb4f4f5ce11710591edc9ebe163ece79bd35cc6fe5146d13399d`.
Native Windows additionally observes initialized data `0x31415926`, BSS after
DllMain attach `0x13579bdf`, direct-call result `571916821`, ordinal-call result
`305419905`, and NULL for sparse ordinal 19. It resolves the forwarder to the
actual kernel32 GetLastError address and successfully frees the DLL. Those call/
initialization/unload observations remain **Windows-only**, not Rust execution.
The complete native report is frozen at `pe/reference/exports-windows-v1.json`,
common/native-only payload SHA-256
`e4353868555f99335062944073385952865c446ba7a3e55cf79110a54713351d`. Future
pairs must match its exact DLL/source/common/native-only identities. A new
joint-target/hole drift test brings Python coverage to 137 passing tests.

An owned no-CRT PE64 DLL is now built once with open Clang/LLVM and transferred
unchanged to the existing Windows PE workflow. Its source/.def contract contains
biased ordinals 17..23 with a sparse hole, two names targeting the same code,
exported initialized/BSS data, an ordinal-only function and a kernel32 forwarder.
The DLL retains an actual absolute data pointer/DIR64 fixup. A read-only Rust CLI
reports its metadata without mapping/entering it; a separate native Windows
observer uses actual LoadLibraryW/GetProcAddress/FreeLibrary and callable function/
data checks. Its query list comes independently from the owned .def contract,
not from the Rust parser. Native-only initialization/function/hole observations
are explicitly separate from common metadata and must not be attributed to Rust.

Local metadata inspection, 31 PE Rust tests on physical M4/x86-64, 17 portable
arm64 tests, 136 Python tests, PE formatting/Clippy and both existing executable
observers passed before the native run recorded above. This is DLL metadata
comparison with actual Windows lookups; no Rust DLL lifecycle or GetProcAddress
bridge is added. Existing native fixtures and original samples are
unchanged. See [the probe contract](pe.md#owned-dll-export-reference-probe).

## DLL export metadata: host-only foundation

The PE crate now inspects DLLs through a separate `Image::parse_dll` API and
reads bounded export metadata through `Image::exports`. Biased ordinals, sparse
holes, aliases, ordinal-only entries, mapped code/data addresses and raw bounded
forwarder text are preserved. Six independent synthetic parser units cover
truncation, wrong image kind/entry, lexical name order, slot/ordinal bounds,
counts/overflow and forwarder termination within the declared directory.
All 30 PE Rust tests pass on physical M4/x86-64, all 16 portable tests pass on
native arm64, and PE Clippy/formatting pass. Both unchanged executable observers
also pass locally, including normal/forced-rebase released u-config cases.

This is host metadata evidence, not DLL loading or native Windows export/runtime
parity. Executable startup gates remain; DLL execution and lifecycle still fail
explicitly. The [bounded metadata contract](pe.md#dll-export-metadata-foundation)
records what was implemented; no original source, shader, asset, frozen fixture
or game-derived data is changed.

## Off-the-shelf PE64: released Windows u-config on physical M4

A subsequent host unit exercises redirected stdin through actual Win64
ReadFile/WriteFile bridges: arbitrary byte readback, exact short-read/EOF counts,
preserved buffer tails, write-denial, high handle bits and stale-handle failure.
All 24 PE Rust tests and PE Clippy pass on physical M4/x86-64. This closes a host
coverage gap; u-config does not consume stdin, and this adds no native Windows
stdin observation or new product capability. Frozen observers are unchanged.

All three PE jobs pass in [42386144339](https://github.com/daeken/Alkyhol/actions/runs/42386144339)
on `1ccfd87`. Native Windows, hosted Mac and a fresh clean physical M4 execution
of the transferred upstream PE agree on all eleven cases, including the literal
lambda/cherry-blossom argument after explicit UTF-8 input decoding. Actual guest
status/stdout/stderr agree; both Alkyhol rebased runs agree with their normal
runs. The full native report is frozen at `pe/reference/u-config-windows-v1.json`,
canonical observation SHA-256
`8906455451a1c0c47b045027e2040a6d160661a237438badb552b38709e57a96`.
The strict comparator now requires this frozen input/binary/result identity and
checks that retained raw streams really produce the compared observations.
Raw listing order may differ; duplicates or any other changed bytes fail. A new
negative protocol test brings Python coverage to 134 passing tests. All 27 jobs
pass at `1ccfd87`: the PE run above and the 24-job
[conformance run 42386144322](https://github.com/daeken/Alkyhol/actions/runs/42386144322).
The strict frozen-reference [PE run 42386424722](https://github.com/daeken/Alkyhol/actions/runs/42386424722)
then passes all three jobs on `9645f17`. Downloaded clean native/hosted reports
and a clean physical M4 run at that same revision agree with the frozen native
observations and exact upstream PE identity. Existing shader/graphics/COM/SDK
comparisons remain intact; the stricter comparator changes no product runtime.

The unmodified official w64devkit 2.11.0 x64 `pkg-config.exe` (u-config 0.35.0)
executes under Rosetta through Alkyhol. Its 46,232 input bytes have SHA-256
`390f6c01b87cd8c71d3b5aa6cabd1cb63539ba6204972d920347f2376ef4debc`;
the actual extracted Mach-O payload agrees. Archive, matching source and public-
domain license are independently pinned; [provenance and bounded contracts](u-config.md)
record the release/recipe. No source port, rebuild, patch, alternate implementation
or relaxed exception/TLS/load-config startup gate is used.

Eleven package/CLI cases pass on physical M4, both normally and with forced
rebasing: real dependency/static flags, version predicates, missing-package errors,
quoted Unicode/empty values, directory listing and validation. All status/stdout/
stderr bytes are checked; only unspecified enumeration order is normalized by
whole lines, with raw output retained. Ten new process-service bridges bring the
whitelist to 34. The released code exposed undefined high stack-slot bits in an
int32 template-handle argument; documented low-32-bit kernel-handle canonicalization
fixes the actual file-open path without interpreting pointers as handles.

Initial local validation passed twenty-three PE Rust tests, ten portable arm64
PE tests, 78 root tests and 132 Python tests, plus formatting/PE Clippy and the
unchanged frozen original PE observer. Subsequent protocol tests bring Python
coverage to 134. Real PTY Unicode output is host unit evidence; released-binary
native comparisons are recorded above and remain finite program-path evidence.
Existing source/COM/Metal implementations and all original shaders/assets remain
intact. The shared token allocator changes ownership plumbing, not object API
semantics; it now reserves unique positive 32-bit-compatible tokens for companion
streams/searches as well as kernel objects.

Initial run 42385649794 on `13ec88b` executes all eleven cases successfully on
hosted macOS and native Windows, and retains the earlier frozen PE comparison.
Its new comparison step rejects input identity *labels*: native Path strings
used backslashes while macOS used slashes. Report keys now use `as_posix()`;
actual input hashes and every guest-output predicate remain unchanged. Normal
launcher execution is now quiet; observers explicitly request `--loader-info`
when checking the mapping diagnostic. This changes host reporting, not the
embedded upstream bytes or successful API semantics.

The corrected-label run 42385854756 then catches a real *input* discrepancy in
the Unicode case: Windows Python decoded the UTF-8 corpus using its legacy locale
encoding. Its released program correctly echoed that unintended mojibake; Mac
correctly echoed lambda/cherry-blossom. All corpus/report JSON now explicitly
uses UTF-8; the unchanged corpus bytes and guest expectations are not weakened.
A literal UTF-8 case-identity test guards this boundary (133 Python tests).
The successful native rerun above uses the corrected input; no parity was
claimed from the mojibake run.

## PE64 binary execution: first physical Rosetta milestone

The strict-reference revision `14c009d` passes all 27 CI jobs: the three
[PE binary jobs](https://github.com/daeken/Alkyhol/actions/runs/42383191376) and all
24 existing [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42383191362).
Downloaded clean native/hosted reports and a fresh clean physical M4 run at that
same revision match the frozen native record, including exact transferred /
extracted PE identity, all 19 words and 12 Unicode-file bytes. Both Alkyhol
forced-rebase executions agree. The original SDK/source/COM/Metal/shader evidence
remains intact; no original sample source or graphics implementation was changed.

The new `pe/` crate maps checked AMD64 Windows EXEs and supplies explicit Win64
bridges to the real Rust kernel. `tools/translate_pe.py` embeds unchanged PE bytes
in a read-only Mach-O section and links that x86-64 host to Alkyhol. This is a
loader/wrapper, not general static code rewriting or a game launch. The owned
no-CRT C/assembly fixture is built to actual COFF/PE using open Clang/LLVM; its
machine code is executed unchanged, rather than recompiled for macOS.

On physical M4/macOS, the fixture passes at its preferred base and after forced
DIR64 rebasing, including all 24 imports, Win64 nonvolatile GPR/XMM canaries,
stack arguments, surrogate-pair filename, zero fill, TLS/error stress, events,
semaphores, timing predicates and file seek/truncation/readback. All 19 report
words and the separate 12 file bytes match independent expectations. Twelve PE
Rust tests pass on the x86-64 process (eight portable parser units plus import
identity, mapping/entry, pseudo-handle semantics and isolated real permission-fault tests); eight portable
tests also pass on native arm64. Root Rust tests (77), Python tests (128), PE
Clippy and formatting pass locally. This is host/Rosetta evidence; native Windows
comparison initially awaited the new workflow; the executed result follows below. Existing
source/COM/Metal/sample implementations and fixtures are unchanged. The exact
bounded [binary contract](pe.md) lists unsupported startup/CRT/COM operations.

Initial CI 42382565164 passed the hosted Mac execution/permission tests but stopped
before Windows execution: checkout converted fixture source LF to CRLF, correctly
failing the exact-source check. The fixtures now pin LF through `.gitattributes`;
no observer predicate or kernel behavior was relaxed. Packaging additionally
extracts the actual linked read-only Mach-O section and verifies the PE snapshot
hash. Execution reports compare that extracted/transferred payload identity.
Three portable packaging-negative units pass (127 Python tests total).

The next native attempt reached fixture assertion 38: the test had mistakenly
classified -1 as an invalid CloseHandle handle. Windows uses -1/-2 for current
process/thread pseudo-handles; documented close semantics have no effect. The
binary bridge now recognizes those identities without touching the owned-handle
table. The fixture tests NULL as genuinely invalid and separately records both
pseudo-handle return/last-error pairs, rather than excluding the discrepancy.
These new records still require native execution evidence; the restricted source
kernel's original invalid-owned-handle contract/tests are unchanged.

Native Windows execution and the comparison job now pass in
[42382929856](https://github.com/daeken/Alkyhol/actions/runs/42382929856) on `77631c7`.
All three PE jobs pass. The transferred PE SHA-256 is
`b07f5490efc122124ab9e21037b9dbeab84fb268dce29b7e138fc7b5f106162b`;
its actual extracted Mach-O payload hash agrees. Clean native Windows, hosted
Mac and physical M4 reports on that same revision agree on all 19 words, the
12 Unicode-file bytes and status zero. Physical/hosted forced-rebase runs agree
with their normal runs. Native pseudo-handle closes return one and preserve both
last-error sentinels; NULL fails with error six. The full native report is frozen
at `pe/reference/windows-v1.json`, observation SHA-256
`142b5e834d87fb520f1d41f63add9d5ffc7741868eca9dba19486bfdbbca2009`.
Live pairs must now match that reference's exact PE identity, source identities
and observations, not merely each other. A joint-drift rejection test passes
(128 Python tests). This remains finite kernel/ABI execution evidence, with no
CRT, game, general DLL, binary COM/graphics or original shaded DXR frame claim.

## COM compute dispatch: physical Metal command-list evidence

The restricted device now records/submits real compute dispatch through D3D12
command lists and queues, with compute PSOs, root buffer descriptors, writable
Private buffers, per-resource UAV barriers and state-aware readback. This is
actual COM submission, beyond the prior direct-Metal PSO unit. The shared C++
probe passes on physical M4/x86-64 with Metal API validation: all 624 words across
six outputs match its independent scalar data/guard checks. It covers DIRECT and
COMPUTE queues, root offsets 256/16/8/16, binding snapshots, repeated root identity,
initial/reset PSOs, zero dispatch and interleaved aligned/unaligned GPU copies.
The unchanged established compute DXIL is used; this is not original MiniEngine
HLSL or an original frame. Native SDK/WARP execution in CI 42379450367 and clean
physical M4 execution agree on all 624 words at `8e4c6b2`, with identical shared
source/fixture hashes. The full native report is frozen at
`cpp/reference/compute-dispatch-windows-v1.json`, canonical observation SHA-256
`263fe289e48e36cfe4745892f83e6b51f513fcf74a3fa79528ccb7a89ecbfaa9`.
Future live pairs must also match this native payload. Frozen readback/source/
joint-drift protocol tests bring the local Python suite to 120 passing tests.
The initial run [42379450367](https://github.com/daeken/Alkyhol/actions/runs/42379450367)
passed all twenty-four jobs on `8e4c6b2`. The strict-reference run
[42379882478](https://github.com/daeken/Alkyhol/actions/runs/42379882478) then passed
all twenty-four jobs on `667c15d`, retaining every earlier SDK/shader/graphics/
resource/descriptor/root/PSO comparison. Downloaded clean native/hosted reports
and a clean physical M4/API-validation report at that same revision agree on
all 624 data/guard words and the frozen payload. The original subsystem v6 caller
also passes cleanly there. Windows debug-layer and native negative-HRESULT or
extra-lifetime parity remain unclaimed.

Seven new host/runtime units pass (65 total), including private shader-state
batch validation/rollback and negative binding/identity/reset/flag cases. Runtime
Clippy/formatting and all 118 Python tests pass. Existing frozen Shared/DEFAULT/
byte-copy and CBV-copy probes still match their native payloads locally. The
unchanged original subsystem v6 caller still passes; the complete original app
relinks and still exits 78 at production device creation. The exact bounded
contract, intended synchronization references and evidence distinctions are in
[compute-dispatch.md](../cpp/compute-dispatch.md). Tables/constants, embedded roots,
graphics and original application rendering remain unavailable. No source,
shader quality, asset or working Rust sample is replaced.

## Compute PSO creation: real Metal executable / original helper milestone

The restricted COM device now creates real nine-slot compute PSO objects through
the existing open DXIL -> SPIR-V -> MSL path. It selects the declared compute
entry, validates reflected buffer bindings against an owned same-device root
layout, and retains the actual Metal executable. Unsupported stages, embedded
roots, resource arrays/textures, root constants, cache blobs and flags fail
explicitly. [The bounded contract](../cpp/compute-pipelines.md) records the exact
limits. Command-list shader binding/Dispatch and graphics PSOs remain unavailable.

Six new runtime units pass (58 total), including direct Metal GPU execution of
the COM-created pipeline on physical M4 with all 96 words checked (76 data and
20 sentinel tail). This uses the established 4932-byte DXIL container with SHA-256
`e3084e25dfa23ef53e98eb1dd6eac386c3695718ac03d6631904b6711cc11d4e`.
It is direct Metal unit evidence, separate from the earlier native WARP compute
comparison and from D3D12 command-list execution. Two new translator units pass;
runtime/shader Clippy/formatting, all 114 Python tests and exact open-DXC fixture
regeneration pass locally.

The shared C++ construction/COM probe agrees on native WARP, hosted Rust/Metal
and clean physical M4/x86-64 at `ded5bbd`. All twenty-three jobs passed in
[42375900712](https://github.com/daeken/Alkyhol/actions/runs/42375900712), retaining
all previous SDK/shader/graphics/resource/descriptor/root comparisons. The first
run failed strict MSVC compilation on an int-to-byte fill in the new observer;
`ded5bbd` fixes that exact type without relaxing assertions or runtime semantics.
The native report is frozen at `cpp/reference/compute-pipelines-windows-v1.json`,
canonical observation SHA-256
`b8eefd69ed8e0166548fa8765e1b38467a4a52fee408ffc9f6c943351e8c6348`.
Future live pairs must match the frozen construction/COM payload. Local strict
checks of downloaded native/hosted and native/physical pairs pass; all 115 Python
tests pass. Cache blobs and shader execution after input disposal are not native
parity claims from this probe.

The strict-reference run [42376529216](https://github.com/daeken/Alkyhol/actions/runs/42376529216)
passed all twenty-three jobs on `6de9bd1`, including native PSO construction,
hosted Metal runtime tests and all prior SDK/shader/graphics/resource/descriptor/
root comparisons. Downloaded clean native and hosted reports match a fresh clean
physical M4 record at that same revision, including shared fixture identities and
the frozen common COM payload. The original subsystem v6 caller also passes
cleanly there. Locally the shader suite has 29 passing tests and one explicitly
ignored physical-RT material-interface test; that ignored test is not new evidence
from this milestone. Exact Sakura/MiniEngine shader artifact regeneration passes.
The original subsystem v6 caller now runs unchanged PipelineState.cpp's compute
finalization/cache path alongside the four previous verified original objects.
It supplies the owned compute test shader; original sample embedded-root DXIL
is still unsupported through this API. No upstream source/header is edited.
The complete original app still relinks and exits 78 at device creation, before
original initialization/rendering. Working Rust samples and assets are unchanged.

## Original root-signature helper: local layout/COM milestone

The unchanged MiniEngine RootSignature.cpp now runs through Rust's real
version-1.0 serialization and owned eight-slot root-signature COM object. The
original helper exercises Finalize, mixed parameter layouts, static sampler
conversion, repeat-finalization and original cache reuse. Its source/object are
verified from the complete pinned 70-unit build alongside the three prior
queue/allocator/heap objects (v5 observation). This is a subsystem execution;
the complete original app still relinks and exits 78 at device creation.

The shared C++ observer records all 980 bytes of six checked root containers,
legacy/versioned equality and real COM identity/metadata/parent assertions on
physical M4/x86-64. Nine new core units pass (77 core tests total), three new
runtime units pass (52 total), both Clippy checks/formatting pass, and all 108
Python tests pass. Root layout/checksum code comes from pinned open DXC with
retained NCSA attribution/license; no gated component is introduced.
[The exact bounded contract and provenance](../cpp/root-signatures.md) records
limits and failures. Native WARP and hosted Rust/Metal producers and root comparison
passed on `5473a58` in [42373677759](https://github.com/daeken/Alkyhol/actions/runs/42373677759).
Downloaded clean reports and the separate clean physical M4 report agree on all
980 bytes and COM assertions. The report is frozen at
`cpp/reference/root-signatures-windows-v1.json`; its canonical SHA-256 is
`f0c59ed82431c950da3780d21b233c267d5b204cfc641ed5a67d7cc46fcdc3fe`.
That overall run caught an older host-only link-boundary test still requiring
serialization to be unsupported. The check now validates actual success/error
blobs and version-1.1 rejection. All five host C++ ABI probes pass locally. No
frozen Windows observer is changed. Strict live pairs must also match the frozen
native bytes/COM payload; local checks of both actual pairs pass. The Python
suite now has 110 passing tests.

The strict-reference run [42373991380](https://github.com/daeken/Alkyhol/actions/runs/42373991380)
passed all twenty-two jobs on `28cf869`: updated host ABI boundaries, native
SDK/WARP and hosted Rust/Metal roots, exact frozen root bytes and all previous
SDK/shader/graphics/resource/descriptor comparisons. Downloaded clean reports
from this revision also match fresh physical M4/x86-64 execution on all 980 bytes
and COM assertions. The verified original subsystem v5 caller passes cleanly at
that same revision. Its four unchanged objects include the original root helper;
this original-helper execution is local and has not itself been compared with a
native C++ instance. The full original app remains compiled/linked and gated at
production D3D12CreateDevice. No GPU root binding/PSO/Dispatch or original-frame
parity is claimed.

## Constant-buffer descriptors and copies: physical Metal evidence

The explicit restricted device now initializes non-null UPLOAD buffer CBVs with
256-byte aligned addresses/sizes, sizes through 64 KiB and complete live buffer
bounds. Only registered same-device resources/heaps are accepted. CBV/SRV/UAV
heap slots own virtual 64-byte records; this is Alkyhol's encoding, not native
Windows descriptor bytes. CopyDescriptorsSimple and multi-range CopyDescriptors
snapshot initialized CBVs by value, including NULL size arrays (one per range).
All ranges, source slots and totals are checked before any destination write.
Visible copy sources, overlap, duplicate destinations, empty ranges, other view
types and null CBVs remain explicitly unsupported. Unsupported void calls abort.

On physical M4/x86-64 with Metal API validation, a shared C++ observer performs
simple copies, source overwrite independence, CPU staging with unequal range
partitions, and scattered unit-size copies. An explicitly named diagnostic API
consumes the four resulting CBV records on the GPU through actual native buffer
addresses. All three readbacks have sixteen expected data words and eight intact
guard words (72 total). No CPU substitute fills those readbacks. Native Windows
CI uses that same C++ copy workload but consumes its views via a real D3D12 root
signature/compute pipeline and a tiny HLSL shader compiled with pinned open DXC.
The committed DXIL header must regenerate exactly with that open compiler; no
gated converter or new proprietary shader compiler is used. This is descriptor-content evidence,
not identical-DXIL Metal translation, general Rust D3D12 root signatures/pipelines/
Dispatch, original rendering or game compatibility.

Seven new runtime units pass (49 total): typed records, live GPU/CPU intervals,
range/size/alignment failures, copy snapshot/order semantics, rejection without
partial writes, real GPU consumption/guards, diagnostic bounds, and retained
native allocation lifetime after COM shell release. That last ownership behavior
is an explicit local extension, not a native resource-lifetime observation.
Registries hold weak references and do not create ownership cycles. Runtime
Clippy/formatting, all 103 Python tests and exact open-DXC regeneration pass.
The unchanged original queue/allocator/heap subsystem caller still passes; the
complete original application relinks and still exits 78 before initialization
at D3D12CreateDevice. Original sources, assets, shaders and Rust apps are unchanged.

Native WARP, hosted Metal and physical M4 clean observations on `b10a7df`
agree on all 72 words and complete observer/fixture/generator identities. The
producer and descriptor comparison jobs passed in
[42371948288](https://github.com/daeken/Alkyhol/actions/runs/42371948288).
That overall run caught a Windows path-spelling assumption in the new generator
protocol test; `d2208ed` corrects that test, without shader/runtime/observer changes.
The full native report is frozen at `cpp/reference/cbv-copies-windows-v1.json`.
Its canonical observation SHA-256 is
`309c6fedf53fb0e3bf1915edda937b7b35d32bbb5ff87c400ebdf6cc169aa769`.
The 4156-byte HLSL consumer's DXIL SHA-256 is
`cf2694b10805c93623f4e73c0f58a4ed2faaed78a48933fd5967a2965be49aa0`.
Future live pairs must also match the frozen native payload; every word remains
an exact comparison field. The freeze adds source/fixture identity and joint-drift
protocol tests, bringing the local Python suite to 105 passing tests. Both actual
native/Metal pairs pass the strict frozen comparator locally.

The corrected run [42372084214](https://github.com/daeken/Alkyhol/actions/runs/42372084214)
passed all twenty-one jobs on `d2208ed`. The strict frozen-reference run
[42372226929](https://github.com/daeken/Alkyhol/actions/runs/42372226929) then
passed all twenty-one jobs on `a0ae65a`, including native WARP/hosted Metal CBV
consumption, frozen payload enforcement, exact open-DXC header regeneration and
all previous SDK/shader/graphics comparisons. Downloaded clean reports from that
same revision also match a fresh physical M4/API-validation run on all 72 words
and the full shared source/fixture identity. Original C++ initialization/rendering
remains gated; the different bounded consumers do not establish general D3D12
Dispatch or full native shaded-frame parity.

The preceding frozen allocation record passed all twenty CI jobs on `5ba4baa`
in [42370649149](https://github.com/daeken/Alkyhol/actions/runs/42370649149).

## Descriptor-heap allocation: local COM/storage evidence

The explicit Metal subset now supplies typed 11-slot ID3D12DescriptorHeap objects
for all four heap types. Shader-visible allocation is limited to CBV/SRV/UAV and
sampler heaps. Real retained Shared Metal buffers reserve 64-byte virtual slots;
CPU handles identify their actual backing addresses and GPU handles use actual
native GPU addresses. Visibility is determined by flags, not handle bits. This is empty heap
storage/metadata/ownership support, not initialized views or shader binding.
View/sampler creation, descriptor copies and command-list binding still abort
with explicit unavailable diagnostics; no successful empty descriptor operation
is substituted for them.

The physical M4/x86-64 shared C++ probe allocates twelve heaps across type,
visibility and node-mask cases, exercises null-output capability checks, inherited
metadata/QI, by-value descriptors/handles, stable starts and retained parent
identity. Each logical last-slot index is computed using that backend's actual
increment. Raw increments remain in reports; only those opaque driver byte strides
are excluded from cross-backend semantic equality. Returned descriptors, handle
presence, index offsets and ownership remain exact comparison fields.

Five new runtime units pass (42 total), including typed layout/bounds, defensive
constructor failures, actual handles/storage and metadata/parent lifetime, all
CPU heap types/range separation and a real Metal blit round-trip through the heap
backing allocation. That last check proves backing storage is GPU accessible;
it does not consume a D3D descriptor. Runtime Clippy/formatting and all 97 Python
tests pass locally. The unchanged original DescriptorHeap.cpp object now joins
the original queue/allocator subsystem caller: all four CPU allocators perform
pool rollover, and both visible heap types perform bump/index/range/exhaustion
checks (v4 observation). All three original object hashes are verified from the
complete original build; no upstream source is edited. The whole original app
relinks and still exits 78 before initialization at D3D12CreateDevice.

New CI executes the same heap probe on native SDK/WARP and hosted Metal and
compares the explicit semantic invariants from clean identical revisions. Native
results for this new allocation workload remain pending. The earlier frozen-byte
reference milestone passed all nineteen jobs on `db66f16` in
[42368218306](https://github.com/daeken/Alkyhol/actions/runs/42368218306).

The first native run, [42369268249](https://github.com/daeken/Alkyhol/actions/runs/42369268249),
failed a probe assertion equating nonzero GPU-handle bits with heap visibility.
No native heap parity is claimed from that failure. The observer now records the
actual value in diagnostics and separately records the all-ones invalid sentinel,
while preserving both fields in comparison. This measures the native distinction
before deciding how the backend should represent it; no frozen oracle is changed.

Native SDK/WARP execution on `0ee8f58` in
[42369744013](https://github.com/daeken/Alkyhol/actions/runs/42369744013) then
recorded nonzero, non-sentinel GPU handles for all twelve heaps, including the
CPU-only heaps. All returned node masks are 1. Actual WARP strides are
`[64,64,32,8]`, while Metal reserves `[64,64,64,64]`; both are retained as driver
facts. The native report is frozen at `cpp/reference/descriptor-heaps-windows-v1.json`.
Rust now exposes the real backing GPU address for CPU-only heaps as well; it
does not infer visibility or binding permission from those bits. This corrects
the mismatch rather than excluding it from comparison. The unchanged observer's
invariants now match locally, and future live pairs must also match the frozen
native invariants. Fresh native SDK/WARP and hosted Metal execution and the strict
invariant comparison passed on `2aaf073` in
[42370246611](https://github.com/daeken/Alkyhol/actions/runs/42370246611).
All twenty jobs passed, retaining the existing SDK, shader, graphics, geometry
and prior COM/resource comparisons.
Downloaded reports also match the clean physical M4 observation from that same
revision. The native observation's full canonical SHA-256, including its actual
strides, is `f492c53e6077155086010c76cf09953aa8ee48561e3073093d6b08cc69efa9c0`.
Only the opaque stride bytes differ between backends; the observed handle-state
mismatch was corrected and remains a comparison field. The Python
suite now has 99 passing tests. No initialized descriptor or native shader use
is established by this allocation reference.

## Byte-granular COM buffer copies: physical GPU evidence

Unaligned offsets and odd copy lengths now lower to a small MIT-licensed native
MSL byte-copy utility; aligned copies retain Metal blits. This is backend memory
copy implementation, not a replacement sample shader, general D3D12 Dispatch or
an alternative shader translator. The per-device utility pipeline compiles lazily
and caches its real result; compilation failure fails submission without a CPU
copy or success fallback. GPU dispatches cover exact ranges in at most 16-MiB
chunks. Encoder switches preserve command order and tracked-resource hazards.

On physical M4/x86-64 with Metal API validation enabled, the shared C++ probe
passes 272 offset-residue/tail cases and adjacent/overlapping writes to an odd
4099-byte Private buffer. All 39,601 smaller readback/guard bytes match the test's
independent expected data. It also checks all 16,777,313 bytes of a larger readback
whose copy crosses a dispatch boundary, using a pattern that changes across that
boundary. Reports retain its checksum and three boundary windows (128 bytes),
not the complete large buffer. Native comparison therefore requires exact small
readbacks/windows/checksum agreement plus each executable's full-range assertions;
it is not a raw large-buffer byte-for-byte native dump comparison.

Three new runtime units pass (37 total): parameter/chunk bounds and overflow,
aligned-path selection, and actual Private compute→blit→compute data/guard checks
with pipeline cache identity. Existing frozen Shared and DEFAULT copy probes
still match their native snapshots. The original MiniEngine subsystem caller
now copies 257 bytes on each original queue type (v3 observation), with allocator
reuse and original fence routing intact. Its original implementation objects,
upstream sources and all sample shaders/assets remain unchanged. All 91 Python
tests, runtime Clippy and formatting pass locally. The complete original sample
relinks and still exits 78 before original initialization at device creation.

A CI probe executes the same byte-copy C++ source on native WARP and hosted
Metal. [CI 42367694508](https://github.com/daeken/Alkyhol/actions/runs/42367694508)
passed all nineteen jobs on `57e6321`, including all previous shader/SDK/graphics
comparisons. Downloaded clean native WARP and hosted Metal records also match
the clean physical M4 record on every observed field: the 39,601 smaller bytes,
128 boundary bytes and large checksum. The native snapshot is preserved at
`cpp/reference/byte-copies-windows-v1.json`, canonical observation SHA-256
`c1016c4a93300ff0b85f43975a2e447fc9b8466b8455487d373ce4413f65868a`.
Both future live reports must match it unchanged. The current Python suite has
93 passing tests. All eighteen prior
jobs passed on `74a890e` in
[42366501362](https://github.com/daeken/Alkyhol/actions/runs/42366501362).
GPU queue waits, zero-length DEFAULT copies, same-resource copies, textures,
general pipelines and original rendering remain outside this milestone.

## DEFAULT-buffer transitions: physical GPU evidence

The explicit Metal subset now allocates DEFAULT buffers in genuine Private
storage with tracked hazards, preserving actual aligned GPU addresses and buffer
descriptors. CPU Map fails; invalid DEFAULT Unmap aborts. DIRECT lists record
complete buffer transitions over the bounded COMMON/COPY/read-only state set.
UPLOAD/READBACK states remain immutable. Byte-granular copies, split/alias/UAV
barriers, textures, descriptor/pipeline rendering and production device creation
remain unavailable.

On physical M4/x86-64, two DEFAULT buffers execute upload/private/private/readback
chains with explicit transitions, COMMON promotion/decay, a CPU-fenced direct-to-
copy-to-direct queue handoff, and a final two-list batch. All 4096 readback bytes,
including untouched a5 sentinels and later partial overwrites, match the finite
workload. Actual fence values are `[1,2,3]`. The workload maps only Shared buffers;
Private storage is not exposed through a CPU fallback. Transition entries validate
D3D state while the classic Metal queue resolves hazards on directly bound tracked
resources; no unsupported rendering command returns successful empty work.

Seven new runtime units pass (34 total), covering state promotion/read combinations,
before-state checks, physical Private storage/map rejection, barrier layouts and
unsupported forms, transactional rollback/cross-queue reservations, no decay between
lists, member retirement, and persistent injected failure. These defensive invalid/
injected cases are local assertions, not native device-removal or synchronization
parity. All 85 Python tests and runtime Clippy pass locally. The prior frozen COM
copy observation still matches exactly, and the unchanged original queue-manager
caller still completes its three-queue copies and allocator reuse. The complete
original sample relinks and still exits 78 at the honest D3D12CreateDevice gate.
No original source, shader, asset or reference expectation is modified.

A native WARP/hosted Metal CI probe compares every byte plus returned DEFAULT
descriptors, heap properties, Map failure and fence values, using identical source
and clean revisions. [CI 42366061032](https://github.com/daeken/Alkyhol/actions/runs/42366061032)
passed all eighteen jobs on `4c09ebc`. Downloaded native WARP and hosted Metal
reports also match the clean physical M4 observation, including all 4096 bytes.
The native report is frozen as `cpp/reference/default-buffer-transitions-windows-v1.json`,
with canonical observation SHA-256
`98868d79e4b04b6e2e76d35c4c06097f59b031622160f7d5860447ed4d081586`.
Future live comparisons must also match that unchanged snapshot; 87 Python tests
now pass locally. This remains finite buffer-workload evidence, not a native
original-frame comparison. The prior frozen-copy/comparison code passed all
seventeen jobs on `c9ef6a6` in
[42365022565](https://github.com/daeken/Alkyhol/actions/runs/42365022565).

## COM command-list buffer copies: physical GPU execution

The restricted Metal constructor now creates actual 9-slot command allocators
and 60-slot GraphicsCommandList0 interfaces. DIRECT, COMPUTE and COPY lists
record immutable buffer-copy operations; ExecuteCommandLists encodes native
Metal blits, rather than copying bytes on the CPU or returning a successful stub.
Closed/recording state, allocator epochs and pending execution reservations are
tracked. Reset can replace a list's recording while its prior snapshot executes;
allocator reset cannot discard active recording/pending storage. Known recording
errors surface at Close and remain failures on Reset. Unsupported rendering,
barriers, byte-granular copies and other void operations still fail explicitly.

On physical M4/x86-64, the new shared C++ probe checks that recording leaves the
destination unchanged, changes upload data before submission, submits a partial
copy, immediately resets that list with another allocator, then submits two lists
containing another region copy and a full-resource copy. A queued fence completes
after the actual GPU work. All 2048 destination bytes, including untouched a5
sentinels, match the finite expected workload. Eleven HRESULT observations cover
recording/reset/close and completed allocator reuse. Six unsupported void paths
separately abort with specific diagnostics. No shader, raster, DXR, original-frame
or feature-level result is inferred from these copies.

Nine new runtime units pass (27 total), covering typed slots/handle ABI, range
overflow/alignment, state transitions, metadata/parent ownership, foreign/busy/
wrong-kind allocators, sticky errors, stale/duplicate submissions, immutable
snapshot/reset ownership and actual native Metal full-buffer copying. Defensive
pending-reset rejection, invalid submission aborts, retained released resource
shells and injected/internal lifecycle cases are local assertions, not measured
Windows edge semantics. Runtime Clippy and all 79 Python tests pass locally.

The original subsystem caller now additionally executes unchanged
CommandListManager::CreateNewCommandList and CommandAllocatorPool allocation /
deferred reuse / Reset logic. It validates a real 256-byte COM GPU copy on each
original queue type, with original fence routing/waits. Its v2 observation records
three copied queues and allocator reuse, separately from the earlier v1 record.
This is physical original-subsystem execution, not the original app initialization
or original rendering. The complete unchanged app relinks and still exits 78 at
the production D3D12CreateDevice gate. Existing shader/asset quality is unchanged.

CI executes the exact same C++ copy probe through native SDK/WARP and hosted
Metal constructors and compares all bytes/state results at a clean identical
revision. Native and hosted execution and their exact comparison passed in
[42364640363](https://github.com/daeken/Alkyhol/actions/runs/42364640363) on `ffc1ec5`.
All seventeen jobs passed, preserving the prior shader/native compute, Windows
SDK, metadata, buffer, queue, geometry and graphics comparisons as well.
Downloaded reports also match the clean physical M4 record from that revision:
all 2048 observed bytes and eleven state results agree. The native observation is
preserved at `cpp/reference/command-copy-windows-v1.json`, canonical observation
SHA-256 `419bf280bdeb59498bc05837f3a367ebe90ead351a8a799aa0962be6f067edcc`.
Future live comparisons must match that unchanged native snapshot as well.
The Python suite now has 81 passing tests. Earlier queue,
buffer, metadata, Windows SDK, shader and geometry comparisons all passed in
[42363716768](https://github.com/daeken/Alkyhol/actions/runs/42363716768) on `357f72f`.

## Original MiniEngine queue manager: physical subsystem execution

A separate caller now links the exact unchanged `CommandListManager.cpp` and
`CommandAllocatorPool.cpp` objects from the verified complete 70-unit build. On
physical M4/x86-64, original CommandListManager::Create initializes all three
actual Rust/Metal COM queues and fences, preserving the original high-bit queue
encoding. Eight original IncrementFence/WaitForFence/IsFenceComplete sequences
per queue complete, with next values `[9,144115188075855881,216172782113783817]`.
Original per-queue idle, manager IdleGPU, Shutdown and repeated destructor shutdown
also complete. The harness supplies the matching original global and a caller;
it changes no original implementation or header.

`tools/run_original_command_manager.py` rejects stale compatibility headers,
source/object changes, incomplete compilation and altered source revision, then
records exact original object/source hashes and commands alongside the Rust
library, caller and executable hashes. Three new Python provenance/observation
tests pass (75 Python total); they are synthetic gates, separate from the actual
physical subsystem run. The raw report is ignored at
`local/original-command-manager.json`. This is local original-C++ queue-management
execution; it has not received a native Windows subsystem comparison. Allocator
creation, command lists, shaders, original app initialization and rendering remain
unsupported or unexecuted. The complete original app still exits 78 at preflight.

## Restricted COM queues/fences: local completion evidence

On physical M4, real x86-64 C++ virtual calls now create COPY queues and fences
through the explicit Metal subset constructor. The shared probe observes values
`[5,7,8,3,9,12,21,2]`, ten expected event wait results, queued rewinds and retained
parent/pending-submission lifetimes. Queue signals submit actual empty native
Metal command buffers; completion callbacks, rather than immediate CPU writes,
advance the fence. This is GPU queue-completion evidence, not COM command-list
execution, shader execution, original C++ initialization or a frame.

Six new runtime units (18 total) pass, covering typed layouts, CPU rewinds and
event thresholds, genuinely blocking null-event waits, out-of-order completion
callbacks, failure sentinels, actual Metal completion, release/lifetime behavior,
supported queue kinds and rejection of foreign-device fences. All 68 root tests,
runtime/root Clippy, 70 Python tests and existing C++/compiler probes pass locally.
Six unsupported queue void-method paths separately abort with explicit diagnostics;
GPU queue Wait and timestamp APIs fail honestly. Event retention after CloseHandle
and injected callback failure are local assertions, not measured native semantics.

The compatibility-header change was followed by a fresh successful compile of
all 70 original C++ units and a complete relink. The original application still
exits 78 at unavailable D3D12CreateDevice, before original initialization. Original
source, assets and shaders remain unchanged. New CI executes an independent WARP
queue/fence probe and compares its exact output with hosted Metal from the same
revision. Previous
buffer observations and their exact original probe bytes are preserved, and live
buffer comparisons now also require equality to that unchanged native snapshot.

Native WARP execution in
[42363080977](https://github.com/daeken/Alkyhol/actions/runs/42363080977) on
`52df42e` observed all eight fence values, ten waits and both lifetime checks
matching Metal. The exact comparison correctly failed on a descriptor mismatch:
native GetDesc normalizes input NodeMask 0 to returned 1, while Metal echoed 0.
That independent native report/source identity is frozen as
`cpp/reference/queue-fence-windows-v1.json`; Rust now normalizes its single-node
queue descriptor to the measured result. The comparator also requires the live
pair to equal this unchanged native snapshot. No comparison tolerance or expected
native output was changed. [CI 42363319667](https://github.com/daeken/Alkyhol/actions/runs/42363319667)
passed all sixteen jobs on `30e7613`, including fresh native WARP and hosted Metal
queue/fence execution and their exact comparison. Downloaded clean-revision
reports also match the frozen observation. The corrected physical M4 probe
separately matches it; no full native shaded frame or original C++ rendering is
claimed. The Python suite at that revision has 72 passing tests.

## Restricted Metal COM device and committed buffers: local evidence

A separately named `AlkyholCreateMetalDeviceSubset` now constructs real retained
Metal-backed ID3D12Device/Resource interfaces, with typed 44/15-slot vtables.
This is explicitly not D3D12CreateDevice or a feature-level claim. That native API
and the original application's startup gate remain unavailable. Options5 reports
no implemented DXR/render-pass/tiled-resource extensions, regardless of physical
Metal RT support; Device5 QueryInterface fails honestly.

On physical M4, actual x86-64 C++ virtual calls create UPLOAD and READBACK buffers
of 257 and 1024 bytes, exercise large by-value resource descriptors and by-value
adapter LUID ABI, map/write/unmap/remap, obtain heap properties and native Metal
GPU addresses, and verify retained device identity after releasing caller handles.
Observed mapped checksums are 4261412018/3966715333 for the two widths. These are
CPU data/ABI observations; the subsequent native WARP comparison is recorded below.
Unsupported void/struct operations are separately checked to abort with a
specific diagnostic, never counted as supported behavior or successful results.

A distinct Rust integration unit creates the same COM-backed shared buffer,
submits a native Metal blit fill, waits for Completed status and verifies every
one of its 512 mapped bytes is 5a. This is physical GPU-backed resource evidence,
not COM command-list execution, shader execution, original C++ rendering or a
native D3D12 feature-level claim. Six new device/resource units (12 runtime total)
and runtime Clippy pass; all 64 Python tests pass locally. The unchanged original
C++ sample relinks with the updated library and still exits 78 at its honest
D3D12CreateDevice preflight. No original source, asset or shader is modified.

A new CI comparison will require clean identical revisions and source bytes for
the independent native WARP and restricted Metal buffer probes. Neither expected
native heap/descriptor values nor successful calls are manufactured by its parser.
Initial native probe runs rejected an unjustified assumption that a device
interface pointer must equal its IUnknown pointer. The probe now verifies COM's
canonical IUnknown identity across round trips and parent interfaces instead;
this does not require arbitrary interface addresses to match. The frozen earlier
metadata observer is unchanged.

Native SDK execution in
[42361555862](https://github.com/daeken/Alkyhol/actions/runs/42361555862) then exposed
a real implementation mismatch: committed-buffer descriptors report alignment
65536 and heap flags 192 (ALLOW_ONLY_BUFFERS), rather than the input zero values.
The full native report is preserved separately. Rust now matches those measured
returns and exposes an actually 64-KiB-aligned subspan of its Metal allocation;
CPU mapping, native GPU address and the direct Metal blit all use the same offset.
No GPU address is synthesized. [CI 42361819479](https://github.com/daeken/Alkyhol/actions/runs/42361819479)
passed all fifteen jobs on `5048884`, including the new live native/Metal buffer
comparison. Downloaded native WARP and hosted Metal records also match the physical
M4 x86-64 report from that same clean revision. All four descriptor/heap-property
records, mapped checksums, address-presence and canonical parent-identity checks
agree without weakening the comparison. A frozen native snapshot/source hash
protects the measured values separately from the implementation. The current
Python suite has 65 passing tests; runtime units/Clippy remain passing.
See [restricted runtime](../cpp/runtime/README.md) for exact supported constraints.

## Direct-Metal DXGI factory/adapter COM: local discovery evidence

The new combined Rust COM runtime constructs actual Metal-backed DXGI factory
and adapter objects. On physical M4, an x86-64 C++ executable called the SDK
interfaces through all 30 factory and 11 adapter vtable slots, checking inherited
metadata, canonical identities, parent ownership after releasing factory handles,
LUID round trips, guarded descriptor writes and explicit unsupported operations.
It enumerated Apple M4, registry ID 4294968282, with actual advisory Metal
working-set budget 19069665280 bytes. This is physical device discovery, not
command submission, DXR support, a native Windows descriptor comparison or a frame.

Descriptions and IDs come from retained Metal devices. Unknown PCI identifiers
remain zero. DedicatedVideoMemory explicitly presents a virtual adapter working-set
budget, not physical VRAM or guaranteed capacity; other unknown memory budgets
remain zero. GPU preference ordering, software/WARP, window/output/presentation
services fail rather than reporting synthetic adapters or success. Factory
private-data forwarding is exercised locally; native DXGI-specific edge semantics
remain unmeasured. The previous native D3D12Object reference is unchanged.

All six new runtime Rust tests and Clippy pass locally; the 67 root tests and root
Clippy pass, all existing C++ fixtures pass, and 59 Python tests pass. Tests cover
layout/LUID bits, constructor failures, actual Metal descriptor properties,
all factory IID versions, selection failures, parent lifetimes and atomic refs.
The original unchanged C++ application relinks against the combined Metal library
(default) and still exits 78 at unavailable D3D12 device creation. The explicit
root-only build remains available; no automatic backend fallback is introduced.
[CI 42359772543](https://github.com/daeken/Alkyhol/actions/runs/42359772543)
passed all fourteen jobs on `c8475d0`, including the new runtime Rust units and
actual x86-64 C++ Metal factory calls. Downloaded hosted evidence identifies
Apple Paravirtual device, registry ID 4294967703, advisory budget 5010800640 and
native Metal RT=false. Physical M4 and hosted observations are deliberately not
compared as identical hardware descriptions. Existing native Windows, metadata,
shader and graphics/geometry comparisons all remain passing.
See [runtime mapping](../cpp/runtime/README.md) for exact APIs, field semantics
and the remaining device/queue/resource and rendering boundary.

## Rust ID3D12Object metadata base: local CPU execution

The new seven-slot Rust vtable implements IUnknown identity/reference ownership
and ID3D12Object private bytes, stored-interface ownership and names. An explicit
`AlkyholCreateMetadataObject32` diagnostic constructor creates a real CPU metadata
object, with UTF-32 source-ABI name input and UTF-16 LE debug-name storage. It
represents no device, fence, resource or GPU capability; its C export is Unix-only.
The original application's D3D12CreateDevice boundary continues to fail honestly.

On the physical M4 through x86-64/Rosetta, actual C++ virtual calls reuse the frozen
native observer's method sequence without changing its source or acceptance data.
Every one of the 34 records agrees with the independent WARP reference, including
buffer sentinel tails, HRESULTs, sizes and foreign IUnknown AddRef/Release/destruction
events. The native constructor is replaced explicitly in a separate host wrapper;
no native fence or backend rendering is inferred from the Rust metadata object.
[CI 42357889974](https://github.com/daeken/Alkyhol/actions/runs/42357889974)
passed all thirteen jobs on `4ea9957`. The new comparator verified a fresh native
WARP report against hosted Rust execution from the same clean revision, retaining
the unchanged observer/source hash and all frozen fields. Downloaded reports also
match the physical M4 x86-64 observation at that revision. Existing Windows SDK,
shader execution/regeneration and graphics/geometry comparisons remain passing.

Seven new Rust units cover vtable identity/lifetimes, defensive null outputs,
copy ownership/empty entries, unaligned interface outputs, UTF-32/UTF-16 names and
invalid input without partial changes, foreign callback reentrancy, concurrent
byte entries and atomic ownership. Foreign AddRef/Release callbacks run outside
the store mutex; immutable snapshots keep retained interfaces alive during reads.
Foreign interfaces still require valid thread/apartment use; no marshaling or
agility is synthesized. These extra concurrency/reentrancy/defensive cases are
host assertions, not new native observations. Null-pointer HRESULTs and empty-name
contents outside the frozen sequence remain local contracts.

All 67 root Rust tests, Clippy and 55 Python tests pass locally, as do existing
C++/WRL, CRT/platform and compiler fixtures. The complete original application
relinks against the updated library and still exits 78 at the same honest device
capability check. The pinned original checkout stays clean; working sample apps,
shaders, geometry, native reference and frozen records are unchanged.

## Native ID3D12Object metadata reference: COM runtime preparation

[CI 42356150734](https://github.com/daeken/Alkyhol/actions/runs/42356150734)
executed the independent SDK observer on `e3218ee`, using a real native WARP
fence (vendor 5140/device 140/software flag 2), Windows SDK 10.0.26100.0 and
MSVC 19.51.36260. Its 34 records capture actual private-data HRESULTs/sizes,
exact output/sentinel bytes, caller-owned IUnknown reference events and Unicode
debug-name bytes. The report identifies clean source, actual compiler/adapter,
source/executable hashes and build command. Diagnostic lifecycle events have no
manufactured HRESULT. The frozen observation canonical JSON SHA-256 is
`cbd399983e710289abfe8925243a08c2b75a6c93434e28aa0288831be572b035`.

Measured details include S_FALSE removing absent bytes, a present zero-byte entry
for non-null zero-length data, and a present pointer-sized entry for a null
interface assignment. Size queries do not AddRef; successful interface reads do.
Insufficient buffers remain unchanged, same-interface replacement performs both
reference calls, and final object destruction releases the retained IUnknown.
SetName preserves UTF-16 bytes including a terminator; an empty name has a
two-byte measured entry size.
These observations are frozen separately from a future Rust implementation;
subsequent native runs must match all records and the unchanged observer source.
See [reference contract](../cpp/reference/README.md) for the full report and limits.

The unchanged observer subsequently matched the full frozen observation in
[42356681229](https://github.com/daeken/Alkyhol/actions/runs/42356681229) on `dbb9264`
and [42356731682](https://github.com/daeken/Alkyhol/actions/runs/42356731682) on
`e2bda90`; all twelve jobs passed in each run, including existing shader/graphics
and Windows SDK comparisons. No expected data was loosened for these reruns.

The observer also compiles to a host object, and all 50 Python tests pass locally.
Those host transport/comparator tests are synthetic checks, not native API evidence.
The original C++ device-creation boundary and working sample apps are unchanged.
No Rust object metadata implementation, Metal fence, C++ initialization or original
C++ rendering is established by this new native reference.

## Complete original MiniEngine C++ compile/link: local source ABI evidence

On the physical M4 macOS host, the pinned original checkout remains clean at
`68a2b1e97f71642924e74c7757ca21756ad221b5`. Open DXC compiles all 184 original
project shader entries, including seven complete DXR libraries. The patched open
Clang build compiles **70/70 original C++ translation units** without rewriting
upstream source. Original Core and Model archives plus ModelViewer link against
Rust's x86-64 static library as a complete Mach-O application. Source, compiler,
header, object, shader-header and executable hashes/commands are recorded under
ignored `local/original-miniengine/`. Link validation rejects stale headers or
changed source/objects rather than claiming a mixed build.

The executable's host entry bridge detects unavailable D3D12 COM device creation
and returns 78 with HRESULT `80004001`. It intentionally does not enter upstream
Release code with null interfaces: original rendering and even successful original
application initialization are **not established**. No original C++ sample frame,
full DXR library translation, window/input backend or device COM implementation
is inferred from compilation. Working Rust reference sample apps are preserved.

Local C++ executables pass actual Rust IUnknown/ID3DBlob virtual calls, WRL
ownership, concurrent references, apartment nesting, all 267 DirectInput metadata
records, layout/task fixtures, CRT wide paths/secure fields/alignment/wide format
strings, UTF-32 text conversions and real kernel file metadata/read/close. The
runtime ABI fixture separately checks explicit unsupported APIs clear output
interfaces and fail; those checks are not positive API implementation evidence.
Three compiler executables cover record storage/destruction, reference/base
adjustment and actual cross-TU template linkage. Standard-mode diagnostics remain;
omitting the real template provider fails to link. All 60 root Rust tests, Clippy and all
44 Python tests pass locally. These new CPU probes use x86-64 via Rosetta, not a
Metal graphics execution path.

Native COM qualification: Windows run
[42354745967](https://github.com/daeken/Alkyhol/actions/runs/42354745967) passed the
serial blob identity/ownership phase, then terminated with heap-corruption status
`c0000374` in the uncoordinated shared-MTA reference stress. Initializing all threads
into the same MTA did not resolve that failure. This does **not** establish its
precise cause or a native guarantee of atomic blob ownership. Shared comparison
now measures explicitly synchronized client reference calls; the original
uncoordinated stress remains a separate Rust-only extension test. Neither native
failure is counted as successful parity or silently discarded. Logs are retained
as CI diagnostics. COM apartment rules are documented by
[Microsoft](https://learn.microsoft.com/windows/win32/com/single-threaded-and-multithreaded-communication);
they do not by themselves promise this object's reference implementation is atomic.

[CI 42355345904](https://github.com/daeken/Alkyhol/actions/runs/42355345904) passed
all twelve jobs on `42fff60`. Independent native Windows SDK/MSVC executables
and hosted macOS Rust executions agree on the synchronized blob probe output and
all 267 normalized DirectInput object records plus format headers. Downloaded
reports also match the physical M4 x86-64/Rosetta probe: exact actual source
SHA-256 values, clean tracked trees and revision `42fff60` agree on all three.
The input-format observation serialization (270 lines including headers/summary)
has SHA-256 `15ad1974f9c0c7eef1283edef8447e506b1102663c3c8f5612d3c9163107ddde`.
This measures public metadata and synchronized COM ownership, not native atomic
references, graphics COM behavior or original C++ rendering. Source line endings
are fixed by Git attributes and report paths use POSIX separators; the comparator
still requires identical actual source hashes rather than normalizing hash inputs. Prior intake
`0c72f0a` passed all twelve earlier jobs in
[42351798268](https://github.com/daeken/Alkyhol/actions/runs/42351798268); this predates
the complete build and cannot validate it by implication. The preceding shader
GetDimensions milestone `b1dcd0b` also passed all twelve jobs in
[42349370073](https://github.com/daeken/Alkyhol/actions/runs/42349370073).

See [original C++ build](../cpp/README.md) and its
[runtime status](../cpp/runtime-status.md) for compiler/header provenance, exact
commands and explicitly unavailable runtime entry points.

## Shader binding validation hardening: local host evidence

Five new Rust remapper unit tests cover descriptor interval boundaries, adjacent
and overlapping ranges in either order, independent namespaces/spaces, repeated
callbacks, conflicting kind/count/alignment, maximum registers/spaces, range
wraparound and unsupported arrays/counters/kinds. Conflicting duplicate callbacks
now fail explicitly rather than retaining the first declaration. A fault stops
later remapping and preserves its original error. These are synthetic host-side
metadata assertions; no new native shader semantics are inferred from them.

All 24 shader tests passed locally (five host units plus 19 integration tests,
including physical M4 execution); formatting and Clippy pass. A real DXC/DXIL
translation test accepts exactly 128 SRV texture slots with a buffer UAV, then
rejects a texture UAV that would require slot 129. The shared SRV/UAV texture
budget is a translator check, not additional native GPU boundary evidence.
All checked-in sample shader artifacts regenerated identically. The working
sample apps, source adapters, native compute fixture and frozen comparators are
unchanged.

## Same-container DXIL compute: native Windows and Metal evidence

The physical Apple M4 ran the new `dxil-compute-v1` observation probe and matched
all 96 independently expected words: 19 uint4 results and a 20-word untouched
tail. It consumes the existing compute fixture unchanged, checking matrix/cbuffer
layout, overlapping namespaces, register space 2, group-shared barriers/reversal,
unsigned wraparound and padded dispatch. Probe input/executable/DXIL hashes and
actual backend identity are recorded by the runner. All 40 Python tests and shader
formatting/Clippy pass locally; six new Python tests check validator/comparator
and subprocess failures and are synthetic assertions, not native GPU evidence.

[Run 42347852341](https://github.com/daeken/Alkyhol/actions/runs/42347852341)
passed all twelve jobs on revision `3d866ad`, including the new native compute and
comparison jobs. An independent native MSVC/Windows SDK program used explicit
D3D12 WARP, consumed the exact DXIL container produced by the hosted shader job,
and matched all 96 words against both the independent scalar contract and
translated Metal. The comparison requires matching container SHA-256, source
revision and clean tracked trees. This is actual native compute evidence, separate
from hosted Metal's lack of RT and the physical M4 observation above.

Downloaded reports also agree with the physical M4 on the exact container hash:
`e3084e25dfa23ef53e98eb1dd6eac386c3695718ac03d6631904b6711cc11d4e`.
The executed native adapter was WARP vendor 5140/device 140/software flag 2;
hosted Metal identified Apple Paravirtual device, while local Metal identified M4.

The native reference links only Windows SDK D3D12/DXGI, with no Alkyhol library,
shader rewrite or Windows-side recompilation. This finite arithmetic/binding/
barrier probe does not establish general DXIL semantics, native full shaded DXR
frames or game compatibility. Existing sample rendering paths and quality are
unchanged.

## Original Sponza material/shadow/reflection HLSL: local evidence

Physical Apple M4 passed all 18 MiniEngine tests, all 18 shader tests and all
34 Python tests. Shader/MiniEngine formatting and Clippy passed. The default
Sponza path executes original material helpers/Hit and SSR RayGen bodies through
explicit queued-hit source adapters, with native shadow/reflection queries and
three reflected arrays containing all 81 material textures. Actual fixed-array
nonuniform texture selection and the full material pipeline interface passed
separate physical Metal tests. Reverse-Z endpoint/unprojection, uint16 attribute
chunk routing and malformed/overflow/aliasing binding metadata have unit checks.
Source hashes and generated artifact equality are checked separately.

At 129×73, the original material frame observed 9,769 shadow queries/8,721 hits,
352 reflection queries/hits, five repaired tangent frames and zero short
self-reflections. Reflections changed 324 eligible floor pixels. All five source
camera presets passed the existing geometry/effect/finite-output checks. Turning
the explicit manual-MSL reference on and then off restored exact original-path
pixels and counters. This does not assert shaded original/manual pixel equality.
The source adapter's tangent repair and primary specular/view extensions remain
explicit; SSAO is neutral and bloom/adaptive exposure remain absent.

A 960×540 camera-3 release capture completed in 0.032s cold, observing 617,850
shadow queries/519,675 hits, 99,450 reflection queries/hits, 108 tangent repairs
and no short self-reflections. The rebuilt live app visibly displayed original
material shading/reflections around 59.7 FPS, and keys 2, 1 and 7 switched views
successfully. These loaded-machine UI/cold observations are not controlled
steady-state benchmarks. Geometry, texture mips and render quality were retained.

[Run 42347064101](https://github.com/daeken/Alkyhol/actions/runs/42347064101)
passed all ten jobs on material migration revision `5984b3e`. Native MSVC Sponza
attributes, Sakura placements, portable Windows SDK observations and the native
WARP checker comparisons continue to agree. Hosted Metal compute/raster tests,
source extraction and exact shader regeneration also passed; that GPU lacks RT.
Native full shaded DXR frame parity and general DXR library translation remain
unmeasured/unimplemented. The preceding Sakura revision `581845c` also passed all
ten jobs in [42344311350](https://github.com/daeken/Alkyhol/actions/runs/42344311350).

## Original Sakura HLSL bodies: local evidence

The physical M4 passed all ten Sakura tests and all sixteen shader tests; all
34 Python tests, source extraction/regeneration checks, and shader/Sakura
formatting/Clippy also passed. Original closest-hit, reflection, camera and
miss/Star Nest bodies execute through a bounded SER-off inline-query adapter.
The complete scene remains 33,983 unique triangles and 13,525 instances; the
sky retains 200×50 iterations. Buffer tests cover the original 36-byte Vertex,
indexed cube and zero-padded original leaf-index reads used by the trunk shader.
An original helper compute fixture verifies interpolation/Fresnel/world-position
and a counted fault/NaN payload when the depth contract is violated.

The default 129×73 original/manual observations had identical hit/reflection
counts but 642 different RGBA8 pixels; no pixel-equality claim is made. Original
HLSL's +X Star Nest observation was 0.8959766 on this M4, separate from earlier
manual-MSL and host scalar observations. No tolerance was widened to equate them.
A full 960×540 original-shader frame was saved locally in 0.046s cold; the live
window visibly rendered around 42 FPS at that resolution. These are live/cold
observations, not an isolated steady-state benchmark or native DXR comparison.

The [adapter record](../shader/samples/sakura/README.md) details source extraction,
bounded recursion/context restoration, local-root/payload specialization and
unsupported-SER failure behavior. Full DXR binary/library translation, SER and
native Windows full-frame DXR comparison remain unverified.

[Run 42342947047](https://github.com/daeken/Alkyhol/actions/runs/42342947047)
passed all ten jobs on original tone/display revision `0cf9620`. That hosted
evidence precedes this Sakura migration and does not imply hosted ray tracing.

## Original tone/display HLSL: local evidence

The physical M4 passed all 15 MiniEngine and 14 shader tests, plus all 31 Python
tests, shader regeneration equality and shader/MiniEngine formatting/Clippy.
Unmodified original tone mapping, SDR display, fullscreen vertices, copies and
shaded live upsampling now execute through the open translation components.
The independent 13×7 GPU test checks exposure 2, the Stanard/sRGB curve, opaque
RGBA capture alpha and log luminance. Shader translation now explicitly pads RGB
outputs for Metal RGBA attachment formats rather than modifying the HLSL.

The full 960×540 shaded/reflection capture was saved locally and the rebuilt
live app visibly rendered around 59.8 FPS at that resolution. The cold headless
frame took 0.051s; neither observation is an uncapped steady-state GPU benchmark.
HDR intermediates use RGBA32Float instead of packed RGB; bloom strength stays 0
and adaptive exposure remains absent. Geometry and shading quality were not reduced.

[Run 42340939662](https://github.com/daeken/Alkyhol/actions/runs/42340939662)
passed all ten jobs on the initial shader revision `b1d643b`.
[Run 42342151549](https://github.com/daeken/Alkyhol/actions/runs/42342151549)
passed all ten jobs on original ray-generation/barycentric revision `d3cf021`.
The hosted shader/Metal results are distinct from physical RT tests above.
These runs precede the local tone/display milestone and do not validate it by
implication. At that revision, original shaded-hit/Sakura migration and native full DXR pixel
comparison remained outstanding; subsequent source migrations are recorded above.

## Original Sponza HLSL migration: local evidence

The physical Apple M4 passed 13 MiniEngine tests after adopting original HLSL
primary-ray generation in every mode and original hit/miss bodies for view 2.
Five-preset A/B tests require exact RGBA8 pixels and per-mesh/miss counts against
the manual Metal intersector using the same generated rays. A separate compute
test verifies the original camera cbuffer, ray records and padded-dispatch guard;
it can run without RT. Source hashes and generated artifacts are checked separately.

All 13 shader tests, all 31 Python tests, regenerated shader equality, rustfmt
and shader/MiniEngine Clippy passed locally. The rebuilt live app showed view 2
and switched back to shaded/reflections; its observed title was around 59 FPS at
960×540. That is an overall live-view measurement, not an uncapped GPU benchmark.

The compiler path is entirely open source (DXC, dxil-spirv, SPIRV-Cross and Rust
adapters); the user explicitly excluded Apple's account-gated converter. The
adapters' fixed source contracts are described in [shader intake](../shader/samples/miniengine/README.md).
At that revision, original shaded-hit and Sakura migration remained outstanding;
the later records above supersede that status. Full DXR library translation and
native Windows full-frame DXR comparison remain outstanding.

## First complete milestone

[Conformance run 42326374803](https://github.com/daeken/Alkyhol/actions/runs/42326374803)
on revision `347e88767ffd114dfb6ac53e971d4603d00bf6f7` completed with all six jobs
passing. These are executed results, not expectations inferred from workflow YAML.

| Check | Actual host/backend | Result |
| --- | --- | --- |
| Core Rust tests | macOS, Ubuntu Linux, Windows | 16 passed on each host (11 kernel, 5 command-state/footprint) |
| PE parser tests | Python on all three hosts | 7 passed on each host; fixtures cover PE32/PE32+, normal/delay and legacy-VA imports, named/ordinal entries and malformed input |
| Native C contract probe | Alkyhol shared library on macOS and Linux | Passed |
| Same C source, Windows SDK | Native Windows runner, MSVC/kernel32 | Passed |
| Differential observation comparison | Separate Ubuntu job downloading all three observations | Exact JSON agreement |
| Offscreen rendering | Metal; `Apple Paravirtual device`, IntegratedGpu, hosted macOS | Clear/copy, padded readback, triangle pixels, replay, invalid states and invalid shader all passed |
| Offscreen rendering | Vulkan; llvmpipe LLVM 21.1.8, Cpu, hosted Linux | Same checks passed; explicitly requested software adapter |
| Style/static checks | Root crate on all three hosts; graphics on macOS/Linux | rustfmt and Clippy with warnings denied passed |

Local development also executed the core tests, parser tests and native C sample
on macOS 27.0.1 ARM64 with Rust 1.99.1 and Apple Clang 21.0.0. Local GPU compilation
was blocked by unavailable cached crates and terminal network restrictions; GPU
execution above happened on the recorded CI hosts, not this developer machine.

The kernel sample checks file contents/counts/EOF, creation error codes, truncation,
closed handles, event saturation/reset/timeout and clock monotonicity. It prints
only deterministic booleans after its assertions; it does not compare absolute
clock epochs, scheduler precision or unspecified last-error values. Sample runner reports from [run 42326549684](https://github.com/daeken/Alkyhol/actions/runs/42326549684)
on revision `1f2135c647b5617b6213a97c3257930e0a017ee6` additionally bind compiler identity, revision, source and
executed library hashes. Artifacts belong to their exact source revision.

## Limits

This is sample-level evidence for the restricted contracts in
[contracts.md](contracts.md) and [graphics.md](graphics.md). The raster sample has
no native D3D12 differential reference. Hosted Metal uses Apple's paravirtual GPU,
not an identified physical Apple Silicon GPU. Software Vulkan evidence is labeled
separately. No game execution, exhaustive API coverage, binary ABI compatibility,
COM compatibility, general HLSL/DXIL translation or game equivalence has been established.
The newer restricted shader milestone is recorded in [shaders.md](shaders.md).

## Expanded tests and Microsoft sample ports

On macOS 27.0.1 ARM64, Rust 1.99.1, Apple Clang 21.0.0 and a physical Apple M4
Metal adapter, local execution passed 32 root Rust tests, 18 Python tests, four
real-device integration tests, all three portable C/C++ samples and the original
clear/triangle/readback example. Root and graphics Clippy checks with warnings
denied passed. The [test map](testing.md) records what each function/unit exercises.

The new [Microsoft ports](../samples/msdn/README.md) cover synchronous file writing
with a readback check and event-gated readers. They include pinned provenance and
MIT attribution. Deliberate API substitutions are documented; the separate upstream
asynchronous file read example is not ported.

[Conformance run 42329817238](https://github.com/daeken/Alkyhol/actions/runs/42329817238)
on revision `20f8785587b05e8330634fdf3d87f2657729b6d8` completed with all six jobs
passing. Downloaded sample reports bind that revision and clean tracked trees.

| Executed check | Host/backend | Result |
| --- | --- | --- |
| 32 root Rust and 18 Python tests; style/static checks | macOS 26.6.2 ARM64; Linux x86_64/glibc 2.43; Windows Server 2025 AMD64, build 26100 | Passed on each host |
| Three portable C/C++ samples | Windows SDK/kernel32; Alkyhol native C ABI on macOS and Linux | Each executed; full `samples` mapping agreed exactly in the comparison job |
| Four GPU integration tests plus clear/triangle/readback sample | Hosted Metal, `Apple Paravirtual device`, IntegratedGpu | Passed |
| Same four GPU integration tests and drawing sample | Explicit software Vulkan, llvmpipe LLVM 21.1.8, Mesa 26.0.8, Cpu | Passed |

The physical Apple M4 result above is separate local evidence. Hosted Metal and
software Vulkan results are labeled as such. No native D3D12 graphics comparison
or additional game compatibility is established by this run.

## Unnamed counting semaphore milestone

[Conformance run 42330264837](https://github.com/daeken/Alkyhol/actions/runs/42330264837)
on revision `4da84e503cba3407b7264ce9f4ad77c7947ec19a` completed with all six jobs
passing. This adds CreateSemaphoreW, ReleaseSemaphore and semaphore waits to the
restricted kernel contract, plus a pinned adaptation of Microsoft's semaphore
worker example. All four portable C/C++ samples executed on Windows SDK/kernel32,
native macOS Alkyhol and native Linux Alkyhol; the complete observations agreed
exactly. Downloaded reports bind the same revision with clean tracked trees.

The semaphore sample checks ten permits shared by twelve workers, zero-timeout
acquisition, returned permits, previous counts, invalid creation/release counts
and maximum-count overflow errors. It asserts an upper bound on active workers,
not a minimum degree of concurrency, waiter order or fairness. C++ threads/join
remain harness substitutions for unsupported Win32 thread APIs.

There are now 39 root Rust tests (seven new semaphore tests), 18 Python tests and
four real-device integration tests. The CI root tests passed on all three hosts;
GPU tests and the drawing sample passed on hosted Metal and explicit software
Vulkan. Local macOS 27.0.1 ARM64/Rust 1.99.1 also passed those tests and all four
portable samples, with GPU execution on physical Apple M4 Metal. Root Clippy with
warnings denied passed locally; CI root and graphics static checks passed.

Named objects, security descriptors, handle inheritance, cross-process semaphores,
access masks and closing a semaphore while another thread waits remain unsupported.
The corpus and these samples do not establish game compatibility.

## Dynamic thread-local storage milestone

[Conformance run 42330786584](https://github.com/daeken/Alkyhol/actions/runs/42330786584)
on revision `e231f9e77f7336d1612b3c71400d07ef76a259dc` completed with all six jobs
passing. The four new exports are TlsAlloc, TlsFree, TlsSetValue and TlsGetValue.
The Microsoft TLS adaptation verifies four workers' separate 256-byte payloads
through a common retrieval function, null initialization, valid null values,
TlsGetValue clearing last-error, main-thread isolation and out-of-range errors.

All five portable samples executed against Windows SDK/kernel32 on Windows Server
2025 build 26100 AMD64, Alkyhol on macOS 26.7.2 ARM64, and Alkyhol on Linux
x86_64/glibc 2.43. Their complete observations agreed exactly. Downloaded reports
bind the same source revision and clean tracked trees. A clean local report on
macOS 27.0.1 ARM64 also matched those observations.

There are now 43 root Rust tests, 18 Python tests and four GPU integration tests.
Root tests and style/static checks passed on all three CI hosts. The GPU tests
and drawing sample passed on hosted Metal and explicit software Vulkan, and
locally on physical Apple M4 Metal. Local Rust was 1.99.1 and C/C++ used Apple
Clang 21.0.0. The four new host assertion tests additionally cover pool exhaustion,
forced slot reuse in an existing thread, opaque address bits and stricter rejection
of unallocated indices. These are separate from native Windows sample evidence:
the portable sample does not compare exhaustion capacity/error, allocation order,
in-range unallocated reads, or forced index reuse.

Dynamic TLS does not implement PE static TLS, DLL initialization/teardown, FLS,
thread creation, memory allocation or pointer ownership. Stored addresses remain
caller-owned host values. Full slot-lifetime rules and deliberate limits are in
[the kernel contracts](contracts.md). No game compatibility is established.

## Native thread callback milestone

[Conformance run 42331502760](https://github.com/daeken/Alkyhol/actions/runs/42331502760)
on revision `925f8f914bd73e5b2390a56ddfbf51fad8120a70` completed with all six jobs
passing. CreateThread now starts a native host callback, GetCurrentThreadId returns
its opaque Alkyhol identity, GetExitCodeThread queries pending/returned status, and
WaitForSingleObject accepts thread handles. Completion remains signaled, including
when a completed callback returns the numeric STILL_ACTIVE value (259).

The pinned Microsoft Creating Threads port executes three actual CreateThread
callbacks with separate parameter records, an event gate, individual finite waits
and explicit handle cleanup. It checks parameter writes, callback IDs against the
creation outputs, pending status, returned DWORD values and repeated completion
waits. All six portable samples executed against Windows SDK/kernel32 on Windows
Server 2025 build 26100 AMD64, Alkyhol on macOS 26.6.2 ARM64, and Alkyhol on Linux
x86_64/glibc 2.43. Their complete observations agreed exactly. Downloaded reports
bind the same revision and clean tracked trees; a clean local macOS report agreed.

All 52 root Rust tests and 18 Python tests passed on each CI host. Nine new host
thread tests include pending/completed/error paths, optional/null outputs, early
handle close, distinct foreign/created thread IDs, returned 259, and a deliberately
blocked Rust TLS destructor proving callback return alone does not publish
completion. A safe Rust worker panic must produce an explicit host error. These
host tests are separate from the Windows SDK sample; the panic path does not
establish native Windows exception handling or DLL callback semantics.

Four GPU integration tests and the drawing example passed on hosted Metal
(`Apple Paravirtual device`) and explicit CPU Vulkan (llvmpipe LLVM 21.1.8).
The same checks passed locally on physical Apple M4 Metal, macOS 27.0.1 ARM64 and
Rust 1.99.1; all six C/C++ samples also passed with Apple Clang 21.0.0. Root local
Clippy and CI root/graphics static checks passed with warnings denied.

The initial thread backend uses a detached host monitor per callback to join the
worker, then publish completion after host TLS destructors. This adds resources
and does not emulate Windows stack reserve/commit, system-wide IDs, DLL lifecycle,
security, inheritance, suspended creation, priority or abnormal termination. Existing
event/semaphore/TLS ports still use their documented C++ thread substitutions;
the new Creating Threads port provides the direct thread-API comparison. No game
compatibility or universal API equivalence is established.

## Whole-resource GPU copy milestone

[Conformance run 42332065590](https://github.com/daeken/Alkyhol/actions/runs/42332065590)
on revision `6093e2d74ea5d2287e61e7d7f56115d3f6570a48` completed with all six jobs
passing. The Rust graphics facade now records CopyResource for distinct matching
RGBA8 2D textures, with explicit CopySource/CopyDest states. It issues an actual
wgpu texture-to-texture GPU copy. Identity, state and both dimension checks finish
before any list command is encoded or submitted.

Three new root tests exercise copy state/token failures, sequential copy chains
and rejection after earlier logical transitions. Two real-device tests compare
every pixel after chained 65×3 copies and reject width/height mismatches after an
earlier recorded clear. They verify source pixels and destination state are still
unchanged, including a subsequent valid transition from the original state.
The new `copy_resource` example first renders and captures a patterned 65×65
triangle, then compares the entire copied destination and original source bytes.
It is original project code guided by the API docs, not a Microsoft sample port.

All six GPU tests and both graphics examples passed locally on physical Apple M4
Metal (macOS 27.0.1 ARM64, Rust 1.99.1), on hosted Metal (`Apple Paravirtual device`),
and on explicit software Vulkan (CPU llvmpipe LLVM 21.1.8, Mesa 26.0.8).
These are separate backend results. There is no native D3D12 copy/raster reference
in this milestone, no generalized format/subresource support and no game evidence.

The 55 root Rust tests and 18 Python tests passed on Windows, macOS and Linux CI,
with style/static checks passing for both crates. All six existing kernel C/C++
samples still executed on Windows SDK/kernel32 and native macOS/Linux Alkyhol;
their complete observations agreed. Downloaded reports bind this exact revision
and clean tracked trees, as does a matching local C/C++ report. Those kernel
comparisons do not validate the new GPU copy operation against D3D12.

## First native D3D12 pixel comparison

[Conformance run 42332990445](https://github.com/daeken/Alkyhol/actions/runs/42332990445)
on revision `ac4dfd463c7572912fdf31d5f407b10c52163e83` completed with all eight jobs
passing, including the new native Windows reference and graphics comparison jobs.
The independent Windows SDK program used D3D12 COM calls with explicit WARP,
compiled original HLSL through D3DCompile, rendered a 65×4 multichannel checkerboard,
called CopyResource and read both surfaces after a queue fence. It did not link
Alkyhol. Every one of the 1,040 source and 1,040 destination RGBA8 bytes agreed with
Alkyhol Metal and explicit software Vulkan, as well as the fixed scenario contract.

| Execution | Observed identity |
| --- | --- |
| Native Windows D3D12 | Windows Server 2025 build 26100; WARP vendor 5140, device 140, software flags 2; MSVC 19.51.36260 x64; Windows SDK 10.0.26100.0 |
| Hosted Metal | macOS 26.7.2 ARM64; Apple Paravirtual device, IntegratedGpu; Rust 1.99.1 |
| Explicit software Vulkan | Linux x86_64/glibc 2.43; llvmpipe LLVM 21.1.8, Mesa 26.0.8, Cpu; Rust 1.99.1 |
| Local physical Metal | macOS 27.0.1 ARM64; Apple M4, IntegratedGpu; Rust 1.99.1 |

Downloaded reports bind the exact revision, clean tracked trees, compiler/SDK and
adapter context, source hashes and executed-program hashes; the clean local
report agreed too. The actual full observations were compared, without reducing
pixels to checksums. All 55 root Rust tests, 26 Python tests and six GPU integration
tests passed on their applicable CI hosts; the two earlier graphics examples and
six existing kernel samples also passed. Eight new Python tests include exhaustive
single-channel perturbation of these two finite surfaces and synthetic process
faults; those tests are not GPU evidence.

This establishes agreement for the fixed [checkerboard render/copy scenario](graphics-reference.md),
using different handwritten HLSL and WGSL frontends and the labeled backends. It
does not establish HLSL/DXIL translation, general raster/copy equivalence, COM facade
compatibility, native invalid-command behavior or game compatibility. WARP and
llvmpipe are software renderers; runtime DLLs and driver binaries are not pinned.

## Reproduce

```sh
cargo fmt --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --offline
python3 -m unittest discover -s tests -v
python3 tools/run_samples.py --output local/kernel-observation.json
cargo fmt --manifest-path graphics/Cargo.toml --check
cargo clippy --manifest-path graphics/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path graphics/Cargo.toml --locked -- --nocapture
cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
```

Create `local/` before requesting a report there. Graphics commands need access to
the crates in their lockfile and a usable backend. Linux without hardware must add
`-- --allow-software` and provide Vulkan/llvmpipe. A missing backend or timeout fails.

## Direct Metal migration, 2026-11-07

Revision `818cfe8` retired wgpu, Naga and the Vulkan graphics path. Local physical
Apple M4/macOS 27.0.1 ARM64/Rust 1.99.1 passed 55 root tests, 26 Python tests,
seven GPU tests, graphics Clippy and the unchanged v1 full-pixel contract. CI run
[42334642328](https://github.com/daeken/Alkyhol/actions/runs/42334642328)
passed all seven jobs, including exact agreement of all 1040 source and 1040 copied
RGBA8 bytes between independent native Windows SDK D3D12 WARP and hosted direct
Metal. The hosted Apple Paravirtual device reports ray tracing **false**; it does
not validate Sakura ray tracing. Physical M4 ray-tracing tests are separate and
require an explicit `--include-ignored` invocation; absence is not a passing ray test.

## Running Sakura Forest on physical Metal, 2026-11-07

Code revision `b72e0790ee8b785d0399ad044cf820d62b8901eb` runs the complete
SORTMODE_OFF rendering port in Rust/MSL. All five public sample assets are pinned
and hash-verified; four BLAS contain 33,983 original triangles, instanced 13,525
times through a native Metal TLAS. The code retains original scene, texture color
spaces, material quirks, two-level reflections and 200×50 fractal sky iterations.

On physical Apple M4, macOS 27.0.1 ARM64 and Rust 1.99.1:

- Eight graphics integration tests passed with `--include-ignored`, including real
  native ray hits/misses, scaled/translated instances, barycentrics and masks.
- Seven Sakura integration tests passed with `--include-ignored`: five CPU/asset
  tests, CLI rejection/help and the actual full-scene rendering test. Repeated PNG
  pixels matched exactly at fixed camera/light; moving the light changed pixels;
  the close camera observed 1,359 primary reflective-cube hits and reflection rays.
- The 129×73 default test observed 687 floor reflection rays, 663 secondary hits and
  24 secondary misses. All primary rays were accounted for and floor output alpha
  stayed opaque; 257×257 close-camera testing exercised cube attenuation/recursion.
- A release 960×540/time-zero capture observed primary counts
  `[151825, 0, 153240, 135415, 60987, 16933]` for floor, reflective cubes, trunks,
  leaves, bushes and sky; 37,272 floor reflections led to 35,837 secondary hits and
  1,435 misses. The initial view has no primary reflective-cube hits; the dedicated
  camera test covers them instead. PNG SHA-256:
  `118e5c5ba39d3488bd0810b71716db06842244bd0ce4c17e7990e7db6b1e225d`.
- The native app window visibly displayed the forest. Held Right/W keys changed
  the camera view. A final release build with correct trunk sRGB handling was
  observed at about 16 FPS at 960×540. This is an observed UI rate under current
  machine load, not a controlled performance benchmark.

Raw logs, source/asset/executable hashes and the capture stay in ignored
`local/sakura/physical-metal-report.json` and `local/sakura/metal-sakura.png`.
Use [the launch/test instructions](sakura.md#running-the-port) to reproduce.

CI run [42335656952](https://github.com/daeken/Alkyhol/actions/runs/42335656952)
passed all eight jobs on that code revision. Native MSVC and hosted macOS Rust
produced identical 662,725-byte placement observations: every material/mesh ID and
scale/position float bit across all 13,525 instances. Both artifact SHA-256 values
are `0e9cfe7ea4a45fd374fb20ef10ba3df8b6e83f365e9beafe06170e009a03a6a3`.
The same run also passed all three hosts' root/kernel sample comparisons, 28 Python
tests and exact native D3D12 WARP/direct Metal checker-copy pixels. Hosted Metal
CPU/asset tests passed; its RT integration test was explicitly ignored because
that device reports no ray-tracing support. This is not hosted Sakura GPU evidence.

No full Sakura native DXR capture has been measured, so there is no claim of
Windows/Metal full-frame pixel equivalence. WIC/Rust JPEG rounding and cross-GPU
floating-point behavior remain comparison work. DirectX SER scheduling modes are
not implemented; the UI says SER off. This source-level application port does not
run original Windows/game binaries and is not evidence of game compatibility.

## Sakura geometry audit and shader units, 2026-11-07

On the same physical M4/macOS 27.0.1/Rust 1.99.1, the full-scene intersection audit
compared 58 Metal nearest-hit queries with an independent CPU slab/Möller–Trumbore
triangle implementation. Queries cover a pixel grid and directions toward nearby
trunks, considering all 13,525 instances as potential occluders. All 58 agreed on
nearest distance; the reported GPU triangle was independently intersected too,
allowing coincident triangles at the same depth. Material counts were
`[11, 1, 32, 13, 1, 0]`. This establishes the queried geometry only, not every view,
all trunks or native Windows/DXR equivalence. It did not reproduce a depth error.

The pinned trunk OBJ contains 17,970 triangle faces and 479 quad faces. Upstream's
ObjModelLoader reads only the first three corners of each face; the port retains
that behavior. This drops the second triangle of each quad. This is a possible
source of visible holes, not a demonstrated explanation for every reported missing
trunk. No default asset/geometry repair was made.

Shader arithmetic helpers now have real Metal compute checks for all five normal
buffer paths, the leaf-index bounds behavior, unnormalized interpolation, original
cube/trunk index mixing, Schlick endpoints/interior/clamping, Euclidean triplanar
weights (including zero), and all three petal colors. These analytic fixtures are
shader-unit assertions. They require Metal compute, not RT hardware. Full scene
rendering after extraction produced the same 960×540/time-zero PNG SHA-256 as the
running milestone: `118e5c5ba39d3488bd0810b71716db06842244bd0ce4c17e7990e7db6b1e225d`.
All nine Sakura integration tests passed locally with `--include-ignored`, plus
Sakura Clippy. Raw output is ignored under
`local/sakura/shader-and-geometry-audit-tests.log`.

A separate exploratory full-fractal scalar-host comparison **did not agree** at an
attempted 0.001 absolute tolerance. For directions +X/+Y/−Z, GPU densities were
`[0.8959779, 0.91883063, 0.63184154]`; independent scalar Rust f32 observations were
`[0.936103, 0.936103, 0.7602065]`. The discrepancy is unresolved, and no larger
acceptance tolerance was adopted. The shader test records these observations and
asserts only the finite density range for this part; it does not count them as
host/GPU equality or native DXR evidence. Original sky math, loop counts and the
working full-frame output are retained. The user clarified that the purple/noisy
sky matches the sample screenshot; the remaining visual concern is trunk coverage.

## MiniEngine Sponza geometry diagnostics, 2026-11-07

Physical M4/macOS 27.0.1/Rust 1.99.1 rendered the original pinned Sponza H3D with
34 meshes, 27 materials and 278,919 triangles. Six MiniEngine integration tests
passed explicitly including pinned assets and actual GPU rendering: three parser/
asset tests, camera, CLI and full-scene view/capture/resize checks. All four views
accounted for the same primary rays and repeated full pixels matched at fixed
view. Original Bary Rays was observed in a live window around 59 FPS at 960×540;
UI switching from 2 to 1 visibly returned to Geometry diagnostic after the controls
fix. The neutral diagnostic is not the original raster/material shading mode.

Two physical graphics ray tests passed, including the new two-mesh far-first /
near-second instance scene and both directions with ray backface culling enabled
or disabled via instance override. Existing Sakura instances retain their prior
culling behavior. The original Sakura geometry/shader audit code revision
`fa757f6` passed all eight existing CI jobs in run
[42336806679](https://github.com/daeken/Alkyhol/actions/runs/42336806679).

The Rust H3D stream contains 46,858,664 bytes, including material/count headers and
every expanded original vertex attribute bit (including unused NaN tangents).
Local SHA-256: `0d937fea9ee97a5f38a8dcec326ae0ce48b154d36613a70358cf1c136478f51e`.
The original H3D asset SHA-256 is
`c52e851f67a37892acf229df7cce8e2b8aff997a087bc41fa8c48beb73b45747`.
A native MSVC mirror of the pinned serialized structs asserts the SIMD record
sizes and independently expands the uint16 index stream. Its CI comparison is
newly defined; no native result is claimed until that job actually completes.
There is no full MiniEngine native DXR pixel comparison yet. Textured lighting,
shadow/reflection modes and postprocessing remain subsequent work requested by
the user; diagnostic rendering does not count as those effects being implemented.

## MiniEngine native intake comparison and shaded Metal effects, 2026-11-07

Revision `2ec2207` passed all nine CI jobs in
[42338140310](https://github.com/daeken/Alkyhol/actions/runs/42338140310).
Native MSVC and Rust agreed on all 46,858,664 expanded Sponza geometry/attribute
bytes, SHA-256 `0d937fea9ee97a5f38a8dcec326ae0ce48b154d36613a70358cf1c136478f51e`.
This upgrades the earlier pending native ABI comparison to observed agreement;
it is not native DXR image evidence. The first run's strict MSVC warning about
implicit trailing alignment padding was fixed by declaring the serialized padding
explicitly, without changing the record size or decoded bytes.

The physical M4 now renders textured lighting, normal/specular maps, native shadow
rays and floor reflection rays. All 50 original DDS mip chains pass parsing; GPU
fixtures sample three distinct uploaded BC1 mips and the original flat-normal
default. Additional actual Metal compute tests cover valid, NaN, mirrored-UV and
degenerate-UV tangent frames, ambient/shadow endpoints and the source tone curve.
The H3D data remains unchanged; renderer-only invalid-frame repair is explicit.

At 129×73, the full-scene effect test observed 9,769 shadow queries, 8,721 shadow
hits, 352 reflection queries/hits, five tangent-frame repairs and zero short
self-reflection hits. Reflections changed 324 floor pixels relative to the otherwise
identical lit frame; primary mesh/miss counts agreed across all views. Shadows
changed the unshadowed image, and their diagnostic mask contained binary visibility.
All five original camera presets also rendered the reflected scene successfully.
Nonfinite output fails rendering; it is not accepted as a black-pixel fallback.
These are local physical Metal assertions, not native Windows effect comparisons.
Source/raster-derived differences and remaining postprocessing are documented in
[miniengine.md](miniengine.md#textured-lighting-shadows-and-reflections).

The shaded code revision `001ac6e8213b724f5c93c31ffc4862704475c26e` passed all
nine jobs in [42339198654](https://github.com/daeken/Alkyhol/actions/runs/42339198654).
This includes actual BC1 mip sampling and shader arithmetic on hosted Metal
(`Apple Paravirtual device`, RT false), all original DDS chain checks, and continued
native MSVC/Rust geometry-byte, Sakura placement and SDK WARP checker-copy agreement.
Hosted compute evidence does not validate shadow/reflection traversal; those
full-scene RT tests were explicitly run on physical M4. All ten MiniEngine
integration tests passed locally with `--include-ignored`, plus Clippy and 29 Python
tests. The refreshed live native app visibly displayed material lighting and floor
reflections at approximately 59.6 FPS at 960×540. Actual key 1 / key 7 UI actions
switched the title/view between shaded lighting and reflections. The window scales
the render texture; fullscreen does not imply native display pixel resolution.
This observed presentation rate is not a controlled uncapped performance claim.

A separate fixed camera-preset-3 release capture at 960×540 observed 617,850 shadow
queries, 519,675 shadow hits, 99,450 reflection queries/hits, 108 tangent repairs and
zero short self-reflections. Raw source/asset/executable hashes and diagnostics are
ignored in `local/miniengine/physical-shaded-report.json`; output is
`local/miniengine/shaded-reflections.png`. Native full shaded DXR pixels and the
remaining original postprocessing passes have not been validated or implemented.
