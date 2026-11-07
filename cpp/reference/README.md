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
