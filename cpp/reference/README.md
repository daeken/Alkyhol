# Native COM reference contracts

`object-private-data-windows-v1.json` preserves the actual 34-record observation
from CI 42356150734 on `e3218ee`, Windows SDK 10.0.26100.0, MSVC 19.51.36260,
using a real D3D12 WARP fence. It contains source/executable hashes, command,
compiler/host/adapter identity and exact output bytes. The custom IUnknown tracks
its own real AddRef/Release/destruction; it is not an Alkyhol object.

Important measured distinctions for the Rust object metadata implementation:

- Missing byte data returns `887a0002`, sets size zero and leaves output untouched.
- Insufficient space returns `887a0003`, reports required size and copies no bytes.
- A null output is a successful size query without acquiring an interface reference.
- Removing existing bytes returns S_OK; removing absent bytes returns S_FALSE.
- Zero-length non-null data remains a present zero-length entry.
- Null interface assignment remains a present pointer-sized entry, not removal.
- Getting a stored interface acquires a reference; replacement and object destruction
  release retained references. Same-pointer assignment still calls AddRef/Release.
- SetName stores UTF-16 debug-name bytes including NUL; an empty name has a
  measured entry size of two bytes. Its post-clear contents were not read back.

The historical row label `interface.clear_missing` is a repeated null assignment
to an existing null-interface entry; the measured size query establishes that it
was not a missing key. The label is retained with the frozen source identity.

The unchanged observer also matched this entire frozen contract in CI
42356681229 and 42356731682; all twelve jobs passed in each run.

This is finite evidence on this SDK/runtime, not all COM behavior. Callback
reentrancy, concurrent setters/getters, allocation failures, invalid pointers,
DXGI object metadata and arbitrary object types remain separate questions.
There is no Metal object implementation, GPU command completion, original C++
initialization or rendering evidence in this reference.

The current observer must still match these exact fields and the original source
hash. Change a contract only with separately measured evidence and an explicit
version/review; do not change this reference to make a Rust candidate pass.

The Rust CPU metadata vtable now executes this complete sequence through a
separate C++ host constructor, preserving this reference and its source. Native
WARP, hosted Rust and physical M4 x86-64 observations agree on `4ea9957`;
[CI 42357889974](https://github.com/daeken/Alkyhol/actions/runs/42357889974)
passed all thirteen jobs. This is inherited metadata behavior, not device/fence
or GPU implementation evidence.

## Native committed-buffer reference

`device-shared-buffers-windows-v1.json` preserves the successful native buffer
observation from `2b5bbe3` in CI 42361555862. That run exposed an implementation
mismatch: buffer alignment is 65536 and committed heap flags are 192, rather than
the caller's input zeros. The implementation was corrected; this reference was
not changed to make it pass. Live native WARP/hosted Metal comparison and physical
M4 observations now agree on `5048884`, with all fifteen jobs passing in
[42361819479](https://github.com/daeken/Alkyhol/actions/runs/42361819479).
The fixture records descriptors, heap properties, mapped CPU data, nonzero-address
checks and canonical parent identity; it does not prove native GPU allocation
internals, feature-level support, COM command execution or rendering.

The exact original probe bytes are archived as `probes/device_buffers_v1.cpp`,
with the frozen report's source hash still checked. The live probe changes only
its host-only negative queue check from DIRECT to unsupported VIDEO_DECODE as
real queues become available. Its native body and buffer observation sequence
remain unchanged. Live native/Metal comparisons must also equal the unchanged
frozen buffer observation, so two backends cannot jointly drift from it.

## Native queue/fence reference

`queue-fence-windows-v1.json` preserves actual native SDK/WARP execution on
`52df42e` in CI 42363080977, MSVC 19.51.36260, SDK 10.0.26100.0. CPU signals
and rewinds, event threshold waits, null-event waits, ordered queue signals and
parent/pending-submission lifetimes all agreed with Metal. The exact comparison
exposed one real mismatch: native GetDesc returns NodeMask 1 for requested 0.
Rust was corrected to match this measurement; the native report is unchanged.
The shared source hash remains checked and both live observations must equal the
frozen snapshot, without manufacturing descriptor fields or completion values.

The queue signal sequence contains no command lists or shader workloads. Native
WARP execution and actual empty Metal command-buffer completion support this
finite synchronization contract, not arbitrary GPU work, fault recovery,
cross-queue GPU waits, feature-level conformance or original C++ rendering.

## Native COM command-list copy reference

`command-copy-windows-v1.json` preserves actual native WARP execution on
`ffc1ec5` in CI 42364640363. It records all 2048 bytes from two real GPU-written
readback buffers, including untouched a5 sentinels, and eleven HRESULT/state
observations. The shared probe changes upload contents after recording, submits
one list, immediately resets that recording with another allocator, submits a
two-list batch and waits on a queue fence. CopyBufferRegion and CopyResource are
both exercised. The source hash and canonical observation SHA-256 are checked
separately from the implementation; all future live comparisons must also match
this unchanged native snapshot.

Downloaded native WARP, hosted Metal and physical M4 observations agree at the
same clean `ffc1ec5` revision. This is finite COM GPU-copy/state evidence, without
shaders, textures, barriers, native removal/fault handling, byte-granular copies,
feature-level conformance or an original rendered frame. The separate original
queue-manager subsystem caller runs locally on physical Metal and is not itself
a native Windows subsystem comparison.

## Native DEFAULT-buffer transition reference

`default-buffer-transitions-windows-v1.json` preserves native SDK/WARP execution
on `4c09ebc` in CI 42366061032. Its two DEFAULT buffers exercise explicit DIRECT
transitions, COMMON promotion/decay, CPU-fenced DIRECT→COPY→DIRECT handoff and a
two-list batch. All 4096 readback bytes, descriptor/heap fields, mapping failure
and three fence values agree with hosted Metal and the physical M4 from that
same clean revision. The runtime uses native Private storage with tracked blit
hazards; no CPU map or CPU copy substitutes for the DEFAULT workload.

Source identity and canonical observation SHA-256
`98868d79e4b04b6e2e76d35c4c06097f59b031622160f7d5860447ed4d081586`
are checked independently of the implementation. Live comparisons must continue
matching this unchanged snapshot. Invalid-state rejection, rollback, overlapping
queue rejection and injected GPU-failure poison remain separate local assertions;
this native positive workload does not establish those edge semantics, shader
execution, arbitrary barriers, full feature-level conformance or original frames.

## Native byte-copy reference

`byte-copies-windows-v1.json` preserves native SDK/WARP execution on `57e6321`
in CI 42367694508. The workload covers 272 offset-residue/tail cases, odd-sized
Private buffers and adjacent/overlapping partial writes. It retains all 39,601
smaller readback/guard bytes. A large copy crosses the 16-MiB utility dispatch
boundary; each executable asserts all 16,777,313 readback bytes, while the report
retains a checksum and three windows totalling 128 bytes. The latter is summary
agreement, not a raw comparison of the full large readback.

Native WARP, hosted Metal and physical M4 records agree at the same clean
revision. The physical run also enabled Metal API validation. Source identity
and canonical observation SHA-256
`c1016c4a93300ff0b85f43975a2e447fc9b8466b8455487d373ce4413f65868a`
are checked separately; future live pairs must also match this native snapshot.
The backend utility is open MSL memory-copy code, not general D3D12 shader or
rendering support. Original sample shader bodies and prior references are intact.

## Native descriptor-heap allocation reference

`descriptor-heaps-windows-v1.json` preserves SDK/WARP execution on `0ee8f58`
in CI 42369744013, with twelve type/visibility/node-mask cases. The first probe
had incorrectly equated nonzero GPU-handle bits with visibility. Native WARP
actually returned nonzero, non-sentinel GPU handles for CPU-only heaps too.
The expanded observer records that fact and diagnostics independently; the Metal
getter was corrected to expose its actual backing GPU addresses without implying
binding permission. Flags govern visibility. No frozen expectation was weakened.

Raw WARP strides are `[64,64,32,8]`; Metal reserves `[64,64,64,64]`. The comparator
excludes only those opaque driver stride values, preserving both raw reports,
and requires exact returned descriptions, handle presence/sentinel state,
indices computed using each actual stride and parent identity. Future live pairs
must also match this unchanged native invariant snapshot. This finite allocation
reference does not initialize descriptor views or establish shader binding/use.

Fresh native WARP, hosted Metal and physical M4 records agree on all preserved
invariants at clean `2aaf073` in CI 42370246611. The frozen native observation's
full canonical SHA-256, including its raw strides, is
`f492c53e6077155086010c76cf09953aa8ee48561e3073093d6b08cc69efa9c0`.
The GPU-handle presence/sentinel fields are still compared, not excluded to make
the implementation pass. Heap flags remain the authority for binding permission.


## Copied constant-buffer views

`cbv-copies-windows-v1.json` is the full native WARP report from CI 42371948288
on `b10a7df`, Windows SDK 10.0.26100.0. The same C++ source creates UPLOAD CBVs,
simple copies, overwrites original slots, stages unequal ranges, and copies
scattered unit ranges with NULL sizes. Native D3D12 consumes all four copied CBVs
through real root-signature/compute-pipeline/Dispatch calls using a tiny HLSL
consumer compiled with pinned open DXC. Its committed 4156-byte DXIL container
has SHA-256 `cf2694b10805c93623f4e73c0f58a4ed2faaed78a48933fd5967a2965be49aa0`;
CI regenerates that header exactly with the open compiler. This introduces no
proprietary or gated shader compiler.

All 72 readback words (16 data plus 8 guards in each of three cases) agree with
hosted Metal and separate physical M4/API-validation execution on that revision.
The complete observation's canonical SHA-256 is
`309c6fedf53fb0e3bf1915edda937b7b35d32bbb5ff87c400ebdf6cc169aa769`.
Future live pairs must also match this frozen payload. Reports preserve source,
fixture/generator hashes, executable hash, compiler, SDK, host and diagnostics.
The Metal consumer is an explicit diagnostic MSL reader of virtual descriptor
records. These finite observations establish copied contents/guards, not general
Alkyhol D3D12 binding/PSOs/Dispatch or identical-DXIL Metal translation. Resource
release while descriptors remain live is tested only as a local ownership
extension; the native probe retains resources. Original C++ rendering stays gated.
This run's producer and descriptor comparison jobs passed; its overall run had an
unrelated new test's Windows path-spelling failure, corrected in `d2208ed`.

The corrected run 42372084214 on `d2208ed` passed all twenty-one jobs. The strict
frozen-reference run 42372226929 on `a0ae65a` also passed all twenty-one jobs,
including every prior SDK/shader/graphics comparison and exact open-DXC fixture
regeneration. Downloaded native/hosted reports from `a0ae65a` match a fresh clean
physical M4/API-validation report and the frozen payload on all 72 words.


## Standalone version-1.0 root signatures

`root-signatures-windows-v1.json` preserves actual Windows SDK 10.0.26100.0/WARP
execution from CI 42373677759 on `5473a58`. The shared C++ observer serializes six
layouts through both legacy and versioned entry points, then creates actual root
objects and asserts capability probes, canonical identity, inherited metadata,
non-Pageable interface type and retained device lifetime. Complete 980-byte
container observations include empty roots, 64 DWORDs of constants, seven root
descriptors, APPEND/explicit offset tables, static sampler bits and stage-disjoint
registers. All bytes, including retail checksums and the two-block padding case,
agree with hosted Rust/Metal and clean physical M4 observations at that revision.
These are CPU layout/COM observations, not GPU use of the root layouts.

The full canonical observation SHA-256 is
`f0c59ed82431c950da3780d21b233c267d5b204cfc641ed5a67d7cc46fcdc3fe`.
Future live pairs must also match the frozen native payload. Source/executable
hashes, SDK/compiler, host, commands and diagnostics remain in the full report.
The unchanged original RootSignature.cpp also executes locally through the Rust
serializer/COM implementation, via a separate verified subsystem caller. This
original-helper execution has not been compared against its native C++ instance.

The root producers/comparison passed in that initial run. Its overall run caught
an older host ABI test which expected serialization to stay unavailable. Updating
that obsolete boundary adds success/blob/error checks; the independent Windows
root observer is unchanged. The strict-reference run 42373991380 on `28cf869`
passed all twenty-two jobs, including the corrected host ABI gate and every prior
SDK/shader/graphics/resource comparison. Fresh downloaded native/hosted reports
match clean physical M4 execution and the frozen 980-byte payload at that same
revision. The original subsystem v5 caller also passes cleanly there. See
[contract/open source provenance](../root-signatures.md) for all subset limits,
pinned DXC derivation and retained NCSA license. GPU binding/PSOs/Dispatch and
original C++ application rendering remain unavailable.


## Compute pipeline construction and COM assertions

`compute-pipelines-windows-v1.json` preserves actual Windows SDK/WARP execution
from CI 42375900712 on `ded5bbd` (all twenty-three jobs passed). The shared C++
source uses the same 4932-byte DXIL container with SHA-256
`e3084e25dfa23ef53e98eb1dd6eac386c3695718ac03d6631904b6711cc11d4e`.
It creates two real PSOs, observes S_FALSE capability checks and exercises
canonical/Pageable identity, metadata after discarding inputs and retained parent
lifetime. Its complete canonical observation SHA-256 is
`b8eefd69ed8e0166548fa8765e1b38467a4a52fee408ffc9f6c943351e8c6348`.
The full report retains compiler, SDK, executable/source/fixture hashes and
commands. Clean hosted Rust/Metal and physical M4 records agree at the same
revision on every common observation and shared fixture identity. Future live
pairs must also match the frozen native payload.

This probe measures construction/COM, not dispatch or GPU shader results after
input disposal. Rust's cached-blob method remains explicitly unavailable and is
not a native parity field. The separate physical/hosted Metal runtime unit
executes the COM-created pipeline through direct Metal, checking 76 data words
and 20 guards; that is separate from the earlier native same-DXIL compute
comparison and from D3D12 command-list execution. The original C++ ComputePSO
helper also runs locally with this owned fixture; original sample embedded-root
DXIL and production rendering remain outside this bounded API contract.
The first run 42375717669 failed strict MSVC compilation on an int-to-byte fill
in the new observer. `ded5bbd` changes only that fill value's exact type; no
runtime semantics or independent native acceptance check is weakened.

The strict-reference run 42376529216 on `6de9bd1` passed all twenty-three jobs,
retaining every prior comparison. Fresh downloaded native/hosted reports match
a clean physical M4 construction/COM report and the frozen payload at that same
revision. The original subsystem v6 caller also passes cleanly there. These
checks do not add native command-list dispatch or original-rendering evidence.


## COM compute dispatch / Private UAV readback

`compute-dispatch-windows-v1.json` preserves actual Windows SDK/WARP execution
from CI 42379450367 on `8e4c6b2`. The identical C++ source uses the established
4932-byte DXIL (`e3084e25dfa23ef53e98eb1dd6eac386c3695718ac03d6631904b6711cc11d4e`)
and submits actual command lists on DIRECT and COMPUTE queues, including
root-buffer offsets, initial/reset PSOs, binding snapshots, redundant root binding,
zero dispatch after real dispatch, per-resource UAV barriers and interleaved
aligned/one-byte copies. It checks and retains every word of six 416-byte
readbacks: 456 shader data words and 168 prefix/tail/suffix guards, 624 total.

The full canonical observation SHA-256 is
`263fe289e48e36cfe4745892f83e6b51f513fcf74a3fa79528ccb7a89ecbfaa9`.
Clean physical M4/API-validation execution at that revision matches every word,
state-sequence assertion and shared source/fixture identity. Future live pairs
must also match the frozen native payload; host-only negative/state/lifetime
extension tests cannot redefine it. The report retains SDK/compiler, executable,
source/fixture hashes, commands and diagnostics. Hosted Rust/Metal agrees as well. The initial run 42379450367 passed all
twenty-four jobs on `8e4c6b2`; the strict-reference run 42379882478 passed all
twenty-four on `667c15d`. Fresh downloaded native/hosted records and clean
physical M4/API-validation execution at that latter revision match all 624 words
and the frozen payload. The unchanged original subsystem caller also passes
cleanly at that same revision.

This is native WARP / physical Metal COM-dispatch evidence with an owned test
shader, not original MiniEngine HLSL or a full Sponza frame. The probe does not
claim Windows debug-layer validation or native negative-HRESULT/resource-lifetime
parity. Exact limits and intended primary synchronization references are in
[the dispatch contract](../compute-dispatch.md). All earlier frozen references
remain unchanged.
