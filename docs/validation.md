# Executed validation

D3D12 graphics uses direct Metal from Rust; the newer D3D9 layer uses wgpu by
explicit user choice. Historical records below describe their exact revisions,
including the earlier retired wgpu renderer. See d3d9.md for current D3D9 evidence.

## Search family and physical native-window backend during the integrated push

At clean `009e4a4`, original DAO proceeds through lexical path resolution to a
wildcard module-directory search. Real A/W enumeration now snapshots actual
entries, preserves file/directory metadata, handles all-files/extension masks,
shares handles across A/W calls and distinguishes absent parent, absent match
and end-of-enumeration. Exact-file frozen observations remain unchanged. An
owned six-case PE and one filesystem unit pass; native wildcard comparison is
pending. Sleep forwards bounded non-alertable waits to the real host scheduler.

The native full-path probe at `009e4a4` reports two sizing-query discrepancies
for normalized W paths and clearing of the first WCHAR on an insufficient
nonzero buffer. The latter is corrected; the probe now captures actual required
size deltas rather than losing them as booleans. Exact W sizing remains pending
that native measurement. Original DAO's ordinary successful full-buffer queries
have executed; no claim of complete full-path parity is made.

The direct AppKit backend is independently exercised on the physical M4 under
Rosetta by `pe/probe_window_host.rs`: an owned window is visible in actual
WindowServer metadata, has requested 640x480 client bounds, moves/resizes,
changes its title and hides. This is a backend probe, **not DAO window creation**.
The guest Win32 class/window adapter is still being integrated. Gateway snapshots
now accommodate sixteen DWORD arguments and synchronous callbacks up to sixteen
DWORDs, with checked suspended-stack frames for future window procedures. Old
integer/FLS paths and Clippy pass. The user's requested checkpoint remains a
real original-game window and an honest D3D9 initialization stop.

## Integrated non-D3D9 startup work: persistent settings and core API families

The user's current goal is one integrated non-D3D9 implementation through the
original program's real window creation and explicit D3D9 initialization stop.
Individual API milestones are implementation checkpoints, not completion of that
request. No real DAO window or D3D9 boundary has yet been observed.

The first bulk batch adds 15 exports: `WritePrivateProfileStringW`,
`ReleaseSemaphore`, `CreateEventA`, `CreateSemaphoreA`, `InterlockedExchangeAdd`,
`GetFileSize`, `GetFileSizeEx`, `GetCurrentThread`, `CreateDirectoryA`,
`SetCurrentDirectoryA`, `GetCurrentDirectoryA`, `GetFileAttributesW/A`,
`WaitForSingleObject` and `CreateFileA`. Existing scalar INI reads now parse real
owned files instead of rejecting every existing file. State is shared with the
existing checked directory, synchronization and file gateways. ANSI paths remain
ASCII-only; file opens retain the existing read-only/share-all/normal-attribute
profile, and waits retain the zero-timeout/non-alertable profile. This does not
claim all imports, threading, windowing or D3D9 are implemented.

Seven successive original owned PEs were observed on native Windows before the
writer implementation in
[42524988291](https://github.com/daeken/Alkyhol/actions/runs/42524988291).
The unmodified report is `pe/reference/pe32-profile-roundtrip-windows-v1.json`,
SHA-256 `f114d548fc190551f33d1eb30fc1e1e918fff0e4e7b11611760a5af18a015b38`.
Physical M4 selects the same literal seven PEs and matches every 168-byte output
and every intermediate INI file byte. Case-insensitive updates preserve original
section/key spelling and order; empty values persist, and new keys are inserted
before the next section. Read LastError is zero and write LastError retains the
sentinel in these observed cases. The grammar is bounded ASCII/CRLF without
comments, duplicates, quoted/edge-whitespace values or Unicode encodings. Unknown
syntax fails before rewriting; it never becomes an absent value or a default.
These contracts follow the measured scalar profile of
[WritePrivateProfileStringW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-writeprivateprofilestringw).

INI access is anchored to the explicitly selected Documents capability through
checked no-follow parent/file handles, identity/link-count checks and real file
locks. Completed writes are flushed to disk. Five host restrictions verify no
mutation on missing profile, outside target, unknown syntax, symlink or hardlink.
A separate stateful core-family PE checks real event consumption, semaphore
release/error atomicity, interlocked wraparound, shared ANSI directory state,
file sizes/attributes and existing-file missing-key reads; eight exclusions
check handle kinds, checked pointers, alignment, names and missing path state.
At clean `04ec60e`, the combined original DAO run creates its real settings
file and continues through the newly supported ANSI current-directory call to
`GetFullPathNameW`. This is progress toward the requested window/D3D9 boundary,
not completion of that request. The native core-family PE executes successfully
in [42526478743](https://github.com/daeken/Alkyhol/actions/runs/42526478743), but
its first comparator catches missing-key/section LastError **2**, differing from
existing-value LastError 0. The runtime and expected finite observations now
preserve that measured distinction; raw first results remain local. Native
rerun is pending. `GetFullPathNameW/A` now use the real process-owned directory,
perform lexical dot/parent normalization without inventing file existence, and
check A/W buffer sizing and file-part pointers. Owned A/W execution and two
logical units pass; its native comparison is pending. Windows creation and
D3D9 initialization are still unobserved.
Local checks pass: 219 Rust PE checks including doctests, 410 Python tests,
Clippy/formatting, the exact settings lifecycle, the core-family PE and prior
missing-INI/current-directory comparisons. Original game inputs/inventories stay
local; this changes no D3D9 implementation and makes no game compatibility claim.

## Original DAO completes profile-directory setup and reaches INI writes

At clean revision `6d242f1`, physical M4 executes the unchanged original EXE's
actual `CreateDirectoryW` prefix walk and creates real directories under its
explicitly selected fresh Documents backing. Existing ordinary ancestors are
observed without a write outside that profile. At clean `7f27cd5`, the final
separator on an existing directory is accepted and original startup reaches
**KERNEL32.dll!WritePrivateProfileStringW**, which remains unsupported. Existing
INI parsing also remains unsupported; no settings-file success is fabricated.
No window/menu/rendering or successful gameplay is observed. Original hashes,
private paths/arguments/tree and complete captures remain local, with prior
fresh-profile and locally unpacked dependency caveats unchanged.

The [native startup job at `7f27cd5`](https://github.com/daeken/Alkyhol/actions/runs/42523873815)
passes the expanded original owned directory-constructor PE. Physical M4 selects
that run's seven literal PE artifacts with identical complete producer/PE hashes.
**All 96 positive-observer bytes (24 DWORDs)** agree with native Windows, including
actual creation, nesting, existing ancestors, duplicates, file collisions,
missing/non-directory parents and existing/new paths with a final backslash.
Independent filesystem checks confirm the returned directory state, absence of
implicitly created parents and unchanged collision-file contents. The raw report
is `pe/reference/pe32-create-directory-windows-v1.json`, SHA-256
`c151a3b75b6f8fb3db741a35dfa567309891418f0fd993e5e107689b9ad1ebab`.
Success preserves the supplied LastError sentinel in these three observed calls;
this exact finite capture is retained without asserting a general documented
success-LastError guarantee.

Creation is restricted to exact ASCII descendants of the selected Documents
backing. The held root and each parent identity are checked; no-follow parent
handles and `mkdirat` anchor real writes to that capability. A single final
backslash names the same directory; other path normalization is not inferred.
Seven host exclusions reject security attributes, writes outside the backing,
traversal, symlinks, Unicode descendants, relative paths and absent backing,
with no escaped mutation. POSIX inherited permissions implement this bounded
host policy; general NT security descriptors/ACL parity, aliases/reparse points,
Unicode descendant names and trailing separators on regular files remain
unsupported. This is a bounded implementation of
[CreateDirectoryW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createdirectoryw),
not a general Windows filesystem or security compatibility claim.

Local checks pass: the initial full Rust PE suite has 216 checks including
doctests, both affected filesystem units pass after the separator change,
407 Python tests pass, project Clippy/formatting pass and actual literal-PE
execution matches the complete native record. Hosted startup, Rust PE and native
startup jobs pass at `7f27cd5`; its broader native job is still finishing. All five
preceding PE jobs pass on `7c6518c`
([42522984117](https://github.com/daeken/Alkyhol/actions/runs/42522984117)),
including the INI path-error correction.

## Original DAO passes 49 INI queries and reaches directory creation

At clean revision `411ad55`, physical M4 executes 49 actual
`GetPrivateProfileStringW` calls in the unchanged original EXE. All six checked
arguments identify scalar queries, an empty default, a 512-WCHAR output and a
dot-relative leaf. The real backed file is absent in the process-owned current
directory, so the caller's own empty defaults are returned. Original startup
continues to **KERNEL32.dll!CreateDirectoryW**, which remains unsupported. No
window/menu/rendering or successful gameplay is observed. Original identities,
private strings/locations and full captures stay local, with prior fresh-profile
and locally unpacked dependency caveats unchanged.

The owned PE executes successfully on native Windows in
[42522573824](https://github.com/daeken/Alkyhol/actions/runs/42522573824).
The first native comparator fails because missing-parent LastError is **3**,
while the initial implementation/expectation used 2 for every absence. The
untouched successful PE capture is preserved as
`pe/reference/pe32-private-profile-missing-windows-v1.json`, SHA-256
`b5cf4426db5fdaca66b129614614cb3f7a21b3d9002274f63ca4e4f7de0e48e8`.
The runtime now propagates checked file absence (2) versus path absence (3).
This parent-path correction does not affect the 49 actual dot-relative leaf
queries above. The native workflow rerun passes at `7c6518c`, as recorded above.

Physical M4 selects that run's eight literal owned PEs, with identical complete
producer/PE hashes. **All 936 positive-observer bytes (234 DWORDs)** now agree
with the raw native capture. Ten queries cover NULL/empty defaults, Unicode,
leading/trailing spaces, truncation, capacity one, exact fit and missing parent;
return counts, LastError, NUL, complete staged buffers, guards and stdcall stack
agree. A separate native API control reads an existing owned INI value through
its dot-relative path, independently establishing current-directory resolution;
that control is not an implementation claim for existing-file parsing.

`--private-profile-files` explicitly declares file-only guest INI state without
registry IniFileMapping redirection. The bounded gateway verifies real absence
before returning a default. Existing contents never become a fallback: parsing
remains unsupported. Absolute Z: paths and explicit dot-relative leaves use the
real process-owned directory. Eight host-only exclusions stop explicitly:
section/key enumeration, zero-size output, invalid storage, bare-name searching,
existing contents, trailing control whitespace and absent profile selection.
This implements the absent-file scalar branch of the
[Win32 API](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getprivateprofilestringw),
not a general INI parser or recovered Windows profile/registry.

Local checks pass: 215 Rust PE checks including doctests before the native error
correction, its two targeted Rust units afterward, 404 Python tests,
project Clippy/formatting and actual identical-PE execution. Hosted startup
passed at `411ad55`; the native comparator failure above remains visible rather
than being described as green CI. All five preceding PE CI jobs pass on
`a32dfa5` ([42522013961](https://github.com/daeken/Alkyhol/actions/runs/42522013961)).

## Original DAO passes its Documents query and reaches INI configuration

At clean revision `97d77d7`, physical M4 executes the unchanged original EXE's
`SHGetFolderPathW` call. All five checked stack words confirm NULL owner/token,
`CSIDL_PERSONAL`, current-path flags and an actual writable output buffer.
The returned Z: path identifies an explicitly selected, existing fresh guest
Documents directory. Original startup continues to
**KERNEL32.dll!GetPrivateProfileStringW**, which remains unsupported. No window,
menu, rendering or successful gameplay is observed. Original hashes, private
arguments/locations and raw captures stay local; the fresh registry/session and
locally unpacked dependency caveats remain unchanged.

The [native startup job at `97d77d7`](https://github.com/daeken/Alkyhol/actions/runs/42521614542)
passes the actual Documents observer and preceding native startup comparisons.
Physical M4 then selects that run's six literal PE artifacts and executes the
positive observer and host restrictions with identical complete producer/PE hashes. The raw native report is
`pe/reference/pe32-documents-folder-windows-v1.json`, SHA-256
`72ca6264014946d0d2dccb937c585c4d1b74c9b02e7d0330720d8e1f302f045e`.
Both direct import and dynamic resolution return HRESULT S_OK, the independent
per-host backed path, intact guards, NUL termination and a balanced stdcall
stack. Paths intentionally differ by host. Native LastError observations are
0 and 1008, while the host preserves the supplied sentinel; these raw differences
are retained, not normalized or represented as LastError parity. The
[documented API](https://learn.microsoft.com/en-us/windows/win32/api/shlobj_core/nf-shlobj_core-shgetfolderpathw)
uses HRESULT as its result channel.

The bounded gateway admits only NULL owner/token, `CSIDL_PERSONAL`, current-path
flags and writable MAX_PATH storage. Directory identity is checked against its
held-open backing before each result. Six host-only exclusions stop explicitly:
other folder, default-user token, default-path flags, invalid buffer, non-NULL
owner and absent backing. This declares fresh guest state, not a recovered
original user profile. Creation/redirection and other folder roles remain unsupported.

The second API bank starts after both existing x87 bodies. All original 120 API
slots, math bodies and rejection gates retain their offsets; overflow is checked.
Local checks pass: 212 Rust PE checks including doctests, 401 Python tests,
project Clippy/formatting, unchanged literal event/semaphore execution and both
bounded x87 suites. The hosted startup and native startup jobs pass; broader
CI jobs at this revision are still finishing. All CI jobs passed on preceding
`d88d468` ([42520781540](https://github.com/daeken/Alkyhol/actions/runs/42520781540)).

## Original DAO passes both synchronization constructors and reaches a user folder

At clean revision `0cda51c`, physical M4 executes the unchanged original EXE's
actual `CreateEventW` and `CreateSemaphoreW`. Both complete four-word captures
confirm NULL security and names; the semaphore starts at zero with a valid
signed maximum. Real backing objects are created, original startup continues
its critical-section/tick work, then reaches **SHELL32.dll!SHGetFolderPathW**.
That API remains unsupported; no window/menu/rendering is observed. Folder
contents/paths have not been invented. Original hashes, arguments, resource
data and full captures remain local, with existing profile/dependency caveats.

The [native startup job at `0cda51c`](https://github.com/daeken/Alkyhol/actions/runs/42520195797)
passes the actual counting-semaphore observation and the preceding native
startup comparisons. All 28 raw words agree with physical M4 after selecting
the same five literal PEs and complete source/file hashes. The record is
`pe/reference/pe32-unnamed-semaphore-windows-v1.json`, SHA-256
`2a840b978afdfad61efed57f1a2972b72fc050f70018af3c2b6274700f26215f`.
Permits are consumed one at a time, zero count times out, and negative/excessive
initial counts or nonpositive maxima fail with native error 87. Earlier 42-word
event evidence stays unchanged and agrees after adding semaphore routing.
Four semaphore restrictions remain host-only exclusions. This follows the
[CreateSemaphoreW contract](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createsemaphorew)
within the admitted process-local scope, not general synchronization parity.

Local checks pass: 211 Rust PE checks including doctests, 398 Python tests,
project Clippy/formatting and actual event/semaphore PE execution. Broader CI
jobs at this revision are still finishing. The API gate table now fills its
first page; the next new import requires a checked layout extension preserving
the existing math-gateway regions and frozen native comparisons.

## Original DAO creates its event and reaches a counting semaphore

At clean revision `f34fd67`, physical M4 executes the original EXE's actual
`CreateEventW` and records all four arguments. The name is NULL, confirming the
unnamed profile rather than inferring it from the earlier three-word trap.
The real manual-reset, initially nonsignaled object is created; original
critical-section initialization continues to **KERNEL32.dll!CreateSemaphoreW**.
No window/menu/rendering is observed. Original hashes, private arguments and
captures remain local, with prior profile/unpacked-dependency caveats unchanged.

All five CI jobs pass at this revision in
[42519149166](https://github.com/daeken/Alkyhol/actions/runs/42519149166).
All 42 native event words agree with physical M4 on identical PE/source hashes,
including real manual/automatic state, zero-timeout consumption, set/reset
idempotence, error state, close and stack balance. The complete native record
is `pe/reference/pe32-unnamed-event-windows-v1.json`, SHA-256
`3a57fecf255eb806b873669037a33183c0e0f61fb8ed6ecf678244f5ab887720`;
all six literal owned PEs are selected explicitly. Five host exclusions remain
separate from the native positive observation.

For the newly reached semaphore, the same typed process-local bridge now owns
real kernel counting state via `CreateSemaphoreW`, checked signed counts and
zero-timeout non-alertable polls. Event modifiers reject semaphore handles.
The owned producer checks a zero-count/large-maximum object, two permits consumed
one at a time, timeout after exhaustion, invalid signed counts and closing.
Its 28-word physical M4 check and four unsupported forms pass. Guest semaphore
release, named/security/inheritance forms and blocking/APC waits remain
unsupported; native semaphore comparison and a new original run are pending.

## Checked unnamed event bridge prepared for the actual DAO constructor

The PE32 gateway now admits `CreateEventW` with NULL security/name parameters
and real manual/automatic initial state through the existing Rust kernel event
implementation. It owns typed process-local handles, closes them during proper
teardown and dispatches their `WaitForSingleObjectEx(...,0,FALSE)` independently
from mutex ownership. `SetEvent` and `ResetEvent` change actual backing state;
manual signals persist, automatic signals are consumed by one successful poll,
and repeated sets do not accumulate semaphore-like permits. Guest names/security,
blocking/APC waits and other handle kinds remain unsupported. The original
constructor's fourth parameter will be recorded by the real 16-byte stdcall gate;
the earlier three-word trap capture did not establish that parameter.

An original owned PE observes 42 words covering both event kinds, initial states,
timeouts/consumption, idempotent set/reset, closing, error state and caller stack.
It and five unsupported call cases pass on physical M4. A Rust unit checks actual
state transitions and stale/type ownership. The identical PE native Windows
comparison is wired into existing CI and pending. This follows Microsoft's
[CreateEventW](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createeventw),
[SetEvent](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-setevent)
and [ResetEvent](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-resetevent)
contracts within this bounded process/thread profile; it is not complete Win32
event namespace or guest-thread support, and no game rendering is claimed.
Local checks pass: 210 PE Rust checks including doctests on the Rosetta target,
398 Python tests, project Clippy, formatting and actual event PE execution.

## Native icon comparisons pass and original DAO reaches event creation

At clean revision `eb4395e`, the physical M4 repeats original EXE startup through
the actual resource icon load and reaches **KERNEL32.dll!CreateEventW**. Its
original EXE/probe hashes and private arguments are retained locally. No native
window, menu or game rendering is observed; existing fresh-profile and unpacked
dependency caveats remain unchanged.

The [native startup job at `eb4395e`](https://github.com/daeken/Alkyhol/actions/runs/42517813846)
passes all native startup checks, including actual icon loading and the separate
SDK/GDI readback of the identical owned PE resource. Its complete record is
`pe/reference/pe32-resource-icon-windows-v1.json`, SHA-256
`3b9a22f85a41334ed546e1f34f0cb0f9687f2a9635e19ef7dd54a257d0c3c6e5`.
The raw GDI snapshot is `pe32-resource-icon-gdi-v1.bin`, SHA-256
`41732b8eea027a41ea80cc67d95515a2db2e101d25d38d32954c786c7efa9ba7`.
The physical M4 reruns all three selected literal PEs with identical complete
producer/file hashes. All ten loader/cache/error/stack DWORDs agree, as do
3,072 defined RGB bytes and 128 AND-mask bytes. The reserved fourth GDI byte is
preserved raw and remains outside the predefined alpha comparison contract.
These are owned icon-resource observations, not native game-pixel parity.

Local checks pass after the fd-lifetime fix: 209 Rust PE checks including
doctests, 398 Python tests, project Clippy/formatting, the icon/audit tests and
the unchanged 76-word mutex ownership/handoff/abandonment reference. The hosted
Rust/PE startup jobs at this revision also pass. The broader PE64 native and
paired-comparison jobs are still finishing. Earlier failed runs remain recorded;
no frozen previous sample, native intrinsic criterion or reference was relaxed.

## Original DAO loads its icon and reaches event creation; native details measured

At clean revision `59e560a`, the unchanged original EXE loads its own real icon
resource into a decoded shared object on physical M4, then continues its
critical-section work to **KERNEL32.dll!CreateEventW**. No window/menu/rendering
is observed. Original file/probe identity, icon data and resource arguments
remain local; existing fresh-profile/unpacked-dependency caveats remain.

The [native icon observation at `3950b73`](https://github.com/daeken/Alkyhol/actions/runs/42517044098)
executes the complete PE and SDK/GDI readback successfully after the LF producer
fix. It measures error **0** on initial/cached loads and **1813** for this missing
integer icon. The initial preserve-error/1814 hypotheses were wrong; the provider
and new checker now match the actual raw ten-word observation. Defined native
GDI RGB and mask bytes agree with the owned host snapshot. No alpha comparison
criterion was changed. The first checker job remains failed pending a fresh run.

That run also exposed a mutex lifetime test failure after all logical references
closed. A new deterministic host-fd duplication test reproduces the stale lease
before the fix. Parallel tests spawn host subprocesses; transient inherited fd
copies can explain that observation even with close-on-exec set. Mutex lease,
owner and namespace locks now explicitly unlock at logical owner destruction;
`CloseHandle` propagates unlock errors. The duplicate-fd reproducer and all six
mutex units pass, including abandonment while a copied owner descriptor remains
open. The original strict fresh-object assertion remains unchanged. Earlier
frozen producer bytes/comparators are unchanged.

## Resource-backed icons prepared for the actual DAO UI import

The PE32 `LoadIconW` provider retains immutable icon resources from each checked
module, selects one exact 32-logical-pixel, 32bpp candidate, validates its group
and BI_RGB DIB extents, and owns real decoded top-down BGRA/AND-mask data behind
a shared process token. Repeated loads return that existing object. A known
missing integer resource returns failure; system icons, named resources,
ambiguous languages/size candidates, PNG/compressed/palette/scaled variants and
mutable resources stop explicitly. The classic 32px profile is finite; no
arbitrary current-display selection is claimed. Original color/alpha/mask
values are retained, with no alpha repair or geometry/render-quality changes.

`ProcessImage32::icon_images` and the explicitly requested probe `--icon-audit`
produce read-only diagnostic copies, tested by an actual owned PE loader. The
owned multi-image resource selects its 32px entry rather than its 16px entry;
all 1,024 source pixels and mask bytes agree with the independent producer.
Two Rust units cover exact/ambiguous selection, row orientation, alpha/mask
preservation and malformed headers/extents. Two unsupported call forms stop.
Native loader error/cache observations and a separate Windows SDK/GDI readback
of the same owned PE resource are wired into CI and pending. The predefined
pixel comparison checks RGB/mask bytes; GetDIBits BI_RGB's reserved fourth byte
is retained raw and is not treated as an alpha guarantee. Host alpha checks
remain a separate category. No icon has yet been drawn in an Alkyhol window.
Local checks pass: 208 PE Rust checks including doctests on the Rosetta target,
398 Python tests, project Clippy, formatting and the real owned icon/audit probes.

This implements the bounded [LoadIconW](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-loadiconw)
resource/object operation; it is not a placeholder successful icon handle.
Original game icons, decoded pixels and resource inventories remain local.

The first [native icon job at `59e560a`](https://github.com/daeken/Alkyhol/actions/runs/42516743206)
compiled the SDK observer but rejected producer identity before running the PE.
The newly hashed Python pixel generator lacked an explicit checkout line-ending
rule. Its bytes are now required to remain LF on both hosts, matching the shared
C producers; identity failures also name the differing owned inputs. No checksum
check was removed. This first run is not native icon execution/pixel evidence.

## Original DAO acquires its mutex and reaches UI icon resource loading

At clean runtime revision `e7481d0`, the physical M4 unchanged original EXE
executes its actual zero-timeout, non-alertable mutex wait and acquires real
ownership. Its original critical-section calls then execute and startup reaches
**USER32.dll!LoadIconW** for a resource in its own module. That call remains
unsupported; no window, main menu or rendering has been observed. Original
EXE/probe identity, private resource arguments and complete captures remain
local. Fresh-profile and locally unpacked-dependency caveats remain unchanged.

The [native startup job at `e7481d0`](https://github.com/daeken/Alkyhol/actions/runs/42514801946)
passes recursive ownership, non-owner release failure, real interprocess
contention/handoff and abandonment after forced owner-process termination.
All **76 raw DWORD payloads** agree between native Windows and physical M4 after
selecting the same five literal PE files and verifying complete producer/PE
hashes. The two hosts' forced-termination exit statuses remain separate raw
observations. The complete native record is pinned as
`pe/reference/pe32-mutex-ownership-windows-v1.json`, SHA-256
`0b141673543db2614757811bf53616ca3209468c9ca09d1596f07a3a5fc4dfc4`.
Timed/APC wait exclusions remain host assertions, not native implementation
claims. Earlier reference, directory, default-attribute and other frozen records
are unchanged. Broader CI jobs at this revision are still finishing.

## Real zero-timeout mutex ownership prepared for the reached DAO call

The PE32 provider now implements `WaitForSingleObjectEx(handle, 0, FALSE)` and
`ReleaseMutex` on its owned named mutexes and one admitted guest thread. A
separate exclusive OS file lock holds ownership across guest calls/processes;
reference leases remain independent. Recursive waits through any live handle
increment the thread/object count, and only the final release clears ownership.
A non-owner release fails with error 288. A live foreign owner produces timeout
258. Persistent ownership markers plus released OS locks report abandoned
ownership (128) after owner process death while another reference remains; a
new object resets that marker. These are state changes, not invented wait results.

Two original owned PE32 processes actually exercise contention, explicit release
and handoff, then forced owner-process death and abandoned acquisition. All raw
readiness/tail words, error state and process outcomes are retained. Recursive
acquire/release, non-owner failure and caller stack balance run separately.
All three scenarios pass on physical M4. Timed and alertable waits stop in two
guest exclusions. Two new Rust units check shared-handle ownership, recursion,
contention, abandonment/new-object reset, last-owned-close rejection and malformed
ownership state. Existing reference-lifetime fixtures and their frozen bytes
remain unchanged. Native comparison and the next original EXE run are pending.
Local checks pass: 206 PE Rust checks including doctests on the Rosetta target,
398 Python tests, project Clippy and formatting. The earlier frozen reference
lifetime observer also passes its identical-PE lifecycle and cross-process
close/death scenarios with the ownership implementation present.

The finite contract follows Microsoft's
[WaitForSingleObjectEx](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-waitforsingleobjectex)
and [ReleaseMutex](https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-releasemutex)
descriptions. Blocking/timed/APC waits, other object kinds, additional guest
threads, initial ownership, closing the last handle while still owned, custom
security/inheritance and general mixed-version namespace sharing remain
unsupported. This does not establish game compatibility or complete Win32 waits.

## Original DAO passes directory setup and explicit mutex attributes; next is acquisition

At clean runtime revision `08b94a6`, the physical M4 unchanged original EXE
selects its real working directory, creates the next real named object using
explicit default/non-inheritable attributes, and advances to
**KERNEL32.dll!WaitForSingleObjectEx** with zero timeout and alertable FALSE.
That call remains unsupported. A real ownership transition and subsequent
release/recursive/abandonment behavior must be implemented; successful reference
creation is not a successful wait. No menu or rendering is observed. Original
EXE/probe hashes, private arguments and the exact launch/captures remain local;
the fresh profile and unpacked-dependency caveats remain unchanged.

The [native startup job at `08b94a6`](https://github.com/daeken/Alkyhol/actions/runs/42513321943)
passes default-attribute identity/lifecycle comparison, the directory observer
and all preceding native startup checks. The complete attribute record is
`pe/reference/pe32-mutex-default-attributes-windows-v1.json`, SHA-256
`148d0d2a8f49bce0c7b3140eaa25bceaaaf7abb209f96af28258030926ec6a4c`.
All four literal owned PEs are selected to avoid compiler-output differences;
the physical M4 rerun agrees with all 16 native lifecycle words on identical
PE/source hashes. Three host exclusions retain explicit failures for inheritance,
custom descriptors and other structure sizes. Prior frozen records are unchanged.
All five jobs at `14c1028` pass in
[42512731501](https://github.com/daeken/Alkyhol/actions/runs/42512731501).

## Directory comparison passes; the next measured mutex uses explicit defaults

The [native startup job at `14c1028`](https://github.com/daeken/Alkyhol/actions/runs/42512731501)
passes the corrected working-directory observation and peer comparison. All 67
raw words validate against the real per-host UTF-16 lengths; non-path behavior
agrees, including missing-leaf error 2 and successful existing-file error 0.
The unchanged initial raw native capture is pinned as
`pe/reference/pe32-current-directory-windows-v1.json`, SHA-256
`bb324725474f42f15ebf8eaf4a29567205950018e1f79b636d69cf981f98bda5`.
It preserves the first checker's failure state (no added decoded fields), while
its successful complete PE execution is now independently validated. The old
failed CI job remains failed. Local and hosted compilers produced different
PE hashes despite identical C sources; all three literal CI-produced PEs are now
selected explicitly. The physical M4 reran those exact native PE bytes and all
67 words validate. Source, fixture and PE identities agree completely.

The original EXE at clean revision `14c1028` again executes directory selection.
Its next mutex call has a checked 12-byte `SECURITY_ATTRIBUTES` with a NULL
descriptor and inheritance disabled. The provider now admits precisely that
explicit default form through the existing backed object policy; custom
descriptors, inheritance, other sizes and initial ownership still stop. A
separate owned producer mixes explicit defaults and NULL attributes to test real
shared identity, both closes and last-close destruction. Its 16-word lifecycle
and three guest exclusions pass on the physical M4. Fresh native observation
and the next original EXE run are pending; no game compatibility is claimed.

## Native directory observation corrects two previously unmeasured error details

The first [native directory job at `c880378`](https://github.com/daeken/Alkyhol/actions/runs/42512402029)
executes the entire owned PE successfully and preserves all 67 raw words before
its checker fails. Native `SetCurrentDirectoryW` on the known missing leaf
returns error **2**, not the preparatory host hypothesis of 3. The real native
existing read-only `CreateFileW` calls clear last error on success; the prior
host bridge preserved it. The provider and new observation checker now use those
measured details. The original producer, binary and all earlier frozen evidence
are unchanged. Size/short/full queries, Unicode lengths, guards, failed-change
atomicity, relative A/B reads and close error preservation already agree.
The initial job remains failed; a fresh native pass is pending.

At clean revision `c880378`, the physical M4 original EXE executes its actual
`SetCurrentDirectoryW` and proceeds to another `CreateMutexW`, this time passing
non-NULL `SECURITY_ATTRIBUTES`. That form still stops explicitly. A diagnostic
now captures its three checked Win32 fields before rejection; it does not grant
unsupported security/inheritance semantics. Original file identity and local
profile/unpacked-dependency caveats remain; no menu/rendering is observed.

## Backed directory state prepared for the reached original startup call

The PE32 `SetCurrentDirectoryW` provider now opens and retains a real directory
reference in process-owned state. It admits existing exact absolute paths in
the explicit `Z:` host-root view, with one optional trailing separator. Checked
filesystem traversal rejects unknown case/Unicode aliases, symlinks, DOS devices,
normalization, other drives and relative directory changes. Missing paths and
non-directory targets return separate failures without changing the previous
directory. Host process cwd is never changed. Renaming/replacing the directory
externally is detected by identity checks and stops explicitly.

`GetCurrentDirectoryW` observes that state with UTF-16 sizing/termination and
checked output storage. After a successful set, ordinary relative or absolute
existing read-only `CreateFileW` requests resolve through the backed directory;
unknown create/write semantics stop. General inherited/per-drive cwd, relative
directory setters, mutation/rename locks, UNC/device paths and arbitrary relative
search APIs are not implemented. The interface follows Microsoft's
[SetCurrentDirectory](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-setcurrentdirectory)
and [GetCurrentDirectory](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-getcurrentdirectory)
contracts within this finite profile.

The physical M4 executes an owned PE through two Unicode directory selections,
real relative reads returning distinct bytes, sizing/short/full output queries,
guards and failed-change atomicity. Two unsupported namespace PEs stop before
emitting success. Two Rust units check actual directory references, unchanged
host cwd, namespace bounds and external rename detection. The identical-PE native
observer is wired into CI; native error/buffer observations and a new original
EXE run are pending, and these host assertions are not game compatibility.
Local checks pass: 204 PE Rust checks including doctests on the Rosetta target,
398 Python tests, project Clippy checks and formatting. Existing frozen
fixtures/comparators and the PE64/source/Metal implementations are unchanged.

## Original DAO executes its multimedia clock and reaches working-directory setup

At clean runtime revision `fce4123`, the physical M4 executes the unchanged EXE
past its actual Winmm clock calls and genuine `GetModuleFileNameW` lookup. The
next reached unsupported import is **KERNEL32.dll!SetCurrentDirectoryW**. Private
module/RVA/argument evidence and original-file/probe hashes remain local; the
capture confirms no runtime edits before building. No menu or rendering has
been observed, and the earlier fresh-profile/unpacked-dependency caveats remain.

The [native startup job at `fce4123`](https://github.com/daeken/Alkyhol/actions/runs/42511459044)
passes the identical Winmm clock PE and requires the peer host observation.
All 47 raw DWORDs are validated on each host against its own independent boot
clock bracket, parent interval and performance-counter scale, plus existing
error/guard/atomic contracts. Native measured 33 ms / 0.0319158 counter seconds;
physical M4 measured 32 ms / 0.031752834 counter seconds. Actual timestamps,
PIDs and rates remain per-host observations, not claimed byte equality.
The complete native record is pinned as
`pe/reference/pe32-multimedia-clock-windows-v1.json`, SHA-256
`5b2fea4322e53c7147e8f7c0ebdbc16c668236e4b92e2a33a9e355328114d0bd`.
The selected identical literal PE has SHA-256
`7dd6ddc967324854e0fb71410440430df9c744122ef7bb74e41b3abc973be7df`.
These are finite clock observations; multimedia timer resolution/scheduling is
not established. All five jobs at the prior mutex milestone `2db79bf` pass in
[42511091789](https://github.com/daeken/Alkyhol/actions/runs/42511091789).

## Original DAO startup creates its named mutex and reaches the multimedia clock

At clean runtime revision `2db79bf`, the physical M4 executes the original EXE's
actual `CreateMutexW(NULL, FALSE, name)` through the backed shared guest session.
Original startup continues to **WINMM.dll!timeGetTime**. The capture records the
pre-build revision, clean runtime status, executable and probe hashes, and the
explicit local object-directory selection. Original EXE bytes are unchanged;
the prior local unpacked-dependency caveat remains. Captures and the game's
object identity stay local. This is startup progress, not a menu/rendering result.

The next reached clock now routes only `winmm.dll!timeGetTime` to the existing
real continuous boot-millisecond provider. It keeps the low DWORD's specified
49.7-day wrap and preserves error state. Kernel32 cannot claim this export.
The provider follows the [documented timeGetTime contract](https://learn.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timegettime)
using actual host elapsed boot time. An owned variant
of the existing clock producer imports the real separate Winmm DLL without
changing the old producer/fixture. The physical host verifies live elapsed time
against independent boot-clock and performance-counter observations, with raw
per-host values retained. Native execution and the next original EXE run remain
pending at this point; timer-resolution requests remain unsupported.

The [mutex milestone CI](https://github.com/daeken/Alkyhol/actions/runs/42511091789)
has passed the hosted PE32 startup and native Windows startup jobs, including
the required identical-PE mutex comparison. Broader PE execution jobs are still
finishing. The physical clock observer, export routing unit, Clippy and
formatting checks pass locally; prior frozen records remain unchanged.

## Next reached mutex call has an independent native lifetime observer

The [native startup job at `09124b9`](https://github.com/daeken/Alkyhol/actions/runs/42510592173)
now passes all three actual named-mutex observations, including an independent
64-bit parent keeping the PE32 child's object alive. The unchanged report is
`pe/reference/pe32-mutex-create-windows-v1.json`, SHA-256
`ac8dd7be930dadb38097535fcdbcd3303a42219a1fcdbe2a87734cf0872a1eee`.
Literal source-pinned lifecycle/single-create PEs are retained too; no prior
frozen evidence or producer was changed to make the backend agree.

The Rust PE32 provider now admits named, initially unowned `CreateMutexW` with
NULL security attributes in a caller-selected `--object-directory` /
`ProcessOptions32::object_directory`. This directory defines an explicit shared
guest session, not the macOS/Windows global namespace. All participating guest
processes must select the same exclusively managed directory. OS shared file
locks represent live references: each real opaque handle owns its own open file;
creation is serialized by an exclusive namespace gate, and the last close or
process death releases the object's existence. Persistent lease files alone do
not count as live objects. Directory-relative opens pin the namespace even across
directory renames, reject symlinks/hardlinks, and retain close-on-exec descriptors.
Full UTF-16 identity records detect hash collisions instead of aliasing names.
Bare/`Local` names share this session and comparisons remain case sensitive.
`CloseHandle` releases the actual reference while preserving caller error state.

On the physical M4, all 32 native DWORDs agree on the identical PEs: creation,
existing-object error 183, multiple references, last-close destruction, stack
balance and preserved close error state. A separate original PE32 holder proves
cross-process lifetime through real guest calls, including destruction after
abrupt host process death. Three Rust units cover local references, name bounds,
case/alias routing and corrupt/symlinked identity rejection. Initial ownership
and missing namespace configuration explicitly stop in two guest exclusions.
Acquisition/waits, release, abandonment, unnamed/Global/private objects,
security/ACLs and inheritance remain unsupported; this is a finite reference
lifetime implementation, not complete mutex synchronization. It has not yet
been exercised by the original EXE at this revision.
Local checks pass: 202 PE Rust checks including doctests on the Rosetta target,
398 Python tests, Clippy with project warnings denied, formatting, and the real
guest cross-process lifetime/exclusion observations above.

The owned named-mutex observer builds for the actual `CreateMutexW` frontier.
It checks new versus already-existing objects, multiple references, actual
last-close destruction, caller stack/error state, and a separate live parent
process holding the same Windows named object. This is more than a non-NULL
handle check. The first [native run at `d7b9911`](https://github.com/daeken/Alkyhol/actions/runs/42509666672)
agrees on all 16 lifecycle words. The separate parent-process portion stopped
before creating its object: Python's Windows `ctypes.set_last_error` takes a
signed C int and rejected the unsigned high-bit sentinel. The caller now
converts that sentinel to signed 32-bit form, preserving its bits. The raw
partial report is retained locally; cross-process evidence remains pending.
At that preparatory revision, compilation and two transport tests passed while
the backend remained explicitly unsupported pending object identity/lifetime
implementation. No game-specific object
names or inventories are included in the public probe.

## Original DAO startup passes a scoped registry query and reaches mutex creation

On the physical M4, the unchanged original EXE now executes its reached
`RegOpenKeyExW` call and continues its original branch to
**KERNEL32.dll!CreateMutexW**. The caller uses a new, explicitly configured
fresh-application registry scope. That subtree is declared absent in a local
backing catalog, so the lookup returns a real scoped `ERROR_FILE_NOT_FOUND` with
NULL output; the game handles that result itself. This is **not** a recovered
Windows install hive or proof that the user's original Windows registry lacks
the key. No key names/values or game-derived inventories are committed or sent
to CI. Original files and the existing unpacked-dependency caveat are unchanged;
no menu, rendering or game compatibility has been observed.

The read-only 32-bit registry provider loads a bounded regular XML backing file
through `ProcessOptions32::registry_catalog` / `--registry-catalog`. Each scope
explicitly declares its own existence and children. Existing keys open into
real owned opaque tokens, relative and ASCII case-insensitive lookups use that
state, and close removes/unmaps the live token. Missing keys are reported only
inside authoritative configured scopes. Unknown hives/handles/scopes, unsupported
views/options/access, non-ASCII or empty key paths, writes/values/ACL semantics
and malformed/ambiguous/oversized catalogs remain unsupported. There is no
global "unimplemented means missing" fallback.

The [native startup job at `9536749`](https://github.com/daeken/Alkyhol/actions/runs/42507259294)
passed SDK widths and the owned native HKCU observer. Its unedited 26-word record
is pinned as `pe/reference/pe32-registry-open-windows-v1.json`, SHA-256
`980f920609e04f17c4d05d414083b7372e16dd7e42f053b9fbbd6bfd6efe42bc`.
The literal owned PE, `pe/reference/pe32-registry-open-v1.exe`, has SHA-256
`37bda422183b97312fe864dd2c8c8561ea6c5f9c0c21837403ea3cd0af14f7d8`.
All producer/PE identities agree with the physical M4. Every output word agrees:
open/relative/case/32-bit-view status, live handles, known missing-child NULL
output, guards, closing, stdcall stack balance and preserved error state. Five
exclusions stop for their actual wrong-view, bad-output, unknown-scope,
stale-handle or unconfigured-profile reason. Native CI now requires the peer
host observation, while retaining its temporary owned-key cleanup.

Local checks pass: 199 PE Rust tests/checks including doctests on the Rosetta
x86_64 target, 395 Python tests, Clippy with project warnings denied, formatting,
and identical-PE registry execution. Catalog tests distinguish unknown state
from scoped absence and reject case duplicates, overlaps, unsupported syntax
and scope/key/depth/size budgets. This remains a finite read-only profile.

## An additional native CPU observation extends the admitted machine profile

The [native system-info observation at `6866375`](https://github.com/daeken/Alkyhol/actions/runs/42506664367)
returned a successful ordinary PE result on an AMD **family-26** host. The prior
profile rejected that previously unobserved family, so that job correctly failed.
Its raw result remains local. The actual signature/model/stepping and all nine
structure words agree with the existing derivation once that observed family is
admitted. AMD families 25/26 are now supported alongside the existing Intel-6
profile, with a targeted test for the newly observed descriptor. The frozen
family-25 report, original PEs and field checks are unchanged; other unknown
families still stop. This does not change the physical M4's Rosetta CPU profile.

## Native registry evidence prepared at the next original startup call

For the reached `RegOpenKeyExW` frontier, the owned observer now builds against
actual Advapi32 imports. Native CI creates only its dedicated temporary HKCU
fixture, observes open/relative/case/32-bit-view lookup, a known absent child,
output guards and handle closing, then removes its owned keys. It refuses to
overwrite a pre-existing fixture. An independent SDK width check is included.
Compilation and two transport tests pass locally; native observations and a
backed registry implementation remain separate pending work. No game key data
is embedded in these public sources or sent to CI.

## Native ordinary/LAA memory-profile checks now pass on identical PEs

The [native startup job at `83ee2c5`](https://github.com/daeken/Alkyhol/actions/runs/42506046791)
passed SDK layout, both original SYSTEM_INFO observations and both separate
real-storage witnesses. The hosted Rust/PE startup jobs also pass. Downloaded
native artifacts have exactly the same complete source/PE identities as the
physical M4 observations. Every system-information field validates against that
host's actual CPUID and independently observed CPU count; both real-storage
contracts agree after checking all raw addresses against their reported bands.
Raw differing CPU facts and ASLR addresses remain preserved per host.

On the physical M4, CPUID is the x86 environment actually exposed by Rosetta,
while the available-processor count is the host observation. This does not turn
an Apple CPU into native Intel hardware or claim general topology/scheduling
parity. The committed original EXE run at `83ee2c5` again reaches the real
`RegOpenKeyExW` stop with its EXE hash unchanged. Registry contents and missing
keys have not been guessed; that is the next application-driven frontier.

## Original DAO startup passes host-backed system information and reaches registry access

On the physical M4, the unchanged original EXE executes past its actual
`GetSystemInfo` call using an explicitly selected host-machine profile. It next
stops at **ADVAPI32.dll!RegOpenKeyExW** in original startup. This is further EXE
startup, not an observed menu/render loop or game compatibility. Original files,
module/RVA/argument captures and derived inventories stay local; the documented
locally unpacked dependency caveat remains.

The complete 36-byte structure describes the admitted x86 guest environment:
actual CPUID family/model/stepping, independently checked active processor count,
a contiguous virtual CPU mask, 4 KiB pages and 64 KiB reservation granularity.
The root EXE's real LARGE_ADDRESS_AWARE bit selects the ordinary 2 GiB or LAA
4 GiB address band (both exclude the native top/bottom reserved bands).
This is an explicit `--host-machine-info` / `ProcessOptions32::host_machine_info`
profile. It admits the observed Intel family-6 and AMD family-25 CPU profiles, 1..32 processors
and 4 KiB host pages; other descriptors stop rather than inventing a machine.
It does not implement general guest scheduling or VirtualAlloc lifecycle.

Those limits are enforced on owned image/gate/TLS, stack/TEB, heap, startup-buffer
and activation-section mappings; their reservation bases align to 64 KiB.
They are not just reported constants. Default profiles retain their previous
mapping contract. Real storage checks run in separately linked small-PAGEZERO PE
witnesses: ordinary Rust test binaries cannot allocate these low guest mappings.
The added witness checks actual image, stack, structure, heap and command-line
addresses against the reported band, image/heap alignment, volatile first/last
heap-byte writes/readback, release, stack balance and guards. Both ordinary/LAA
versions pass on the physical M4. Two excluded profiles stop for their actual
missing-profile or bad-output reason.

The [native startup job at `474575b`](https://github.com/daeken/Alkyhol/actions/runs/42504218530)
passed the independent x86 SDK layout check and recorded every ordinary/LAA
SYSTEM_INFO byte plus actual CPUID leaves on its native AMD family-25 host.
The report is pinned as `pe/reference/pe32-system-info-windows-v1.json`, SHA-256
`9e6d3399fb6d2c2798aa3bad982b0a89ea83cd8eadccb1d4065dc96d3ab6a6ee`.
Literal owned ordinary/LAA PEs are `pe32-system-info-v1.exe` (SHA-256
`a4ba580aae1f25c6877ac71065724a5cc50feccd1270ebf94720e66547b40cd6`)
and `pe32-system-info-large-v1.exe` (SHA-256
`9e4f778705b1c3608f7b1a904f69a5661cac570346cfdc3623c3972c9616bca6`).
Original producer/PE identities are unchanged. Every structure field is checked
against its own actual CPU/parent count and selected address model; differing
raw CPU facts remain explicitly per host. This is finite profile evidence,
not identical hardware or general system-information equivalence. The additional
real-storage witnesses still await native execution.

Local checks pass: 196 PE Rust tests/checks including doctests on the Rosetta
x86_64 target, 392 Python tests, Clippy with project warnings denied, formatting,
the complete original SYSTEM_INFO PEs and separate real-storage probes. The
native workflow now requires the peer host record rather than accepting a
missing cross-host comparison. No unsupported call returns fabricated success.

## Native startup checks pass at the allocator milestone; next API evidence prepared

[PE execution CI at `2bab359`](https://github.com/daeken/Alkyhol/actions/runs/42503166104)
passed all five jobs, including native Windows. The 28 heap-lock words and ten
immediate impossible-resize-failure words now agree on hosted Rosetta execution
and native Windows. The immediate sample confirms error 8 right after that failed
call. The existing 64-word frozen resize record and all prior native startup
comparisons also pass at this revision. Physical M4 evidence is recorded separately
below; this is not game rendering or game equivalence.

For the actual `GetSystemInfo` frontier, an owned observer now builds ordinary and
large-address-aware PE32 files from the same source. It records all 36 raw structure
bytes and actual CPUID leaves; an independent Windows SDK layout check is wired
into CI. These native observations are pending. No GetSystemInfo backend success
is claimed merely because the observers compile.

## Original EXE allocator startup advances to GetSystemInfo

The physical M4 executes the unchanged original EXE past `HeapLock`, into its
original custom allocator initialization, and through a matching `HeapUnlock`.
The next measured unsupported import is **KERNEL32.dll!GetSystemInfo**. A first
30-second attempt exhausted its diagnostic budget; a separate attempt with a
300-second budget advanced to this real API stop. No initializer, allocation or
synchronization call was skipped. The existing 512 MiB per-allocation profile
bound remains active: the original allocator's own retry logic handles budget
failures. This is not evidence of native allocation-size/performance parity.
The game has not reached an observed menu or render loop. Exact game bytes,
identities, arguments and instruction observations remain local.

Heap locks now carry real per-heap recursion and owning-thread state, and matching
releases mutate that state. Unknown handles, unbalanced/wrong-owner release,
contention requiring guest scheduling and nonserialized heaps stop explicitly.
The latter's native behavior is
[undefined](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heaplock).
The 28-word owned process/private heap witness passes on the physical M4 for
recursion, allocations/resizing/free while owned, releases, stack and error state;
three restriction PEs stop for their actual reason. Cross-thread blocking and
native comparison of this new witness remain separate unverified work.

Adding the next reached APIs filled the first gate layout. Stubs now occupy one
owned page and the unchanged x87 bodies occupy disjoint ranges in a second RX
page. Template/patch bounds, page extent and overflow checks remain enforced.
All **24 sqrt and 16 atan admitted-profile rows** still agree byte-for-byte with
the native `a3bb58f` observations after this move, including full recorded state.
The frozen numeric PEs, producers and acceptance fields are unchanged.

Local checks pass: 194 PE Rust tests/checks including doctests on the Rosetta
x86_64 target, 385 Python tests before the two new heap evidence tests, those
additional tests, Clippy and formatting. The actual original graph reaches the
new `GetSystemInfo` frontier; native validation of new heap locking is pending.

## Native API observations verify entry helpers and correct heap failure state

The [native startup job at `a3bb58f`](https://github.com/daeken/Alkyhol/actions/runs/42502001666)
passed onexit/startup (53 words), cursor ABI/guards/error (nine comparable words,
with coordinates explicitly per host), both ANSI directory contracts (nine
comparable words each, with actual paths independently checked), and top-level
filter registration (11 words). Source and identical-PE identities agree with
the physical M4 records. These finite observations do not establish exception
filter delivery, screen/DPI equivalence, full directory semantics or game parity.

The same job correctly failed the heap-resize comparison. Its successful raw
64-word native execution observes error **8** after the impossible-size resize
sequence and retains that state after size/prefix/free checks. The prior host
assumption that this sequence preserves the incoming error was wrong. The bounded
backend now records the native-observed error for that impossible-size failure;
the original block remains live and unchanged. Other allocation-budget/failure
profiles are not declared native-equivalent.

The unedited successful native raw report is pinned as
`pe/reference/pe32-heap-realloc-windows-v1.json`, SHA-256
`479ab62b9259690906c9cc3b5570fc36e84ddbdfd2beb27370e5f0be5cb065cb`.
The literal owned PE is `pe/reference/pe32-heap-realloc-v1.exe`, SHA-256
`9554b9ee95c3f06161495b73cfb2b67b342e0c327ceb8bef163a7863f368dbd5`.
All original producer/PE identities remain pinned. All 64 observed words now
agree on the physical M4. A **separate** new ten-word witness samples last error
immediately after the failed call; its host assertions pass, but native execution
is pending. The original producer and composed-sequence observation are unchanged.
This avoids silently claiming that the old delayed sample isolated every API's
individual error effect. The native job remains failed; no full-CI success claim
is made for that revision.

## DAO EXE startup records its top-level filter and reaches heap locking

The unchanged original game entry now executes past its reached
`SetUnhandledExceptionFilter` call, records its callback in process-owned state,
and continues original startup/critical-section initialization. Its next measured
unsupported import is **KERNEL32.dll!HeapLock**. The game has not reached a main
menu or observed render loop; all original-file/capture and unpacked-dependency
caveats remain unchanged.

The bounded API stores a checked executable guest callback (or NULL), returns
the actual previous registration and supports replacement/clear. It does not
eagerly invoke the filter or modify the host's signal/exception policy. This is
[filter registration](https://learn.microsoft.com/en-us/windows/win32/api/errhandlingapi/nf-errhandlingapi-setunhandledexceptionfilter)
only: delivering an unhandled guest exception through that filter and resuming
modified CONTEXT are still unsupported. Instrumentation faults still stop; no
successful exception handling is fabricated.

An owned 11-word PE passes on the physical M4 for initial empty state, repeated
registration, replacement, clearing, no eager callbacks, stdcall stack and error
state. A bad callback stops explicitly. Callback-region boundary tests, two
transport tests, Clippy and formatting pass. Native identical-PE comparison is
wired into the existing CI as independent evidence and pending.

## Independent native startup observations retain upstream failures

The [native startup job at `646a0e9`](https://github.com/daeken/Alkyhol/actions/runs/42501028073)
returned a successful raw 204-row atan observation, but its strict frozen-baseline
comparison failed: two status-word C1 bits differ under **0x027F**, a profile the
backend already explicitly rejects. Other raw bytes in that observation agree.
The cause is not isolated; neither CPU/CRT equivalence nor a host regression is
inferred from this observation. The original report, PE and acceptance fields
remain unchanged, and the run remains failed. The raw differing native report
is retained locally.

Later independent native onexit/heap/cursor/directory observations now execute
even after an earlier comparison fails, preserving their own raw output and strict
checks. This does not suppress failed steps or change the job result. It prevents
an unsupported-profile reference discrepancy from dropping evidence for APIs
actually reached by DAO. The established 16-row 0x037F native/M4 comparison remains
the finite admitted-profile evidence; no new general atan parity is claimed.

## The unchanged DAO EXE enters its original startup code

On the physical M4, Bink's original `GetWindowsDirectoryA` and subsequent
`GetSystemDirectoryA` now query explicitly configured backed directories.
Bink and Umbra initialization return, and the unchanged **original game EXE
entry is invoked**. Its startup executes original CRT/TLS/heap code and stops
at the next unimplemented import, **KERNEL32.dll!SetUnhandledExceptionFilter**.
This is entry/startup execution, not a main menu, render loop or game compatibility.
The original EXE's hash remains unchanged; local module/RVA/argument captures
and the launch profile stay ignored. The previously documented locally unpacked
dependency caveat remains; no unchanged-all-DLL claim is made.

`ProcessOptions32::windows_directory` is a separately verified per-user role,
selected with `--windows-directory`; it is never inferred from the shared Windows
root or System32. This preserves the API's
[distinction under Terminal Services](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getwindowsdirectorya)
without claiming Terminal Services emulation. The two reached ANSI functions
support complete ASCII-backed output only, preserving error state and the buffer
tail. Missing roles, non-ASCII paths, insufficient buffers and invalid output
storage stop explicitly. Existing Unicode directory behavior is unchanged.

The owned identical-PE witnesses pass on the physical M4 for both APIs, checking
actual paths against their independently configured roles, lengths/termination,
untouched tail/guards, stdcall stack and error state. Distinct user, shared and
system roots ensure a mistaken role alias cannot pass. Eight restriction cases
stop for their actual reason. Native Windows comparisons are wired into the
existing workflow; they are pending. Two transport tests verify all fields,
source path identity, termination and byte extent. Local checks pass: 193 PE
Rust tests/checks including doctests on the Rosetta x86_64 target, 381 Python tests,
Clippy with project warnings denied and formatting.

## Native onexit/startup comparison catches and corrects ANSI last error

The [native startup job at `035552d`](https://github.com/daeken/Alkyhol/actions/runs/42500607924)
completed the revised owned 53-word exit-table/startup PE with exit status zero,
empty stderr and successful Windows SDK layout checks. The strict comparator
correctly rejected one difference: complete `GetModuleFileNameA` output clears
last error to zero, as W already did. The backend now performs that native-observed
update. The earlier assumption that A preserves last error was incorrect.

The literal identical PE (SHA-256
`c50a2d232a4cb00e0590def5be9fb1f1a867bb01330d98d54c0ad2b29ed1770b`)
is now `pe/reference/pe32-crt-onexit-v1.exe`. Its unedited native raw report is
`pe/reference/pe32-crt-onexit-windows-v1.json`, SHA-256
`159566d99483db524c217eb47edfa744f303e3cf2bca60219ccd44de1dd9a0e2`.
All producer source and PE identities are pinned. All 53 output words now agree
on the physical M4, covering lock recursion, real preallocated empty/populated
callback tables, invocation order/side effects, caller stacks, guards, inherited
startup handles and A/W path/error relations. Five excluded profiles stop with 78.
Three transport/frozen-evidence tests pass, and actual original startup still
reaches the same `GetWindowsDirectoryA` blocker. No game entry/parity claim is made.

That native job remains a failed job: the comparator caught the ANSI discrepancy,
and later heap observations were consequently skipped. This record claims only
the completed 53-word observation and independent SDK check. A fresh full pipeline
with the corrected implementation remains separate.

## Original Bink startup reads the physical desktop cursor

Bink's original `GetCursorPos` now performs a real read-only Quartz event query
on the physical M4, followed by its tick query, thread-notification policy change
and actual module-path query. Its next measured unsupported call is
**KERNEL32.dll!GetWindowsDirectoryA**. The game EXE has not entered. Original game
files, per-module captures and inventories remain local with the existing unpacked
dependency caveat unchanged.

The cursor bridge writes two checked signed 32-bit LONG fields from the actual
host query. Its explicit profile uses Quartz logical desktop coordinates with
floor quantization, including negative monitor coordinates. Nonfinite/out-of-range
coordinates, an unavailable host event source and invalid guest storage stop.
No cursor is moved and no input events are injected. This is not DPI virtualization,
a Windows input-desktop model or screen-topology parity. See the
[GetCursorPos contract](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getcursorpos).

The owned PE passes on the physical M4 with actual coordinates, POINT guards,
stdcall stack balance and preserved last error. A bad-pointer PE stops with 78.
Conversion boundary tests and two transport tests pass, as do Clippy and formatting.
Native SDK/identical-PE execution are added to CI and pending. Only ABI/guard/error
fields are compared across hosts; actual coordinates remain explicitly per-host
observations, not fixed cross-machine equality claims.

## Original dependency startup reaches Bink

The first native exit-table witness at `7c3275d` failed with access violation
before producing its buffered record. That witness incorrectly attempted a NULL
initial table; its successful host-only behavior was not native evidence. The
backend now stops that unverified case, and the next witness uses allocated
storage with an empty entry span, matching the table ownership required in the
actual startup path. Expected callback/ABI/error fields are unchanged. The failed
native report is retained locally. The revised native comparison remains pending.

On the physical M4, the original bundled CRT's reached `HeapReAlloc` now resizes
an actual live allocation with preserved content. Dependency traversal completes
the PhysX-related and bundled C++ runtime initializers, then enters SpeedTree
and Bink. Bink's first measured unsupported import is **USER32.dll!GetCursorPos**.
**The game EXE has not entered.** No game bytes/inventories are committed; the
previously documented locally unpacked dependency caveat remains.

The bounded allocator retains live ownership, uses reserved capacity for shrink
or small growth, and otherwise allocates/copies/frees real low-address storage.
`HEAP_ZERO_MEMORY` clears only newly exposed bytes; in-place-only growth fails
without mutation when owned capacity is insufficient. Allocation failure retains
the original block and last error. Unknown/interior/freed pointers, foreign heaps
and exception-mode flags stop explicitly. The profile uses the existing 512 MiB
allocation budget and single guest thread; it does not claim native fragmentation
or multithread scheduling behavior. These contracts follow the
[Windows HeapReAlloc specification](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heaprealloc).

The owned 64-word PE passes on the physical M4 for both process and private heaps:
actual stdcall stack balance, eight-byte alignment, requested sizes, preserved
prefixes, zeroed regrowth after shrink, in-place shrink, impossible-size failure
with the old block retained, error state and eventual free. Three negative PEs
stop for their actual pointer/heap/flag reason. Metadata unit checks separately
exercise failed in-place growth and invalid ownership without dereferencing fake
addresses. The two transport integrity tests, targeted Rust heap test, formatting
and Clippy pass. Identical-PE native comparison is wired into CI and pending.

## D3DX9 initialization completes; original PhysX startup reaches heap reallocation

On the physical M4, actual original D3DX9 constructors now return and dependency
traversal enters PhysX initialization. Real recursive CRT lock 8 and
`__dllonexit` registration allow this advance. The latter validates live owned
table storage, allocates/copies/appends/frees actual guest memory and returns the
registered callback. Registration does not eagerly execute callbacks: the
original caller still owns table traversal. Other lock IDs, contention,
unbalanced ownership and invalid storage stop explicitly; this is not a general
CRT destructor/termination implementation.

The original PhysX startup next consumes `GetStartupInfoW` using the same explicit
68-byte inherited-stream profile as A (no desktop/title/private inheritance
block). `GetModuleFileNameA` supports complete ASCII-backed registered paths;
truncation, non-ASCII origins and unknown modules still stop. The next actual
unsupported import is **HeapReAlloc** in the original bundled CRT. The game EXE
has not entered. The existing locally unpacked dependency caveat remains; game
bytes, module/RVA captures and derived inventories stay local.

The owned `crt-onexit-0.exe` records 53 words on the physical M4: recursive
cdecl calls and stack preservation, preallocated empty and populated callback
tables, preserved entries and guards, reverse invocation order/side effects,
A/W startup layouts and actual inherited handle relations, and complete ASCII
A/W path agreement with error state. Five restriction PEs stop with 78 for their
actual lock/pointer reason. Windows SDK layout checking and identical-PE native
execution are wired into CI; those results are pending for this milestone.
Local checks pass: 192 PE Rust tests/checks including doctests on the Rosetta
x86_64 target, 374 Python tests, Clippy with project warnings denied and formatting.

The [native job at `74f870f`](https://github.com/daeken/Alkyhol/actions/runs/42497907786)
also completed all 204 frozen atan observations and the additional 16-row domain
witness. The identical domain PE (SHA-256
`c215904f56006e639db2aff613dea4bb51f7c06cc2571027724f3d4fe825d314`)
was downloaded and executed on the physical M4. All 16 raw and decoded rows
agree with Windows, including the five new signed/boundary cases, x87 control/
status/stack, MXCSR and both error states. This leaves the documented 0x027F and
other unsupported profiles excluded; no FP criteria or frozen fixtures changed.

## Original D3DX9 constructor passes arctangent

On the physical M4, the original constructor executes beyond `_CIatan` and now
stops at **msvcrt.dll!_lock**. The table and DLL initialization have not completed;
**game EXE entry has not run**. Original EXE/CRT bytes are unchanged, and the
existing unpacked local dependency caveat remains. No game compatibility is claimed.

The [native startup job at `5f2a746`](https://github.com/daeken/Alkyhol/actions/runs/42495734383)
completed 204 actual arctangent observations through the unchanged raw 80-bit
transport. Its operator-specific imports were independently checked with LLVM.
The full native report is `pe/reference/pe32-ci-atan-windows-v1.json`, SHA-256
`ce77f0714d4d0d7e85e415c00888d9288cf7eafd6803d778bb252fbbd28286b2`.
The literal owned PE is `pe/reference/pe32-ci-atan-v1.exe`, SHA-256
`f69a886c8305efa3fbb62deab9ca439bcbac081987437d021e205933155c60d0`.
All six producer sources and the full native baseline remain pinned; no sqrt
producer, fixture or acceptance field was changed for the new operator.

The initial two-control host candidate **failed** comparison: native C1 differed
under 0x027F, and the tiny operand lacked native sticky precision under both
controls. Results agreed for the corresponding observed operands, but those
state differences prevent parity. The failed report and candidate PE/source are
retained locally. The cause of the C1 difference has not been independently
isolated, and 0x027F is explicitly rejected by the supported backend.

The admitted 0x037F fully masked round-to-nearest profile computes on the actual
x87 stack, narrows the return value to binary64 as observed, and preserves sticky
precision for nonzero finite operands while retaining the rest of the FP
environment. Canonical signed operands in the binary64 finite magnitude range
and signed zeros are admitted; the needed extra physical stack slot is checked.
All **11 corresponding native rows** now agree on the physical M4, including
80-bit results, stack/control/status, MXCSR and both error states. A separate
16-row identical-PE witness adds five signed/boundary cases; those additional
native comparisons are pending. Five host-only exclusions (other precision,
too-small extended magnitude, NaN, too-large magnitude and 0x027F) stop with 78
for the actual profile reason. This is not general arctangent, math-error-hook or
floating-point startup equivalence.

Local checks pass: 191 PE Rust tests/checks including doctests on the x86_64
Rosetta target, 372 Python tests, Clippy with project warnings denied, formatting,
the five atan profile/restriction cases plus the valid witness, the existing
sqrt cases and CRT cases. The actual original graph advances to the `_lock`
import. The five added signed/boundary rows still need native execution.

## Original D3DX9 constructor passes the first x87 intrinsic (earlier milestone)

For the next reached `_CIatan` call, an operator-specific native observer reuses
the unchanged owned numeric/state transport through explicit C/assembly wrappers.
LLVM import inspection confirms the PE calls `_CIatan`, not `_CIsqrt`. The pinned
sqrt producer is unchanged. Its build is successful; actual native arctangent
results and a backend implementation remain separate pending work.

On the physical M4, the original constructor now executes past `_CIsqrt` using
a dedicated guest-i386 x87 gate. The next actual unsupported import is
**_CIatan**. The DLL constructor table has not completed and **game EXE entry
has not run**. The existing local unpacked-dependency caveat remains; no game
compatibility or unchanged-all-DLL claim is made.

The [native startup job at `b13af7d`](https://github.com/daeken/Alkyhol/actions/runs/42491731440)
completed all 204 owned `_CIsqrt` observations: 17 raw 80-bit operands in 12
precision/rounding contexts. It retains actual results, lower-stack witness,
control/status words, MXCSR, last error and errno. The full report is frozen at
`pe/reference/pe32-ci-sqrt-windows-v1.json`, SHA-256
`7b1d29fd99c35c12a318f0a0d70661a943c78b25c181ba4f1065faed10930374`.
The literal owned PE is `pe/reference/pe32-ci-sqrt-v1.exe`, SHA-256
`8035d5e88c0f91dbb5c4b03e7bf91c31b9d43690ee82a230afc1cf5b4c1a389f`.
All four original producer sources remain byte-pinned. Negative/NaN and other
control-mode observations are preserved as evidence, not silently declared
implemented by the backend.

The bounded implementation admits nonnegative canonical operands within the
binary64 finite magnitude range (including the minimum subnormal and signed
zeros), control words 0x027F/0x037F, and the measured MXCSR controls. It computes
on the actual x87 operand, with mode-dependent binary64 result narrowing. The
maximum finite operand distinguishes a real double-rounding difference between
these modes in the native record. All **20 corresponding native rows** match
on the physical M4, including 80-bit results and every recorded state/error
field. This is not a plain integer gateway or an assumption that host sqrt
matches the intrinsic. All GPRs/flags and lower FP stack are preserved by the
gate. Its checked rejection path retains the real snapshot on owned guest stack
storage and stops unsupported profiles with 78.

A separate 24-row identical-PE witness adds two inputs per control mode, including
an 80-bit operand just above a binary64 square-root rounding threshold. These
four new rows subsequently pass the native comparison described below.
Four host-only restrictions (other precision, negative, NaN, oversized magnitude)
stop for the actual profile rejection reason; no NaN or errno success is fabricated.
The `_errno` accessor supplies stable aligned zero-initialized guest storage;
the witness checks its initial value, alias stability and writable sentinel.
General errno/math-error hooks, arbitrary rounding/exception masks and guest-thread
FP startup-state equivalence are not established. Original inputs, snapshots and
module/RVA traces stay local.

Local milestone checks pass: 190 PE Rust tests/checks including doctests on the
x86_64 Rosetta target, 370 Python tests, project Clippy with warnings denied,
formatting, all five bounded sqrt/restriction PEs, the 13 existing CRT PEs and
the 25 startup cases. The actual original graph repeats the `_CIatan` stop after
passing the square-root call. These checks do not establish full floating-point
or original-game equivalence.

The [native startup job at `4281503`](https://github.com/daeken/Alkyhol/actions/runs/42494716584)
passes the identical-PE bounded comparison: all **24 records** agree with the
host, including the four new rounding-boundary/pi rows. The exact CI-produced
domain PE also executes on the physical M4 and agrees on every recorded result,
stack witness, control/status, MXCSR, errno and last-error field. Its SHA-256 is
`3f7624c7488a0a1f95e5e91823f6603a1aae3b6d4e46fe05f0895e6737867037`.
All four unsupported-profile cases still stop honestly, and native Windows
reproduces all 204 frozen full-observer records without altering that baseline.
The original constructor's square-root call now completes on the M4; the next
_CIatan call remains unimplemented.

## Original D3DX9 constructors execute through the CRT gateway (earlier milestone)

For the next actually reached `_CIsqrt` blocker, an owned native observer now
loads raw 80-bit operands and calls the real x86 intrinsic directly from
assembly. It captures 17 numeric inputs in 12 precision/rounding contexts,
including an underlying x87 stack witness, control/status words, MXCSR, last
error and errno. The observer does not convert through a C `double` signature.
At this earlier instrumentation milestone, native execution was pending and its
build/transport tests made no backend parity claim. The Microsoft [intrinsic contract](https://learn.microsoft.com/en-us/cpp/c-runtime-library/cisqrt?view=msvc-170)
documents the specialized x86 convention. Inputs are original numeric fixtures,
not extracted game data.

On the physical M4, the original D3DX9 helper now calls `msvcrt.dll!malloc` and
receives actual owned low-address storage. The game EXE remains unchanged and
its entry has **not run**. Its reached GetVersion call now uses the existing
legacy NT 6.2/build 9200 persona, sharing fields with GetVersionExA and agreeing
with the frozen native packed value. `_initterm` now enters that DLL's original
constructor callbacks. The next measured unsupported call is **_CIsqrt** inside
one of those callbacks. The table has not completed. This math intrinsic needs
its x87 operand/result convention, not an integer cdecl stand-in.
The local unpacked dependency caveat remains;
this is not game compatibility or unchanged-all-DLL execution.

The CRT call required real cdecl gateways with caller stack cleanup. It also
makes the implemented system CRT namespace visible during original bundled CRT
initialization. Its reached configuration lookups now operate on owned error-mode,
application-type, terminate-handler and unexpected-handler state; they do not
silently report that namespace or those registered exports as missing. Handler
targets are checked against executable guest storage. NULL is the documented
restorable default for these setters, not a fabricated successful callback.
See Microsoft's [malloc](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/malloc?view=msvc-170),
[error policy](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/set-error-mode?view=msvc-170)
and [termination policy](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/set-terminate-crt?view=msvc-170)
contracts. No third-party implementation code was copied.

An owned PE exercises 0/1/17/4099/65539-byte allocations, 8-byte alignment,
nonoverlapping data retention/checksums, actual cdecl stack restoration, policy
mutation/getters, previous-handler identities and last-error preservation. Four
separate owned PEs execute real cdecl callbacks and propagate actual ExitProcess
statuses 37/38/37/37, including an unexpected handler returning into terminate and
an initializer escaping through ExitProcess. Constructor tests also check order,
NULL skipping, empty ranges, reentrant CRT calls and live mutation of a later
entry by an earlier constructor. The table is not prematurely snapshotted.
Eight additional host-only paths stop for their measured reasons: default abort,
returning terminate callback, invalid callback target, oversized allocation,
invalid error-mode value, invalid application type, inverted constructor range and
invalid constructor target. They are not run against
native Windows and are not claimed as native error semantics. Existing stdcall
FLS callback behavior is separately regression-checked.

Default abort/reporting, errno/new-handler allocation failure, free/realloc,
arbitrary guest threads and C++ exception unwinding remain unsupported. The new
CRT PE/state and callback-exit observations have been added to the existing
Windows comparison workflow. The first native attempt at `0f49bf7` failed before output with
0xC0000139 (entry point not found). A separate kernel-only observer completed in
the [native startup job at `a9be6e7`](https://github.com/daeken/Alkyhol/actions/runs/42489647337)
and isolated the unavailable import: **__get_app_type is absent**, returning
NULL/error 127 from GetProcAddress. The other nine names, including
__set_app_type, are present; successful lookups preserve last error. The getter
was removed from the compatibility exports, and this exact optional lookup now
returns the measured missing-export result. Other unknown exports still stop.
This is a measured distinction between an absent export and an unimplemented API.

The literal owned kernel-only PE is frozen at `pe/reference/pe32-crt-exports-v1.exe`,
SHA-256 `7e4579a9123da58054bb49b371f9975da7ffc07b7e5f8f50f019db981eba871e`.
Its raw report is `pe/reference/pe32-crt-exports-windows-v1.json`, SHA-256
`596b2963e9a5cc85c03b4b82c7212d5e2bb6f017bd048ef5fdf2d1a022c473f9`.
Only the completed 36-word export/packed-version collector is frozen evidence;
that raw report deliberately also retains the subsequent failed CRT-state PE.
It is **not** a successful whole-job or whole-CRT observation. The collector's
exact sources and literal PE stay pinned while the separate state/constructor PE
is corrected to avoid the nonexistent static import. GetVersion is 0x23F00206,
last error is preserved, and the observer releases its acquired DLL reference.
Host assertions are not a native original-game observation. Original
files, traces, identities and launch inputs remain ignored/local.

Local checks pass: 188 PE Rust tests/checks including doctests on the x86_64
Rosetta target, 366 Python tests, project Clippy with warnings denied, formatting,
all thirteen host CRT PEs and the existing 25 startup cases (including real
stdcall FLS callbacks). The frozen export collector is native-only. The actual
DAO graph passes corrected optional-export handling, CRT configuration, D3DX9
malloc, GetVersion and the original constructor dispatch before stopping at
_CIsqrt. No constructor is skipped and no math result is fabricated.

The corrected [native startup job at `b3b862f`](https://github.com/daeken/Alkyhol/actions/runs/42490671383)
now passes. Its **80 normalized words** match the hosted Rosetta observation;
the identical CI-produced PEs also pass on the physical M4 and agree with all
80 native words and all four callback/initializer exit codes (37/38/37/37).
The state/storage/constructor PE SHA-256 is
`5eadf38f46d1d627f94dc9a46de7c978f04fb67a4aa137dd4b4fe22aad8881ac`.
Both default-handler tokens are actually NULL in this native observation, and
the optional `__get_app_type` lookup returns the measured NULL/error 127 result
on both backends. Pointer identities are compared through caller checks rather
than cross-machine addresses. All eight host-only unsupported cases still stop
with 78 for their exact expected reasons. This establishes the finite owned CRT
contract, not original-game execution on Windows or general CRT/C++ EH parity.

## Dependency startup reaches original D3DX9 initialization (earlier milestone)

On the physical M4, DLL initialization now advances beyond FMOD and enters the
original D3DX9 helper. The reached DisableThreadLibraryCalls calls update real
per-module notification policy. Its handle checks, repeat behavior and module
isolation are unit-tested. Process initialization/teardown remain enabled; guest
thread creation and native delivery/suppression comparison remain unsupported.
Process DLL static TLS is rejected before entry, so the gateway cannot claim
success for an active-static-TLS module. The next measured unsupported operation
is **msvcrt.dll!malloc** during D3DX9 initialization. **Game EXE entry has not run**;
the existing unpacked local dependency caveat still applies. No game files or
derived inventories are committed.

The [native startup job at `9ef66ab`](https://github.com/daeken/Alkyhol/actions/runs/42486833009)
completed the calendar observer and independent SDK layout check. The raw report
is frozen at `pe/reference/pe32-calendar-filetime-windows-v1.json`, SHA-256
`472812221c1704d1bdb0727fb5becc562ffba9235dc7c6ba5c8240d8c1237fa5`.
The identical owned, non-game PE is `pe/reference/pe32-calendar-filetime-v1.exe`,
SHA-256 `e1383422f2651afcbd699732bb9dea658e22d370315938127b6055b0204d890e`.
All **444 bytes** agree on the physical M4 at normal and forced placement:
epochs, Gregorian centuries/leap day, submillisecond behavior, maximum positive
signed FILETIME, and both invalid high-bit patterns. Native invalid patterns
return zero/error 87 and leave all 16 SYSTEMTIME bytes untouched; the Rust gateway
now does the same. Inputs and surrounding guards are preserved.

That native job subsequently **failed** the startup clock/atomic PE with status
0xC000001D and no output. This is not successful native clock evidence. The failed
raw report and original inputs remain local. A too-short two-million-iteration
tick polling budget is a hypothesis, not an isolated fault observation. The
owned probe now retains a finite 100-million-iteration ceiling and the same
20-second parent deadline, and emits its accumulated words plus a failure-stage
tag before either explicit diagnostic trap. All PID, rate, boot-time, error,
guard and atomic acceptance checks are unchanged. The revised PE passes on the
physical M4. No frozen baseline is altered to accept the failed run.

The revised observer subsequently completed in the [native startup job at
`905214e`](https://github.com/daeken/Alkyhol/actions/runs/42487490117). Its exact PE,
SHA-256 `c41b400516ed906e10fd91cc3811a77f2abf841466ad826be00858788e165acd`,
also executes successfully on the physical M4 after downloading the unchanged CI
input artifact. Both executions pass the unchanged PID, boot-time bracket,
counter-rate, atomic-bit, guard and last-error checks. Windows reports a 10 MHz
counter and the M4 gateway reports its real 1 GHz provider; these are separate
per-host clocks, not identical counter words. Both report a 32 ms tick interval;
measured counter intervals are 20.8626 ms on Windows and 31.867875 ms on the M4,
within the existing 25 ms quantization bound. The exact cause of the earlier
unreported trap remains unisolated. Native calendar checks and the other startup
comparators also pass at `905214e`; the broader workflow is separate.

This milestone passes 186 PE Rust tests/checks including doctests on the physical
M4's x86_64 Rosetta target, 361 Python tests, project Clippy with warnings denied,
formatting, the exact native calendar PE at normal/forced placement and the revised
live clock/atomic PE. The actual DAO profile repeats the D3DX9 malloc stop.

## Original CRT completes; dependency startup reaches FMOD (earlier milestone)

On the physical M4, original `msvcr80.dll` initialization now returns successfully.
It reads actual assembly-manifest metadata, executes local/calendar FILETIME
conversions under an explicit guest UTC profile, and closes its real search
handle. Dependency traversal then enters FMOD initialization. Its cookie/lock
setup executes real host process ID, boot ticks, performance counter,
InterlockedCompareExchange and InterlockedExchange calls. The next measured
unsupported operation is **DisableThreadLibraryCalls** in FMOD. That initializer
has not completed; **the game EXE entry has not run**. The EXE and original CRT
remain unchanged. The already documented local unpacked diagnostic dependency is
still in use, so this is not unchanged-all-DLL execution or game compatibility.
Original files and module/RVA/argument captures remain ignored/local.

The [native startup job at `60d977e`](https://github.com/daeken/Alkyhol/actions/runs/42483854331)
completed the owned UTC FILETIME observer. Its literal PE is frozen at
`pe/reference/pe32-local-filetime-v1.exe`, SHA-256
`89d7153190d43e0840708c6d671b32c3cd97cb534ebed38b6b23e57db601be58`.
The raw native report is `pe/reference/pe32-local-filetime-windows-v1.json`,
SHA-256 `11c4b77655abc70fd00549ef44af733176c53602c9f0ebe7a18fe166f4096f2f`.
All 106 DWORDs agree on the physical M4: native UTC qualification, results,
unchanged last error, guards, source preservation and both exact-alias cases.
High-bit FILETIMEs are valid for this measured UTC local conversion. Unknown and
unsupported guest timezones stop before producing results. Guest UTC is explicit;
the host timezone is unchanged and localized timezone names are not compared.

Valid FILETIME-to-SYSTEMTIME conversion uses bounded Gregorian integer arithmetic.
Host units check the 1601/1970 epochs, submillisecond truncation, 1900/2000/2024
leap boundaries and maximum positive signed FILETIME. Native calendar-field and
invalid-range observations have been added to the existing Windows workflow but
have not yet run. High-bit calendar inputs currently stop explicitly instead of
inventing a native error code.

The new owned startup clock/atomic PE executes on the physical M4 and checks the
actual child PID, live boot ticks, counter frequency/rate, old/new atomic bits,
guards and unchanged last error. Counter storage accepts genuine DWORD alignment,
including the original dependency's observed buffer. Counter/tick epochs and PIDs
are checked per host, not compared as cross-machine constants. Native execution
of this new PE is pending. The tick comparator originally used CLOCK_MONOTONIC
and correctly failed an approximately 5.6-second absolute offset; its failed raw
report is retained locally. Apple's [Libc clock implementation](https://github.com/apple-oss-distributions/Libc/blob/main/gen/clock_gettime.c)
identifies CLOCK_MONOTONIC_RAW as the continuous Mach timebase. The SDK-typed
reader now selects that API, and the unchanged tight boot-time bracket passes.
This checks gateway/scaling agreement through another host API, not independence
from the operating system's underlying clock or a measured cause for that offset.

Local milestone checks passed: 186 PE Rust tests/checks including doctests on
the x86_64 Rosetta target, 359 Python tests, Clippy with warnings denied for the
project, formatting, the exact frozen UTC PE and the live startup clock/atomic PE.
The vendored XML parser retains its existing dependency warnings. These checks
do not substitute for pending native calendar/clock execution or original-game
runtime observations.

## Original CRT obtains real assembly-manifest file metadata (earlier milestone)

On the physical M4, the original CRT now executes FindFirstFileW successfully
against its genuine backed assembly-manifest file. It receives a real owned
search handle and the file's actual size, name and creation/access/write times.
The next measured unsupported call is **FileTimeToLocalFileTime**; game entry
has **not run**. All original data and detailed captures remain ignored/local.

The corrected owned observer at `9630990` completed on native Windows in the
[startup job](https://github.com/daeken/Alkyhol/actions/runs/42481671891).
Its valid search/exhaustion/close lifecycle, missing file and missing parent
produce 21 normalized DWORDs. Successful calls preserve last error; exhaustion
sets 18, real missing files set 2, and missing parents set 3. Guarded outputs
remain untouched on those failures. Native success fills the first 36 metadata
bytes, the actual terminated filename and the first alternate-name WCHAR; the
reserved DWORDs and unused name tails retain the caller's sentinel. These holes
are preserved by the Rust gateway. The SDK independently confirms all relevant
592-byte structure offsets.

The raw native report is frozen at `pe/reference/pe32-find-file-windows-v1.json`,
SHA-256 `b09164fb590477d5bb6bb8d20eca305387ab99446443ada0d29944eee9297c82`.
The literal **owned, non-game** 4 KiB PE is retained as
`pe/reference/pe32-find-file-v1.exe`, SHA-256
`a1d7b6e3ca1b958fd91f71c22cae13c38820912739113158825e88c8a4e15fa5`.
Its exact C/DEF/SDK-check sources and input identities remain mandatory. Local
Apple Clang 21 emits different instruction bytes from this CI-produced image;
that recompile is not represented as the identical native PE. Select the literal
reference explicitly with `observe_pe32_find_file.py --frozen-fixture`, then use
`--host`. CI does the same. The source builder remains available and fails an
identity mismatch instead of silently substituting different code. The cause of
the compiler-output difference has not been independently isolated.

Normal and forced-relocation runs of that literal PE on the physical M4 agree
with every native non-time record byte and all 21 normalized words. Only the
three real FILETIMEs are normalized across machines: each is first checked
against independently read actual filesystem metadata on its own host. Windows
uses its parent process's stat data; macOS uses a separate SDK-typed C stat reader
to retain full birth-time precision. This is filesystem evidence, not a frozen
clock value or unmeasured native original-game comparison.

The PE32 search profile admits exact ordinary canonical Z: paths and real
regular-file/directory metadata. It returns owned inaccessible low-address
handle tokens, checks each actual path component and preserves absent-file vs
absent-parent errors. Unsupported aliases, DOS devices, wildcard searches,
symlinks/reparse points, ambiguous spellings and exhausted budgets stop explicitly.
Its translated filesystem has no generated 8.3 aliases; initial DOS attributes
are archive for regular files or directory for directories, plus backed POSIX
read-only state. This is an explicit filesystem profile, not an NTFS attribute
or ACL dump. Closed/unowned handle operations stop; no native stale-handle
failure behavior is claimed. Remaining owned searches close after DLL teardown.

Local validation passes 182 PE tests/checks, 354 Python tests, Clippy and both
literal-PE M4 runs. Prior PE64 metadata behavior remains unchanged; only internal
visibility is shared. Current-code CI is pending. The next native-only observer
now builds for the actual FileTimeToLocalFileTime call, measuring a verified UTC
Windows timezone, valid/boundary inputs, aliases, errors and output guards. Its
execution and a host local-time implementation are still pending; host-local
DST behavior is not guessed.

## Exact-file observer correction (no successful file-search evidence yet)

The first owned file-search observer at `a3443c6` exited on native Windows with
`0xc0000005` and no emitted record. Its raw failed report and exact input identities
are retained locally; no successful API or game result is claimed. That observer
combined valid lifecycle operations with use of an already closed search handle.
The fault location was not measured, so a stale-handle cause remains a hypothesis.
The corrected, still-unfrozen observer records bounded progress markers and covers
only a valid exact-file search, exhaustion, one close and genuine missing file/
parent queries. Closed-handle native behavior is not an acceptance assumption.
The Windows SDK's independent 592-byte structure/offset check passed. All prior
frozen comparisons remain unchanged; the new native execution is pending.

## DAO executes the shared Windows-directory query

The original CRT now executes GetSystemWindowsDirectoryW on the physical M4 and
reaches **FindFirstFileW** for the actual backed assembly-manifest path. File
search still stops explicitly after a checked diagnostic pattern read; no
successful handle or missing-file result is fabricated. Game entry has **not
run**. All original path/argument captures remain ignored/local.

The shared Windows-directory role is separately and explicitly configured in
`ProcessOptions32` / `--system-windows-directory`, validated as an existing host
directory before any initializer. It is not inferred from System32 or DLL paths.
A small refactor shares checked UTF-16 sizing/copy code with the existing system
directory API while retaining independent roles and existing restrictions.
The actual local profile uses its genuine Windows folder and preserves original
CRT origins; this does not install Windows or replace those origins.

The unchanged owned i386 API-name variant ran on native Windows at `d993c7c`
in the [startup job](https://github.com/daeken/Alkyhol/actions/runs/42480098934).
All 18 normalized words agree with the physical M4, including full/exact/short/
zero sizing, terminators, guards, untouched short output and preserved last error.
Each complete path is independently checked: Windows reports its real OS root,
while Alkyhol reports its actual supplied profile directory, including supplementary
UTF-16 units. The raw native report is frozen at
`pe/reference/pe32-system-windows-directory-windows-v1.json`, SHA-256
`eac73c7dbf686427f111d279655157bf15c36b0a3998ba33f8cc257e06b42ae7`.
Source and identical-PE identities are mandatory, without an initial-capture bypass.

Five host cases pass (one positive with distinct root/System32 roles and four
restrictions), as do all five prior system-directory cases, two new frozen
transport tests and Clippy. The next owned native-only exact-file observer now
builds with open Clang/LLD; its Windows execution is pending. It will record the
592-byte find-data layout, real per-host timestamps/size/name, output guards,
search exhaustion, handle close/stale-handle behavior and genuine absent-file/
absent-parent results. Its SDK layout is checked independently with MSVC. No
original game/runtime files enter that observer or CI. It does not yet establish
host file-search support or native full-original-graph parity.

## DAO passes its original CRT assembly lookup

The physical M4 now executes past the original CRT's FindActCtxSectionStringW
call with real scoped binding data. The next measured unsupported operation is
its dynamic lookup of **GetSystemWindowsDirectoryW**. The game EXE entry has
**not run**. This is original CRT startup progress, not game compatibility or a
native full-game comparison. The original EXE and runtime DLL bytes remain
unchanged; all original identities, file provenance and captures stay local.

The process profile explicitly registers the unmodified extracted CRT assembly
manifest and publisher-policy manifest. Both policy ranges are parsed with exact
inclusive version bounds, family/platform/token checks and a backed target
assembly. Missing, out-of-range, wrong-token, duplicate or unknown policies fail
before initialization. This is a caller-trusted installed-policy inventory,
**not** Windows Installer, catalog/PKI validation, automatic WinSxS search or a
native observation of publisher-policy installation. Native policy trust and a
full original Windows graph comparison remain unmeasured. No unavailable trust
API returns invented success. The provided modules retain their actual source
paths and the separate system-directory role; no path substitution bypasses the
original check. The third CRT member is backed by a byte-identical extracted file
and is not imported, mapped or executed.

Three additional owned native layouts passed at `4a6ef89` in the
[Windows startup job](https://github.com/daeken/Alkyhol/actions/runs/42478710865).
An empty resource-2 DLL has a real non-NULL active context during its own and its
manifest-free provider's attach/detach, yet every DLL-key query misses. Its
explicitly created/activated empty context has a second distinct handle; EXE
default/restored scope remains NULL. Exact-version token-bearing private assembly
metadata also binds without a fabricated certificate validation claim. Finally,
a three-file assembly produces a 324-byte section with a real three-bucket hash
dictionary; two files are deliberately unimported, so no execution of those files
is claimed. The complete raw native report is frozen at
`pe/reference/pe32-assembly-layout-windows-v1.json`, SHA-256
`447013659b9d99cc5e25bcb62daf7328cf547333c904c04044aaae6416622b79`.

All three identical-PE layouts now run on the physical M4 and match every frozen
context relationship, scalar and section byte. Empty DLL contexts own real
reference-counted handles without inventing DLL data. One- and three-file
sections use owned low read-only buffers; other dictionary sizes remain
unsupported. Token identities and the known SHA-1/Identity-transform metadata
schema are parsed, following the separate native private-loader observation that
those hash declarations were not enforced during this runtime binding. This does
not establish signature validation or shared-assembly installation integrity.
The default empty-only/diagnostic loader contract stays separate.

An owned host-only policy model additionally runs the unchanged token-bearing PE
chain against an actual newer registered manifest and an explicit matching policy.
Its scoped lookups agree with the frozen exact-version native layout. Five
negative policy profiles stop for their specific reason with no initializer
output. **This is host model evidence, not identical native policy configuration.**
Local checks pass 181 PE tests/checks, 349 Python tests, Clippy, the six old scoped
activation cases, all three new layouts and six policy-model cases. Current-code
CI remains separate/pending; the native captures above identify their own exact
revisions. The next native-only owned observer builds the existing C directory
sizing/guard probe with its API name selected as GetSystemWindowsDirectoryW.
Its Windows result and host implementation are still pending.

## Rust PE32 activation scopes execute the frozen Windows observer

The native metadata experiment at `3e755c7` completed in the
[Windows startup job](https://github.com/daeken/Alkyhol/actions/runs/42478104697).
Windows rejected both the newer-version private assembly and the combined
newer-version/token case before observer output, with process status
`0xc0150002`. The exact-version case with deliberately incorrect SHA-1 metadata
and noInheritable executed all seven lifecycle events successfully; every scope,
query scalar and complete section byte matched the baseline. Thus private
version matching cannot simply be relaxed to use the supplied newer CRT. This
finite private-loader observation also distinguishes file-hash metadata from a
runtime integrity check; it does not establish catalog verification, shared
WinSxS installation or policy trust.

The raw CRLF report is frozen in
`pe/reference/pe32-assembly-metadata-windows-v1.json`, SHA-256
`b6d946b0858252e9e18eb6688b61962ac5cadab602a8f5cd8f9a12ba59e06b4c`.
Builds and reruns require exact original owned inputs and all three native
outcomes. No original game/runtime data is used. The follow-up native-only
`observe_pe32_assembly_layout.py` now builds the three remaining layouts relevant
to the actual graph: an empty resource-2 DLL, a three-file assembly (two files
remain unimported), and an exact-version token-bearing private assembly. These
new Windows observations are pending; no host restriction has been relaxed
based on them. Synthetic checks preserve the existing frozen key requirement
while permitting the separate empty-context observer to record real misses.

The direct blocker is still DAO's original signed CRT assembly/publisher-policy
binding. Game entry has **not run**. The next runtime component now executes:
the unchanged owned EXE and two DLLs from the native scope experiment below run
on the physical M4 through Alkyhol's new explicit assembly profile. Normal and
forced-relocation runs match every frozen lifecycle observation, normalized
context identity, query scalar and complete 124-byte redirection section. Actual
module paths are checked independently against each host's provided files.

`ProcessOptions32::activation_assemblies` opts into a bounded inventory of real
assembly manifests. Each file declaration must resolve to the genuine provided
module source; exact unsigned x86 identity/version matching is required. The
current admitted schema is an empty EXE default plus resource-2 DLLs with one
exact dependency on a single-file assembly. There is no filesystem search or
version fallback. Unknown elements, hashes/signing/policy metadata, multiple
files/dependencies, ambiguous resource languages, nonempty EXE defaults and
unmeasured empty resource-2 DLL scopes stop explicitly. The complete opt-in graph
is validated before **any** initializer, even if that code would never query an
activation API. The older empty-only/diagnostic profile remains separate.

A manifest-free dependency inherits its first importer's scope; a resource-2 DLL
supplies its own scope. Each original initializer and reverse teardown runs in
that saved scope, restoring the caller afterward. The runtime owns real low
read-only section buffers and opaque inaccessible handle tokens, with reference
counts for GetCurrentActCtx, returned query handles, created contexts and explicit
activation stack entries. CreateActCtxW currently admits the measured module +
resource-2 form; activation cookies are checked in stack order. Unknown handles,
stale cookies, unsupported forms and unbalanced initializer activation stop.
FindActCtxSectionStringW preserves native output holes, tail bytes and last error
on success; a real supported-scope miss retains all output and sets 14007.

Four host-only restrictions use the same unchanged PEs: absent assembly,
duplicate identity, a substituted module origin and a mismatched assembly version.
Each stops with status 78, no observation output and its specific reason. Parser,
scope-inheritance and complete-section tests cover the corresponding logical
units. The full pre-final PE suite passed 176 tests/checks; a subsequent focused
run includes the additional role/version refusal test. All 343 Python tests and
Clippy pass. The host and native startup jobs passed at `8affdd7` and `3e755c7`, including
the complete frozen comparator; broader CI results remain recorded separately.

A local run of the original DAO graph with the default profile still reaches the
same original FindActCtxSectionStringW frontier and stops 78. Opting into the new
profile with an empty inventory refuses the unsupported original manifest
attributes before original initialization; it does not turn those dependencies
into absent keys. Original signed identity/hash metadata, multiple CRT files and
publisher-policy redirects remain the next implementation work. All original
captures and dependency files stay ignored/local, and no DAODecomp code changed.

## Measured Windows DLL activation scopes for DAO's actual blocker

DAO still stops in its original CRT's FindActCtxSectionStringW call; game entry
has **not run**. An owned native observer now mirrors the relevant loader pattern:
an empty-default EXE imports a resource-2 DLL, which statically imports a provider
without an embedded manifest. The provider exists only in a private assembly
folder whose manifest declares an exact version, so a flat-directory fallback
cannot masquerade as binding. All code/manifests are original MIT fixtures, not
extracted game/runtime data. macOS builds the unchanged i386 PEs with open
Clang/LLD; **this observer has executed on native Windows only**.

At `de02c26`, the actual [Windows startup job](https://github.com/daeken/Alkyhol/actions/runs/42476456674)
passed all prior frozen comparisons and this new experiment. The provider and
importer both see the same non-NULL activation context during PROCESS_ATTACH
and reverse PROCESS_DETACH. Both FindActCtxSectionStringW flag forms find the
provider key in those scopes. EXE entry sees a NULL current context and real
missing-key results (14007), with every keyed-output byte untouched. Explicitly
creating/activating the importer's resource-2 manifest exposes the key under a
second distinct context; deactivation restores the NULL/default miss. This is
finite evidence for these owned PEs, not a measurement of DAO on Windows or a
license to union every DLL manifest globally.

For successful queries, Windows returns format version 1, a 20-byte redirection
record, a 124-byte containing section and roster index 2. The record's five
DWORDs are `[20, 2, 0, 0, 0]`. Flag 0 returns a NULL context handle; flag 1 returns
an owned reference to the actual current context. Both preserve last error.
The ulFlags field and bytes 40–63 retain their sentinel rather than being filled
with invented values. The complete section bytes and all scalar observations,
including guards and opaque-handle identity relationships, are retained. Each
complete module path is independently checked against the actual loaded source,
including the provider's assembly subfolder.

The raw CRLF native report is frozen at
`pe/reference/pe32-activation-startup-windows-v1.json`, SHA-256
`1c781050c8866bf4e7900086a4cd49814b418fc0c4ed8e6d0bf7f6fe6fa0f88e`.
Builds and native reruns require all original source/PE/manifest identities and
compare every decoded observation after path validation and handle-ID
normalization; there is no initial-capture bypass. Four synthetic transport and
comparison tests reject truncation, lost events, wrong sources, damaged guards,
inconsistent section/data and changed scopes/payloads. They are not additional
native observations. All 342 Python tests passed before freezing; the additional
comparison test and exact fixture rebuild pass locally afterward.

The Alkyhol runtime has not gained assembly activation from this experiment.
Its original-graph failure remains explicit. The next implementation needs real
per-DLL dependency scopes and actual assembly/publisher-policy binding from the
local original metadata, including supported payload ownership, rather than a
fabricated successful lookup. Windows observations here establish scope and
redirection bytes for a private, unsigned owned assembly; original signed CRT
assembly and publisher-policy semantics remain separate work.

## DAO reaches its original dependent-assembly lookup

On the physical M4 at clean `9a0474f`, the original CRT now executes GetSystemDirectoryW and calls
FindActCtxSectionStringW itself. The runtime stops explicitly because activation
of the original DLL manifests/dependent assemblies is unimplemented. It has not
returned a fabricated successful binding or an invented missing-key result. The
game EXE entry has **not run**. The actual query uses flags 0, NULL extension GUID,
section 2 and cbSize 64; its original filename key and identities stay ignored/local.

`ProcessOptions32` and `load_files_configured` accept an explicit backed guest
system-directory role. The library verifies an existing directory before guest
entry, retains its canonical Z: spelling and implements checked UTF-16 copy/sizing.
Absent configuration, non-directory/missing profile paths and invalid output stop
explicitly. The default file/byte loaders supply no system-directory role.
The probe opts in with `--file-origins --system-directory /actual/host/directory`.

The local DAO driver now assigns a separate directory in its ignored guest profile
to this role. It is currently empty; host services use Rust gateways. This is not
a Windows installation, side-by-side binding or a relocation of the original CRT
files. The CRT retains its genuine extracted source path, so the original caller
continues into its activation lookup rather than skipping that check because of
a substituted directory. Diagnostic lookup logs retain the checked output size
and a bounded UTF-16 key prefix; all original-game captures remain local.

An owned i386 PE checks full/exact/insufficient/zero capacities, last error,
terminators and guards. Its 18 normalized DWORDs agree with the host profile's
independently known complete directory path, including supplementary UTF-16 units.
Four restrictions cover no role, read-only output, a regular file used as a
directory and a missing directory; the latter two stop before guest entry. Native
Windows comparison passed at `9a0474f`: all 18 normalized words agree, including
last-error preservation and untouched insufficient-buffer output. Each backend's
complete path is validated separately. The native collector records Windows'
actual directory and checks it exists under its OS root; this does not claim
WOW64 filesystem-redirection support or equality with the local profile path.

Local validation passes 154 PE host tests and 18 compile/documentation checks,
339 Python tests, Clippy, all 25 prior startup cases and five new directory cases.
All five [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42473762953)
and all 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42473762951)
passed at `9a0474f`, including the unchanged historical native comparisons.
The complete raw CRLF native directory record is frozen at
`pe/reference/pe32-system-directory-windows-v1.json`, SHA-256
`d0885526615a19f311d62283d3d6012b8ae9b1d591d1ee3d2fec1e907b4fbbd1`.
Exact input identities and the observed contract are required without an initial
capture bypass. All previous frozen reports and fixture source/PE identities remain.

The next work is actual original assembly binding, using the locally extracted
original assembly/publisher-policy metadata and native observations of activation
during DLL startup. The current empty-default context profile remains insufficient
for that graph; directory configuration alone does not make it empty or resolved.
The owned native experiment above now mirrors a CRT without an embedded
manifest imported by a resource-2 DLL, recording active contexts during both DLL
initializers, EXE entry and teardown, plus the actual DLL-redirection payload.
Do not infer the active context by unioning all manifests. Microsoft's
[resource-ID rules](https://learn.microsoft.com/en-us/windows/win32/sbscs/using-side-by-side-assemblies-as-a-resource)
and [assembly-file searching](https://learn.microsoft.com/en-us/windows/win32/sbscs/searching-for-assembly-files)
provide the interface background; original-graph behavior still needs observation.

## DAO real module origins and path queries

On the physical M4 at clean `14cd103`, the original CRT now executes GetModuleFileNameW and
GetLongPathNameW and reaches **GetSystemDirectoryW**, the next measured unsupported
import. The activation query itself and the game EXE entry have **not executed**.
Original identities, source paths, module/RVA/argument captures and extracted DLLs
remain ignored/local.

`ProcessImage32::load_files` reads the explicitly supplied regular files and
retains their actual canonical origins before any DLL initializer. Its Z: namespace
directly mirrors the real host root; it does not infer module paths from basenames
or claim a Windows installation/system directory. Existing bytes-only loaders stay
separate and refuse filename queries without genuine origin metadata. The startup
probe selects this explicitly with `--file-origins`; DLL search remains absent.

GetModuleFileNameW supports NULL/current-EXE and actual owned EXE/DLL bases,
UTF-16 lengths, terminators, modern truncation/insufficient-buffer results and
checked output bounds. GetLongPathNameW currently supports only exact registered,
already-long module paths, reverified against their real source files, with sizing
and staged in-place copies. Arbitrary paths, short aliases, unknown modules,
missing origins and invalid outputs stop explicitly; no synthetic filesystem
absence, alias expansion or source path is invented.

An owned unchanged i386 EXE/DLL fixture calls the filename API during the real DLL
initializer, then tests NULL/explicit EXE and DLL handles, full/exact/truncated/zero
capacities, last error, guards, UTF-16 supplementary characters and long-path
sizing/in-place behavior. Normal and forced-relocation M4 runs agree on 105
normalized DWORDs. Each complete returned path is separately checked against its
independently known actual source; full absolute paths and their lengths are
machine-specific and are not compared for cross-machine byte equality. Four
host-only restrictions stop for their specific expected reason, including a
bytes-only DLL stopping before EXE entry. The complete native Windows comparison
passed at `14cd103`: all 105 normalized words agree, and all three full paths
independently identify their real source on each host. The actual directory/drive
strings remain intentionally host-specific.

The first native observation at `dc2e1dc` returned status 0 and complete paths,
but its comparison failed: successful GetModuleFileNameW calls cleared last error
to 0 instead of preserving the seeded value. All other normalized fields agreed.
The runtime and explicit expected record now follow those measured native zeros;
the C/DLL sources and executed PE inputs remained unchanged. The corrected complete
comparison passed in the next native run. GetLongPathNameW's seeded last error
was preserved in that same independent observation.

Local checks pass 154 PE host tests plus 17 compile/documentation checks,
336 Python tests, Clippy, all 25 prior startup cases and six new path executions.
The existing frozen activation/source/PE identities remain unchanged. At
predecessor `d1c31f9`, both dedicated startup jobs passed the strict frozen
activation comparator; the historical PE job separately hit an upstream u-config
HTTP 504 and skipped its dependent historical comparisons.

At `14cd103`, both dedicated [startup jobs](https://github.com/daeken/Alkyhol/actions/runs/42471707745)
passed, including all original frozen startup/activation observations and the
new path comparison. Its unchanged raw CRLF native report is frozen at
`pe/reference/pe32-module-path-windows-v1.json`, SHA-256
`9a4ac6ec8aa07c071e3b43739cc9faab01a2eb246081463b132c24f61395d26e`.
Exact source/EXE/DLL identities and the observed contract are now required; there
is no initial-capture bypass. The historical PE job again failed on the unchanged
upstream u-config HTTP 504 download; its dependent native/comparison jobs did not run.
All 24 conformance jobs separately passed at `dc2e1dc`.

The next system-directory API needs an explicit, backed guest-system directory
configuration. It must not relabel the CRT source directory as the system directory
to bypass the subsequent original activation check. Side-by-side binding remains
unknown/unimplemented for the original DLL graph.

Read-only inspection of the bundled redistributable also located its original
CRT publisher-policy manifest and catalogs. Extracted policy bytes, parsed ranges
and exact installer provenance stay ignored/local. This is package metadata,
not native publisher-policy application or evidence of correct original binding.

References: [GetModuleFileNameW](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-getmodulefilenamew),
[GetLongPathNameW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getlongpathnamew)
and [GetSystemDirectoryW](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemdirectoryw).

## DAO bounded activation lookup and next original CRT call

On the physical M4 at clean `7b39f23`, the original CRT now resolves FindActCtxSectionStringW to
a real checked gateway, continues through its original module-handle queries,
and reaches **GetModuleFileNameW**. That is the next measured unsupported import.
The activation query itself and the game EXE entry have **not executed**.
Original module/RVA/argument captures remain ignored/local.

The new activation gateway implements only missing DLL keys in a verified empty
embedded default context, with flags 0/1, NULL extension GUID, section 2 and
40/64-byte keyed-data structures. It bounds the actual guest UTF-16 key and output
storage, leaves output bytes untouched and returns FALSE/error 14007. It does
not infer an empty context from a missing manifest: external/system defaults,
multiple manifest languages, nonempty EXE manifests and nonempty DLL manifests
remain explicitly unsupported. In particular, DAO's original DLL assembly binding
is still unknown; exporting this partial API does not fabricate its result.

Namespace-aware XML parsing uses unmodified, vendored, open-source roxmltree
0.20.0 (MIT/Apache-2.0, with its upstream ISC tree credit). The published archive
SHA-256 is `6c20b6793b5c2fa6553b250154b78d6d0db37e72700ae35fad9387a46f487c97`.
Source/license hashes are checked; builds need no new downloads. Alkyhol disables
DTDs, bounds manifest size/node count and rejects every unsupported element or
attribute. Parsing is not assembly resolution or language fallback.

One unchanged owned PE exercises dynamic/direct gateway identity and four
empty-context calls (both sizes and flags), producing 33 exact DWORDs on M4.
Five host-only restrictions stop after a boot marker: nonempty EXE/DLL contexts,
absent embedded default, read-only output and unsupported flags. They are not
executed natively or counted as Windows failure semantics. The new positive PE
passed native comparison at `7b39f23`: all 33 words and every source/PE/DLL identity
agree with M4. The unchanged raw CRLF native record is frozen at
`pe/reference/pe32-activation-miss-windows-v1.json`, SHA-256
`39123e61b237bb99d74dfaf1f7a894b4ad3eedf631f72d4a3bb4ca0df04f8e9e`.
The positive PE SHA-256 is
`d67f11dcb64c5c5ada0bfd81983e33e23d8191fcb7f0bb58f7fcb21acaf3bda1`.
The report, exact input identities and measured output are required without an
initial-capture bypass.

Local validation passes 151 PE host tests and 16 compile/documentation checks,
332 Python tests, Clippy, all 25 prior startup cases and six new activation cases.
The unmodified third-party parser emits compiler deprecation/lifetime warnings
on the newer Rust toolchain; Alkyhol's own Clippy checks pass with warnings denied.
All five [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42468373009)
passed on predecessor `4cecfeb`, including the strict frozen activation comparator
and the previously download-blocked historical comparisons. That is separate
from the new gateway's result.

At `7b39f23`, both dedicated [startup jobs](https://github.com/daeken/Alkyhol/actions/runs/42469244460)
passed, including the original frozen activation observer and the new 33-word
comparison. The historical PE job separately hit HTTP 504 downloading unchanged
i386 peports, so its dependent historical Windows/comparison jobs were skipped.
No fixture, checksum or historical acceptance condition was changed.

The next blocker needs genuine loaded-module path metadata: the current process
API receives module basenames and bytes, not retained source-file origins.
GetModuleFileNameW must not invent a DLL directory or relabel the CRT directory
as the Windows system directory to bypass the later activation check. Subsequent
path and activation calls in the inspected original caller remain unexecuted.

## DAO activation-context resource groundwork

The next actual DAO blocker remains the dynamic lookup of
FindActCtxSectionStringW in the original CRT. The EXE entry has **not run**.
Read-only inspection of its original caller also identifies module/path queries
before the activation query itself; those calls have not yet executed here.
No absent-context result, successful assembly binding or missing-function result
is fabricated to skip this check.

`Image::resources()` now checks the PE type/name/language tree and returns exact
file-backed borrowed payloads with their original IDs/UTF-16 names, language,
code page and RVA. Directory-relative offsets and image-relative data RVAs remain
separate. Truncation, duplicate keys, invalid levels, misalignment, out-of-bounds
names/payloads, reserved data and traversal budgets fail explicitly. This is
metadata inspection, not resource-language fallback or activation support.

On the physical M4, all twelve supplied original images passed this inspection;
all seven embedded manifest payloads agree byte-for-byte with the prior independent
local extractor. Those originals and derived metadata remain ignored/local.
Some original dependencies declare assembly dependencies, so treating the whole
process as an empty activation context would be unjustified.

An owned i386 PE with an empty default manifest, a separate file-redirection
manifest and a Unicode-named binary resource now builds with open Clang/LLD.
Host inspection agrees exactly with the linked payloads. The dedicated native
startup job extracts the same resources with Windows APIs and observes actual
Create/Activate/GetCurrent/Find/Deactivate/ReleaseActCtx behavior, including both
40-byte legacy and 64-byte current keyed-data callers. Independent Windows SDK
layout assertions are included. These observations have now passed on native
Windows at `1a7fe5f`; no activation API implementation or original-game progress
beyond the existing stop is claimed.

Local checks pass 148 PE host tests plus 16 compile/documentation checks,
330 Python tests, all 25 existing startup microtests and Clippy. The five new
resource units cover exact RVA/payload separation, counted UTF-16 names,
truncation, malformed trees and duplicate/count errors. Collector tests are
synthetic format checks and do not establish activation semantics.

At `388dbb7`, hosted resource inspection, all existing startup cases and the
independent native SDK layout assertions passed. The new activation observer did
not execute: its collector rejected Windows path separators in the source-key
inventory before launch. Source keys now use portable forward slashes without
relaxing any source/PE hash check. Native activation observations remain pending.
The broader historical PE job again stopped at the unchanged upstream u-config
HTTP 504 download; its dependent historical comparisons did not run.

The corrected collector at `684a8dc` did execute the owned PE on Windows and
retrieved both manifest payloads, then the observer trapped while accessing its
additional Unicode-named test resource. That run is **failed evidence**, not a
successful activation observation. The owned resource name is now stored in
uppercase form and failure records include the native resource error. Activation
queries were unobserved in those failed attempts.

At `1a7fe5f`, both dedicated [startup jobs](https://github.com/daeken/Alkyhol/actions/runs/42468131948)
passed, including SDK layout checks, exact native/host resource bytes and the
complete activation observer. The unchanged raw CRLF Windows report is frozen at
`pe/reference/pe32-activation-windows-v1.json`, SHA-256
`8f965ed831084348e098cbbe73613c456c197249b4de9ee7746f8ed813e65bb1`.
The identical executed PE SHA-256 is
`98d5bbcc1da381b7d196f014ed17063bfc5eedccf620f1abe6bdcc99a922d526`;
all source/PE identities and exact observed scalar records are now required.

In this finite native observer, missing keys return FALSE/error 14007 and leave
the output untouched. Both 40-byte and 64-byte callers find the owned file while
its real context is active: format 1, 20-byte data, 140-byte section and roster 1.
Successful lookups preserve the seeded last error, clear hActCtx without the
return-handle flag, and return the actual context with that flag. The bytes from
offset 40 through 63 remain untouched even for the 64-byte caller. Deactivation
removes that file from lookup. This is native context-lifecycle/ABI evidence for
our owned manifests, not proof of original CRT assembly binding or host activation
support. No original game input was sent to Windows.

The same run's historical PE job separately failed on HTTP 504 while downloading
the unchanged i386 peports release, so its historical native/comparison jobs were
skipped. Its existing fixture/checksum was not changed. The new frozen comparator
has passed locally against the downloaded raw native capture; its next CI result
remains separate from the completed native observation above.

References: [PE resource tables](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format#the-rsrc-section),
[FindActCtxSectionStringW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-findactctxsectionstringw),
[ACTCTXW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-actctxw)
and [keyed-data layout](https://learn.microsoft.com/en-us/windows/win32/api/winbase/ns-winbase-actctx_section_keyed_data).

## DAO real wall clock and live heap-size queries

The physical M4 now executes the original CRT's GetSystemTimeAsFileTime and HeapSize
calls, with real UTC output and allocation metadata. Its next observed stop is
GetProcAddress for FindActCtxSectionStringW. That function has **not been called**;
activation-context arguments/results and side-by-side assembly behavior remain
unobserved/unimplemented. No successful activation or absent-context result is
invented to bypass the original runtime check. The game EXE entry has **not run**.
Original identities, traces and dependency data remain ignored and local.

The new unchanged owned PE checks the 8-byte FILETIME write and guard, process and
private heap requested sizes, HEAP_NO_SERIALIZE on the private heap, last-error
preservation and real frees. Its actual timestamp is bracketed against parent UTC
on each machine with 100 ms allowance for the non-precise Windows clock; raw
clock values are deliberately not required to agree between machines. The identical PE also passed native Windows at `b22a43b`, including the clock
bracket and requested-size/error/guard fields. Separate wrong-heap and read-only-output programs
stop with status 78 after a boot marker; these are host restrictions, not native
observations of undefined Windows inputs. Synthetic record-validation negatives
reject wrong epoch/time bounds, width, guards, padding-sized heap results and
lost error preservation. They are not original-game evidence.

Local validation passes 142 PE host tests, 16 compile/documentation checks,
325 Python tests, Clippy and all 25 targeted host startup cases. The existing
FILETIME epoch/range tests remain intact. All prior frozen reports and original
fixture source/PE identities remain required unchanged.

For predecessor `935dcad`, all five [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42463404929)
and all 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42463404917)
passed. Twelve identical-PE exact-output startup cases passed native Windows,
including mapping edges, atomic wraparound, translated-domain SSE2/CPUID checks
and the finite x87 division witness. This closes their prior pending native result;
it establishes neither general floating-point parity nor game compatibility.

At `b22a43b`, both dedicated startup jobs passed: all 25 host cases and 13 native
positive cases (12 exact-output cases plus the separately bracketed clock/heap
case). All 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42464946185)
passed. The broader [PE workflow](https://github.com/daeken/Alkyhol/actions/runs/42464946179)
failed twice while fetching the unchanged historical u-config fixture: the upstream
server returned HTTP 504. Its dependent historical Windows/comparison jobs were
therefore skipped; no success is claimed for those jobs at this revision. The
fixture, its required checksum and acceptance criteria were not changed.

Read-only inspection also found the original EXE's embedded manifest. Original
manifest bytes and parsed assembly identities are retained only in ignored local
storage for the next activation-context milestone; no game-derived manifest or
inventory is committed.

References: [GetSystemTimeAsFileTime](https://learn.microsoft.com/en-us/windows/win32/api/sysinfoapi/nf-sysinfoapi-getsystemtimeasfiletime)
and [HeapSize](https://learn.microsoft.com/en-us/windows/win32/api/heapapi/nf-heapapi-heapsize).

## DAO neutral-locale case mapping and later CRT initialization

Original bundled CRT execution on the physical M4 now passes its lowercase and
uppercase table setup, refcount decrements and processor-feature query. The next
actual blocker is GetSystemTimeAsFileTime during later CRT initialization. The game
EXE entry has **not run**. Original identities, raw trace/RVA details and extracted
dependencies remain local; this is diagnostic startup, not game compatibility.

At `9a51997`, all five [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42462059740)
and all 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42462059732)
passed. Ten positive identical-PE startup cases passed native Windows, including
CTYPE1 order/termination edges. Its separate native-only LCMapStringW observation
captured both complete 256-unit batch results and 256 individual lower/upper pairs
for locale 0 and flags 0x100/0x200. Every call succeeded, every single-unit mapping
has length one, and concatenation agrees with each independent complete batch.
The full unmodified report is frozen at pe/reference/pe32-lcmap-windows-v1.json;
all prior PE/source identities and both raw native reports are required unchanged.

The same original observer now executes on M4 and agrees byte-for-byte with the
entire native capture, including source units, result lengths, data and last-error
records. The runtime applies the measured mappings to supported sequences without
host-locale substitutions; other locales/flags/units stop explicitly. New owned PE
edges exercise sizing, insufficient buffers, negative source lengths, in-place
mapping, output guards and decrement wraparound. New CPU checks compare actual
translated CPUID SSE2 availability, execute SSE2, exclude the original Pentium-only
FDIV erratum and run the classic x87 division witness. These new edges await their
native result; the CPU witness is not general floating-point parity evidence.

Local validation passes 141 PE host tests plus 16 compile/documentation checks,
322 Python tests, Clippy and all 22 targeted host startup cases. The existing
independent startup/Windows jobs run these unchanged candidate PEs separately
from historical regressions. Native full-game startup/rendering remains unmeasured.

References: [LCMapStringW contract](https://learn.microsoft.com/en-us/windows/win32/api/winnls/nf-winnls-lcmapstringw),
[processor-feature definitions](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-isprocessorfeaturepresent)
and [measured mapping provenance](../pe/data/ctype1/README.md).

## DAO CTYPE1 classification from measured native data

The physical M4 now executes the original CRT's GetStringTypeW probe and its actual
256-character classification batch. The next observed call is LCMapStringW during
case mapping. The game EXE entry has **not executed**. Original-game capture details
remain ignored/local; the public evidence below uses only our own PE fixtures.

At clean `9e852ec`, eight unchanged positive PEs passed native Windows (92
normalized DWORDs). The separate original CTYPE1 observer captured 256 raw WORDs
from native Windows. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42460883696)
and all 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42460883747)
passed. The complete unmodified native startup report, including its source/PE
identities and CRLF bytes, is frozen at pe/reference/pe32-startup-windows-v1.json.
Its bytes, all existing PE identities and original three source files are required;
there is no initial-capture bypass. Those original C/DEF files remain unchanged.

The runtime uses those measured CTYPE1 data for the Windows-1252 Unicode repertoire,
with provenance in pe/data/ctype1/README.md. It applies the data to supported
sequences; it does not recognize test vectors or use host-locale approximations.
Unknown code units and CTYPE2/3 fail explicitly. The identical frozen observer now
executes on physical M4 and agrees with all 256 native WORDs. Separate owned PE
cases test reordered/duplicate units, embedded NUL, NUL termination and output
bounds, plus actual stops for unknown units/info levels. These new edge cases
are host assertions pending their native observations, not general NLS parity.

Local validation passes 140 PE host tests plus 16 compile/documentation checks,
322 Python tests, Clippy and all sixteen targeted host startup cases. A new
native-only LCMapStringW observer
captures both batch results and per-unit lower/upper mappings for the next actual
blocker. It is built but not counted as host execution or successful API support.
The existing startup checks and identical-PE Windows execution now have separate
jobs from historical PE regressions; historical jobs and frozen comparators remain.

References: [GetStringTypeW contract](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-getstringtypew)
and [measured data provenance](../pe/data/ctype1/README.md). Finite native API
observations and original CRT diagnostic execution do not establish game behavior
or original-game native Windows equivalence.

## DAO environment and legacy ANSI code-page milestone

On the physical M4, original bundled CRT execution now passes environment-block
acquisition, sizing/conversion/copy/free, command-line acquisition and Windows-1252
code-page setup. The next actual blocker is GetStringTypeW in CRT character
classification. The game EXE's own entry has **not executed**. Original identities,
raw argument/RVA captures and extracted dependencies stay local and ignored.

The ANSI profile is explicitly Windows-1252. Unmodified Microsoft's 256-entry byte
mapping and 698-entry best-fit mapping come from Unicode's public WindowsBestFit
archive; the file identity, URL and Unicode License V3 are retained under
pe/data/windows-1252/. This is mapping provenance, not native semantic evidence.
CP_ACP, GetACP, IsValidCodePage and GetCPInfo consistently describe the installed
1252/65001 profile. Other pages/flags and Windows-1252 surrogate conversion remain
unsupported. The established explicit UTF-8 path is preserved.

Twelve targeted host startup cases pass. New owned PE32 cases check a real Unicode
environment snapshot including empty values and a surrogate pair, separate copies
and correct free ownership, cached A/W command text, Windows-1252 best-fit/default
replacement, conversion sizing, bounds and CPINFO layout. Eight positive binaries
(92 normalized DWORDs) passed native Windows at `9e852ec`. Its separate native-only
PE also recorded 256 CTYPE1 WORDs; that initial native-only observation has now
become the finite host comparison recorded above. No classifier results were
guessed or fabricated to advance the game.

Local validation passes 139 PE host tests, 16 compile/documentation checks,
322 Python tests and Clippy. Byte/Unicode data preservation, BMP best-fit selection,
empty environment termination and typed free rejection have targeted host checks;
they are distinguished from the pending native observations above.

For completed predecessor `408f11d`, all six identical positive PE files passed
native Windows (64 DWORDs), hosted Mac and physical M4. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42459905829) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42459905828) passed.
That is finite module/TLS/callback/locking/stream evidence, not native original-game
startup, rendering or game compatibility.

References: [environment block contract](https://learn.microsoft.com/windows/win32/api/processenv/nf-processenv-getenvironmentstringsw),
[matching release API](https://learn.microsoft.com/en-us/windows/win32/api/processenv/nf-processenv-freeenvironmentstringsw),
and [mapping provenance](../pe/data/windows-1252/README.md).

## DAO CRT module/TLS/locking and standard-I/O milestone

DAO remains the sole application driver. The original bundled CRT now executes
through module/dynamic-export lookup, real TLS/FLS setup, pointer encoding,
recursive critical-section initialization/use, atomic increments and standard-I/O
setup on the physical M4. The next observed original call is GetEnvironmentStringsW,
which stops explicitly. The game EXE entry has **not run**. Exact game/dependency
identities, original argument/RVA captures and the private launch remain local;
there is no native Windows original-game startup/frame comparison.

Targeted owned PE32 microtests execute direct FS reads of TLS and thread identity,
dynamic module/function lookup, opaque pointer roundtrips, actual reentrant FLS
cleanup through nested CPU transitions, recursive section operations, DWORD atomic
wraparound, configured STARTUPINFOA streams and real pipe types. Unknown dynamic
exports and builtin ordinals stop with status 78 before post-call output. The four
negative cases are host instrumentation evidence only. Six positive PEs contain 64
normalized output DWORDs and are supplied unchanged to the existing Windows CI job;
native results passed at `408f11d` as recorded above. No original game bytes or derived inventories enter
these tests. Thread/fiber switching, contended guest synchronization and automatic
thread-exit FLS cleanup are not claimed. Pending FLS cleanup stops explicitly.

Local validation passes 137 PE host tests plus 16 compile/documentation checks,
322 Python tests, Clippy and all ten targeted startup cases. The existing frozen
105-DWORD process fixture also retains its physical M4 normal/rebased startup,
termination and five host rejection results. These checks preserve previous
evidence; they do not turn this original-game diagnostic run into native parity.

For the preceding `78468cf` milestone, all three targeted positive PE files (24
DWORDs total) passed native Windows; all three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42458454808)
and all 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42458454911)
passed. This closes the earlier pending native result, not the new TLS/locking
milestone or game compatibility.

API contract references used for the new bounded implementation:
[FLS allocation](https://learn.microsoft.com/en-us/windows/win32/api/fibersapi/nf-fibersapi-flsalloc),
[FLS free callbacks](https://learn.microsoft.com/en-us/windows/win32/api/fibersapi/nf-fibersapi-flsfree),
[process pointer encoding](https://learn.microsoft.com/en-us/previous-versions/bb432254%28v%3Dvs.85%29),
[recursive critical sections](https://learn.microsoft.com/en-us/windows/win32/sync/critical-section-objects),
[STARTUPINFOA](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/ns-processthreadsapi-startupinfoa),
and [obsolete SetHandleCount](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-sethandlecount).

## DAO-driven diagnostic startup and first runtime blockers

Latest priority is DAO alone, using tests needed for observed startup blockers.
The local original EXE/dependency launch now executes original bundled CRT entry
code on the physical M4. Guarded-import faults identify the actual original module
and fault/return locations. Real process/private heap support, legacy version
reporting and an environment query moved the observed stop to builtin module
lookup during CRT initialization. The EXE's own entry has not run. Original file
identities, detailed inventories/captures and extracted dependencies remain ignored
and local. One packed DLL is unpacked only in a local diagnostic copy; the original
is preserved. This is diagnostic execution, not original-game behavioral parity.

Unknown external-service functions **and data** occupy inaccessible guard pages.
Separate original owned PE32 microtests execute a real function-fetch fault and a
real data-read fault, each after a boot marker; both stop explicitly with status 78
and correct attribution. Their strict-loader counterparts reject before entry.
Additional small tests execute heap calls with an IAT in RX storage, exercise
zeroed allocation/reuse/free and compare legacy version/environment results with
fixed expected records. Physical M4 checks pass. The identical positive PE files,
source/file identity manifest and expected records now run in the native Windows
workflow; their native result passed at `78468cf` as recorded above. At that milestone, all 134 PE host tests, 16 compiler checks, 322 Python tests,
portable metadata tests and Clippy pass locally. The diagnostic guard restores and
queries prior host fault actions/alternate-stack state on successful return; owned
faults stop instead of claiming Windows exception recovery.

Bounded delay-import validation preserves the original compiler thunks/helper
rather than replacing delayed operations with success. Targeted metadata tests
reject malformed descriptors, invalid handle/table storage, reserved ordinal bits,
missing thunk relocations and missing terminators. Existing strict parsers keep
their prior restrictions. Existing frozen PE64/PE32/native/source/Metal evidence
remains a separate category. See docs/pe32.md for the explicit diagnostic contract.

## Closed PE32 process graph: physical M4 startup/calls/teardown evidence

The original MIT process observer statically imports two unchanged frozen PE32
DLLs, shares a dependency, performs actual module/symbol lookup and indirect
calls, then calls a real imported third DLL that executes ExitProcess(7).
At clean revision `76698e6`, native Windows, hosted Mac and physical M4 agree
on all **105 raw DWORDs** and exit status 7 from the identical positive EXE/DLL
graph, including actual forced relocation on Mac. Both strict paired comparisons
pass. Static initialization happens once, then original DLL code runs in reverse
teardown order. The owner restores/clears actual host selector state and rejects
restart. All 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42452681390)
pass; native and Mac [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42452681333)
passed and the paired comparison is recorded above. The complete unchanged native
report is frozen at `pe/reference/pe32-process-windows-v1.json` and required by
default. The initial-capture bypass is removed. These are finite startup and
termination observations, not general CRT or game compatibility.

Four host preflight restrictions (missing DLL, duplicate DLL, unused DLL and
missing symbol) produce explicit status 78 before any guest output. A genuine
failing initializer calls its dependency, receives immediate cleanup, then the
previously initialized dependency rolls back. The 52 raw DWORD rollback sequence
and status 78 are separate host evidence, not native failed-static-startup parity.
Metadata/unit tests cover graph order/shared dependencies/cycles, identities and
module/export ownership during initialization and teardown; protocol negatives
require every raw word, exact inputs, clean matching revisions and error outputs.
See docs/pe32.md for the bounded API and remaining unsupported startup features.
All 133 PE host tests, 16 compiler checks, 321 Python tests and Clippy pass.
Portable metadata tests pass; physical bounded-SEH, linked-DLL and released
i386 peports regressions retain their outcomes. Eight additional physical M4 launcher checks pass: normal/rebased execution,
metadata-only inspection of successful/failing initializers and explicit invalid
option rejection. The public `--exe-name`/`--dll NAME=PATH` path preserves the
released utility behavior. No game instruction has executed.

## Released i386 peports: native Windows, hosted Mac and physical M4

The original released i386 peports from the official w64devkit 2.11.0 x86 archive
passes all 16 exact existing peports corpus cases on the physical M4, both normal
and forced rebase. Its original PE SHA-256 is
`387d317019c36e86a26fa35818353d67dd47e9558e0b10819a2bb27a67fdeab9`;
archive, source and license identities are independently checked by the fetcher.
The Mach-O payload hash is checked against those exact released PE bytes.
No source rebuild or executable patch is involved.

This exercises actual original 32-bit startup, four-byte argv, UTF-8/UTF-16 sizing
and conversion, Unicode paths, CreateFileW/ReadFile/CloseHandle, standard streams,
and controlled non-returning ExitProcess through the Rust Win32 gateways.
Actual host selector restoration and FS descriptor cleanup are required after
ExitProcess. Exact status and raw output/error bytes cover help, imports, exports,
file/stdin errors and subsequent-file recovery. The old released PE64 application,
corpus and frozen native records remain unchanged.

At clean revision `e1772cf`, all 16 cases agree across native Windows, hosted Mac
and physical M4, including actual forced relocation on Mac. Both strict paired
comparisons pass. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42451577637)
and all 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42451577672)
pass. The complete unchanged native record is frozen at
`pe/reference/peports32-windows-v1.json` and required by default. Initial-capture
bypass is removed. This finite utility evidence remains separate from game
compatibility or general Windows/CRT startup.
Focused unit checks cover four-byte argv/overflow, DLL+symbol binding identity,
logical permissions, invalid conversion/I/O inputs, LocalFree ownership rejection,
exit-state reentry refusal and real owned-file cleanup without closing an
unrelated open handle. All 130 PE host tests, 15 compiler checks, 316 Python
tests and Clippy pass locally. Portable metadata tests pass. Frozen bounded
SEH, linked-DLL and compiler-GS physical M4 regressions retain their outcomes.
These checks do not establish arbitrary CRT,
general DLL/thread teardown or game compatibility. No game code has run.

## Bounded PE32 hardware SEH: native Windows, hosted Mac and physical M4

At clean revision `aacce9b`, native Windows, downloaded hosted Mac reports and fresh
physical M4 execution agree on all **32 raw DWORDs** from the identical positive
SafeSEH executable, with exact source/PE identities and actual forced relocation.
Both strict paired comparisons pass. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42448838248) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42448838253) pass,
including the actual Win32 SDK/MSVC layout/disposition assertions. The complete
unchanged native report is frozen at `pe/reference/pe32-seh-windows-v1.json` and
required by default. The initial-capture bypass is removed. These finite
observations compare selected integer/PC/flags/FS behavior, not every context byte
or general Windows exception compatibility.

The new opt-in `ExceptionImage32`/`--seh` profile handles a real UD2 through two
FS-linked cdecl handlers, enforces the owned SafeSEH table and resumes changed
integer/PC/carry state through Darwin's actual signal return. Physical M4 normal
and forced rebase return the same 32 DWORDs: observed exception code, handler order,
seven interrupted register canaries, original PC/SP properties, modified EAX/EBX,
carry, preserved chain and direct FS last-error mutation. This is actual 32-bit
fault → 64-bit Rust → 32-bit handlers → resumed 32-bit execution, not a synthetic
context-only assertion. The native alternate stack resides above 4 GiB; its guard
pages and actual signal-stack bounds are checked. Host signal/alternate-stack
settings and segment state are restored and queried before successful return.

Four owned host-only negative binaries exercise forbidden imported handler calls,
unsupported context writes, a real nested UD2 and code outside the SafeSEH table.
Each returns explicit diagnostic/status 78 with no fabricated successful body.
They are profile-restriction tests, not Windows exception outcome comparisons.
The positive unchanged PE and independent Win32 SDK/MSVC layout/disposition
assertions run in the existing native workflow. Their completed native comparison
is recorded above; the four host-only restrictions remain a separate category.

All 124 PE host tests, 15 compiler checks, 310 Python tests and Clippy pass locally.
Portable metadata checks pass. Installed Darwin SDK C assertions check the signal
ABI. Frozen default-load/compiler-GS observations, including the deliberate raw
SIGILL trap, and linked-DLL observations retain their physical M4 behavior.
Default execution is preserved; this experimental profile must be selected.
Follow-up hardening rejects a blocked host SIGILL mask and handler changes to the
TEB chain head, and documents exclusive host signal configuration during entry.
Focused units/protocol checks, Clippy and all five physical M4 cases pass after
that hardening. It adds no new native exception-semantic claim.

The local game's explicit bounded exception **inspection** now stops at unsupported
imports before mapping/entry; no game instruction executes. This does not validate
its handlers. Access violations, software RaiseException, FP/debug state, imported
handler APIs, nesting, unwind, system/DLL handlers and CRT/C++ exceptions remain
unsupported. See [PE32](pe32.md) for the full contract and primary interface sources.
The next work targets original program startup/imports and its DLL lifecycle;
full checks remain batched at meaningful milestones.

## PE32 linked DLLs: native Windows, hosted Mac and physical M4

Strict-reference revision `8ddc477` passes all three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42446160760) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42446161188).
The initial capture revision `cf234fc` also passes all three PE and 24 conformance
jobs linked below. These precede the hardware-exception profile above.

At clean revision `cf234fc`, actual native Windows and downloaded hosted Mac
reports and fresh physical M4 execution agree on every one of the 81 raw DWORDs,
with the same exact DLL/driver/source identities, including the frozen original
dependency. Both strict comparisons pass for normal/forced-rebase execution.
The complete actual native report is frozen unchanged at
`pe/reference/pe32-linked-windows-v1.json` and required by default; the initial
capture bypass is removed. See the [PE run](https://github.com/daeken/Alkyhol/actions/runs/42445695193)
and [conformance run](https://github.com/daeken/Alkyhol/actions/runs/42445695166)
for that revision's CI. Full conclusions are recorded only after completion.

`LoadedLibrary32::load_linked` now binds named/ordinal code and writable data from
explicitly borrowed initialized dependencies. Related mappings share one guarded
guest stack and TEB, including direct FS accesses and last-error state. Actual
dependency regions retain their real permissions in imported API buffer checks;
code-call targets remain restricted to the queried module's initialized code.
The borrow keeps dependencies alive through consumer detach. Distinct guest-thread
contexts, duplicate/reserved/path basenames, missing imports/exports and reentrant
loading fail before a consumer initializer. No DLL search/cache or successful stub
is added; EXE dependency startup and exception dispatch remain unsupported.

Physical M4 normal/forced-rebase execution of a new owned consumer with the
**unchanged frozen original test DLL** returns the same 81 DWORDs. Its initializer,
export and destructor call that dependency's actual code, including a genuine
ordinal-only import. Shared data mutations survive across DLLs and a real WriteFile
reads a buffer in the dependency's mapped data. The original DLL's FS identity
check remains true when called through its consumer, exposing separate-TEB mistakes.
Consumer detach calls the still-live dependency before the latter's detach.
Host-only preflight negatives emit no extra initializer records. A compiler check
rejects dropping the dependency while its consumer remains live. This is current
finite execution evidence, now independently compared to Windows at the clean
revision above. No game instruction has executed. Zero import ordinals remain
unsupported even if metadata contains a zero-based export, preserving the
existing runtime policy without guessing native semantics for that case.
All 119 PE host tests, 14 compiler checks, 306 Python tests and Clippy pass locally.
The existing frozen PE32 integer/stdcall, static TLS data and isolated DLL
observers retain their behavior on the physical M4 after sharing the CPU state.

## Isolated PE32 DLL milestone: native Windows, hosted Mac and physical M4

Strict-reference revision `6f01c39` passes all three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42444812611) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42444812577).
This precedes the shared-thread dependency milestone above.

At clean revision `346f3e0`, actual Windows i386, hosted Mac and physical M4
execution of the identical DLLs agree on all **80 raw DWORDs**. Both downloaded
native/hosted and native/physical comparisons pass, including input identities,
forced HIGHLOW relocation and the separate host stack-cleanup negative. The
complete actual native report is frozen unchanged at
`pe/reference/pe32-library-windows-v1.json` and required by default. No initial
capture bypass remains. The [PE workflow](https://github.com/daeken/Alkyhol/actions/runs/42444241381)
passes all three jobs. The [conformance workflow](https://github.com/daeken/Alkyhol/actions/runs/42444241402)
is still running when this record is written; its full conclusion is not claimed.

The physical M4 executes the unchanged owned accepted/rejected PE32 DLLs through
real dynamic attach, integer exports and detach. Normal/forced-rebase runs emit
the same 80 raw DWORDs. A host-only wrong-convention run executes the real cdecl
body, rejects its actual post-return ESP, performs one detach and returns explicit
status 78. Missing/data exports and excessive arity are rejected before entry.
Host assertions separately cover call-frame alignment/bounds and DLL header/entry
kind isolation; a compiler check enforces the process borrow through destruction.
All 117 PE host tests, 13 compiler checks, 302 Python tests and Clippy pass locally. Physical M4
execution of the existing frozen PE32 integer/stdcall, FS/TEB and compiler-TLS-data
observers also retains their prior outputs and host cleanup checks after the CPU
return-gate change. These development reports are separate from clean native CI.

The native i386 driver consumes those identical DLLs through Windows
LoadLibrary/GetProcAddress/FreeLibrary. The existing workflow now transfers and
compares every raw word, exact DLL/driver/source identities, clean revision,
forced relocation and separate host ABI-negative cleanup. This is finite dynamic
DLL lifecycle/calling-convention evidence; it does not establish arbitrary DLL,
exception or game compatibility.

This isolated API is a foundation for dependency startup, not a process loader.
DLL dependencies/TLS, SafeSEH/SEH, CRT and game startup remain unsupported.
Existing EXE/source/Metal paths and frozen reference inputs remain unchanged.
The legacy-cookie strict comparison below is already complete independently.

## PE32 legacy load configuration / compiler GS: native Windows, hosted Mac and physical M4

Strict-reference revision `1a844a6` passes all three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42442892062) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42442892051).
Downloaded clean Windows/hosted Mac reports and fresh clean physical M4 execution
at that same revision pass both strict comparisons: all eleven positive records
and the separately classified controlled compiler-check trap, with exact frozen
source/PE identities, actual forced relocation and successful-return FS cleanup.

The frozen native canonical record at `pe/reference/pe32-load-config-windows-v1.json`
comes from `f566909`: all seven positive executables return 16 DWORDs, and the
controlled corrupt-frame case emits its two-DWORD marker then terminates with
0xc000001d. All three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42439989219)
and all 24 [conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42439989250)
pass on that capture revision. Actual Windows SDK/MSVC assertions confirm the
explicit IMAGE_LOAD_CONFIG_DIRECTORY32 offsets and DWORD widths.

At `15c09ba`, the independently captured four-case length matrix shows that the
**declared structure size** controls the observed default-cookie initialization:
size 72 changes the default with either directory length; size 64 preserves it
with either length. All four canonical bases/length variants execute successfully.
The complete actual record is frozen unchanged at
`pe/reference/pe32-load-config-geometry-windows-v1.json`. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42440571972) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42440571956) pass.
Only the directory-length DWORD is patched in each selected frozen base.

Alkyhol now parses both lengths and the longer backed legacy span, retains checked
cookie/SafeSEH RVAs, and validates immutable metadata/exact pointer relocations,
writable cookie storage, sorted initialized-code handler targets and alias bounds.
Cookie-only startup seeds the 72-byte default using host system entropy. The tested
64-byte default, zero/small/custom seeds and absent/null-cookie cases are preserved.
Only the cookie DWORD is written. Opaque random bytes/RNG algorithms and security
strength are not compared. Exhausted/unavailable entropy fails before guest entry.

Physical M4 development normal/forced-rebase runs agree on all **eleven positive
records**, including the four length pairs. The compiler uses inline checks for
valid frames, so the probe's `(check_count==2)` field is 0: successful calls do not
enter the slow checker. A deliberately invalid compiler frame enters the owned
checker and executes UD2: the physical host observes SIGILL (-4), with exactly the
native begin marker. Raw Windows/Unix trap statuses remain separate; this is a
controlled negative test, not a successful run or Windows SEH dispatch evidence.
Strict same-clean-revision paired CI/physical comparison passes as recorded above.

The focused metadata/entropy units, eight protocol tests, original unchanged PE
execution and Clippy pass locally. Full checks are batched at milestones by the
user's preference. Protocol negatives reject unexplained crashes, probe failures,
changed data/inputs, missing actual relocation/cleanup and missing frozen records.
Existing original fixture/source identities remain required.

SafeSEH metadata is retained for inspection; exception dispatch remains an explicit
startup gate. The user's local game now reaches that specific gate when the full
legacy span is read. No game code executes, no unsupported exception behavior is
reported successful, and all game-derived details stay local. The next priority is
the game's exception/DLL startup path rather than further isolated format probes.
See [PE32](pe32.md).

## PE32 compiler static TLS data: native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `f07f550`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42438791566) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42438791677).
Downloaded clean native Windows/hosted Mac reports and fresh clean physical M4
execution at that same revision agree on all 24 raw DWORDs and exact frozen
source/executed PE identities. Both strict paired comparisons pass, including
actual forced relocation, host segment cleanup and one-body second-entry rejection.
Existing frozen PE32/PE64, released application, DLL/source/SDK/COM, shader and
Metal comparisons remain green. This covers SizeOfZeroFill=0 with no callbacks;
the separate nonzero-zero-fill discrepancy remains unresolved and unsupported.
The local game clears static TLS data preflight but still stops at unsupported
load configuration before entry. No game code executed.

Native Windows at `8efdd37` executes the separately identified `pe32-tls-data`
observer and returns all 24 raw DWORDs. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42438139539) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42438139490) pass
on that observation revision, before the new Alkyhol TLS data comparison. The
complete actual native report is frozen unchanged at
`pe/reference/pe32-tls-data-windows-v1.json`. Exact source/executed PE identities
are required by default. The existing frozen corpora and original inputs remain
unchanged. No game code/data enters CI.

This fixture requests SizeOfZeroFill=0 and an empty callback array. Real compiler
SECREL/index/FS accesses observe initialized and zero-initialized TLS variables,
a relocated global-pointer initializer, proper alignment and template containment.
Mutating private TLS data leaves the original image template unchanged. The pointer
writes the actual mapped global, and last-error state remains coherent. Physical
M4 development normal/forced-rebase execution agrees on all 24 words. Strict
same-clean-revision CI/physical comparison passes at `f07f550` as recorded above. A host-only repeat request
must emit one complete baseline body, restore selectors/clear its FS descriptor,
and reject the second entry with explicit status 78; this is not native CRT restart.

Alkyhol now validates PE32 TLS metadata (VA-to-RVA conversion, exact pointer
relocations, immutable metadata, bounded/aligned extents, writable index and alias
checks). Metadata inspection can retain checked callbacks without executing them.
Startup rejects every nonempty callback list and nonzero SizeOfZeroFill. Supported
initialization copies the mapped template **after** HIGHLOW fixups/import binding
into separate aligned low storage, writes opaque index zero for this isolated EXE,
and installs its TEB vector. This finite evidence compares indexed data behavior,
not native slot numbering or allocation layout. Dynamic TLS APIs, DLL TLS, guest
threads, callbacks, CRT and the game remain unimplemented.

All 109 PE host tests/12 compiler checks, 38 portable tests, 290 Python tests and
Clippy pass locally. New synthetic assertions check metadata/VA shape, flags,
permissions, index/template/IAT/HIGHLOW aliases, requested zero fill, callback
validation/rejection and logical buffer bounds. Protocol negatives enforce all
raw/decoded words, source/payload/revision identity, actual relocation, native/host
diagnostics, one-body repeat rejection, missing references and joint drift. Existing
PE32 integer/stdcall, self-export and FS/TEB development runs retain frozen behavior.
The local game clears static TLS data preflight and now stops at unsupported
load configuration before entry. This does not execute or initialize game code.

## PE32 nonzero TLS zero-fill discrepancy: diagnostic evidence only

Native Windows at `c01a053` executes the original `pe32-tls` observer, but its OR
of 16 bytes requested through SizeOfZeroFill is **255**, not the expected zero.
The complete actual record is retained unchanged at
`pe/reference/pe32-tls-zerofill-observation-windows-v1.json`. At `8efdd37`, three
additional independent executions of the unchanged PE observe **255, 255, 247**;
the complete raw report is retained unchanged at
`pe/reference/pe32-tls-zerofill-repeats-windows-v1.json`. Source/PE identities agree.
These are diagnostic captures, not accepted native/Alkyhol parity profiles.
Cause, actual allocation extent and safe post-template access remain unmeasured.
Do not turn those values into expected initialization semantics or ignore them
inside a successful comparison. Alkyhol explicitly rejects this runtime feature.

The separately pinned `pe32-tls-data` requests no additional bytes and reads only
declared template/private data. Its byte-for-byte comparison does not weaken the
original probe or qualify as nonzero-zero-fill parity. Compiler-zero-initialized
variables within the template are a distinct, successfully measured case. All
three [PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42437522172) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42437522174) pass
on the first capture revision; the diagnostic step checks execution/output capture,
not zero-fill conformance. See [PE32](pe32.md).

## PE32 FS/TEB: native Windows and physical M4

All 27 CI jobs pass at strict-reference revision `2ad7650`: three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42436726834) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42436726819).
Downloaded clean native Windows/hosted Mac reports and fresh clean physical M4
execution at that same revision agree on all 16 raw DWORDs and exact frozen
source/executed PE identities. Both strict paired comparisons pass, including
actual forced relocation, host-only same-image repeat and selector/descriptor
cleanup. Existing frozen PE32/PE64, released application, DLL/source/SDK/COM, shader
and Metal comparisons remain green. Static TLS/callbacks and game entry remain
unsupported; this is the measured FS/TEB foundation.

Native Windows at `d25c411` returns all 16 raw DWORDs from the original owned
FS/TEB observer. All three
[PE jobs](https://github.com/daeken/Alkyhol/actions/runs/42436164368) and all 24
[conformance jobs](https://github.com/daeken/Alkyhol/actions/runs/42436164413) pass
on that capture revision, before the new Alkyhol FS implementation/comparison.
The complete downloaded native report is frozen unchanged at
`pe/reference/pe32-teb-windows-v1.json`. Exact executed PE and C/.def source hashes
are required by default. No original fixture or game code/data is modified.

The probe observes FS:[0x18] self identity and stack containment through FS:[4/8].
SetLastError is visible at FS:[0x34], and a direct write there is returned by
GetLastError. All 10,000 imported round trips retain that coherence and FS identity.
Physical M4 development normal/forced-rebase execution agrees on all 16 words.
The host-only repeated-entry diagnostic emits two complete identical blocks on
one loaded image, with two actual selector-restoration/descriptor-cleanup checks.
This repeats an owned callable body; it does not measure native process restart.
Strict same-clean-revision paired CI/physical comparison passes at `2ad7650`
as recorded above.

Each PE32 image now owns a minimal low one-page TEB with self/stack/last-error
fields. Original assembly saves/restores host FS, restores it before Rust import
calls and reinstalls guest FS on return. GS is untouched. Actual DS/ES/SS/FS/GS
selectors must match after entry; host FS must initially be unused. A process-wide
exclusive lease prevents concurrent entries from retargeting the shared LDT slot.
Occupied slot 514 fails before replacement; it is cleared before successful return.
Two entries on the physical M4 observed that cleanup/reinstallation successfully.
This does not measure hidden segment bases or all hardware limit/fault behavior.

All 105 PE host tests/12 compiler checks, 35 portable tests, 281 Python tests and
Clippy pass locally. Host assertions separately cover descriptor bit fields/base/
limit, initialized TEB extents/field values and memory permissions, last-error
cleanup and exclusive-lease rejection. Protocol negatives enforce all raw words,
scalar types, crashes, native diagnostics, actual relocation/cleanup, both repeat
blocks, missing references and frozen identity. Existing PE32 integer/stdcall,
self-export, geometry and certificate development runs retain their original
finite results. Strict paired CI retains every older frozen comparison.

Static TLS/callbacks, PEB, CRT, SEH and guest threads remain unsupported. An empty
SEH-chain marker and zero unsupported TEB fields are not implementations of those
facilities. The local game still stops at its unchanged startup gate; no game code
executes. This is FS/TEB foundation evidence for later TLS, not game compatibility.
See [PE32](pe32.md).

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

## Integrated PE32 file/window bootstrap (2026-11-08)

Physical M4/Rosetta owned PEs exercise actual file contents, sharing, seeking,
truncation, live descriptor metadata and actual AppKit window ownership with
original x86 callbacks. These new families are not frozen native evidence yet.
CI now schedules the identical PE files on Windows. Original DAO passed real log
creation/writing and reached its thread-name RaiseException before any window or
D3D9 call. The run used e94ac05 plus explicitly recorded dirty runtime sources;
local captures preserve that distinction. Native e94ac05 search-family observation
succeeded; wide full-path sizing remains an explicitly unresolved mismatch.

Native Windows 42537596292 passed both new owned file/window families. The literal
CI-produced PEs were then run on physical M4; full input identities and output
words agree. See pe32.md for hashes. This is finite API/window evidence, not DAO
window or game compatibility. The workflow retains failures in the existing
frozen atan comparison and wide full-path sizing. The new software-SEH/process
families passed physically; their native comparison is pending. All 163 PE Rust
library tests and PE Clippy passed after the software/process changes, and the
root kernel integration tests passed. Original DAO now executes its thread-name
handler and reaches CreateThread; threading remains the next integrated blocker.

Physical M4 now passes the owned original-x86 worker family, including static and
dynamic TLS/FLS isolation, real contention, callback destruction, suspended starts,
non-returning ExitThread and joins. Original DAO workers execute their DLL startup
and main startup reaches its window class and LoadBitmapW resource request. No
DAO HWND/window or D3D9 initialization is claimed. Native worker comparison is
pending. Software exception record/context equality remains unverified after a
native combined-field mismatch; raw per-field capture was expanded rather than
weakening a frozen fixture. Host worker scheduling serializes CPU entry and yields
at APIs; it does not demonstrate unrestricted preemptive Windows threading.

The latest local GUI batch passes 167 PE Rust library tests (including an actual
CoreGraphics row/color round trip), PE Clippy across all targets, and physical
owned bitmap, primary/worker GDI-DIB, worker-window and message-queue PEs. Original
DAO's splash bitmap was captured from its actual on-screen native window; image
and traces remain ignored local. Its next resource-manager run hit the execution
budget, so reaching D3D9 or complete startup is not claimed. Native Windows GUI/
SDK comparisons are scheduled in the existing workflow and remain pending.

The original DAO splash now survives a corrected GUI publication lifetime: the
main thread holds the result lock through its final wakeup, so a timed-out
submitter cannot destroy a request while the notifier still uses its condition
variable. A 512-request owned handoff stress test passes. New real asynchronous
directory notifications use native kqueue descriptors and an active monitor
between guest reads; the owned PE observes file creation, retained changes,
completion, cancellation and untouched guards. All 170 PE library tests and PE
Clippy pass physically. Windows comparisons for this new directory family are
scheduled, not yet observed. Its finite ASCII namespace, budgets, fixed filter,
rescan/error cases and lack of APC/IOCP delivery are explicit. The actual DAO
run passes three original directory-monitor requests and next stops in PhysX at
GetAdaptersInfo, before D3D9. This is still startup evidence, not game support.

CI 42544367465 passed the PE producer and PE64/native comparison jobs; the native
PE32 job retained the known full-path/software-context mismatches and exposed a
new message fixture assumption: an unfiltered native window queue includes OS
messages as well as the owned test messages. The new fixture now requests its
specific custom/paint message ranges, retaining the same content, ownership,
callback, guard and lifetime assertions. The older observation remains a failed
comparison; a new native run is required before claiming agreement.


The next physical DAO run reaches original PhysXCore.dll initialization, including
its original CUDA-runtime dependency startup, through a checked explicit dormant
DLL graph. Original initializers run at LoadLibrary, not at preparation time.
The current actual stop is WS2_32 ordinal 115 (WSAStartup), before D3D9. The owned
DLL witness executes original attach/export/worker/detach code and rejected attach;
its new native comparison is scheduled. Read-only typed scoped registry queries
and mixed A/W handles pass their owned PE. All 172 PE library tests and Clippy pass.
GetAdaptersInfo is an explicit unsupported API failure, not successful networking.
Additional host-service gates leave all previous offsets intact. Full-path,
message/worker/GDI and software-context native mismatches remain recorded in CI
42546413828; that run's directory notification step passed. No new full-game,
GPU PhysX or native game equivalence claim follows from this startup progress.

Original PhysX passes its ordinal Winsock startup and, with the explicit extended
legacy workstation persona, next reaches CreateFileMappingA. Physical M4 runs the
owned Winsock PE against an independent real loopback server, including exact
payload bytes, ordinal/name identity, initialization references and buffer guards.
All 175 PE library tests and Clippy pass. Windows SDK/identical-PE checks are
scheduled; this is not yet native Winsock agreement or complete networking.


## DAO non-D3D9 startup boundary reached

On physical M4/Rosetta, the unchanged original DAO EXE passes original PhysX
initialization, creates its own hidden 1024×768 main client window, requests its
original cursor files and reaches the guarded Direct3DCreate9 call. Status 78 is
an explicit unimplemented D3D9 stop, not successful device creation, rendering,
menu or gameplay. All game files, screenshots, raw traces and derived inventories
remain local. D3D9 is still excluded from the task.

The current local batch passes all 176 PE library tests, PE Clippy, new shared
section/brush/cursor/affinity-error/DLL-worker PEs and the previous bitmap,
primary/worker GDI and message regressions. Each is finite owned evidence. New
native Windows comparisons are scheduled. The preceding CI 42550220024 passed
Winsock, mixed-registry, deferred-DLL and directory checks; unrelated recorded
full-path, message/worker/GDI and software-context mismatches remain unaltered.
Literal CI Winsock/registry PE files subsequently ran on physical M4 and agreed
with their native Windows input identities and every normalized output word.
Their hashes and scope are recorded in pe32.md. None of this establishes full
native Windows DAO behavior or game compatibility.

A clean physical rerun at `0136bde` confirms the unchanged EXE identity, hidden
1024×768 main client, 69 successful original cursor constructions and explicit
status 78 at Direct3DCreate9. Detailed captures and dependency identities stay
local. CI 42555112613 is pending; the new native checks are not results yet.


## First D3D9 binary COM/GPU milestone

The user authorized D3D9 implementation and Microsoft tutorial witnesses. The new
separate wgpu crate performs actual GPU color clear and padded readback. Physical
M4 runs unchanged Microsoft Tutorial 01 InitD3D/Render/Cleanup bodies inside a
finite PE32 witness, through real binary COM identities and owned system-memory
surface readback; all 195 RGB pixels and the COM/scene/stack assertions pass. The
interactive entry loop is replaced by the explicit witness, not claimed unchanged.
All 177 PE library tests, backend tests, explicit GPU test and Clippy pass locally.
Native D3D9/SDK comparisons are newly scheduled, not yet observed. No shaders,
triangles, full D3D9 compatibility or DAO frames are claimed by this clear test.

CI 42555112613 passed the newly added brush/cursor/SDK/DLL-worker checks, as well
as the previous worker/GDI comparisons. Full-path, message and software-context
failures remain. Its section test exposed a wrong lifetime assumption: native
views retain backing data after all handles close, but not the name. Runtime
namespace leases now belong to handles; a new logical object uses a new backing
inode so existing views remain independent. The owned test checks that measured
behavior and locally passes; new native confirmation is still required.

The clean original DAO rerun at `71ef46e` confirms the factory/adapter path and
GetAdapterModeCount stop. CI 42559007382 passed hosted Metal and the identical-PE
producer, but its independent Windows SDK assertion rejected the adapter record
size: PE32 D3DADAPTER_IDENTIFIER9 is 1100 bytes, not the initially assumed 1104.
All checked member offsets passed. The runtime write and assertion are corrected,
and the PE witness now guards both ends of the exact 1100-byte output. The initial
failed native job remains recorded; the native pixel step did not run in it.


The next physical milestone runs unchanged Microsoft Tutorial 02 vertex-buffer
and rendering bodies through PE32 COM/wgpu, producing the original colored
triangle. Its witness checks lock bounds, ownership and drawing after external
vertex-buffer Release, and records all 90,000 RGB pixels. The native contract
requires exact coverage and per-channel RGB error at most one, declared before
the native triangle observation. All 177 PE library tests, five backend tests
(including two explicit physical GPU tests) and Clippy pass. This remains a
restricted transformed/diffuse fixed-function profile, not legacy shader support
or DAO rendering. CI 42559416643 passes corrected SDK layout checks but stops
before native pixels at a platform-dependent identity-key mismatch. Canonical
source keys and byte-preserving Git attributes correct the observer; failed runs
remain failed evidence. See d3d9.md for precise supported/unsupported operations.

CI 42560083730 passes the corrected SDK checks and the full native D3D9 clear
witness, agreeing in all 217 words with hosted Metal. The same-PE triangle's
90,000 coverage decisions agree and all RGB channels differ by at most one, but
its overall witness fails a wrong invalid-input premise: the native lock result differs from the
expected INVALIDCALL value for an invalid range; its raw HRESULT was not retained
and acceptance versus another error remains unknown. The failed result remains recorded. The
next witness uses valid GetDesc metadata instead; owned-storage bounds remain
enforced and separately tested. All 178 PE library units pass. CI 42560238748's
Linux Vulkan-software job independently passes all five backend tests via Mesa
Lavapipe; this is software Vulkan evidence. Full native triangle witness agreement
is pending. Hashes, counts and limitations are in d3d9.md.

CI 42560726454 passes Metal, Vulkan-software, Windows SDK and native clear jobs/
steps. Its triangle control/metadata/stack values all pass; its comparison fails
because the observer includes nine metadata words in one pixel slice. A shared
23-word-prefix extraction corrects the parser without changing pixel coverage or
tolerance requirements. The corrected comparator passes the preserved raw
Windows/Metal outputs; four regression units cover extraction, RGB thresholds,
coverage and exact lengths. The old CI remains failed pending a new complete run.
Literal earlier CI PE files also ran on physical M4: full clear output agrees with
Windows, and the triangle finite pixel audit matches coverage with RGB error <=1,
while retaining the old overall invalid-lock mismatch as a failure.

CI 42561453681 at 9dbab80 passes the complete D3D9 workflow: hosted Metal, native
Windows D3D9/SDK and Linux Mesa Vulkan-software. Both original C++ tutorial
witnesses pass. The identical transferred PEs subsequently run on clean physical
M4 runtime 9dbab80: clear agrees in all words, triangle coverage agrees at all
90,000 pixels and RGB channel error is <=1, with complete control/stack checks.
The acceptance thresholds are unchanged. SHA-256s and scope are in d3d9.md. The
local 178 PE, five backend and four observer tests plus Clippy pass. Other PE/
conformance failures remain separate; no full D3D9 or DAO rendering is claimed.


DAO now executes adapter-mode enumeration through a retained actual CoreGraphics
display snapshot, then conservative GetDeviceCaps and format/multisample queries.
The virtual adapter exposes X8R8G8B8 with logical desktop coordinates consistent
with the existing window bridge; unknown/dynamic native refresh remains zero.
Enumeration does not implement fullscreen mode switching. The 304-byte caps
record exposes the existing transformed-color/one-stream/one-target renderer and
explicitly reports no programmable shader or texture support yet. Unsupported
texture/sRGB/depth queries fail rather than promising unimplemented rendering.
An owned query PE checks enumeration consistency and guarded mode/caps outputs;
physical checks and all 180 PE library tests/Clippy pass. New native query/SDK
comparisons are scheduled. Original DAO then emits its real “Failed to detect a
supported video card” error; it has not created a D3D9 device. Its message-box
path also exposes the existing host-main-thread GUI pumping limitation. This is
not an API stub or a game-success claim. Legacy shader lowering is the next
substantive rendering dependency, and unsupported shader versions stay zero
until real execution exists. Detailed captures and game-derived inventories
remain local.

The D3D9 adapter-query milestone 0b3d13a passes CI 42563476060: hosted Metal,
native Windows mode/caps/SDK comparisons and Linux Mesa Vulkan-software. The
next bounded shader milestone executes owned literal SM3 bytecode via open pinned
vkd3d-shader/Naga/wgpu, through both real PE32 COM and the physical M4 backend.
It includes semantic stage linkage, explicit triangle PointSize/half-pixel
adapters, actual matrix constants, bytecode/declaration roundtrips and retained
binding lifetimes. Its owned shader artifact regeneration is exact; 180 PE library
units and PE Clippy pass. Native Windows shaded-pixel comparison is still pending.
This is not execution of original DAO shaders or full SM3 conformance: shader
versions remain unadvertised. Actual DAO selection rejects an unsupported texture
sRGB-format query before its later capability callback; the prior shader-version
explanation was inference. No DAO device/frame has been observed. See d3d9.md for
compiler pins, explicit configuration, tests and remaining scope.

D3D9 CI 42567868544 at 039b7f9 passes hosted Metal, native Windows SDK/D3D9 and
Mesa Vulkan-software. The identical owned legacy-shader PE agrees on every one
of 2,099 output words, including both frames, and the transferred PE also agrees
on physical M4 with the recorded current runtime. The earlier compiler-setup CI
42567565446 failed before creating the PE/native observation and remains failed.
New BGRA8 mip/sRGB texture COM and SM2/128-arithmetic-slot prerequisite witnesses
pass locally on physical M4; their native comparisons are scheduled. Successful
explicit open compiler initialization enables the bounded SM2 profile; unsupported
operations continue to fail. Original DAO passes its real adapter gate and reaches
CreateDevice, where unsupported presentation settings remain the next boundary.
There is still no original-game shader/frame observation. The exact contracts,
limits, source pins and retained diagnostics are in d3d9.md.

With corrected capability field offsets, original DAO successfully creates its
windowed immediate A8R8G8B8 device with multithreaded software vertex processing,
then enters the original renderer's state initialization. Supported render state
now affects actual GPU blending, culling, write masks and sRGB output. Physical
GPU/state-PE checks pass; new native state comparisons are pending. No DAO frame
has been rendered. The bounded no-depth profile is still explicit.

CI 42569366247 is not fully successful: Metal/Vulkan-software pass; native Windows
passes the exact 2,117-word texture/sRGB witness and previous witnesses, but rejects
the owned long SM2 pixel shader at creation (exit 6). The owned HLSL compiler had
emitted FRC directly to oC0, forbidden by SM2's MOV-only output rule. Fixture-only
normalization now preserves all arithmetic in r0 then writes oC0 with MOV;
regeneration/provenance records it, and raw failure HRESULTs are retained on new
runs. Original game shaders are not rewritten. Native corrected-SM2 and state
comparisons remain pending. A separate new caps field-offset mistake is fixed
and covered by independent SDK offset assertions. See d3d9.md for exact evidence.

Original DAO's initial renderer state/texture-stage/sampler/stream/index reset now
completes and it executes real native window information, position and style work.
WINDOWINFO has a bounded 60-byte layout; its creator version explicitly names the
bridge's Win32 4.0 class profile. Owned physical checks cover actual frame/client
transitions and same-owner positioning. No game frame exists yet. All 181 PE
library tests and Clippy pass before the next input/resource blocker; the new
native comparisons remain pending.

CI 42570648162 passes the native 2,125-word blend/write-mask state witness and
previous witnesses but fails overall: the long ps_2_0 shader still returns native
0x8876086c (retained), and Vulkan-software exposes a single-sample mask difference.
The owned stress input now uses ps_2_a's extended budget with the required output
MOV; no math/tolerance is relaxed. The backend explicitly ignores MULTISAMPLEMASK
on single-sample targets per D3D9, and the updated native witness tests that rule.
The revised PE checks pass on physical M4; complete new CI is pending.

## DAO renderer reset: independent native/Metal/Vulkan results

CI [42574139390](https://github.com/daeken/Alkyhol/actions/runs/42574139390)
passes all three D3D9 jobs on `aaa9e61`: hosted Metal, independent native
Windows SDK/D3D9 and Linux Mesa software Vulkan. The extended SM2a fixture,
texture/sRGB, 2,133-word render-state witness, window transitions and earlier
finite witnesses pass their existing exact/tolerance rules. This supersedes
neither retained failed SM2 runs nor unmeasured full game/frame behavior.

## DAO DirectInput lifecycle and first renderer resources

The unchanged local DAO EXE now passes original DirectInput8 creation, A/W
32-bit COM gates, keyboard/mouse creation, original data formats, cooperative
settings and buffer-size properties. Interfaces own real low RO vtables/RX
stdcall gates and checked storage; devices retain their factory. The explicitly
named virtual system devices use a Quartz event backend. Unsupported effects,
controllers, action maps, event notification handles, background/exclusive
acquisition and confinement return failure. ClipCursor(NULL) releases this
bridge's unconfined cursor; a range fails with Win32 error 120. DAO handles this
failure and continues; storing a rectangle is never reported as confinement.

Acquisition requires an actual foreground owned window and a successful native
event tap; otherwise it fails. In the observed startup, the acquisition attempts
failed before a native tap was acquired. Live keyboard/mouse event delivery is
**not established** by this run. The own-process keyboard tap, foreground-gated
mouse tap, bounded event queue and relative accumulators are implemented; host
queue/overflow/wrap tests are synthetic units, not native input observations.

Original rendering now reaches backbuffer/depth-attachment queries, default
viewport inspection and a 1x1 texture lock. GetDepthStencilSurface returns actual
absence (D3DERR_NOTFOUND with null output) because this device requested no
automatic depth buffer. GetViewport and unchanged full-target SetViewport share
the real target extent; custom viewports remain explicitly unsupported. The
explicit whole-texture RECT now shares NULL locking and whole-level upload;
subrectangles remain unsupported. Original DAO successfully uploads its first
1x1 texture, then requests a full-size render-target texture (usage RENDERTARGET),
which is the next unsupported resource. The earlier run exposed an original null
dereference after our rejected explicit RECT; neither run rendered a frame.

All 183 PE library tests, integration/doctest targets and Clippy pass on the
physical M4 x86-64/Rosetta host before that texture-lock change. The owned input
lifecycle PE passes locally, covering both ANSI/Unicode layouts, COM identity and
parent lifetime, data formats, buffer/application properties, guarded descriptors,
untouched output on nonacquired state and stdcall stack balance. Its raw APPDATA
default is retained, with native Windows comparison pending. The expanded D3D9
query witness passes all 233 words locally, including absent depth and viewport
queries. SDK declarations are independently checked in CI; original game data,
traces and shaders remain ignored local. No DAO frame, menu or gameplay exists yet.

Input CI 42579539422 passes hosted Rosetta and independent Windows SDK layout
checks, then rejects before native execution: the new fixture directory lacked
an LF checkout rule, so actual C++ source hashes differ on Windows. The explicit
Git attribute now preserves the same source bytes; identity checks are retained,
not normalized or relaxed. This failed run supplies no native input observation.

Input CI 42580016760 clears the exact-byte identity and independent SDK checks,
but the native process times out after the entry routine returns: DirectInput
can keep process threads alive. The owned console witness now explicitly calls
ExitProcess after flushing output, and timeouts retain partial raw captures.
This changes the witness process lifecycle, not the input expectations or game.
The timed-out run is not counted as native input parity.

Native input run 42581056533 exits successfully and retains all 60 raw words,
but exact comparison fails. The witness incorrectly equated an interface pointer
with its IUnknown pointer; it now checks the actual COM canonical-IUnknown law.
The keyboard object's requested offset zero is reserved, and native Windows
returns 0x80070002 leaving the property sentinel unchanged. The virtual keyboard
now exposes only its actual mapped physical scan positions. The same reserved
query remains an explicit negative observation in the witness; valid keyboard
APPDATA/object queries use DIK_A (30), while mouse queries retain X offset zero.
Native mouse APPDATA's observed default is 0xffffffff. These corrections preserve
failures and add valid requests; they do not discard unknown native outputs.
New keyboard-positive and complete input comparisons remain pending.

## DAO render-target allocation through first shader creation

Original DAO successfully creates a real full-size BGRA8 render texture, obtains
its cached surface, queries actual dimensions/viewport, uploads a WRITEONLY vertex
buffer and a 256x1 texture and reaches first vertex-shader creation. The original
D3DX DLL's C++ operator new import is the next actual stop. Game bytes/shaders
remain local and no DAO draw, frame or live input event is claimed.

Physical M4 passes the 2,118-word owned --targets PE: real target clear/readback
and sampled frames, guarded resource metadata, 20 target/storage/lifetime checks,
container recovery and target-sized viewport reset. Shared render/sample views
use one actual GPU texture and shared initialization. Storage remains available
after external references and surface handles go away; feedback/uninitialized
reads fail. The independent GPU test now renders into the texture and samples its
real colored interior in a subsequent draw. All 183 PE library tests and remaining
integration/doctest targets pass; PE/backend Clippy pass. Native Windows and Linux
software Vulkan comparisons for this new milestone are pending. CI 42579539633
and 42580016729 passed all three earlier D3D9 jobs, before these render targets.

## Input comparison passes; original D3DX allocation and memory path

Input CI [42581759670](https://github.com/daeken/Alkyhol/actions/runs/42581759670)
passes both hosted Rosetta and native Windows jobs on 9d6af8a. All 64 raw words
agree, including canonical IUnknown, guarded A/W descriptors, mapped/reserved
keyboard objects, raw APPDATA defaults, properties, nonacquired output and stack.
The physical M4 report has the identical PE SHA and identical complete output;
its unused shared-header revision differs from the CI source pin, so it is not
presented as an identical source build. Subsequent input CI 42581978223 also passes.
This is lifecycle/ABI evidence, not delivered native input or game compatibility.

The original D3DX DLL now executes C++ scalar new, memcpy and actual shader
constant-table setup. Scalar/array new and malloc share a real private CRT heap;
matching delete/free release owned blocks, including null no-ops. Exhaustion,
foreign pointers and unimplemented exception/new-handler policy stop explicitly.
Checked memcpy/memmove/memset, strchr and unsigned-byte strncmp use actual guest
storage and correct cdecl cleanup. Traversal/size bounds fail before pretending
to complete. No game call is replaced by a successful zero return.

The owned 121-word PE passes on physical M4: independent live allocations across
zero/small/page-spanning sizes, preserved payload checksums, paired frees, null
delete, copy returns/guards, overlapping moves, memset truncation/zero count,
unsigned narrow comparisons/search and stack balance. All 183 PE library tests,
remaining integration/doctest targets and Clippy pass. Native CRT comparison is
pending. Original DAO now gets through many original vertex/pixel shader creation
and D3DX constant-table calls, but no original draw is observed. It eventually
expects a depth surface after our format queries reject all candidates; the
resulting null dereference is the next actual blocker. A depth attachment must
be implemented and negotiated, not fabricated in GetDepthStencilSurface.

## Native render targets/CRT agree; real DAO D16 negotiation

D3D9 CI 42581978233 passes all three jobs on 199279f, including the exact
2,118-word render-target witness. Subsequent 42583172076 also passes. Input/CRT
CI 42583171880 passes both jobs on 632b227; all 121 raw CRT words match independent
native MSVCRT. The physical M4 has the same CRT PE SHA and exact complete output.
These observations do not establish original shader drawing or live game input.

The actual original depth failure came after rejecting every offered depth format.
Real D16 storage, capabilities, depth matching, automatic device attachment and
explicit attachment APIs are now implemented. The original EXE negotiates and
creates D16 and passes the earlier null dereference. Only that actual format is
advertised; no D24/stencil substitution is hidden. The 128-object prototype bound
then stopped its real shader batch; it is raised to a still-bounded 4,096, without
dropping shader calls. It now reaches dynamic vertex/index buffer allocation.

Physical M4 passes the 2,100-word owned depth PE and all seven GPU tests, including
real D16 clear/occlusion/write/compare and lifetime behavior. PE library (183),
integration/doctest and Clippy checks pass. New native Windows/software Vulkan
depth comparisons are pending. No original DAO draw, menu or gameplay is claimed.

## D16 comparison passes; original DAO reaches swapchain queries

D3D9 CI 42584925886 passes all three jobs on 2803e76. All 2,100 D16 witness words
agree on native Windows, hosted Metal and the physical M4 with identical PE SHA;
Linux software Vulkan also passes its real depth/GPU tests. This is finite D16
occlusion evidence, not full scene/depth precision or game-frame parity.

Original DAO now creates and initializes actual dynamic vertex and 16-bit index
buffers and reaches GetSwapChain. New default-pool buffer storage supports 64 MiB
bounded allocations, real metadata/parent ownership and appropriate dynamic lock
flags. Programmable indexed draws use actual GPU INDEX buffers with checked
signed-base/minimum/stream/start-index translation. Physical M4 passes a shared
four-vertex quad GPU test and all 2,100 words of an owned indexed PE, including
both source index widths, DISCARD/NOOVERWRITE and storage after external Release.
Native indexed comparison is pending. All 183 PE library tests, remaining targets
and Clippy pass; no original game draw, frame or live input event is claimed.

## Indexed comparison passes; DAO enters font setup

CI 42586300117 passes all three D3D9 jobs on 890f0fe. Native Windows and hosted
Metal agree on every indexed witness word; physical M4 has the same PE SHA and
same full 2,100-word output, including 16/32-bit source indices and dynamic locks.

The original EXE now passes implicit swapchain access, device/backbuffer metadata
and further renderer initialization, then enters Scaleform font setup and stops
at its actual unnamed CreateMutexA call. The new owned 235-word query PE passes
on physical M4 and covers real chain identity/parent/backbuffer lifetime and chain
Present, in addition to previous window/depth/viewport evidence. All 183 PE library
tests, remaining targets and Clippy pass. Native swapchain comparison is pending.

D3D9 failures are now explicitly logged even when ignored by the original caller.
This run also retains failures for D24S8, three texture requests and 15 packed
vertex declarations. Continuing through a handled failure does not implement
that resource or establish rendering quality. No original DAO draw or game frame
has yet been observed. These remaining resources must be implemented as needed.

## Original DAO font mutex and real thread ownership

D3D9 CI 42587447080 passes all three jobs on b0abac8, including the expanded
235-word swapchain query/presentation witness. No original frame is implied.

Original DAO now executes its unnamed CreateMutexA call and progresses through
further font/resource setup to another file-open profile boundary. The new A/W
unnamed mutexes have inaccessible low handle tokens, actual per-thread recursive
ownership, timed/non-alertable waits that release the CPU lease, correct foreign
release failure and ownership abandonment during real worker teardown. Closing a
handle releases its actual record; closing during a pending wait remains outside
the defined profile. Existing named-W/session/file-lock contracts and their frozen
observations are preserved; named ANSI conversion and APC waits fail explicitly.

The owned PE passes all 39 words on physical M4: real x86 worker contention,
zero/timed waits, recursion preventing early handoff, wrong-owner error 288,
acquisition after release, actual worker-exit abandonment, initial ownership,
A/W entry points, handle closure and stack balance. Two ownership units and all
185 PE library tests, remaining integration/doctest targets and Clippy pass.
Native Windows comparison is pending. New texture failures are retained rather
than counted as successful loads: original DXT5 requests remain unsupported.
The game still has no observed draw, menu or gameplay.

## Native mutex agreement and actual settings file security

Input/startup ABI CI 42588914270 passes both jobs on ba89035. Native Windows,
hosted Rosetta and physical M4 agree on the same 39-word mutex PE and its SHA,
including real worker contention and exit abandonment. D3D9 CI 42587447080 also
agrees with the physical M4 on the same complete 235-word swapchain query PE.

The next actual file boundary uses SECURITY_ATTRIBUTES {12,NULL,TRUE} for an
original settings write. This now selects the same real default-security file
open as NULL attributes and retains the logical inheritance flag in the owned
lease. GetHandleInformation reports that stored flag for live files, while
known unsupported non-file objects and invalid handles fail explicitly. Guest
child creation/inheritance transfer remains unavailable; this is current-process
metadata, not a claim that host compiler subprocesses inherit guest file handles.
Custom security descriptors and template handles remain unsupported.

The expanded file witness retains its previous sharing/content/seek/truncate
checks and adds true/false inheritance metadata, closure and invalid-output
preservation (36 words). Actual final file contents remain empty as before.
Physical M4 passes it, all 185 PE library tests, remaining targets and Clippy.
Native expanded-file comparison is pending. The real game progresses past the
settings write; no original game draw or frame is established.

Native file-security CI 42592214809 executes successfully and agrees on every
new default-security/inheritance case, but its final negative check fails: the
witness incorrectly treated pseudo process handle -1 as an invalid handle.
That case now supplies actual NULL, retaining the same false/error-6/untouched
output requirement. Pseudo/non-file handle information remains explicitly
unsupported by this file-only implementation. The failed raw native report is
retained and is not counted as complete expanded-file parity.

## Original DXT5 font atlas uploads

The unchanged original EXE now creates, locks and uploads its original DXT5 font
textures through actual BC3 GPU storage, preserving compressed data and mip/color
space choices. It passes the settings-write boundary and reaches an 8-bit palette
DIB request during text setup. No original draw, menu or gameplay is observed.

Physical M4 passes 10 backend host units plus the module integration check, nine
actual GPU tests, all 185 PE library tests and remaining targets, and PE/backend
Clippy. The owned 2,118-word BC3 PE checks every mip's surface metadata/identity,
container, complete encoded block roundtrip, deferred lifetime and exact full
linear/sRGB frames. Direct GPU tests independently cover BC1/2/3 with alpha,
all four mip levels including 1x1/2x2 tails and shared initialization guards.
New native Windows/software Vulkan BC comparisons are pending; no game data or
derived shader/texture inventory is uploaded by these CI steps.


## DAO indexed font bitmaps and continued archive loading

D3D9 CI 42593980363 passes all three jobs on 7c6472d, including native Windows
execution of the identical BC3 PE and software Vulkan GPU tests. Input/startup
CI 42592868000 passes the corrected 36-word file security witness and the existing
input, CRT and mutex comparisons. These are finite API observations, not DAO
frame parity.

DAO's actual font setup requests top-down 8-bit BI_RGB DIB sections with 256
RGBQUAD colors. These now own real indexed bytes with DWORD row stride,
orientation, palette decoding and exact-color writeback. Padding and unchanged
indices are preserved; unsupported quantization, other compression and palette
modes stop explicitly. The owned 33-word PE checks guarded DIB metadata, raw
indices/padding, GetPixel, exact FillRect writes, both orientations, RGB copies
in both directions and object/stack lifetime. It passes on physical M4 along
with all 185 PE library tests, remaining targets and Clippy; native comparison
is pending.

The private guest now has an explicit empty COM class registration catalog:
no optional guest COM servers have been registered. This is selected virtual
installation state, not a captured Windows registry and not an inference that
unknown keys are absent. All queries outside declared scopes still stop. The
private run records profile/catalog hashes alongside runtime and original EXE
identities. With that configuration the unchanged original EXE continues into
its original archive searches/opens and a larger original shader batch. No
original draw, Present or game frame is established yet. Original files and
all derived traces remain local.

## DAO audio startup reaches real COM apartment state

The original FMOD path now calls CoInitialize against real per-guest-thread
apartment/balance records. STA/MTA mismatch fails without acquiring a balance;
S_FALSE acquires one, final CoUninitialize permits a new mode, and actual worker
teardown clears that worker's state. OLE/DDE, RPC, proxies and registered class
activation remain unimplemented. CoCreateInstance returns CO_E_NOTINITIALIZED
before initialization and REGDB_E_CLASSNOTREG only when *both* configured HKCU
and HKLM class-registration scopes establish absence; registered or unknown
classes stop explicitly. No success object or audio endpoint is fabricated.

The new 25-word identical-PE witness passes on physical M4: nested initialization,
mode changes, real worker isolation, uninitialized/missing-class failures with
NULL output and guards, cleanup and stack balance. All 189 PE library tests,
remaining targets and Clippy pass. Native COM comparison is pending.

CI 42595589598 on 39b237b agrees on 32 of 33 indexed-DIB words but fails the
reverse-copy raw-index assertion. Its raw native observation is retained. The
witness omitted required GdiFlush synchronization before reading DIB bytes after
GDI drawing (CreateDIBSection documentation explicitly requires this). It now
flushes before each such read, retaining the original RGB/index/padding criteria.
All admitted Alkyhol GDI writes are synchronous, so its calling-thread flush has
no pending batch or errors. The synchronized witness passes locally; native
retry is pending, and this correction is not yet counted as native DIB parity.

The next integrated game run retains the configured empty class inventories and
original EXE/source/profile identities. Its startup evidence, including any
failed audio initialization, remains private. No original DAO frame is claimed.


## Packed DAO vertex fetch and native COM evidence

Original declaration types SHORT2, D3DCOLOR and UBYTE4N now expand using D3D9
input rules into float4 GPU registers. Float1/2/3 missing components use zero/one;
float source bits, guest storage, topology and source indices remain intact.
Indexed conversion reads only initialized referenced vertices. Original POSITIONT
bypass declarations and other integer/half formats remain explicitly unsupported.

Four backend fetch units cover signed extremes/defaults, every 8-bit normalized
value, color channel order, unchanged NaN/signed-zero bits and invalid extents.
The owned packed PE renders both signed-short positions/ARGB colors and normalized
RGBA bytes through identical legacy shaders. All 2,099 words pass on physical
M4, including both full frames and retained COM/guest stack behavior; the existing
2,100-word indexed witness passes again after the canonical fetch change.
Backend units and PE/backend Clippy pass. New native packed comparison is pending.

CI 42596516259 on c2ad72a passes the 25-word COM comparison on native Windows and
hosted Rosetta, including both guarded activation failures. The physical M4's
literal PE identity and output match that native report. Original FMOD handles
the configured missing audio enumerator class and continues to waveOutGetNumDevs,
which remains the next actual startup blocker; there is no fabricated audio
endpoint or original rendered game frame.

That CI still fails the same indexed-DIB reverse-copy assertion after GdiFlush.
Synchronization is now correct, but it was not the cause of the mismatch. The
next observer preserves all previous criteria and adds all 16 raw destination
bytes (including padding) to determine the actual Windows color-index conversion.
No complete native DIB agreement is claimed. Both failed raw reports remain
retained locally.


## DAO proceeds through audio discovery, configuration and network workers

D3D9 CI 42596983817 passes all three jobs on a7a832f. Physical M4 and native
Windows execute the identical packed PE and agree on all 2,099 words, including
both complete frames. This is finite packed-fetch evidence, not original game
frame parity. The later D3D9 run 42597604307 also passes.

FMOD's actual device-change check now gets real current CoreAudio input/output
stream device counts; every call re-enumerates, so hotplug is not hidden behind
a permanent snapshot. Actual SDK C calls run in the x86_64 host. This adds no
playback, recording, synthetic device or Windows hardware identity. The original
caller continues through the absence of optional DirectSound, using NULL-module
GetProcAddress against the real EXE export table. Missing exports still fail;
forwarders, bad storage and arbitrary foreign module handles remain unsupported.
An EXE without an export table has genuine empty exports, not an unknown module.
The owned 16-word export PE passes locally, including actual function calls,
names, ordinals, missing errors and stack cleanup; native comparison is pending.

Relative INI directory spellings now use the checked process-owned current
directory (including explicit dot-relative descendants). Bare filename search,
foreign namespaces, dot aliases and escaping remain outside the profile; writes
retain the existing scoped Documents capability. The original SKU query advances
only after actual backing absence is checked. An owned 21-word PE checks existing
child-file values, both spellings, real absent leaf/parent errors and output
bounds, with new native comparison pending. Existing frozen INI fixtures and
criteria are unchanged.

Original DAO then creates two more workers. The reached non-alertable one-event
WSAWaitForMultipleEvents now waits on the actual kernel event while releasing the
CPU lease. WSA create/set/reset/close share real manual-reset Win32 event objects.
General multi-object wait sets and APC dispatch remain explicitly unsupported.
The owned 23-word PE passes real worker signaling, manual reset, wait-all with
one object, timed polls, invalid count/closed handle errors and stack balance.
The integrated game run reaches GetDiskFreeSpaceExA as its next unsupported call.
All 191 PE library tests, remaining targets and Clippy pass locally. New native
export, INI and event comparisons are pending. No original DAO draw, Present,
audio playback, menu or gameplay is claimed.

Native palette evidence from CI 42597604351 shows the Windows RGB-to-index
BitBlt path quantizes even colors that exist exactly in the supplied palette:
the raw payload differs, while all padding remains intact. The raw owned native
capture is preserved at pe/reference/pe32-palette-quantizer-windows-v1.json
(SHA-256 226be28fb8994fadba8f4eccc1c674942e59ba133cb8b0f8435620226a190598).
The prior exact-color reverse-copy implementation was not equivalent. BitBlt
*to* indexed destinations now stops explicitly until that path is implemented.
The admitted 31-word witness covers real palette storage, orientation, guards,
GetPixel, exact FillRect and indexed-to-RGB copying; a separate exclusion PE
ensures unsupported quantization cannot report success. Both pass locally.
Earlier failed captures and their stricter reverse-copy criterion remain retained;
they are not counted as complete conversion parity. New native comparison covers
only admitted operations, not the excluded Windows quantizer.

## DAO now queries actual available disk space

GetDiskFreeSpaceExA now resolves the checked process current directory for NULL
or the supplied ordinary directory, then gets actual statvfs user-available,
total and free block counts through SDK C layouts. DWORD-wide guest addresses
receive checked 64-bit byte values; optional outputs are supported and failure
leaves them unchanged. Block multiplication is checked; no fixed free-space
capacity or guessed quota is supplied. This is host filesystem accounting,
not an observation of Windows disk capacities.

The owned 14-word PE passes actual positive accounting, output guards, optional
fields, real absent-parent error and stack cleanup on physical M4. Two units
cover overflow/distinct accounting and actual native SDK execution. All 193 PE
library tests, remaining targets and Clippy pass. Native finite API/guard
comparison is pending; disk sizes themselves are deliberately not compared
across machines. The original game passes its actual NULL-path space query and
next requests a lowercased settings path. The strict exact-spelling filesystem
profile stops there rather than fabricating file absence. No original frame or
audio output is claimed.

## DAO foreground transition and real LAN socket setup

The unchanged original EXE now reads its lowercased settings spelling through an
explicit `--casefold-file-opens` profile. Existing ASCII components must resolve
to actual unique names; canonical backing paths/inode checks, real missing
leaf/parent errors, reparse rejection and sharing remain enforced. Only existing
read-only opens opt in. Strict search/metadata paths and their frozen exclusions,
Unicode folding, ambiguous aliases and write capabilities are unchanged. The
owned 13-word file PE and a new filesystem unit pass on physical M4.

OpenSCManagerW is explicitly unavailable: NULL SC_HANDLE plus Win32 error 120,
with no successful service object/database invented. The original caller handles
that failure and proceeds to its foreground-window transition. This does not
implement SCM or imply a rendered frame. Actual startup then configures its LAN
socket using real SO_BROADCAST, IPv4 bind, manual events and sendto. The backend
normalizes Darwin's option bit into Win32 BOOL; endpoints and datagrams come from
actual BSD sockets. Immediate recvfrom and one-buffer non-overlapped WSARecvFrom
perform real receives with checked source/data/output ranges and CPU release.

The owned 35-word UDP PE passes real loopback packet roundtrips, actual assigned
endpoint data, option toggles, import-name/ordinal identity, descriptor/address
and buffer guards, nonblocking errors and stack cleanup. No game packet/asset is
part of the fixture. The separate 12-word exclusion proves asynchronous receives
return SOCKET_ERROR/WSAEOPNOTSUPP without changing outputs or claiming pending
I/O/completion. UDP truncation, nonzero message flags and general scatter arrays
remain outside this profile. Original DAO requests overlapped receive, receives
that explicit unsupported error, then stops at a real multi-object Winsock wait.
No original game draw, Present, audio playback or functional multiplayer is
claimed. All 195 PE library tests, remaining targets and Clippy pass locally;
new native case-open/UDP comparisons are pending.

Native CI 42599680071 passes the 16-word NULL-module export, 21-word relative INI
and 23-word single-event comparisons. Physical M4 agrees with the same literal
PE identities (the INI PE was downloaded and executed unchanged). CI 42600483781
also passes the expanded disk-space comparison. Both overall runs still fail
because the admitted palette comparator used Python path/string concatenation
after successful native execution. The raw native/hosted 31-word palette outputs
and PE/source identities already agree exactly; the comparator path is corrected
here, with full workflow retry pending. This does not add support for the
separately excluded Windows RGB-to-indexed quantizer.

## DAO advances through atomic network-worker event sets

CI [42609033378](https://github.com/daeken/Alkyhol/actions/runs/42609033378)
passes both jobs on 2096e47. The identical owned case-insensitive open, 35-word
UDP, disk-space, single-event, relative INI, NULL-module export, COM and admitted
31-word palette-DIB comparisons pass on hosted Rosetta and native Windows.
The palette comparator correction is therefore verified. This does not add
Windows RGB-to-indexed quantization or asynchronous UDP support.

Manual-reset event any/all waits now register as one set under a shared kernel
dispatcher. Initial any-ready priority is the lowest index; registered waits
latch the completing signal at set time, and wait-all requires actual overlapping
signaled states. Timeout and registration are ordered with set/reset. Host main
thread tasks run between wait slices while the same registration stays live;
there is no polling gap that discards a completed pulse. Events retain their
manual signal and every registration is removed on completion/timeout. Automatic
events, semaphore sets and alertable/APC delivery remain outside this profile.

Four root units check priority, timeout/error cleanup, duplicate rules, actual
registered any/all signaling and signal/reset within host task delivery. The
extended owned WSA PE passes all 41 words on physical M4, including two actual
worker signals, any/all states, timeouts, duplicate error and guest stack balance.
All 195 PE library tests, remaining targets and Clippy passed during this
milestone; full root tests and Clippy also passed. Native comparison of the new
41-word event-set witness is pending. Previous native 23-word evidence is retained.

Original DAO passes its two-event network wait and next stops at Netapi32 Netbios.
Its overlapped receive still returns an explicit WSAEOPNOTSUPP failure; no pending
operation, callback or completion is invented. No original DAO draw, Present,
menu, gameplay or audio playback has been observed. Original game evidence stays
local and the unchanged EXE identity is retained.

## Original DAO proceeds through NetBIOS failure and automatic-event waits

CI [42610901249](https://github.com/daeken/Alkyhol/actions/runs/42610901249)
passes both jobs on 83bf613. The 41-word manual event-set witness now agrees on
native Windows, hosted Rosetta and physical M4 with identical source/PE identity
(PE SHA-256 3150485bcb0e06058a58e9c6ea34f41f9aa3248caceaf036a4334a60d271a0b6).
The literal native artifact is retained locally. D3D9 CI 42610901334 also passes
all three jobs on this revision.

The reached synchronous Netbios NCBRESET now fails explicitly with NRC_OPENERR
(63): Alkyhol has no NetBIOS transport. Checked 64-byte Win32 NCB storage receives
only its failure/completion status; no adapter data, hardware identity or pending
request is supplied. Other commands and asynchronous completion remain excluded.
The original unchanged caller actually retries ten adapters, handles the failure
and starts another network worker. This is failure-path compatibility, not an
implemented NetBIOS service. The [NCB contract](https://learn.microsoft.com/en-us/windows/win32/api/nb30/ns-nb30-ncb)
and [Netbios status contract](https://learn.microsoft.com/en-us/windows/win32/api/nb30/nf-nb30-netbios)
are the public API references; a unit verifies preserved payloads and rejection
without mutation of unsupported asynchronous/callback profiles.

That worker revealed an automatic-reset event in its actual wait set. Single
event and set waits now share one dispatcher, so each automatic signal can
satisfy only one eligible wait and a partial wait-all does not reserve signals.
Successful all-waits atomically consume automatic members while retaining manual
members. Initial any-ready priority, saturated sets and completion latching remain
preserved. Semaphore sets, APCs and wait-all through distinct aliases to one
automatic object are explicitly outside this profile. Seven root event-set units
cover these rules, actual competing waiters and host task delivery; complete root
tests and root/PE Clippy pass. All 197 PE library tests pass locally. The expanded
63-word owned WSA PE passes physical M4, including actual automatic-event worker
signaling and mixed reset modes; new native comparison is pending.

The integrated original game passes that wait, starts two further workers and
reaches real recvfrom. The next unsupported import is Winsock ordinal 18 (select).
The unchanged EXE SHA is retained with private source/probe/config/capture
identities. No original game draw, Present, audio playback, menu or gameplay is
established; asynchronous receive remains an explicit unsupported failure.

## DAO socket readiness, gamma capability and worker-return contracts

CI [42612102370](https://github.com/daeken/Alkyhol/actions/runs/42612102370)
passes both startup/API jobs on 24dd10d. All 63 mixed-event output words, source
and literal PE identity agree with physical M4 (PE SHA-256
7b7f9602a385e31cb765f0fea4b051386693c7e228c82e4b77ee69cf826fd1d9).
D3D9 CI 42612102250 passes all three jobs, and conformance 42612102213 passes.

Winsock select now translates checked 32-bit counted descriptor arrays to actual
SDK fd_sets, waits on retained BSD socket descriptors with the CPU lease released,
and writes only genuinely ready members and counts. Timeout input remains
unchanged; empty sets/invalid sockets/invalid timeouts fail. Unique descriptors
can participate in different sets, so readable+writable membership counts twice.
In-flight selectors retain descriptors and make unsupported close/final-cleanup
cancellation fail explicitly. Pending/failed TCP connect completion and its
Windows exception/SO_ERROR projection remain unavailable (WSAEOPNOTSUPP), without
changed output arrays or invented readiness. UDP and synchronously connected
loopback TCP are admitted; arbitrary descriptor aliases, oversized sets and
nonzero message/async profiles remain separate. The public
[select](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-select)
and [fd_set](https://learn.microsoft.com/en-us/windows/win32/api/winsock2/ns-winsock2-fd_set)
contracts describe the source representation. The actual original TCP status poll
receives the explicit unsupported error and handles it; this does not establish
working original game networking.

The owned 42-word PE passes on physical M4: real loopback send/receive readiness,
write readiness, counted-array guards, empty/invalid errors, read-only timeval,
CPU-released blocking wait with a real sending worker, and caller-cleaned worker
return. Native comparison is added to the existing identical-PE workflow and is
pending. The worker entry uses literal RET rather than RET 4; its actual return
value becomes GetExitCodeThread without rewriting its machine code. Original
DAO independently exposed the same caller-cleaned entry form. The startup
trampoline now admits exactly the observed argument-retaining and stdcall
one-argument return extents; other stack returns still stop. General nested
callback convention checks are unchanged.

Original DAO also reaches SetGammaRamp. This is a void D3D9 method, and the
[documented unsupported-mode behavior](https://learn.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-setgammaramp)
is no effect and no error return. Alkyhol advertises neither gamma capability and
now checks the implicit chain, flags and complete 1,536-byte const ramp, then
explicitly logs that no display change occurred. No gamma/calibration support or
successful display operation is invented; GetGammaRamp remains unsupported. The
extended original-tutorial query witness passes 237 words and all 195 blue GPU
pixels on physical M4, including both void-call flag forms, ramp guards and stack
cleanup. This tests the ABI and input preservation, not applied display gamma.
All 197 PE units, Clippy and nine observer tests pass locally. New native query
comparison is pending; the original tutorial sources remain unchanged.

PE workflow 42612102237 still has the earlier native full-path, message, GDI copy
and software-context discrepancies. Its Linux comparison job also failed before
testing because a macOS-only dependency-fetch condition left Naga absent in
offline resolution. That job now fetches locked dependencies after installing
Rust on Linux as well; retry is pending. Frozen native criteria are unchanged.
No original DAO draw, Present, audio playback, menu or gameplay is established.

## Original Bink startup and real luminance movie planes

Original DAO now opens its original intro movie through the original Bink DLL,
starts Bink's original worker and reaches BeginScene plus movie-plane locks.
The original EXE/DLL/assets remain local with recorded identities. The requested
1ms timeBeginPeriod/timeEndPeriod policy is explicitly unavailable: TIMERR_NOCANDO
(97), no accepted request, timer-resolution promise or scheduler change. Original
Bink continues after that failure. This is not an implemented multimedia timer
service. The [timer request contract](https://learn.microsoft.com/en-us/windows/win32/api/timeapi/nf-timeapi-timebeginperiod)
is separate from actual native host Sleep and high-resolution counter calls.

Win32 WaitForMultipleObjects now uses the same real event/thread dispatcher as
Winsock waits, independently of WSAStartup. The owned 66-word PE exercises a
Win32 wait before Winsock initialization, then all prior manual/automatic worker
and wait-set cases. It passes physical M4; native comparison is pending. Semaphore
sets and APC delivery remain excluded.

Bink requests 1,280x720 Y and 640x360 chroma L8 textures. These now have actual
one-byte CPU mip storage and lossless GPU expansion to (L,L,L,1), with unchanged
source bytes and original dimensions. Linear samples preserve every luminance
value; no YUV-to-RGB replacement shader, downsampling or substituted movie is
introduced. Render-target/vertex-texture/sRGB L8 paths remain unavailable. The
public [D3DFORMAT contract](https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3dformat)
describes luminance and default channels. All original game shaders remain under
the existing open legacy-bytecode frontend.

Fresh managed L8 READONLY locks may expose their actual already-owned CPU
allocation, as requested by Bink while establishing previous/current buffers.
Initial byte values are not promised to equal Windows allocation contents. A
read-only unlock neither uploads zero pixels nor marks GPU storage initialized;
sampling still fails until an actual writable upload. The native-facing owned
witness tests the fresh read-only lock/return/const profile without comparing
undefined initial values, then writes known data and compares real rendering.
Other formats retain their existing uninitialized-shadow exclusion. See
[LockRect](https://learn.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3dtexture9-lockrect).

A format unit tests all 256 luminance bytes, alpha/default channels and odd mip
extents; physical GPU execution tests odd 3x2/1x1 mips, six luminance extremes,
alpha 255, uninitialized sampling rejection and the explicit sRGB exclusion. All
15 backend units, other non-ignored backend targets, backend/PE Clippy, 199 PE
units and nine observer tests pass. The new identical-PE L8 variant passes all
2,118 words on physical M4, including two complete original-shader frames, COM
and lock guards and guest stack. It is added to existing hosted Metal, native
D3D9 and software Vulkan validation. New native L8 agreement is pending.

D3D9 CI 42614135841 passes all three jobs on e5e68ab. Its 237-word query PE
(including gamma void-call/input preservation) agrees with the physical M4's
identical input and output. This is ABI/input and existing blue-frame evidence,
not applied display gamma. The Linux PE comparison job also passes after the
locked-dependency fetch correction; older native startup discrepancies remain.

### Preserved negative-timeout mismatch

The first socket-select native comparison, CI 42614135717, fails. Windows accepts
the owned {-1,2000} timeval instead of returning the witness's expected WSAEINVAL.
That clears the read count, causing the next reused-set observations to differ
too. The raw 42-word result is retained unchanged in
pe/reference/pe32-select-negative-timeval-windows-v1.json (SHA-256
38949ae9bfa73c5327c6141492c9ba786290bc2bbbaeacfb0bcfeb8401c2efbe).
The original source/PE identity and stricter failed assertion remain in history.
Its ordinary readiness cases and literal caller-cleaned worker return succeed,
but the entire witness is not counted as native agreement.

Non-normalized timeval inputs now stop explicitly before host select/output
mutation until their actual Windows normalization is established; they do not
return a fabricated invalid-parameter error or successful timeout. A separate
exclusion PE verifies that stop. The positive 39-word witness retains the
ordinary nonnegative timeout, all readiness/guard/error criteria and literal RET
worker, and initializes each subsequent descriptor set independently. Both
positive and exclusion witnesses pass locally; admitted native retry is pending.
No complete negative-timeout semantics, original game frame, audio playback,
menu or gameplay is claimed.

The next integrated run exercises a concurrent resource-loader path and stops at
the existing 16MiB per-call I/O budget before a movie frame. Request details and
any larger-read implementation remain the next investigation; no inferred read
size or successful original frame is claimed. The fresh L8 read-only behavior is
verified by the owned PE, while original Bink's subsequent complete draw is still
pending.

## Original frame-loop boundary and complete large-read budget

The integrated original game now repeatedly executes its BeginScene, Clear,
DrawPrimitive, EndScene and swap-chain Present calls. Clear/Present themselves
succeed, but the movie's triangle-strip DrawPrimitive returns D3DERR_NOTAVAILABLE;
subrectangular viewport requests are also rejected by the current profile. This
is original frame-loop execution, not a successfully rendered movie/game image.
Those graphics profiles are the next rendering work. An actual concurrent loader
request asks for 75,075,791 bytes; no shorter successful read is substituted.

The existing checked synchronous I/O staging budget is now 128MiB per call. Full
guest range/count/output checks remain enforced and exactly the actual transferred
bytes are copied. Larger requests still stop explicitly with request diagnostics.
The owned 14-word PE passes a complete 80MiB+3 read, a digest over every byte,
untouched pre/post/EOF tail guards and a subsequent real EOF read. Only generated
owned data enters that test; game data remains local. PE Clippy passes; a native
identical-PE comparison is added and is pending. This is a larger bounded read
profile, not the complete DWORD-sized asynchronous I/O surface.

CI 42617007511 on b2bbec6 passes all three D3D9 jobs. All 2,118 L8 words and both
complete original-shader frames agree byte-for-byte on physical M4, hosted Metal
and native Windows with identical source/PE identities. The fresh READONLY lock
also succeeds natively; initial allocation bytes are deliberately not promised or
compared. CI 42617007466 passes both startup/API jobs. Physical M4 agrees with the
identical native 66-word pre-Winsock Win32/event-set PE and 39-word admitted select
PE. The preserved negative-timeout discrepancy remains an explicit exclusion.
These are finite owned observations, not native DAO frame or game equivalence.

## Original DAO intro rendering through its own binary shaders

The integrated original EXE now renders an identifiable original Dragon Age
intro frame, using the original Bink decoder, original full-size L8 YUV planes
and original legacy pixel shader. The actual 1710x1107 Present readback is
captured privately; no game image, shader or inventory enters the repository.
This is physical M4/Rosetta execution with the dirty source identities retained
by the private driver, not a native-Windows full-frame comparison. The next movie
opens, then low guest-address mapping stops explicitly. No main menu, gameplay
or audio playback is claimed.

The reached drawing profile adds triangle strips without dropping triangles or
changing winding (including degenerates), subviewport state/clipping and a
POSITIONT fixed vertex adapter. POSITIONT bypasses the bound vertex shader while
preserving the application's observable shader binding and constants. The
adapter reconstructs clip W from RHW and links actual COLOR/TEXCOORD semantics
to the unchanged following pixel shader. It is generated with open Naga, not a
replacement movie shader. The [Microsoft declaration rules](https://learn.microsoft.com/en-us/windows/win32/direct3d9/mapping-fvf-codes-to-a-directx-9-declaration)
require bypassing vertex processing for POSITIONT. Nondefault POSITIONT depth
ranges, fixed untextured subviewports, subviewport Clear and stencil Clear remain
explicitly unavailable; unsupported varyings are not invented.

This supersedes the earlier fresh-managed-L8 read-only upload description:
[new managed textures are initially dirty](https://learn.microsoft.com/en-us/windows/win32/direct3d9/texture-dirty-regions).
The first L8 read-only unlock now uploads its actual existing owned CPU bytes if
no prior GPU copy exists. It does not promise Windows initial allocation values,
change application bytes or suppress the backend's uninitialized sampling guard.
Later read-only unlocks do not upload. Original Bink uses this previous-buffer
initialization path.

The physical GPU test exercises nonuniform RHW, original owned pixel bytecode,
a four-color 2x2 texture, full/subviewport clipping and invalid viewport
rejection without output changes. The owned 2,125-word PE exercises the same
render path, POSITIONT with a still-bound VS, Get/SetViewport guards and failure
preservation, shader/COM retention, two complete 32x32 frames and guest stack.
It passes on physical M4; identical-PE native and hosted comparisons are added
and pending. Existing 200 PE units and backend units passed; targeted GPU and
observer tests pass. The earlier L8 comparison remains finite known-byte data,
not original DAO frame equivalence.

CI 42618856909 passes all three D3D9 jobs on 775b18a; startup/API comparison
42618857208 and conformance 42618857330 also pass. The separate broad PE workflow
42618857300 still has earlier native discrepancy failures; no all-CI-green claim
is made.

## Bounded PE32 low-address hole retry

The initial mmap path could choose memory above the non-LAA guest boundary and
immediately stop, without searching reusable holes below it. A fallback now
queries actual host mappings and attempts non-overwriting anonymous mmap hints
inside the unchanged admitted interval. Every resulting extent and alignment is
checked before guest use; rejected mappings are released. MAP_FIXED is never
used. Errors, actual exhausted address space and bounded retry exhaustion remain
explicit failures. GetSystemInfo's original ordinary/LAA ranges are unchanged.

The physical x86_64/Rosetta host probe allocates three adjacent low regions,
releases the middle one, finds and reuses that exact hole, checks neighboring
sentinels survive, checks occupied-range failure leaves output untouched,
checks real zero-fill and invalid bounds/alignment/size, then protects/unmaps
those real mappings. It passes with the SDK C compiler; 200 PE units and PE
Clippy pass. The probe is added to hosted macOS CI. This is host allocation
implementation evidence, not another native Windows API observation.

An attempted direct mach_vm_allocate(FIXED without OVERWRITE) returned
KERN_INVALID_ADDRESS for an otherwise free low-address region under this
Rosetta host; the retained implementation instead uses the working MAP_32BIT
mmap path with actual extent checks. No fixed overwrite or fabricated allocation
success is used. Integrated original-game retry is pending.

## Native oversized-viewport discrepancy and presentation lifetime repair

D3D9 CI 42622475396 on fb66f00 passes hosted Metal and software Vulkan, but its
new native POSITIONT witness fails. Native Windows accepts the deliberately
oversized x=31,width=2 viewport on a 32-wide target and changes the stored state,
instead of the witness's expected INVALIDCALL/preservation. Drawing afterward
therefore produces blue frames. The complete original 2,125-word raw observation
is preserved unchanged in pe/reference/pe32-positiont-viewport-overflow-windows-v1.json
(SHA-256 ee85f642b8c3aa5b4db8ea287ea7e49a536871ec0c96a6b20cf065639efcd6c1).
No rendering equality is inferred from that failed record. The original source
and expectation remain in history.

The admitted POSITIONT witness now makes only valid full/subviewport requests
and checks their exact returned state and sentinels; all other original
shader/binding/RHW/strip/full-frame/stack checks remain. It has 2,121 words and
passes physically; new native comparison is pending. Oversized PE viewport
requests now stop explicitly for missing clipping emulation, rather than claim
Windows rejected them. Direct backend extent rejection is still a host constraint,
not evidence of Windows INVALIDCALL semantics. This supersedes the previous
invalid-viewport native expectation.

An actual host vmmap during original intro rendering showed 264 retained CG
images occupying 1.8GiB, alongside substantial bitmap/CoreAnimation allocations.
The standalone 160-frame 640x480 presentation probe measured 596,156,416 bytes of
resident growth with the original lifetime. Nested pools scoped to each native
Present and event-pump operation reduce the same probe's observed growth to
3,244,032 bytes. The current image is retained by its real window/view; content,
frame count and rendering quality are unchanged. The probe rejects growth of
96MiB or more after warm-up and is added to hosted macOS CI. This is host resource
lifetime evidence, not a Windows frame-equivalence claim.

An integrated run with the bounded allocator passes the earlier next-movie
boundary and reaches original d3dx9_36.dll calling RegOpenKeyA. That run did not
need the fallback, so it is not additional proof of a successful integrated
fallback allocation. The host gap/reuse probe remains the direct fallback
implementation evidence. The exact original D3DX request addresses the optional
HKLM Software\\Microsoft\\Direct3D configuration; the private macOS registry
profile now explicitly declares that uninstalled configuration absent.

Legacy RegOpenKeyA now supports authoritative absent scoped keys and null/empty
subkeys of the two predefined backed hives. It writes the real resulting handle
or NULL, preserves LastError and uses its full three-argument stdcall ABI.
Unknown registry scopes and existing-subkey default security-mask semantics
remain explicitly unsupported; no default write permission is fabricated. The
owned 13-word PE checks absence, both same-hive forms, output guards, LastError
and guest stack. It passes physical M4; native comparison is added and pending.
201 PE units and Clippy pass. The next integrated retry is pending.

## Original D3DX instruction discovery

The next integrated run passes both original D3DX RegOpenKeyA requests and stops
at IsProcessorFeaturePresent(7), the optional 3DNow facility (not SSE2). The
reached instruction-discovery family now reads actual CPUID: MMX and SSE/SSE2
from leaf 1, 3DNow from extended leaf 0x80000001 only when supported. The admitted
x86-64 host supplies OS XMM state support. Unknown feature meanings still stop
explicitly; an absent extended leaf means that instruction facility is absent,
not that unknown API operations return zero. See the [Microsoft feature identifiers](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-isprocessorfeaturepresent).

The owned 13-word PE compares each reported feature to its independently executed
CPUID, preserves LastError and guest stack, and executes an XMM operation when
SSE2 is actually reported. It passes on physical M4; separate bit-selection units
cover missing extended leaves, basic/extended bit independence and unsupported
identifiers. Native identical-PE comparison is added and pending. The old literal
map4 startup PE now legitimately returns from its MMX query and emits its second
marker; its source/PE identities are unchanged. Its former unsupported boundary
remains historical, with the other locale/flag/character exclusions preserved.

The 80MiB owned read comparison now also uses the literal CI-produced PE on the
physical M4: source, PE and generated-data identities and all 14 output words
agree with native Windows from 42618857208. The independently compiled local PE
has a different identity and was not used to claim identical-binary agreement.

## Original D3DX transform-state frontier

With instruction discovery implemented, original DAO reaches
IDirect3DDevice9::SetTransform(D3DTS_PROJECTION). Set/GetTransform now retain the
actual 16 DWORD matrix bits per valid world/view/projection/texture state, with
independent identity defaults. MultiplyTransform premultiplies the supplied
matrix, as specified by the [Microsoft contract](https://learn.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-multiplytransform).
Nonfinite multiplication and unsupported state identifiers fail explicitly.
Set/Get storage itself does not sanitize matrix bits. This state does not alter
programmable shaders or POSITIONT draws; unsupported fixed-function drawing
paths still fail instead of silently ignoring a required transformation.

Two units check independent/default states, signed-zero/NaN storage, invalid
state atomicity, finite-only multiplication and noncommuting scale/translation
order. The owned 2,100-word PE checks five identity defaults, actual set/get,
premultiplication and output guards, followed by the original two programmable
shader frames to detect accidental fixed-state influence. It passes on physical
M4; native and hosted comparison are added and pending. All 204 PE units and
Clippy pass. Integrated retry is pending.

Startup/API CI 42623855118 on 373dd3b passes the Rosetta producer. Its native
legacy-registry step does not execute: the new artifact was uploaded but omitted
from the explicit native download list. There is no native semantic result from
that step. The registry and instruction-feature artifact transfers are now
included; this plumbing correction does not change an observation or comparator.

## Preserved POSITIONT texture-threshold observation

D3D9 CI 42623855024 on 373dd3b passes Metal production and software Vulkan.
Native Windows passes every admitted POSITIONT call/guard/stack assertion, but
the exact full-frame comparator still fails on three of 2,121 words. Native
Windows and hosted Metal differ at pixel (8,22) in both frames (blue versus
red), and frame 1 pixel (20,19) (white versus green). Source/PE identities agree. The complete native report is preserved unchanged in
pe/reference/pe32-positiont-rhw-threshold-windows-v1.json (SHA-256
 d74739dd0e996a9ed2a006b9351cd1a79090dae0691ca5557fef40b649be803a).

Independent exact screen-space arithmetic for those original owned triangles
gives perspective V exactly 1/2, the 2x2 point texture's color boundary. The
physical M4's separately compiled witness differs only at (20,19), but that
separate compilation is not counted as identical-PE evidence. Floating
interpolation/clipping precision is a hypothesis for the difference, not a
measured internal Windows implementation. The exact comparator is retained and
continues to fail; no tolerance, omitted pixel or changed shader/vertex fixture
is used to make it pass. This remains a known finite rendering discrepancy,
separate from original DAO intro rendering and unmeasured full game parity.

## Original renderer full-surface StretchRect

Original DAO passes the projection/view transform calls and requests a full-size
backbuffer-to-render-texture StretchRect with null rectangles and filter NONE.
That equal-size, single-sample BGRA8 attachment path now copies actual content,
without changing render bindings, viewport or shader state. A8-to-A8 uses an
actual GPU image copy. X8 source/destination copies use actual readback/upload
and force only the logically opaque alpha channel; RGB bytes are preserved.
Partial rectangles, scaling, filters, depth/system-memory surfaces, self copies
and in-scene StretchRect remain outside this implemented profile.

The physical GPU test checks uninitialized-source failure, unchanged outputs,
all copied bytes, alpha preservation/opacity, resource alias lifetime and
size/self-copy exclusions. The owned 2,100-word PE checks three actual surface
copy directions, all 1,024 pixels for each phase, A8 alpha and X8 RGB, then both
unchanged programmable shader frames and stack. It passes physical M4; native
and hosted comparisons are added and pending. Backend and PE Clippy pass.
Integrated original-game retry is pending.

## Color StretchRect within original scenes

The first integrated StretchRect retry reveals that DAO calls this color copy
inside its BeginScene/EndScene pair. The initial implementation rejected that
case; the following original postprocess draw then stopped on an uninitialized
texture. The [documented in-scene restriction](https://learn.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-stretchrect)
is under the additional restrictions for depth/stencil copies, not ordinary
color copies. Color attachment copies now execute within the scene too. The
owned three-direction copy witness wraps each copy in actual Begin/EndScene and
passes physically. Depth/stencil copying remains unimplemented. This supersedes
the earlier blanket in-scene exclusion. New native comparison remains pending.

Startup/API CI 42625398706 on 91d0016 passes both jobs, including the corrected
artifact transfers. Both 13-word registry and instruction-feature PEs have now
also run as the literal native-CI-produced binaries on physical M4: source/PE
identities and every output word agree with native Windows. These finite API
observations do not resolve the separately retained native texture-threshold
rendering differences.

## Original title/UI rendering with backed stencil planes

Original DAO now selects D24S8 in its own CreateDevice request, creates its
requested separate 256x256 D24S8 surface, renders through original indexed and
postprocess draws, and sustains a visible Dragon Age: Origins window. A private
WindowServer capture shows original title artwork and UI shapes. Text is still
missing; full menu behavior and gameplay are not demonstrated. The original
renderer repeatedly requests unsupported A8 textures, with an earlier A4R4G4B4
request too; those real format requests are the next investigation. The validation
process was stopped deliberately after capturing this sustained rendering; this
is not a clean original-game exit or a game-compatibility claim.

D24S8 now uses a real wgpu Depth24PlusStencil8 attachment; D16 remains a separate
format. Depth and stencil aspect initialization are checked independently.
Stencil clear, reference/masks, comparison, all eight update operations,
stencil/depth-failure paths and separate faces affect actual GPU state. Disabled
Z testing does not suppress enabled stencil. Clear validates both aspects before
mutation and preserves the other aspect. Capability reporting advertises the
eight implemented operations, retaining the other prior exclusions. Original
shader bodies and geometry remain unchanged.

The physical GPU test executes all eight operations, clamp/wrap boundaries,
replace on each failure path, masked writes, aspect initialization and independent
front/back faces. All existing ignored GPU regressions, 19 backend units, 204 PE
units, observer tests and backend/PE Clippy pass. The new owned 2,100-word PE
checks real D24S8 creation/binding/descriptors and actual replace/equal/not-equal
masks over both complete shader frames; it passes physically. The prior D16 PE
also passes. Hosted and native comparison are added and pending. The
[format specification](https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3dformat)
allows drivers to use additional depth precision while fixing stencil width;
no bit-for-bit native DAO depth/frame agreement is inferred from these finite
checks.

## Original DAO main menu and font formats

The original 32-bit EXE now reaches and visibly renders its real main menu on
physical M4 under Rosetta: title art, buttons and text are present. The same
private capture before A8 support showed the art/shapes without text; after the
format implementation the menu labels render. This is an observed original-game
menu milestone. Menu interaction, character creation, gameplay, saving and audio
playback remain unverified. The initial validation session ended at its explicit
600-second budget, not by a clean game exit. Captures, source identities and the
complete game-derived logs stay local.

The actual A8 and A4R4G4B4 requests now have backed one-/two-byte CPU mip layouts
and lossless GPU uploads. A8's missing RGB components are zero, the documented
[format exception](https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3dformat);
its alpha byte is preserved. A4R4G4B4 expands each nibble to the corresponding
8-bit channel without modifying guest bytes. Uncompressed subrectangle locks
return a checked offset with full row pitch; actual CPU updates are uploaded,
preserving the other bytes. Compressed partial locks and A8/L8 sRGB remain
explicitly outside the implemented profile.

Units check all alpha bytes, all 65,536 four-bit-channel words, mip pitches and
bounds/overflow. A physical GPU test samples every A8 byte and every A4R4G4B4
word through unchanged owned legacy bytecode and verifies full RGBA output.
All existing GPU regressions, 20 backend units, 205 PE units and backend/PE
Clippy pass. The owned 2,100-word PE checks both format descriptors, full and
partial updates/read-only coherence, two complete RGBA frames (including alpha),
COM lifetime and stack. It passes physically; native/hosted comparisons are added
and pending. The observer tests explicitly reject alpha-only mismatches.

D3D9 CI 42630692048 on d472f28 passes Metal and software Vulkan. The identical
2,100-word stencil PE also passes native Windows with exact agreement to hosted
Metal. The overall native job still fails on the retained POSITIONT threshold
pixels and a new StretchRect pixel-value assertion (stage 8). The latter raw
report is retained unchanged in pe/reference/pe32-stretch-color-copy-windows-v1.json;
its first failing phase/value were not included by that original witness.
The next witness now records phase, actual pixel, expected pixel and mask on
failure, without changing its acceptance criteria. No native color-copy equality
or all-CI-green claim is made.

## DAO interactive input: boot-clock correction

Quartz delivered host-injected keyboard and mouse events to the acquired real
event taps on physical M4/Rosetta, but menu activation was not established by
those initial attempts. The trace exposed a real clock-domain error: event
timestamps used uptime excluding sleep, while GetTickCount/timeGetTime used
the continuous boot clock. The measured difference on this laptop exceeded two
hours. Microsoft's [DIDEVICEOBJECTDATA contract](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/ee416611(v=vs.85))
requires the same clock as GetTickCount/timeGetTime.

Input capture now maps the original Quartz timestamp through a fresh observed
absolute/continuous clock pair. It preserves delivery age, includes elapsed
sleep, scales in widened arithmetic and wraps only at the final DWORD. Invalid
clock samples/future timestamps lose acquisition explicitly. Unit cases cover
sleep-offset changes, delayed events, non-unit scaling, overflow and wrap; a
physical clock check brackets the mapped timestamp by boot ticks. All 207 PE
units and PE Clippy pass. Setting ALKYHOL_PE32_TRACE_INPUT enables acquisition,
loss and at most 256 event diagnostics per acquired source; it does not inject
input or modify guest state. Original game interaction is being retested;
neither these synthetic units nor host-injected events establish gameplay or
native Windows input equivalence. Native focus-change messages remain a
separate unimplemented path.

D3D9 CI [42635100920](https://github.com/daeken/Alkyhol/actions/runs/42635100920)
on 443a41e passes Metal and software Vulkan; its native A8/A4R4G4B4 full-RGBA,
subrectangle, stencil and transform comparisons pass. The overall native job
still fails the preserved POSITIONT and StretchRect assertions. No acceptance
criteria or frozen observations were changed for this clock correction.

## DAO native mouse messages reach the original window procedure

AppKit client mouse move/drag and button events now enter the owned Win32 message
queue with observed client/screen coordinates and timestamps. PeekMessage filters,
PM_NOREMOVE and owner-thread DispatchMessage retain their normal paths; no guest
callback runs inside AppKit. Modifier/button transitions and registered
CS_DBLCLKS handling are explicit. Nonclient handling and capture remain outside
this first milestone. Queue overflow fails rather than dropping input silently.

On the physical M4, the owned window probe's optional `--mouse` posts a real move,
down and up through WindowServer, then observes all three through AppKit: exact
75,90 client coordinates, actual global coordinates, button masks and nonzero
timestamps agree. Three targeted units cover signed packing, button/modifier
transitions, invalid coordinates and class-dependent double clicks. The other
209 PE units passed before the final message-packing unit was added; the three
mouse units and PE Clippy pass afterward. This is host input-routing evidence,
not an identical-input native Windows message-order comparison.

In the original DAO run, an actual host click now dispatches WM_LBUTTONDOWN to
the original game procedure and stops honestly at its SetCapture import. The
menu has not yet activated New Game: native mouse capture/release is the next
observed blocker. No original instruction or UI state was patched to reach it;
captures and game-derived disassembly stay local.

## Original DAO accepts New Game and enters scene loading

The original main menu now responds to host mouse input. On the physical M4,
an actual move/down/up sequence passed through WindowServer/AppKit, the owned
Win32 queue, DAO's original window procedure and its SetCapture/ReleaseCapture
calls. DAO left the menu, opened its original loading UI and uploaded further
original textures. It then stopped explicitly at IDirect3DDevice9::CreateCubeTexture
(managed A8R8G8B8, edge 4, one level). Character creation/gameplay and full-scene
rendering are not yet established. The EXE, assets and original instruction
stream were not changed; all game-derived traces/captures remain private.

Foreground owned-window mouse capture now routes real AppKit drag/up events
outside the client to the selected Win32 window. It uses the existing native
WindowServer drag stream and does not intercept another application's clicks.
Release, GetCapture and WM_CAPTURECHANGED update actual routing and owner-thread
state; hiding/destroying/losing the native key window invalidates capture.
Background and foreign-thread capture are explicitly outside this profile.
The physical owned-window `--capture` probe observes correct move/up delivery
at client (-40,-30), release, repeated acquisition and invalid native-window
rejection. Acquisition does not succeed merely by storing a guest handle.

The new owned 19-word PE passes physically: initial/same/repeated acquisition,
queries, both capture-change callbacks, release, teardown and stdcall stack.
It pumps real activation events before acquisition; the earlier immediate
ShowWindow/acquire attempt correctly failed the foreground prerequisite.
All 210 preceding PE units and the additional API-routing unit pass, as does PE
Clippy. Three observer tests reject every failed assertion, malformed/truncated
report and extra output. Existing CI now transfers this literal PE and exact
source identities for native Windows comparison; that result is pending.

## DAO cube textures and the next scene-loading boundary

The original New Game path now passes CreateCubeTexture and its original face
uploads. The next observed stop is the original D3DX helper's system msvcrt
setlocale(LC_NUMERIC,NULL) query. Character creation, a shaded 3D game scene,
gameplay and audio remain unverified. No original game shader, mesh or asset was
replaced to reach this point; private source snapshots and traces identify each
experimental run without committing game-derived data.

SampledTexture now owns either one 2D layer or six actual cube layers; the
Texture2D source alias preserves existing users. Cube views bind as Cube to the
translated original shader. Face/mip uploads preserve independent CPU/GPU data,
format/color-space behavior and initialization guards. Every sampled face/mip
must have been uploaded; an absent one fails explicitly. Cube render targets,
automatic mip generation and general resource methods remain unsupported.

The PE interface supplies the distinct cube COM identity/type, 24-byte LockRect
and 12-byte UnlockRect gates, checked face/mip addressing, cached surface aliases,
GetContainer/GetDevice, reference retention while bound and raw lock coherence.
The 64 MiB CPU shadow budget is checked across all faces before allocation.
Independent Windows SDK vtable-offset assertions and the existing CI now cover
the cube ABI; capability flags are not expanded into an unsupported full profile.

Physical M4/Rosetta GPU tests pass all six direction centers across three mip
levels, including full target RGBA, distinct colors/alpha, incomplete-upload
rejection, wrong shape/face/mip/byte extents and invalid allocations. This tests
the owned original SM2 cube sampler through the open compiler pipeline, not a
substitute fixed shader. All prior ignored physical GPU regressions and backend
Clippy pass. Source/bytecode/SPIR-V/reflection regeneration agrees exactly with
the pinned open compiler; new owned shader provenance contains every hash.

The separate identical-PE cube resource witness passes physically: all 18 face
mips, descriptors, raw writes/reads, six surface aliases and containers, invalid
face/mip failure, parent and bound lifetime. Its independent native Windows
comparison is added and pending; it is not a native cube-sampling frame oracle.
All 211 preceding PE units and the additional cube address/gate unit pass; PE
Clippy passes. Thirteen D3D9 observer tests reject failed resource stages and
malformed output. Existing POSITIONT and StretchRect native discrepancies remain
unchanged and are not covered up by this milestone.

## D3DX initial C locale and real string copies

The original D3DX helper now passes its system-CRT numeric-locale query and
immediately reaches `_strdup` of that returned value. The initial C locale is
backed by owned guest string storage. Queries and selecting C succeed for all
six documented categories; unimplemented locale changes return NULL without
changing the current profile. Invalid categories still stop explicitly because
their invalid-parameter handler path is unimplemented. This does not change
Win32 ACP/NLS or the host process locale.

`_strdup` now scans checked guest bytes through the first terminator, allocates
real writable storage in the existing CRT heap and copies every byte including
the terminator. The result is independently mutable and belongs to CRT free.
NULL/unterminated/unmapped inputs and the 1 MiB traversal budget remain explicit
unsupported boundaries, as does the pre-existing exhausted-malloc policy.

All 214 PE units and PE Clippy pass. The owned 22-word literal PE passes
physically: all-category query/selection, rejected locale preservation,
independent C-string copies, high bytes, empty strings, free and cdecl stack.
Three observer tests reject every altered assertion and malformed report.
The identical native Windows comparison is added to existing CI and pending.
The next original-game run is still needed to establish progress beyond the
string-copy boundary; these finite tests are not character creation/gameplay.

On f3fa9a1, hosted Metal's complete D3D9 producer passed, including the new cube
GPU/resource tests. GitHub then reported internal server errors before creating
the remaining jobs; the input/PE workflows likewise initially created no jobs.
The latter have been requested again. These infrastructure failures are not
counted as native observations or passing comparisons; the prior retained native
rendering differences remain unchanged.

## Original loading passes CRT helpers; texture mip mappings are packed

The clean 72954a1 run accepts New Game and passes the original D3DX setlocale and
`_strdup` calls. It continues loading original scene textures until the existing
512 low-storage mapping budget is exhausted. This is an allocator boundary, not
a successful scene/gameplay milestone. Captures and the complete original trace
remain local; no game data or shader was omitted to reach it.

Previously each texture face/mip consumed a separate low mapping, despite even
one tiny level reserving a full host allocation unit. A texture now reserves one
checked mapping containing all its face/mip shadows. Subresources have disjoint
16-byte-aligned ranges, original pitch/length/dimensions and independent lock and
initialization state. Surface aliases point at the same real subresource bytes.
The 512-map and 64 MiB shadow limits remain intact; no mip or face is dropped.
All padding and total sizes are checked before allocating GPU/CPU storage.

All 215 PE units and PE Clippy pass. New units test face/mip disjointness, raw
block extents, byte formats, alignment, overflow and allocation-budget rejection.
The existing owned cube resource, full-RGBA font/subrectangle and BC3 mip PE/GPU
witnesses all pass physically with packed storage. Their existing assertions
and original bytecode are unchanged. The next clean original-game run is needed
to establish whether packing clears the observed allocation boundary.

## Character-generation resources expose COM-header mapping overhead

The clean packed-mip run passes the earlier stopped allocation and opens
additional original character-generation resources, then reaches the same
512-map guard at a later texture allocation. It still has no verified character
creation frame or gameplay. The remaining tiny COM headers each consumed an
entire low mapping, independently of their actual descriptor storage.

COM headers now occupy stable 16-byte slots in real 64 KiB writable arenas,
separate from immutable vtables/executable gates. Header slots never move or get
reused during a process lifetime, including while an object survives through an
internal binding. Crossing an arena boundary allocates new checked storage.
The 512-map and 4,096-live-object guards remain unchanged; no success is returned
without an owned object and valid vtable. Resource payloads, references, lock
and shader behavior are unchanged.

A new owned literal PE passes physically: two batches each keep 4,094 requested
declarations live, query every descriptor with guards, release all external
references, recover the retained binding/parent and cross an arena boundary.
Native drivers may cache equivalent declarations; the test makes no cross-object
pointer-uniqueness assumption. The existing cube resource and full-RGBA font
PE/GPU witnesses also pass with pooled headers. All 215 PE units, PE Clippy and
14 D3D9 observer tests pass. Existing CI now transfers the allocation witness for
native Windows comparison, which remains pending during GitHub's server errors.
The next original-game run is still needed to identify the next loading boundary.

## Allocation stops clear; original half-precision vertex layouts

The clean e605635 replay passes both earlier allocation stops and continues
through original character-generation resources. It then rejects FLOAT16_4 and
FLOAT16_2 declarations before an original null-pointer cleanup fault. Inspection
of original instruction bytes shows an attempted interface Release through that
null value. A causal connection to the rejected declarations is a hypothesis
until the next replay; the original cleanup code is not patched or skipped.

The two [documented declaration types](https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3ddecltype)
now have checked 4-/8-byte layouts and binary16-to-binary32 fetch. FLOAT16_2
supplies the specified zero/one defaults; four-component values widen all four
lanes. No original vertex, index or shader data is modified. Exhaustive units
compare every finite encoding with an independent f64 arithmetic construction,
check infinities/NaN classification, signed zero, subnormals, vector defaults
and malformed extents. This does not establish native NaN-payload equivalence.

All 21 backend units and backend Clippy pass. A new owned literal PE uses these
actual declaration types with the unchanged owned shader bodies, checks COM and
stack behavior and renders two complete RGB frames physically. The exact native
Windows comparator and independent SDK enum checks are added to existing CI and
pending. Fourteen D3D9 observer tests pass; existing acceptance rules for earlier
witnesses and retained failed native observations remain unchanged. The next
original run must establish whether the cleanup boundary is cleared.

## DAO loading reaches signed SHORT4 declarations

The clean 5f5449c replay passes FLOAT16_2/FLOAT16_4 validation. The next rejected
original elements are SHORT4 (BLENDINDICES) at offsets 44, 52 and 56. Original
cleanup still faults through a null interface at the previously recorded site;
clearing declaration rejection has not yet established a character creation
frame. The game code and asset bytes remain unchanged and local.

[SHORT4](https://learn.microsoft.com/en-us/windows/win32/direct3d9/d3ddecltype)
now fetches four signed, unnormalized 16-bit components. Exhaustive units check
all 65,536 signed encodings and a mixed vector checks component order. Partial
or oversized elements still fail. All 22 backend units, backend Clippy and 14
D3D9 observer tests pass on this host. A new owned literal PE passes physically
on the M4, including two complete RGB frames using SHORT4 positions and the
unchanged shader bodies. Existing CI transfers that identical PE for native
comparison and independently checks the SDK enum; those results remain pending.
The next original-game replay must establish whether its cleanup fault clears.

### Native comparisons recover after GitHub runner failures

CI [42678993014](https://github.com/daeken/Alkyhol/actions/runs/42678993014)
on 5f5449c executes all three D3D9 jobs: Metal producer and software Vulkan pass.
Native Windows passes the identical cube resource/surface-alias, COM arena and
half-precision full-frame comparisons. The native D3D9 job remains failed on the
previously retained StretchRect alpha and POSITIONT pixel differences; those
assertions were not weakened. This establishes these finite native contracts,
not original full-scene or game parity. The DirectInput workflow
[42678992722](https://github.com/daeken/Alkyhol/actions/runs/42678992722) and
conformance [42678993303](https://github.com/daeken/Alkyhol/actions/runs/42678993303)
also pass on that revision.

## DAO character-resource declarations use separate vertex streams

The clean 7786a27 replay passes the SHORT4 elements. Its next rejected element
is a FLOAT4 POSITION on stream 1; original cleanup still encounters the null
interface. That observation narrows the earlier format hypothesis: accepting
SHORT4 alone does not clear the original loading fault. No original instructions
or assets were changed to bypass it.

Device bindings now independently own streams 0–15, including offsets, strides,
retained buffer references, query references and teardown. Declarations retain
their actual stream numbers; both normal and indexed draws fetch each attribute
from its own checked source. Each source must be unlocked, in bounds and
initialized for the referenced vertices. Streams retain the existing 64 MiB
fetch budget and 2,048-byte stride bound. Zero/divided/instanced frequency modes
remain explicitly unsupported; frequency 1 is real per-vertex fetch. Fixed
transformed-color rendering remains the existing bounded stream-0 path.

All 216 PE units, PE Clippy and 14 observer tests pass locally. The owned
`--streams` literal PE renders two complete RGB frames physically on the M4:
ordinary and indexed draws use stream 15 positions and separately padded stream
1 colors, nonzero source/element offsets and distinct strides, with stream 0
unbound. It queries all 16 binding/frequency slots, rejects slot 16 and exercises
buffer lifetimes after external release. Existing indexed and original Microsoft
Tutorial 02 fixed-function GPU witnesses pass without changing acceptance rules.
Existing CI now transfers the identical stream PE for native Windows comparison,
which remains pending. The next original replay is still required to determine
whether the observed loading cleanup fault clears.

Native Windows also passes the identical SHORT4 full-frame PE on 7786a27 in
[42680558107](https://github.com/daeken/Alkyhol/actions/runs/42680558107).
Metal and software Vulkan jobs pass; the native job still fails only the retained
StretchRect/POSITIONT checks. This does not establish full original DAO rendering.

## Separate streams advance DAO to further buffer allocations

The clean 63f3169 run accepts New Game and the stream-1 declaration. It proceeds
through further original character-resource vertex/index allocations without
encountering the prior null-interface fault before its next stop: the existing
512-mapping guard at CreateVertexBuffer. A character-creation frame and gameplay
remain unverified. Native Escape key events were tried during intro playback;
no original startup instruction or asset was omitted.

Vertex/index CPU shadows now use disjoint, 16-byte-aligned slots in stable 4 MiB
writable arenas. Larger buffers retain their checked dedicated mappings. Slots
never move or get reused during the process lifetime, including while a resource
is owned solely through a binding. The 512-map, 4,096-live-object and 64 MiB
per-buffer limits remain unchanged. This reduces mapping overhead without
claiming a resource-memory reclamation implementation.

All 217 PE units and PE Clippy pass physically on this host; 15 observer tests
pass. The new owned `--buffer-storage` PE passes physically: one batch each of
4,094 live vertex and index buffers checks guarded descriptors and every word
of unique buffer contents after later allocations, crosses arena boundaries,
and checks retained bindings, parent identity and raw bytes after all external
references are released. The separate-stream ordinary/indexed complete-frame
PE also passes with arena storage. CI transfers the identical allocation PE for
native comparison, pending. The next clean original replay determines whether
this clears the observed allocation stop; no original game files were committed.

## Buffer arenas reach an original scene draw; stencil initialization stops it

Clean d974dba passes the prior buffer allocation guard and original cleanup,
loads more character resources, and reaches a new indexed scene draw. It stops
because the active depth/stencil object's stencil aspect is not marked
initialized. The preceding original call clears color/depth (flags 3), not
stencil. No scene frame is yet verified. Existing first-unique logging cannot
reconstruct all repeated render-state transitions; a focused diagnostic now
records the actual failing stencil state and distinguishes absent stencil from
uninitialized stencil. Unsupported Clear regions also report their viewport and
target dimensions. These diagnostics change no draw/clear success criteria.

The new separate-stream native comparison on 63f3169 in CI 42682096185 fails
at its unbound-stream query assertion (stage 11), before either rendered frame.
This is distinct from the older POSITIONT/StretchRect failures. The exact native
report is frozen at `pe/reference/pe32-separate-streams-unbind-windows-v1.json`
(SHA-256 `42cd5cbd4db78bb5a654e5fd84c057cee4e3cb10920e6a78bc7be119786c26c5`). The report
establishes that the combined NULL/offset/stride assertion differs, but does not
identify which returned field differs. The witness now emits those raw fields
on this same failure without weakening its assertion. Native multi-stream
full-frame agreement remains unestablished until this contract is diagnosed.

## DAO stencil diagnostics identify depth-range clear rejection and fresh storage

Clean 9fc21bf confirms full-size color/stencil clears were rejected solely
because the viewport's MaxZ was zero. [Clear](https://learn.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-clear)
uses the viewport rectangle; its depth mapping interval does not change clear
coverage. Full-rectangle clears now accept nondefault depth ranges. Partial
regions/rectangle arrays remain explicit unsupported operations.

The later 146x146 scene draw binds D24S8, enables depth and stencil, compares
stencil ALWAYS and increments on pass; it has cleared color/depth but not the
new stencil aspect. PE32 depth/stencil allocation now explicitly clears newly
allocated GPU stencil storage to zero. This is a real queued GPU operation and
a documented Alkyhol initial-storage policy, not a fabricated initialized flag,
a removed stencil operation, or a claim about native initial stencil bytes.
Depth still requires its actual clear. Low-level backend uninitialized-aspect
guards and their existing GPU assertions remain intact. Native rendering that
depends on unspecified initial stencil contents remains unmeasured.

The existing stencil-mask PE still passes its complete frames, now with clears
under [0,0] and [0.25,0.75] viewport depth intervals. A new owned literal
`--fresh-stencil` PE passes two complete RGB frames physically: implicit then
explicit D24S8 surfaces, color/depth-only clears, ALWAYS stencil comparison and
increment-on-pass. Its RGB visibility is independent of the unspecified native
initial stencil byte; the test does not compare that byte or assert native
zero-initialization. The full backend stencil GPU operation/aspect test,
PE Clippy and 15 observer tests pass. Existing CI transfers the identical new PE
for native comparison, pending. The next clean DAO replay determines whether
scene rendering advances; no character-creation frame/gameplay is yet claimed.

The buffer-storage native comparison passes on d974dba in CI
[42683201703](https://github.com/daeken/Alkyhol/actions/runs/42683201703);
its Metal and Vulkan producer jobs pass. Native stream stage 11 and the retained
POSITIONT/StretchRect differences still fail, so the overall workflow is not green.
The 9fc21bf Metal producer is blocked by backend Clippy's redundant `.into()` on
the new diagnostic String; its native diagnostic did not run. The PE Clippy pass
does not lint dependency crates. This diagnostic-only lint is being corrected
separately from the rendering behavior and native evidence.

## New Game reaches the opening movie, then original float depth bias

Clean 4d99a60 passes the prior stencil stop and continues New Game into the
original opening movie. After a native Escape event, later original scene setup
requests D3DRS_DEPTHBIAS = 0x3727c5ac and stops. This is progress beyond the
loading/preview draw, not a verified character-creation scene or gameplay frame.

Constant and slope bias now retain their actual finite float32 state values.
A Naga fragment-entry adapter implements the
[documented float offset](https://learn.microsoft.com/en-us/windows/win32/direct3d9/depth-bias)
from fragment position and depth derivatives, after clipping, clamped to [0,1].
The original fragment body is called unchanged; all original color outputs are
retained and the biased fragment depth is returned separately. This avoids
rounding the normalized float constant into wgpu's integer hardware-bias units.
Both fixed transformed-color and translated original-shader paths use the same
adapter. Zero bias leaves the module untouched. Nonfinite values, explicit
shader depth output with nonzero bias, and explicit early-depth modes fail
rather than silently selecting different behavior. Fine native depth precision,
slope derivative/quantization boundaries and their full-scene effects remain
unmeasured; this is finite implementation evidence, not a native frame oracle.

All 25 backend units, 217 PE units, both crates' Clippy checks and 16 observer
units pass locally. IR units validate output interfaces, existing/new fragment
position input, no-op identity and explicit failure cases. Physical M4 tests
compare complete frames for positive/negative constant and slope bias on D16
and D24/stencil attachments, retaining every covered pixel. The owned `--bias`
PE uses unchanged shader bytecode and tests float state roundtrip, positive-bias
occlusion and negative-bias visibility in two complete RGB frames; it passes
physically. Independent SDK enum/capability checks and identical-PE native
comparison are added to existing CI and pending. The diagnostic String lint
from 9fc21bf is fixed without changing its failure semantics. A clean original
replay is required to identify the next scene boundary.

## DAO reaches occlusion query creation; precise GPU counters

Clean 0c6e568 passes the original nonzero depth-bias setting, executes further
indexed scene draws after the opening movie, and stops at CreateQuery type 9
(occlusion). This is not yet a verified character-creation frame or gameplay.
Native CI [42687444752](https://github.com/daeken/Alkyhol/actions/runs/42687444752)
on that revision passes the identical float-bias and fresh-stencil complete-frame
comparisons; Metal and software Vulkan jobs pass. The workflow remains failed
on the retained POSITIONT/StretchRect differences and separate-stream stage 11.
Its new stream diagnostic reports S_OK, NULL buffer, offset 0, stride 32 after
unbinding stream 0, rather than the test's assumed zero stride. That finite
report is frozen at `pe/reference/pe32-separate-streams-unbind-fields-windows-v1.json`
(SHA-256 `9626754adc62738eb982f5f4385045365b809e3cad93b6106160beaeb493e632`).
The original failed stream fixture/report is retained; its correction is separate.

Occlusion queries now have real PE32 COM identity, parent references and exact
stdcall slots, type/size queries, begin/end ranges and nonblocking GetData.
Draws resolve actual GPU counters asynchronously; overlapping scopes share each
measured draw and sum their own ranges. Pending reads return S_FALSE without
writing the caller's result. FLUSH submits/polls actual work; it does not invent
completion. Reuse drops old counters; dropped scopes are weakly tracked. Active
scopes are bounded at 128 and each range at 4,096 draws. Unissued data, unsupported
issue sequences/query types, readback/device errors and DWORD overflow fail
explicitly. Only the current single-sample profile is admitted, so samples equal
pixels. No boolean result is substituted for a sample count.

The first physical test caught upstream wgpu-hal's Metal boolean-query mode:
its two visible draws returned two rather than the independently read-back
pixel total. A pinned MIT/Apache-2.0 wgpu-hal 27.0.4 source vendor now provides an
opt-in precise-count profile: Metal Counting, Vulkan PRECISE with its required
device feature, and D3D12 OCCLUSION. Other query types and the unselected upstream
profile remain unchanged. The archive SHA-256 was measured and equals the
original Cargo checksum. Original licenses, upstream revision, a reviewable
patch and changed-file hashes are in `vendor/wgpu-hal/ALKYHOL.md` and its adjacent
manifest. The precise Metal profile is desktop macOS only; Vulkan adapters
without precise-query support fail device creation with an explicit diagnostic.
This is an Alkyhol profile, not a claim that WebGPU promises precise counts.
Host runtime identities now include vendored sources and Cargo manifests.

The physical M4 query test agrees with independent color readback for visible
samples and overlapping ranges, returns zero for depth-hidden geometry, and
checks reuse, empty ranges, invalid sequencing, cross-device ownership and
scope-budget cleanup. The existing full ignored GPU suite also passes. The new
owned literal `--occlusion` PE passes physically: original shader bodies and COM
stack/identity checks, overlapping counts 552/276, actual depth-hidden count 0,
empty reuse and two complete RGB frames. All 218 PE units, both crates' Clippy
and 17 observer tests pass. Independent SDK query enum/vtable checks and the
identical-PE native counter/frame comparator are added to existing CI; native
counts and precise Vulkan/D3D12 execution remain pending. The next clean original
replay must establish whether this clears DAO's observed query boundary.

### Native app packaging and activation target verification

The query-enabled DAO host reaches its main menu, but the unbundled diagnostic
process cannot be selected by the desktop-control service. A local app bundle
now wraps the identical compiled host executable; it does not alter the guest
EXE. Its first run exposes an AppKit activation setter rejection. Host setup now
reads the resulting activationPolicy and requires actual Regular policy (0),
rather than treating the setter acknowledgement alone as the target condition.
A non-Regular result still fails with the requested/resulting values. All 218 PE
units and PE Clippy pass. The candidate run physically opens the original splash
and is recognized as Alkyhol DAO by native accessibility control. Its captured
runtime is marked dirty before build and includes the exact source/binary hashes;
committing that identical source does not relabel the earlier capture as clean.
The local app bundle/launch records and original captures remain ignored local
data. The original-query scene replay is still in progress.

### Precise query native agreement; original focus-change capture boundary

CI [42691891879](https://github.com/daeken/Alkyhol/actions/runs/42691891879)
on cc666a3 passes the Metal producer, software Vulkan GPU suite and native
identical-PE occlusion comparison. All actual counters (552/276/0), COM assertions
and complete frame words agree with native Windows D3D9. Precise counting is
therefore physically observed on Metal and software Vulkan; the vendor's D3D12
counting branch still has no independent execution evidence. The overall native
job remains failed on separate-stream unbind metadata, StretchRect and POSITIONT.

The original query-enabled run reaches the menu. Native desktop control then
changes focus and the original game calls SetCapture while its window is not
key. The old foreground-only backend stops there, before exercising query
creation; no original query-game/scene success is yet claimed. This is distinct
from the finite owned query evidence above.

[Win32 SetCapture](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcapture)
also admits background capture, restricted to events over the visible owner.
The backend now retains that restricted scope and its real Win32 ownership/
callbacks. It routes only actual AppKit/WindowServer events; outside-client
routing is admitted solely for established foreground capture. A real upgrade
to foreground enables the existing drag/up stream; subsequent foreground loss,
hide/destroy or another-window down clears it. No other process's clicks are
intercepted and foreground/key state is never fabricated. Hidden/foreign-thread
capture and complete Win32 focus/activation notification parity remain outside
the implemented profile.

The new two-window owned PE passes physically: background acquisition, transfers
in both directions, GetCapture, three synchronous WM_CAPTURECHANGED callbacks,
release, destruction and stack checks. The existing foreground lifecycle PE
also passes unchanged assertions. All 219 PE units, PE Clippy and four capture
observer units pass. Existing CI now compares the identical background PE on
native Windows, pending. Native delivery of arbitrary background mouse sequences
is not established by these lifecycle assertions. The next original replay
checks whether focus transitions clear the observed startup barrier.

### Process-delivered DirectInput mouse events and original query progress

At clean 300c6c4, actual window mouse messages reach DAO but process-targeted
desktop-control clicks do not reach the session-scoped DirectInput mouse tap;
keyboard events already use a process tap and arrive. The mouse now uses the
same CGEventTapCreateForPid delivery point, once per event, with the existing
foreground, loss, bounded queue and timestamp checks intact. This removes the
global session mouse tap rather than combining duplicate sources.

The dirty candidate replay on 300c6c4 captures exact runtime/source hashes. A
real native click now produces both WM_LBUTTONDOWN/UP and DirectInput component
512 transitions with identical timestamps, selects original New Game and plays
the original opening movie. Native Escape skips it. The original EXE proceeds
through CreateQuery(type 9), then stops explicitly at device slot 84,
DrawIndexedPrimitiveUP. This establishes original query creation progress, not
a rendered character-creation frame or full query-result/game correctness.
The targeted input queue/overflow unit and PE Clippy pass on the Rosetta host.
Private traces and captures remain local; the original EXE is unchanged.

### Caller-memory D3D9 draws after original query creation

The original New Game replay described above reaches device slot 84 with a
triangle-list call, eight vertices, twelve primitives, INDEX32 and a 12-byte
stream-zero stride. DrawPrimitiveUP and DrawIndexedPrimitiveUP now feed the same
fetch/shader/depth/stencil/query renderer as buffer draws. Checked guest ranges,
finite triangle-list/strip bounds, INDEX16/32 decoding and nonzero MinVertexIndex
normalization are enforced. Declarations referencing other streams are rejected.
The caller's data is consumed before returning; stream zero is unbound afterward,
and indexed UP also unbinds the index buffer, releasing retained COM bindings.
Unsupported primitive types and out-of-profile layouts still fail explicitly.

The owned --user-draw PE executes on physical M4 through Rosetta: four independent
complete 32x32 frames cover plain list/strip and indexed list/strip, both index
widths, nonzero minimum index, shader constants, caller-storage overwrite directly
after return, retained stream-one state, index-binding differences and stack
balance. All 4167 output words pass. No earlier draw can conceal a missing UP
frame. The new bounds/format/strip/ABI unit, all 220 PE units, PE Clippy and 18
observer units pass. Independent SDK slot assertions and the identical literal
PE/four-frame Windows comparator are wired into existing CI; native results are
pending. Synthetic comparator units are acceptance-guard tests, not GPU evidence.

The documented binding and caller-storage contracts are from Microsoft's
[DrawPrimitiveUP](https://learn.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-drawprimitiveup)
and [DrawIndexedPrimitiveUP](https://learn.microsoft.com/en-us/windows/win32/api/d3d9/nf-d3d9-idirect3ddevice9-drawindexedprimitiveup)
references. Complete original DAO scene/frame and native shaded parity remain
unverified until the clean original replay passes this boundary.

### Original UP draw progress and fresh color-target storage

Clean 1983dbe passes original DAO query creation and no longer terminates at the
DrawIndexedPrimitiveUP dispatch slot. A later audit below finds that the original
fixed-function XYZ invocation returns INVALIDCALL; it is not a passing draw. DAO
continues despite that error and reaches a later render-texture pass and stops in the backend's combined
initialized-content/triangle-extent guard. Given the adapter's bounded nonempty
packed float4 stride and primitive normalization, uninitialized target storage
is the inferred remaining condition; that old combined error did not capture
its individual values. The error now records initialization, byte size, stride
and indexed state to distinguish any recurrence.

The PE render-texture allocation path now performs a real GPU clear to zero,
then returns the same shared render/sample allocation. This is an explicit
Alkyhol initial-storage policy, not a claim that Windows allocations start at
zero. Backend guards remain intact; no successful draw is fabricated. The
extended --user-draw witness (version 23) draws a complete fullscreen quad into
a fresh render texture with no guest Clear, reads every RGB word, then executes
the prior four independent UP frames. All 5192 words pass physically on M4; 18
comparator units and Clippy for both crates pass. The fullscreen frame overwrites
every pixel, so its pending identical-PE native comparison does not depend on
unspecified native allocation contents. Version-22's four-frame evidence remains
reproducible at 1983dbe. Original character creation/gameplay is still unverified.

### Original fresh-target progress and alpha-test boundary

Clean c77d399 passes the original fresh-render-texture guard and proceeds to
further scene/postprocess state. The next explicit stop is ALPHATESTENABLE=1;
preceding original state selects ALPHAFUNC=GREATEREQUAL (7), ALPHAREF=8. The trace
also records SetTexture(stage 0,pointer 3) returning INVALIDCALL; that invalid
pointer is not remapped or accepted. Its cause remains an independent unknown.
No completed character-creation scene/frame is established.

Alpha testing now wraps the original fragment body, compares COLOR0 alpha with
ALPHAREF/255 using the selected one of eight comparisons, and discards before
color/depth/stencil/query contributions. All original outputs, including an
existing depth output, remain intact. Disabled/ALWAYS preserves module identity.
Explicit early-depth-test mode, absent COLOR0 and invalid parameters fail. Both
fixed transformed-color and translated original pixel shaders use the adapter.
The NEVER case uses a structured discard and typed return because Metal's
discard_fragment is not noreturn; the initial direct-discard form failed actual
Metal pipeline compilation and is not retained as passing evidence.

Physical M4 executes 64 fixed/translated combinations across all comparisons and
zero, half-alpha/adjacent references and one-alpha boundaries. Every pixel and
actual occlusion count matches the expected surviving coverage. The owned
--alpha-test literal PE passes 2099 words, original bytecode, retained state,
stack and two complete RGB frames: half alpha rejects at reference 128 and
survives at 127 under GREATEREQUAL. Native identical-PE comparison is pending;
this is finite float-reference evidence, not a claim about all native precision
or game materials. All 27 backend units (two GPU units separately ignored),
both crates' Clippy and 19 observer units pass. Existing CI runs the new GPU test
and native comparator; original DAO replay with alpha testing is next.

### Alpha-enabled original replay and audit correction

Clean 0fd342b on physical M4 passes alpha-enable state and proceeds to shader
resource binding. The next fatal failure is an actual wgpu validation error:
texture binding 16 expects D3 while the retained texture view is D2. The preceding
original SetTexture(stage 0,pointer 3) still returns INVALIDCALL. No original
CreateVolumeTexture call appears in the retained trace. This is a resource-kind/
lifetime investigation, not evidence that a missing 3D allocation is the sole
cause; do not substitute a D2 texture or accept the invalid pointer.

Audit correction: the original occlusion-box DrawIndexedPrimitiveUP call disables
both shaders, binds an untransformed position declaration and uses stride 12. It
returns INVALIDCALL in our current programmable/POSITIONT adapter. Original DAO
ignores that draw failure and proceeds; this must not be described as successful
original occlusion geometry or query-count correctness. The finite owned UP
fixtures exercise programmable draws successfully, but the actual original XYZ
fixed-function transform path remains unsupported. Implement that real path and
its query/depth transforms before treating the query-box milestone as complete.
The later D3/D2 texture error is an additional observed blocker. Character
creation and gameplay remain unverified; no scene-frame parity is claimed.

CI [42698287554](https://github.com/daeken/Alkyhol/actions/runs/42698287554)
on 1983dbe passes hosted Metal, software Vulkan and the native identical-PE
version-22 UP comparison: all 4167 words and four full frames agree. The overall
native job still fails the earlier separate-stream unbind, StretchRect and
POSITIONT comparisons. CI 42699413184 on c77d399 stops its Metal producer on a
rustfmt difference in the added error diagnostic; no fresh-target native result
comes from that run. That formatting is corrected in 0fd342b. Fresh-target and
alpha native results remain pending; physical evidence is recorded above.

### Fixed XYZ visibility/query geometry

The original null-VS/null-PS occlusion-box path now has explicit generated
fixed-function stages. POSITION0 float3 is expanded with W=1, then transformed
by the retained WORLD, VIEW and PROJECTION matrices in D3DMATRIX row-vector
order. The ordinary fetch, clipping, viewport, cull, depth/stencil, bias and real
query renderer executes it. This finite path requires disabled color writes and
alpha test and no vertex blending; lighting/material/texture colors are not
substituted. Application shader bindings and constants stay independent. Other
fixed-function layouts/color draws still return an explicit unsupported failure.

The owned --xyz-query PE disables both application shaders, binds only an XYZ
declaration and submits INDEX32 box geometry with 12-byte source stride. It uses
noncommuting world/view transforms and projective W. Physical M4 returns actual
516/258/0 outer/inner/depth-hidden samples, empty reuse and all 2103 report words
pass. Shader/declaration/stream bindings are restored for the two existing full
frames. A separate actual-GPU test compares the generated transform's query
coverage against independently CPU-transformed clip positions rendered through
the original owned shader, checks depth rejection and every unchanged color byte.
The independent reference uses separate row-vector arithmetic, not the generated
shader code. The shader-interface unit, 221 PE units, 20 observer units and both
crates' Clippy pass. Native identical-PE XYZ comparison is wired into existing
CI and pending. The original DAO replay is next; finite owned geometry is not
yet evidence that the original box draws successfully.

### Original XYZ draw confirmed; backed volume textures

Clean cd599a8 on physical M4 submits the original XYZ/INDEX32 UP calls without
the prior INVALIDCALL, reaches actual IDirect3DQuery9::GetData, and then stops at
the same D3/D2 shader binding validation. This is original draw/query progression,
not native DAO query-count or complete scene-frame parity. The earlier local
trace shows original D3DX querying volume formats (resource type 4), all rejected,
before the later invalid texture pointer. That sequence motivates real volume
support but does not independently prove the pointer's origin.

The backend now allocates actual uncompressed BGRA8 3D textures, bounded to 512
per axis/64 MiB, and uploads all z slices of each independently shrunk mip. D3
views, W addressing and existing point/linear/mip sampling use that allocation.
No 2D view substitution is used. Uninitialized mips still fail sampling. The PE
profile implements managed/default Usage=0 A8R8G8B8/X8R8G8B8 volume textures,
GetLevelDesc/GetVolumeLevel/LockBox/UnlockBox and shared Volume9 aliases with
correct container/device retention. Partial boxes retain full row/slice pitches.
Read-only and NOSYSLOCK flags are bounded explicitly; compression, dynamic usage,
render-target volumes, dirty-update APIs and other formats remain unsupported.
Caps and CheckDeviceFormat now advertise only the backed finite volume profile.
Private initial CPU bytes are an allocation policy, not a native-value promise.

Physical M4 volume sampling passes distinct z slices, true linear z interpolation,
mip selection, uninitialized-mip rejection and invalid extents/uploads. The owned
--volume PE passes all 2100 words: all mips/voxel bytes, partial-box update, cached
volume identity, shared locks, GetContainer/GetDevice, bound lifetime, stack and
two complete frames sampling distinct red/green z slices with original owned
SM2 bytecode. Independent SDK layouts/IIDs and identical-PE native comparison are
added to existing CI; native/original DAO volume results remain pending. The
legacy HLSL, bytecode, linked vertex SPIR-V/reflection, provenance hashes and PE
include regenerate exactly using pinned open vkd3d. No game shader data is in
those fixtures. All 222 PE units, 21 observer units, both Clippy checks and the
unchanged six-face cube resource/lifetime regression pass.

CI 42700895627 on 0fd342b passes hosted Metal, software Vulkan, native alpha-test
and version-23 fresh-target/UP witnesses; the native job still fails earlier
separate-stream, StretchRect and POSITIONT differences. These finite results do
not imply game compatibility.

### Original volume population and L8 extension

Clean 5142eb4 reaches real original CreateVolumeTexture and writes/unlocks all
eight mips of its color volume. The next observed request is 512x512x4, one mip,
L8 (50), managed Usage=0. The BGRA-only constructor returns UNAVAILABLE, after
which original DAO faults on a null pointer at EXE RVA 0x474ab7. No failure is
reclassified as a passing scene.

Linear L8 volumes now retain one-byte CPU rows/slices and mip lengths; actual
GPU uploads expand all original voxel values into linear luminance RGB with
opaque alpha. No sRGB L8 sampling is accepted. The volume PE is extended to
version 27: 4168 words, original resource/alias checks, raw L8 mip checks and four
complete BGRA/L8 slice frames pass physically. Both volume GPU tests pass,
including L8 slices/interpolation and sRGB rejection. All 222 PE units, 21
observer units and both Clippy checks pass. Native extended-volume and original
L8 replay remain pending.

CI 42703229672 on cd599a8 passes hosted Metal, software Vulkan and native XYZ
comparison, including all 2103 words and actual 516/258/0 counters. The overall
native job retains its prior independent mismatches.

The user now requests larger coherent D3D9 implementation batches. The next
batch groups resource creation, locks/staging and transfer operations across
2D/cube/volume resources, with targeted tests and one original DAO replay at
the batch boundary. Do not resume an individual-call replay loop or create
unrelated sample/coverage projects. Existing finite profiles still fail
unsupported operations explicitly.

### Coherent D3D9 resource and transfer batch

The owned --transfers PE adds standalone render-target allocation/description,
ColorFill under disabled color writes/sRGB state, full-pixel partial-fill and
pitched UpdateSurface checks, source-lock and bounds rejection, partial writable
staging locks, SYSTEMMEM binding rejection and UpdateTexture across 2D/cube/volume
resources. Actual GPU reads check all three 2D mips, matching mip suffixes,
NO_DIRTY_UPDATE/explicit dirty restoration, UpdateSurface into a sampled texture,
all six cube faces at both mips, and both BGRA/L8 volume slices and lower mips.
The unchanged normal shader frames are rendered after resource/state restoration.
All 2100 report words pass on physical M4; no original game data is used.

Two actual attachment GPU tests pass, including padded source rows, exact alpha,
neighbor preservation, invalid extents/short input/overflow and rejection of
partial writes into uninitialized GPU storage. All 224 PE units, 22 observer
units and both crates' Clippy pass. Win32 stdcall cleanup tests cover every new
method; real-SDK slot/flag assertions and identical-PE native Windows/software
Vulkan comparisons are wired into the existing workflow and remain pending.
This is a resource/transfer milestone, not a full D3D9 implementation or a
verified original character-creation frame. The next action is one original DAO
replay at this batch boundary; individual-call replay development is superseded
by the user's request for coherent batches.

### First visible original DAO character-creation scene

Clean 845c2a2 on the physical M4 reaches Character Generation after New Game and
skipping the original opening movie. CUA directly observes a textured male human
character, textured room/columns/floor, character-generation controls and the
opening background tutorial. These are original executable/asset scene and UI
observations, not the intro movie or an owned replacement scene. The original EXE
SHA remains 2346caee611002d4dff6ecef68eb315b63a37004dd98b39c11a9c26c0b45bfb3.
The running capture records an empty pre-build runtime diff, all source hashes,
original EXE/probe/driver identities and first-unique stderr lines with repetition
counts. Original files/captures remain ignored and local.

The trace passes the prior 512x512x4 L8 allocation and subsequent original volume
locks/unlocks, with further original indexed draws and no fatal diagnostic at
observation time. Some finite-profile HRESULT failures remain visible; this is
not an assertion that every call succeeds. The new ColorFill/UpdateSurface/
UpdateTexture operations were not observed in this startup capture, so their
owned GPU tests are separate evidence. In particular, the progression from
5142eb4 includes abffcbd's previously unreplayed L8 support: the larger resource
batch is not isolated as the cause of this scene becoming visible.

The user is exploring the running build. Leave its original diagnostic process
and runtime sources intact during that exploration. No frame rate, full-frame
native Windows parity, complete visual correctness or gameplay is established.

Previous CI 42707022286 (abffcbd) passes hosted Metal. Software Vulkan's filtered
BGRA volume midpoint is [128,0,127,255] versus the test's exact [128,0,128,255];
this finite rounding difference remains recorded without changing the oracle.
Native version-27 volume PE fails at initial CreatePixelShader with INVALIDCALL
(stage 6), before volume allocation. Thus it provides shader-rejection evidence,
not a native volume-resource comparison. The pinned open HLSL compiler emitted
SM2 TEXLD directly into oC0 from a swizzled v0, contrary to Microsoft's documented
SM2 temporary-destination and coordinate-register restrictions. A separately
identified legal shader witness is needed; do not relabel the rejected binary
as native success. Existing streams/StretchRect/POSITIONT native mismatches also
remain. CI 42709101205 tests 845c2a2; results are still pending at this record.

### User exploration: client double-click stop and performance observations

The 845c2a2 run stops after the user reports clicking a skill's +1 control.
The observed fatal boundary is DefWindowProc receiving WM_LBUTTONDBLCLK (0x203),
not an established failure in skill-allocation logic. Status is 78 after 347.37 s.
The adapter now handles the default no-action return for left/right/middle
client double-click notifications; nonclient and other mouse defaults remain
separate rather than receiving a blanket successful return. Existing native
mouse queue generation already respects CS_DBLCLKS. Two focused host units and
a new 22-word owned --client-default PE pass, including all three default
returns, unchanged capture ownership, capture callbacks and stack cleanup.
Clippy passes. Identical native Windows comparison is wired and pending; the
original +1 path is not yet reverified with this change.

Performance is visibly poor but no frame-rate measurement has been made. The
optimized original run emits 268,887,676 diagnostic bytes (first-unique retained
stderr is 14,399,224 bytes); pipe formatting/deduplication therefore remains
work even when repeated lines are not written to disk. Code inspection separately
shows SPIR-V parsing/lowering, WGSL generation, shader-module/pipeline creation,
new vertex/index/constant/binding resources and one submission per legacy draw.
Every Present performs a full GPU-to-CPU framebuffer readback before AppKit
presentation. These are concrete optimization targets, not measured percentage
attributions. Cache shader/pipeline preparation first, then measure logging and
presentation independently; preserve all geometry, shaders and image quality.
The user's interaction is the driver; keep originals/captures local.

### Quick Play: live synchronization capacity

The user reaches Quick Play in clean 3c867f2; the original run then stops at
`PE32 event handle budget exceeded`, status 78 after 178.95 seconds. A secondary
worker RefCell panic is also captured during termination. The primary observed
limit is Alkyhol's 128 simultaneously owned events, not an established crash
inside the game's Quick Play logic. The live registry removes entries on close;
this is a live-count ceiling, not a cumulative allocation counter.

The event/semaphore adapter now supports a combined 16,384 real live backing
objects. This is an explicit Alkyhol resource bound, not a claimed Windows
limit. A full registry returns NULL plus ERROR_NOT_ENOUGH_MEMORY; actual kernel
creation failures preserve their errors. No phantom event or successful zero
handle is returned. Closing either type frees capacity. Creation/state-change
results also leave the registry borrow before invoking a fatal diagnostic.
Named/security/alertable profiles remain outside existing supported contracts.

On physical M4/Rosetta, the new 14-word owned event-capacity PE passes with 512
live A/W events plus 512 semaphores: all object identities are distinct, manual
signals persist, automatic signals consume, semaphores retain independent permit
counts, and closing/replacing half preserves all other objects and stdcall stack.
Three host event units pass, including actual 16,384-slot shared exhaustion,
NULL/error-8 results, reclaimed capacity and error-preserving cleanup. The
synthetic comparator rejection test and Clippy pass. All six existing frozen
event PEs retain their earlier admitted and excluded outcomes; frozen bytes and
native observation are unchanged. Identical-PE native Windows comparison for
this new larger live-object witness is wired into existing CI and pending.
Original Quick Play after this change is pending; character creation on the
preceding revision remains the observed scene milestone. Game captures/data stay
local, and no DAODecomp sources were changed.

### Save-slot prefix lookup after the expanded-handle build

The next clean original run, 0c71617, stops after 98.70 seconds while enumerating
save-slot directories with a literal stem followed by '*'. The fatal trace is
`wildcard search pattern is outside the measured all/extension profile`, status
78. The user also reports accidentally closing the app; no causal assumption
about that input is needed to identify the captured API boundary. The prior
128-event-budget diagnostic is absent from this run; Quick Play completion is
still not established.

The search gateway now admits ASCII literal stem prefixes followed by one '*',
with case-insensitive prefix matching, real file/directory metadata and existing
handle iteration/close behavior. Empty suffixes are included. Literal parent
paths, reparse/special-entry exclusions, budgets and existing all/extension
profiles remain checked. Question marks, interior/repeated stars, DOS dot/8.3
aliases and Unicode pattern case rules are not silently approximated.

Five filesystem host units pass, including actual prefix enumeration, extension
regression, nonmatches, missing-file versus missing-parent errors and symlink
rejection. The new identical owned 16-word PE passes A/W enumeration over four
real matching entries, exclusions, attributes, file sizes, exhaustion error 18,
missing errors 2/3 and stack cleanup. The two literal frozen exact-file runs
still match their native Windows observations; frozen source/PE/reference inputs
are untouched. Comparator transport rejection tests and Clippy pass. The new
prefix native comparison is wired into existing CI and pending. Original
save-slot progression after this change is pending; all game/profile captures
remain local.

### Further scene loading: shared low-address CPU storage

Clean a858431 passes the earlier save-slot search boundary and continues loading
more original graphics resources. After 160.79 seconds it stops at
`D3D9 low-storage budget exhausted` during a further declaration creation. The
allocator counted every retained COM/texture/input mapping against an early
512-mapping bound. No missing geometry, substituted textures or completed
in-game frame is inferred from this loading progression.

Small writable, nonexecutable CPU allocations (up to 1 MiB) now occupy disjoint
16-byte-aligned slots in real 4 MiB low-address arenas. Larger allocations keep
separate mappings, as do immutable records and executable storage with their
existing independent OS protections. All guest-visible bytes/pointers stay
stable until process teardown; slots are not recycled, and this change does not
claim CPU shadow reclamation. The explicit ceiling is now 4096 actual mappings,
with checked low-address-space placement, lengths and protection failures. It is
not an unlimited-memory or Windows quota claim. No pixel, mip or resource payload
is clipped to meet that ceiling.

On physical M4/Rosetta the new eight-word --texture-storage PE passes 768 live
managed textures, every mip/texel, metadata and raw lock pitches, followed by
external releases, 128 later allocations, retained texture binding and a shared
surface alias retaining unchanged bytes. The existing 4094-vertex plus
4094-index-buffer rollover/lifetime PE also passes after rebuilding its checked
header identity. The new identical-texture-PE native Windows comparison is wired
into existing CI and pending.

The actual allocator test is linked with the required small PAGEZERO through
`cargo rustc --manifest-path pe/Cargo.toml --target x86_64-apple-darwin --lib
--profile test -- -C link-arg=-Wl,-pagezero_size,0x1000`, then run with the exact
ignored test name `runtime32::d3d9_32::tests::actual_small_guest_allocations_pack_disjoint_bytes_and_keep_protection_classes`.
It passes 1100 actual writable allocations crossing arena boundaries (two
mappings), checks all unchanged bytes/alignment, read-only/write admission and a
separate large mapping. A first ordinary test-host attempt failed low-address
allocation because that host reserved PAGEZERO; the specifically linked run is
the physical allocation evidence. The normal suite explicitly skips this one
link-dependent test: 229 other PE units pass. Eighteen observer units, including
new strict transport rejection checks, and Clippy pass. Original scene loading
after the allocator change remains pending; user game/profile data stays local.

### User-reported Orzammar cinematic and NULL pixel samplers

The user reports that clean f0a3bb4 progresses into the in-engine Orzammar
introductory cinematic, then stops partway through it. The captured run lasts
944.04 seconds and terminates with `pixel shader texture slot is not bound`.
The cinematic identification is the user's observation; no full native Windows
frame comparison or independently captured cinematic-frame assertion is added.
The earlier low-storage stop is absent from this run.

D3D9 NULL pixel sampling is now implemented as opaque black (0,0,0,1), using
private initialized one-texel 2D/cube/volume GPU views matching the shader's actual
float-image declaration. A per-device lazy cache retains those three resources;
GetTexture/application COM state is unchanged. This applies only when a slot has
no application binding. Real bound resources still undergo initialization,
feedback, type and lock checks; invalid assets are not replaced with a NULL view.
Array/integer/depth/multisample/unsupported image declarations remain failures.

The expected result is independently specified in Wine's existing native-facing
[unbound sampler tests](https://github.com/wine-mirror/wine/blob/master/dlls/d3d9/tests/visual.c)
(the 2D/cube/volume unbound checks), not inferred from a desired DAO picture.
No Wine test code or shader bytes were copied into the MIT fixtures. The owned
version-29 --null-textures PE uses the existing legal SM3 2D bytecode, alternates
NULL/real/NULL/real bindings, checks actual GetTexture values and all RGBA values
in four complete frames, including opaque-black alpha versus the real texture's
quarter alpha. All 4168 words pass on physical M4. Its identical native Windows
comparison is wired and pending, so this is not yet new native evidence.

The physical GPU test separately passes 2D/cube/volume NULL output, real nonopaque
texture output and repeated unbinding over complete targets; real uninitialized
textures still fail. A host unit validates dimensions and rejects unsupported
image classes. Twenty-nine backend units, nineteen observer units, both crates'
Clippy and the new physical GPU test pass. The normal backend unit suite also
retains its two explicitly GPU-dependent skips. Original cinematic progression
after this change remains pending.

The user also requests a future pure-Rust SharpRetro runner and coordination
with “Design iterative decompiler workflow.” That coordination is complete;
verified Rust entry points, ownership, memory/ABI boundaries and incomplete CPU
state/exception areas are recorded in sharpretro.md. This is an integration plan,
not an implemented or validated replacement for the current Rosetta runner.


### SharpRetro native ARM64 PE32 execution foundation

The user paused further Rosetta API iteration after a later Orzammar cutscene
crash and requested the JIT binary path for side-by-side comparison. Clean
614363a's preserved local capture terminates at original daorigins.exe+0x24242a
calling KERNEL32!GetLocalTime. That API remains unchanged; the cinematic identity
and later progression are the user's report. No new native full-game comparison
or corrected-frame assertion is inferred.

The isolated jit32 crate now executes the existing owned no-CRT PE32 ABI binary
unchanged through open SharpRetro Bits32 decoding/lifting and Tier0-generated
ARM64, using Apple MAP_JIT plus write protection/cache invalidation. No Rosetta or
interpreter fallback executes inside this runner. The external SharpRetro pin is
1ab08109ac75cfe65d2393828706bac11f01cc91; all relevant Rust trees, generated
artifacts and generator/ISA inputs have checked SHA-256 identities in
jit32/sharpretro-source.json. Its active unrelated Decompiler files are untouched.
Runtime execution is Rust/libSystem; open .NET is needed only to regenerate the
upstream ignored Rust sources in a fresh checkout.

Guest memory is disjoint checked logical storage in an owned sparse 4-GiB
reservation, with read/write/fetch permissions, overlap/wrap checks and no W+X
admission. Each compiled instruction guards its memory access before Tier0's
native load/store. A denied access exits to Rust, restoring the pre-instruction
register/flags snapshot. Unsupported multi-store/read-after-store operations,
atomics, loops, FP/SIMD and 16-bit stack operations do not receive successful
results. Both instruction execution and import transitions are bounded. Current
imports are four real Alkyhol kernel services with checked stdcall marshalling;
no arbitrary guest pointer enters the host-pointer ffi.rs API. The initial stdout
contract explicitly requires a regular file.

Cumulative execution exposed two generated-lifter bugs: CALL emits an eight-byte
return-PC write while subtracting four from ESP in Bits32, and PUSH Ib-sx def84
mistakes its encoded immediate width for the stack operand width. Explicit local
adapter corrections and 32-bit stack wrapping are tested with original opcode
bytes, adjacent stack values and exact register/flag assertions. Tracked shared
SharpRetro sources remain unchanged; these findings were sent to the coordinated
decompiler chat. Twelve targeted physical M4 tests pass, including actual native
ARM execution, CALL/PUSH/RET footprint and stack wraparound, unmapped and protected
faults, RMW rollback, cache/budget resume, unsupported operations and real kernel
service marshalling. One Python transport test rejects changed bytes/status,
missing or extra diagnostics, budget exits and missing cumulative evidence.
Runner Clippy passes; the upstream CPU dependency still reports its own warnings.

The PE SHA-256 is da101b3b451c8d511dd02e6316dc3101ef5c1b13cd56add33c029f9a476b191c,
identical to the existing frozen native Windows observation. On physical M4 each
JIT execution completes 100,089 guest instructions, 20,005 service transitions
and 99 compiled instruction blocks, returning status zero. Both preferred-address
and forced-rebase output are exactly 32 bytes / eight words, agreeing with both
actual Rosetta runs and the existing frozen Windows result. The native reference
and fixture sources are unchanged. tools/check_jit32.py verifies identities
before and after execution and records code/source identities and host details
in ignored local/jit32/comparison.json. Windows was not rerun for this milestone;
its evidence remains the separately identified frozen native observation.

DAO itself does not run through this JIT milestone. Original DLL initialization,
TEB/FS/TLS, callback continuations/reentry, x87/MXCSR/exception state, threads and
the larger Win32/COM/D3D9 service marshaller remain required. The old low-pointer
Rosetta dispatcher cannot be reused by casting guest offsets into host pointers.
No game-equivalence or general CPU-completeness claim follows from this finite
PE result. The D3D9/Rosetta/source/Metal paths and frozen acceptance data remain
unchanged.

A direct read-only startup preflight of the user's local original DAOrigins.exe
returns `static TLS initialization is not implemented by JIT32` before executing
any original game instruction. That is the next loader boundary; it was not
bypassed. Its report remains in ignored local/jit32/dao-preflight.stderr. The JIT
host itself is verified as an arm64 Mach-O executable.


### Native ARM JIT: static TLS, DLL attach and the first original DAO initializer

The new Process API materializes a checked static dependency graph using only
explicitly supplied originals; unused available DLLs do not acquire imports,
TLS blocks or initializers. It chooses disjoint addresses, applies HIGHLOW
relocations, binds named/ordinal code/data exports and checked forwarders, and
constructs actual single-thread TEB/static-TLS storage. TLS indices/templates
retain relocated pointer initializers and requested alignment. A separate
Image::check_emulated_startup32 validates callback metadata while preserving the
old diagnostic/native startup restrictions for existing clients. Unknown CLR,
unsupported auxiliary directories, missing dependencies/exports and dependency
cycles remain failures. Nonzero TLS SizeOfZeroFill remains outside admission.

The scheduler invokes original dependency TLS callbacks before original DLL
entries, then EXE TLS callbacks, retaining a stopped call across budgets/import
exits. Static attach receives the module base, reason 1 and a backed non-NULL
opaque reserved indicator as specified by
[DllMain](https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain); no Windows
startup-structure contents are fabricated. Stack/nonvolatile violations and
FALSE attach prevent application entry. General API callback reentry, dynamic
loading and detach/failed-load rollback notifications are still unimplemented.
Synthetic tests establish the scheduler's tested ordering/state transitions;
there is no new native Windows DLL-order capture in this milestone.

The real service state supplies checked owned process/private heaps, size/free/
validation/reallocation (including byte retention, zeroed growth and in-place
shrink), recursive single-guest-thread critical sections, dynamic TLS slots,
reversible pointer encoding, host-backed process/thread IDs and time/counter
outputs, owned case-insensitive ASCII environment A/W buffers, and explicitly
selected NT 6.2/build 9200 version queries. Environment starts explicitly empty,
not inherited from host secrets. Executable heaps, unsupported flags, unknown
calls and unavailable threading/dispatch do not receive successful stubs.
Freed storage loses guest access; the marshaller synchronizes direct TEB last-error
and dynamic TLS slot state. Clock/lock/environment/identity assertions are host
units, not cross-Windows observations. The existing frozen load-config cookie
policy is retained: only a default cookie with declared 72-byte metadata is
seeded from actual entropy; 64-byte/other seeds are preserved. No SEH delivery is
claimed from inspecting that metadata.

The unchanged existing TLS-data PE (SHA-256
967e546b33fdaddd01c107d0d3e0dfce5cb68f3bf4ef0a6d999945660d4de12a) executes
135 guest instructions/four services/135 blocks. All 24 output words agree with
actual Rosetta and the frozen Windows result. The unchanged literal heap-resize
PE (9554b9ee95c3f06161495b73cfb2b67b342e0c327ceb8bef163a7863f368dbd5)
executes 3789 instructions/46 services/449 blocks; all 64 words agree with both
actual Rosetta and frozen native Windows. Each JIT and Rosetta test is run at
preferred and forced-rebased addresses. tools/check_jit32_startup.py verifies
source/PE/reference identities before and after execution and records raw bytes,
host, code identity and counts in ignored local/jit32/startup-comparison. Windows
was not rerun here; the unchanged frozen observations are the independent native
evidence. The original eight-word ABI comparator still passes unchanged.

The TLS witness also exposes a generated CPU omission: absolute A0-A3 memory
operands ignored the segment override. The adapter now honors FS/other segment
bases, wraps Bits32 effective addresses before adding a segment, and keeps LEA
segment independent. LEAVE and register SETcc forms are admitted. Targeted ARM
checks cover exact moffs/ModRM/LEA bytes, 32-bit wrapped index addressing, LEAVE
fault rollback and all 16 SETcc predicates over all 32 relevant flag combinations.
These synthetic CPU checks are distinct from the finite frozen Windows outputs;
they do not establish whole-CPU equivalence. SharpRetro's tracked sources and
pinned revision/artifacts are unchanged.

A real original DAO startup probe now maps/binds 12 modules and enters the original
msvcr80.dll initializer. It executes 191 guest instructions and seven real service
transitions, then stops explicitly at KERNEL32!GetModuleHandleA, return caller
msvcr80.dll+0x2fe3. The explicit NT 6.2 persona and empty environment are recorded;
original module/input hashes, registers/flags/caller, elapsed time and diagnostics
stay under ignored local/jit32. This is original dependency code executing through
native ARM64, not complete CRT initialization, EXE entry, gameplay or shaded-frame
parity. The earlier static-TLS preflight rejection and working Rosetta DAO build
remain preserved. Module lookup/service exports are the next reached blocker.

Twenty-eight JIT/process/service/CPU tests pass on physical M4, and the new
emulator-preflight test brings the portable image32 suite to 18 passing tests.
The x86-64/Rosetta PE suite also passes 229 units plus those 18 parser tests,
retaining its existing one PAGEZERO-dependent skip. Runner Clippy and two strict
Python transport tests pass. Shared original fixtures, native references and
DAODecomp are untouched; no game-derived inputs/inventories are committed.

### Native ARM shared PE32 runtime and original EXE startup

The new `jit32/shared-runtime` feature executes original guest instructions with
SharpRetro on the physical M4 while reusing the existing Win32/COM/D3D9 services.
It has no Rosetta or interpreter fallback. The native host has ordinary hard
PAGEZERO protection; checked biased backing keeps guest pointer fields 32-bit.
API extents and CPU page/arena access checks remain separate. See
[the execution contract](sharpretro.md#native-arm-execution-through-the-existing-pe32-services).

`tools/check_jit32_shared.py` runs the unchanged ABI, TLS and heap witnesses at
preferred and forced-rebased addresses. All 8 + 24 + 64 output words agree with
the frozen Windows observations; PE/source/reference identities are checked
before and after, with positive native CPU/service counters. This is a new native
ARM execution path, not a new Windows measurement. The existing standalone JIT
and actual Rosetta comparators remain intact.

Physical M4 CPU tests now cover reached divide/#DE behavior, REP progress and
fault boundaries, atomic/vector alignment, XMM bit preservation, 16/32-bit flag
and register-bank stack layouts, real TSC bracketing, x87 environments, transfers,
comparisons and precision. An independent owned x86 assembly observer in
jit32/tests/x87_reference.c ran under actual Rosetta: precision control, four
integer rounding modes, both reduced-precision double-rounding directions and
11 sin/cos float-store cases agree with the ARM tests. These are finite host
observations; they are not native Windows observations or proof of the game.
Software CRT math also matches the unchanged accepted frozen Windows sqrt/atan
components: 20 + 11 rows, checking result80/control/status/lower operand, without
editing those inputs or acceptance data. General exceptional/uncertified FP
paths still fail explicitly; no full shaded frame or transcendental hardware-bit
parity is inferred from these rows.

The original DLL graph now completes under native ARM and enters DAOrigins.exe's
own startup. The recorded original-input run reaches 332,460,348 guest
instructions / 798,587 services before FSINCOS; with bounded trig it advances to
337,870,384 instructions / 821,784 services and a memory-SAR lowering limitation.
The input binaries' SHA-256 values agree before/after. Raw game-derived state,
addresses, bytes and logs remain local. No JIT window, gameplay or Rosetta
cutscene parity is claimed here. The Rosetta game boundary remains separately
preserved, as do the working source/PE64/Metal sample paths.

On this physical M4, 39 JIT CPU/process/service tests pass, including the reached
trig and SIMD transfer cases. The x86/Rosetta PE suite passes 229 units plus 18
image32 tests, retaining its existing one PAGEZERO-dependent skip. The new
syntax-adapter test and three strict Python transport tests pass; runner Clippy
passes with warnings denied for the runner itself. These counts do not include
new full native-Windows or game-frame measurements.

The DLL inputs retain the existing explicitly configured staged/unpacked
dependencies; their exact identities remain in the local capture. The original
DAOrigins.exe file is unchanged.

The preexisting comparators also pass again: unchanged standalone JIT and actual
Rosetta executions match the same frozen 8-word ABI and 24/64-word TLS/heap
results at both bases. The new shared service path matches those 96 words at
both bases independently; no frozen sources, PEs or Windows references changed.

The subsequent memory-shift milestone passes 41 physical M4 JIT tests. Forwarding
is confined to an identical SSA address/width with a dominating store; 120
shift/input/count cases check actual native ARM results/defined flags, plus
non-dominating/mismatched-read rejection and write-fault rollback. These host
assertions add no new Windows observation. An actual original-input execution
advances to 338,719,466 instructions / 825,382 services and its owned software-
exception snapshot entry, after writing its real startup log. The existing
thread-naming filter must be entered with real CPU state next. Input hashes
agree before/after; no native window or parity is inferred from this stop.

The native software-exception entry passes the unchanged owned 15-word SEH
contract with two original handlers executed, plus explicit rejection of all
three excluded continuation modes. The separate actual Rosetta observer also
passes the same four PEs. The x86 PE suite remains 229 units + 18 image tests
(one existing skip); 41 native ARM JIT tests and runner Clippy pass. These are
host tests and do not add a new native Windows exception observation. General
FXSAVE/FXRSTOR, native signal delivery and C++ unwind remain unsupported; the
native software API uses its real saved CPU registers and existing checked
handler traversal instead of a fabricated snapshot.

Original-input execution with native SEH reaches real worker entry procedures,
Win32 class registration and an Apple M4/Metal D3D9 factory. Original platform
validation rejects the currently declared scalar CPU's SSE/SSE2 capabilities;
shutdown then stops at the existing owned-mutex-close exclusion. A local bounded
instruction trace confirms those actual original CPUID branches. Input hashes
remain unchanged. No JIT-created window, game frame or Rosetta cutscene parity
has yet been observed. Frozen shared ABI/TLS/heap comparisons still pass all
96 words at both bases with no fixture/reference changes.

An opt-in incomplete SSE/SSE2 development CPU profile now enters the original
feature branches without patching game code; missing instructions remain fatal.
The reached original FMOD scalar conversion is implemented as a six-form family
with software rounding and guest MXCSR. Physical M4 passes 43 default JIT tests
(one separately invoked observer test ignored by default). The independent
actual Rosetta assembly/native ARM comparison passes 512 conversion rows,
including all rounding modes, DAZ, signed boundaries and NaN/infinity status.
Runner Clippy passes. These rows are not new native Windows or physical Intel
observations, and the experimental profile does not promise full SIMD support.
Original DAO now proceeds beyond platform validation into original ERF resource
indexing; no new native JIT window/frame is established by this interim run.

The reached FLDPI gap is implemented with the remaining x87 constant family.
An independent original-assembly observer under actual Rosetta agrees with
native ARM for all 84 constant result80/status rows (7 constants × 4 RC × 3 PC).
Native code now packs immutable instructions in owned MAP_JIT arenas; lifetime,
disjoint execution and large-allocation tests pass. The full native suite is
45 passing tests, with two external observation tests explicitly invoked by
separate checkers (512 conversion + 84 constant rows pass). Runner Clippy and
all unchanged shared frozen Windows ABI/TLS/heap words at both bases pass.
This storage change does not remove guest instructions/checks or establish an
FPS result. The next original-input replay also restores the explicit open
D3D9 compiler configuration used by Rosetta; no new game frame is claimed yet.

The native zero-timeout wait optimization passes the same 26-word original
worker/TLS/FLS witness and new 12-word event-poll/worker-progress PE before/after.
The scheduler's existing service quantum remains active; blocking waits and
Rosetta remain unchanged. These are finite native ARM host assertions, with no
new native Windows timing or game performance claim.

The reached SIMD environment/math group passes 54 native CPU tests and the
independent actual Rosetta 512 conversion + 1,024 arithmetic rows, with guest
rounding/result bits/status and explicit failure coverage. Three external
observation tests are invoked separately from the default suite. Fixed-width
GPR/XMM move aliases, per-lane integer operations and MXCSR state are checked;
the unimplemented numerical domain remains explicit. The original loader's
subsequent stop is a real denormal SIMD operand. Loading reached the preceding
math boundary in 91.6s versus 577.6s at the previous nearby boundary, with
uncontrolled host load; no FPS/game equivalence claim follows. Original input
binary hashes remain unchanged and there is still no native JIT window/frame.

Masked SIMD edge semantics now pass 6,144 additional independent actual Rosetta/
native ARM rows, retaining all 512 conversion and 1,024 normal arithmetic rows.
The fixed edge matrix covers denormal inputs, DAZ/FTZ, NaN payload priority,
invalid/divide/overflow responses and unbounded-exponent rounding boundaries.
55 default CPU tests pass (four external observers invoked separately), plus
Clippy and unchanged shared frozen Windows ABI/TLS/heap comparisons. Unmasked
#XM delivery remains unsupported; no full CPU or native Windows parity follows.

The native WNDPROC validator now normalizes guest code addresses against biased
host ranges. The unchanged owned window witness passes all 32 words and 13
original callback invocations; its previously zero process status alone was not
a pass. Original DAO subsequently creates/shows its own native splash, draws the
original bitmap and runs its message procedures. A physical window-specific
capture visually confirms the splash artwork while original resource loading
continues. This is the first original native-JIT splash window, not a D3D9 game
frame/menu/cutscene or Rosetta progress parity. Raw captures and game data remain
local; no game assets are committed.

Original native DAO's subsequent graphics enumeration exposed a real immediate
width/carry bug in the JIT: an equal DWORD shader-version comparison incorrectly
set CF alongside ZF. An owned regression fails before the correction. The checked
builder now masks integer literals to their declared IL width, preserving the
original adapter caps and game checks. On physical M4, the independent actual
Rosetta assembly comparator passes 960 rows across CMP/SUB/ADD/ADC/SBB and
8/16/32-bit widths, both carry inputs and multiple original encoding/storage
forms, comparing results and all six arithmetic flags. Reproduce with
`tools/check_jit32_immediates.py`. The 56 default tests (five independent observers
separately invoked), runner Clippy, existing 512/1,024/6,144 SIMD rows and frozen
shared ABI/TLS/heap comparisons at both bases pass. No frozen Windows data or
SharpRetro sources changed. The corrected original replay is ongoing; game-frame
and Rosetta cutscene parity remain unverified.

The corrected original-input native replay completes adapter acceptance and
creates the actual D3D9 device, then initializes DirectInput before an explicit
BSR stop in original Umbra. Inputs remain unchanged (193.9s uncontrolled wall
time; no FPS inference). BSF/BSR are now admitted using the existing pinned
templates, with native host tests for all source-bit positions, 16/32-bit widths,
register/checked-memory forms, partial writes, zero-source ZF and fault rollback.
57 default CPU tests pass, five external observers remain separate. A subsequent
original replay is pending; no native JIT game frame/menu or cutscene is claimed.

The following original replay passes Umbra and creates the full game window,
original shaders and vertex/index buffers, then stops explicitly at UNPCKHPD
in the original EXE. Original hashes agree before/after (633.8s uncontrolled wall
time; 816,434,379 loading-thread guest instructions). There was no original
Present. Four legacy 32/64-bit low/high floating unpack forms now pass native
bit-interleave, alias, status-preservation, aligned-memory and fault tests using
the pinned templates. The shared runner's new explicit finite budget setting
preserves the billion default; owned ABI execution stops without output at 1,
passes eight frozen Windows words at 1,000,000 and rejects invalid limits.
59 default CPU tests and Clippy pass. The next original experiment explicitly
chooses ten billion transitions per entry; neither budget extension nor these
host assertions establish a game frame or Rosetta progress parity.

The ten-billion original replay passed unpacking and reached original matrix
SHUFPS after further texture/viewport setup, at 825,096,084 loading-thread
instructions; inputs remained unchanged (457.1s uncontrolled wall time). No
Present was observed. SHUFPS/SHUFPD pass all 256 selector values on native ARM,
including source aliasing, aligned memory, exact bits, preserved registers/
status and fault rollback. A physical original-startup sample identified a
window-polling worker holding the CPU lease while waiting for AppKit. Native
PeekMessage now keeps only a host App Arc during that wait, ending the UI borrow
and releasing/reacquiring the lease; guest event processing remains serialized.
The existing Rosetta path is retained. New owned polling/window execution passes
36 exact words before/after, and existing 32 window + 26 worker/TLS + 12 poll words
pass afterward. All 60 default CPU tests and runner Clippy pass. Owned total
process times were ~0.835s both ways, not a measured game performance gain. No
new native Windows observation or game-frame/cutscene parity follows.
The x86_64 PE crate compile check also passes; this is not a new Rosetta runtime
or native Windows execution observation.

The polling/shuffle original replay reaches variable PSRLQ in the EXE at
829,550,784 loading-thread instructions, with original inputs unchanged. Its
138.7s wall time compares to 457.1s at the preceding nearby SHUFPS stop under
uncontrolled host load; no controlled speedup or FPS claim is made. No Present
has yet been observed. Eight XMM variable packed shift forms now pass targeted
native tests across all supported lane widths, directions, aliasing, register/
aligned-memory count sources, full low-64-bit saturation and ignored upper count
bits, preservation and fault bounds. MMX remains explicitly unsupported. The
61 default CPU tests and runner Clippy pass; original replay follows.

Original execution clears PSRLQ and stops at runtime BTS in the EXE, after
829,889,181 loading-thread instructions (179.2s uncontrolled wall time, original
inputs unchanged). No Present was observed. The explicit Bits32 BT/BTS/BTR/BTC
adapter handles register/imm8 masking, signed register-index memory bit strings,
partial register writes, CF from the old bit, segment/address wrap and guarded
aligned atomic modifying forms. LOCK registers, 16-bit memory addressing and
the existing unaligned-atomic domain remain explicitly excluded. Independent
actual x86/Rosetta assembly agrees with native ARM on all 112 signed memory/CF
rows via `tools/check_jit32_bits.py`. Native bounds/permission/fault and register
tests pass; the full default suite is 65 passing tests with six independent
observers separately invoked. Runner Clippy passes. These add no native Windows
observation or original native-JIT game-frame/cutscene parity claim.

The next original native run clears BTS, loads AddIns.xml and reaches repeated
CMPSB's unwired intrinsic at 1,415,045,498 loading-thread instructions (217.5s
uncontrolled wall time, inputs unchanged). No Present was observed. Checked
CMPS/SCAS byte/word/DWORD lowering now produces all arithmetic flags, direction-
correct pointers and exact count/early-stop behavior while retaining per-REP-
iteration fault commitment. Independent actual Rosetta assembly agrees on all
432 input/result rows via `tools/check_jit32_strings.py`. Native tests also retain
completed iterations when the second memory source faults, and skip invalid
sources on zero repeat count. Segmented/16-bit-address strings remain explicitly
unsupported. All 67 default CPU tests and runner Clippy pass; seven external
observers are invoked separately. No new Windows or full game/frame parity is
inferred from these finite host comparisons.

Latest CI inspection on e9dc101 found the PE/Rosetta workflows stopping at Rust
formatting checks in the native bridge files; formatting is restored without
changing fixtures or runtime behavior. Separate native Windows full-path,
message/paint, SEH and capture observations still report differences; those
failures are retained, not suppressed or relabeled as passes. The new standalone
native x64 SIMD observation job measures the owned comparison matrix without
game inputs and refuses ARM/translated Windows hosts. Its result is pending.

Native SIMD observation run 42765765607 succeeds on native x64 Windows Server
2025, AuthenticAMD Family 25 Model 1. The first attempt stopped at a binary
stdout transport-size check; Windows binary mode corrects that transport, without
changing any matrix inputs/results. The complete 18,432-row raw table SHA-256 is
668ad960788af5dfbaae4f265635672a5afc98b25867519b8c5909e1b1b6f2c3.
Its source's native CRLF hash and verified LF-normalized hash are both preserved.
The frozen table and provenance are jit32/reference/sse-compare-windows-amd64-v1.*;
the complete Rosetta counterpart is frozen separately. Results/masks and EFLAGS
agree for every row; MXCSR DE differs in 152 ORD/UNORD NaN/denormal rows. The
runner implements the measured native Windows AMD profile, matching every row
with `tools/check_jit32_compare_windows.py`. The strict Rosetta comparator still
fails; it does not drop or bless those differences. No physical Intel or general
CPU model equivalence is inferred.

The latest original run reaches CMPPD after settings initialization at
1,617,727,162 loading-thread instructions (227.5s uncontrolled wall time, original
inputs unchanged); no Present was observed. The eight legacy SIMD comparison
forms now pass 71 default CPU tests/Clippy, with memory/alignment, aliasing,
immediate-mask and explicit unmasked-exception failure coverage. Eight external
observers are invoked separately. Original replay follows; game-frame and
Rosetta cutscene progress parity remain unestablished.

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
table are frozen; `python3 tools/check_jit32_sqrt.py` replays it. The existing
native Windows observer workflow now includes this unit; its result is still
pending. No older frozen comparison is weakened. A fresh original replay is
required to verify this square-root implementation in DAO, and Rosetta's
Orzammar progress parity remains unestablished.
