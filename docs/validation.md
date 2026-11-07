# Executed validation

Current graphics development uses direct Metal from Rust. The historical records
below describe their exact revisions, including the retired wgpu/Vulkan backend.
See the latest direct-Metal record for current implementation evidence.

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
limits and failures. Native Windows SDK/WARP comparison for this new milestone
is pending; no GPU binding/PSO/Dispatch or original-frame parity is claimed.

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
