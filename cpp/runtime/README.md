# C++ COM runtime over Metal

This crate supplies a real Rust `CreateDXGIFactory2`, factory interfaces through
IDXGIFactory6, and IDXGIAdapter/IDXGIAdapter1 discovery over retained native Metal
devices. The root crate's `cpp-metal-factory` feature suppresses only its explicit
unavailable factory export; this combined static library owns the replacement.
Link this library by itself, not alongside a separately built root static library.

Implemented operations include canonical IUnknown identity, atomic references,
private-data ownership, indexed enumeration, opaque adapter LUID lookup,
unspecified-preference enumeration, descriptors, factory parent ownership and
current-inventory checks. The adapter retains its factory, which retains actual
Metal devices. New hardware snapshots determine IsCurrent; no dummy adapter is
created when Metal is unavailable.

All factory/adapter vtable slots have the SDK source signatures. Swap chains,
windows/outputs, stereo, registration, WARP/software adapters, presentation
features and explicit GPU-preference sorting fail explicitly. D3D12CreateDevice
still fails; no device, resource, queue, fence, shader execution or rendering is
advertised by adapter discovery.

## Descriptor mapping

The source ABI uses 32-bit WCHAR. Description is the actual Metal name, truncated
to 127 code points plus NUL. The opaque LUID encodes the actual 64-bit registryID;
it is not a Windows adapter LUID or a PCI address. Unknown PCI vendor/device/
subsystem/revision values remain zero. Adapter flags do not claim a software
rasterizer; hosted Metal may still be virtual rather than physical hardware.

DedicatedVideoMemory is the **virtual adapter's advisory Metal working-set budget**,
not physically dedicated VRAM, total RAM or guaranteed allocation capacity. It
comes directly from Apple's recommendedMaxWorkingSetSize property. Dedicated and
shared system-memory fields remain zero because no separate budget is known.
This explicit virtual budget also allows unchanged upstream adapter-selection
code to rank Metal adapters without manufacturing physical memory measurements.
See [Apple's definition](https://developer.apple.com/documentation/metal/mtldevice/recommendedmaxworkingsetsize).

Private-data methods compose the tested CPU store. Microsoft documents retained
interface ownership for [IDXGIObject](https://learn.microsoft.com/windows/win32/api/dxgi/nf-dxgi-idxgiobject-setprivatedatainterface);
new factory/adapter metadata round trips are host evidence. Their DXGI-specific
edge cases have not received an independent native Windows comparison; the prior
frozen reference is for ID3D12Object and is preserved unchanged.

```sh
cargo test --manifest-path cpp/runtime/Cargo.toml --locked
python3 tools/run_cpp_metal_factory.py --output local/cpp-metal-factory.json
python3 tools/build_original_miniengine.py --link-only
```

Original sample linking now selects this Metal COM library by default. The prior
root-only implementation remains an explicit `--runtime root` option for CPU
reference work; there is no automatic fallback. The original application still
exits 78 at its D3D12 device preflight, before original initialization.

[CI 42359772543](https://github.com/daeken/Alkyhol/actions/runs/42359772543)
passed all fourteen jobs on `c8475d0`. The hosted C++ probe enumerated Apple
Paravirtual device (no native Metal RT); the physical M4 probe enumerated Apple
M4. Each checked all vtable slots and parent/descriptor invariants against its
own actual Metal inventory. Neither is original C++ rendering evidence.

## Restricted device/resource milestone

`AlkyholCreateMetalDeviceSubset` is an explicit, separately named constructor for
real Metal-backed ID3D12Device/ID3D12Resource source-ABI work. It is not wired into
D3D12CreateDevice and makes no feature-level claim. The native API remains
unavailable until the runtime can satisfy its contract; original sample startup
therefore remains gated. The subset does not advertise Device5 or DXR capability.

The full typed device/resource vtables have 44/15 SDK slots. Supported behavior is
IUnknown/Object inheritance, naming/private data, one-node and adapter identity,
UPLOAD/READBACK committed buffers (normal properties, node masks 1, flags zero,
row-major, one subresource, initial GENERIC_READ/COPY_DEST respectively), coherent
Shared-storage Map/Unmap, by-value GetDesc, actual Metal GPU addresses, heap
properties and retained parent-device lifetime. Zero-width and invalid scalar
buffer shapes fail. The DEFAULT-buffer extension is described below; other
resource types/flags/custom heaps/alignment remain unsupported.

HRESULT operations that are unimplemented fail and clear interface outputs.
Unsupported void/struct getters with no meaningful error result abort with a
specific diagnostic instead of silently continuing with successful empty results.
The allocation-info getter uses the SDK's invalid-size sentinel. Options5 reports
no implemented DXR, render passes or tiled-resource extension; this is runtime
capability, independent of the physical Metal device's ray-tracing property.

A Rust integration unit fills the actual COM-owned shared buffer with a native
Metal blit command, waits for successful GPU completion and checks all 512 bytes
through Map. This is actual resource/backend evidence, not command submission
through COM, a shader/rendering test, feature-level conformance or game compatibility.
A shared C++ upload/readback probe records returned descriptors, heap properties,
CPU-visible data checksums, GPU-address presence and parent lifetime on Metal and
independent Windows SDK/WARP. [CI 42361819479](https://github.com/daeken/Alkyhol/actions/runs/42361819479)
passed all fifteen jobs on `5048884`: native WARP, hosted Metal and physical M4
observations agree on all returned buffer records. This adds no COM queue,
feature-level or original C++ rendering claim.

Native WARP observations normalize committed buffers to 65536-byte descriptor
alignment and ALLOW_ONLY_BUFFERS heap flags (192). The implementation preserves
these measured results with a genuinely aligned subspan of a larger Metal buffer;
Map, GPUVirtualAddress and backend operations use that common offset. Requested
width remains unchanged. The native record is frozen separately under
`cpp/reference/device-shared-buffers-windows-v1.json`. Native COM identity is
checked through canonical IUnknown round trips, without assuming all interface
pointers share one address.

## Restricted command queues and fences

The explicit subset constructor now supports typed 19-slot ID3D12CommandQueue
and 11-slot ID3D12Fence interfaces, retaining the device and composing the same
metadata store. DIRECT, COMPUTE and COPY queues accept normal priority, no flags
and a single node; other configurations fail. Fences accept no sharing flags.
GetDesc normalizes the default node mask 0 to 1, matching the independent WARP
measurement preserved in `cpp/reference/queue-fence-windows-v1.json`.
CPU Signal can advance or rewind the fence value. SetEventOnCompletion retains
a validated event reference, signals already satisfied waits immediately and
supports the null-event blocking form. Retained notifications cannot alias a new
handle after CloseHandle; that extra ownership contract is tested locally.

Queue Signal submits a real empty Metal command buffer and updates the fence
only from its completion callback. Submission and callback ordering preserve
queued rewinds; pending timeline state survives release of its COM shell. Failed
Metal completion records failure and uses the device-removed fence sentinel,
rather than reporting the requested value as completed. Fault handling is a host
assertion, not a native device-removal comparison. Queue objects and pending
completion state own their respective native/runtime lifetimes.

The queue/fence-only probe contains no command lists or shaders. GPU queue Wait
and timestamps/clock calibration return E_NOTIMPL; tiled mappings and marker/event
void methods abort with explicit diagnostics. Supported buffer-copy submissions
are described below; invalid/unsupported submissions abort separately. The
production D3D12CreateDevice and original application startup remain gated.

```sh
python3 tools/run_cpp_queue_fence.py --output local/cpp-queue-fence.json
```

The shared C++ probe observes CPU signals/rewinds, independent event thresholds,
null-event waits, actual queued completion, queued rewind order and parent/pending
submission lifetimes. Native WARP and Metal execute the same source through
different device constructors; CI compares exact outputs from a clean identical
revision. Local Metal execution alone is not native synchronization evidence or
original C++ rendering.

Native WARP and hosted Metal observations match the unchanged frozen queue/fence
contract on `30e7613`; [CI 42363319667](https://github.com/daeken/Alkyhol/actions/runs/42363319667)
passed all sixteen jobs. That run also preserves all prior buffer, metadata,
Windows SDK, shader and geometry comparisons.

After a full original C++ build, a separate original-subsystem caller also runs:

```sh
python3 tools/run_original_command_manager.py --output local/original-command-manager.json
```

It links the verified, unchanged original CommandListManager and
CommandAllocatorPool objects and exercises original creation of three queues,
high-bit fence routing, repeated increments/event waits, idle and shutdown.
This runs those original queue-management functions on physical Metal; it does
not invoke allocator creation, command lists or the original application's entry
point. Source/object/header changes require rebuilding before this caller runs.

## Restricted command allocator/list copies

The subset now supports ID3D12CommandAllocator and GraphicsCommandList0 (9/60
typed SDK slots). List creation/reset accepts DIRECT, COMPUTE or COPY, node 0/1,
a matching same-device allocator and a null initial pipeline. It implements
inherited identity/metadata/parent ownership, GetType, Close/Reset,
CopyBufferRegion and CopyResource. Queue execution accepts matching closed lists
and encodes real native Metal blits. Signals share that submission order and
cannot report completion before prior workload callbacks retire.

Copies accept UPLOAD/DEFAULT sources and DEFAULT/READBACK destinations, with
byte-granular offsets/lengths. DEFAULT state constraints and byte lowering are below.
Full-resource copies require matching buffer widths. It neither rounds nor drops byte-granular requests;
unsupported copies fail at Close and remain failed on Reset. No rendering state,
draw, dispatch, texture copy, bundle or indirect command is supported. The only
supported barriers are the bounded DEFAULT-buffer transitions described below.
Their void methods abort with a named diagnostic. Zero-list execution remains
explicit unsupported. Invalid/open/stale/duplicate submissions also abort,
rather than silently dropping work or claiming native device-removal behavior.

The allocator owns the recording/epoch reservation; closed list snapshots retain
actual native buffers until GPU completion. Reset installs a fresh recording,
while old submissions keep their prior operations. Pending reservations retire
before subsequent fence notifications. Reusing an allocator before its workload
finishes is defensively rejected; native D3D12 calls that case undefined behavior.
These restrictions follow the documented [allocator reset](https://learn.microsoft.com/windows/win32/api/d3d12/nf-d3d12-id3d12commandallocator-reset),
[list reset](https://learn.microsoft.com/windows/win32/api/d3d12/nf-d3d12-id3d12graphicscommandlist-reset)
and [submission](https://learn.microsoft.com/windows/win32/api/d3d12/nf-d3d12-id3d12commandqueue-executecommandlists)
contracts within the explicit subset; host assertions are not native observations.

```sh
python3 tools/run_cpp_command_copy.py --output local/cpp-command-copy.json
```

The original queue-manager caller now runs actual allocator creation/reuse and
COM GPU copies on all three original queues (v2 observation). Its original C++
implementation objects remain verbatim. This does not enable production device
creation, full original initialization or rendering.

Independent native WARP and hosted Metal copy observations agree byte-for-byte
on clean `ffc1ec5` in [CI 42364640363](https://github.com/daeken/Alkyhol/actions/runs/42364640363).
All seventeen jobs passed, including the existing native shader/SDK comparisons.
The separate physical M4 observation from that same revision also matches.
All 2048 destination/sentinel bytes and eleven state results are preserved in
`cpp/reference/command-copy-windows-v1.json`; the parser/comparator cannot replace
or normalize them to manufacture agreement. Other effects and fault semantics
remain outside this finite comparison.

## DEFAULT buffers and transition lowering

The subset also creates DEFAULT buffers in native Metal Private storage, with
tracked hazards and no CPU mapping. UPLOAD/READBACK retain coherent Shared storage.
Private buffers preserve the same actual 64-KiB-aligned GPU subspan and normalized
buffer descriptors. Map returns an error and clears its pointer; Unmap on DEFAULT
aborts instead of exposing Private contents or pretending a CPU map exists.

DIRECT lists accept complete transition barriers for DEFAULT buffers (subresource
0 or ALL_SUBRESOURCES). The supported state set is COMMON, COPY_DEST and read-only
subsets of GENERIC_READ. Split, aliasing, UAV, other resource classes and explicit
barriers on other queue types remain unsupported. Zero-byte DEFAULT copies are
unsupported. Shared-heap states are immutable.

State checks cover the entire ExecuteCommandLists batch atomically. COMMON can
promote for a copy; DEFAULT buffers decay at the batch boundary, never between
lists in one batch. A promotable BeforeState is accepted after decay, following
the [D3D12 buffer rules](https://learn.microsoft.com/windows/win32/direct3d12/using-resource-barriers-to-synchronize-resource-states-in-direct3d-12).
Failed preparation changes no scheduled state, and uncommitted reservations roll
back. Pending private-buffer ownership blocks overlapping work on another queue;
CPU fence completion allows handoff. GPU queue Wait remains unavailable.

Transition entries validate the D3D contract; native copy hazards are resolved by
the tracked-resource, directly bound `MTLCommandQueue`/blit path, as described by
[Metal resource synchronization](https://developer.apple.com/documentation/metal/resource-synchronization).
This uses the classic Metal queue API, whose synchronization contract differs
from MTL4CommandQueue. State validation is not a substitute for native GPU copies.

```sh
python3 tools/run_cpp_default_buffers.py --output local/cpp-default-buffers.json
```

The shared probe covers two private buffers, chained copies and transitions,
implicit promotion/decay, a CPU-fenced DIRECT→COPY→DIRECT handoff, and a two-list
batch. It checks all 4096 readback/sentinel bytes and records native returned
descriptors, mapping failure and fence values. Local failure injection and
cross-queue rejection are defensive host contracts, not native fault evidence.

Native WARP and hosted Metal observations match at clean `4c09ebc` in
[CI 42366061032](https://github.com/daeken/Alkyhol/actions/runs/42366061032), which
passed all eighteen jobs. The separate physical M4 record from that revision
matches every observed field and byte. The native snapshot and original source
hash are preserved under `cpp/reference/default-buffer-transitions-windows-v1.json`;
future live comparisons must also agree with it. This is buffer-workload evidence,
without a native rendered-frame, shader or feature-level claim.

## Byte-granular copy lowering

The macOS [buffer blit API](https://developer.apple.com/documentation/metal/mtlblitcommandencoder/copy(from:sourceoffset:to:destinationoffset:size:))
requires aligned offsets. Copies that cannot use the aligned blit path execute
the open `byte_copy.metal` utility through a real native compute encoder. It copies
individual bytes in bounded 16-MiB dispatch chunks, checks the final partial group
and never rounds a copy or overwrites neighbouring guards. Private storage stays
GPU-only. No CPU mapping/copy, original sample shader change or gated converter
is involved. Original HLSL rendering translation is unchanged.

Each device lazily caches the actual utility pipeline or its compilation error.
Encoder switches between consecutive blit/compute work preserve order. The kernel
source, runtime and executable hashes are included in host probe provenance.
This does not expose GraphicsCommandList::Dispatch or general shader pipelines;
those APIs remain unsupported. Zero-length DEFAULT and same-resource copies also
remain outside the subset.

```sh
python3 tools/run_cpp_byte_copies.py --output local/cpp-byte-copies.json
```

The shared workload contains 272 source/destination residue and tail cases,
4099-byte Private copies/overlays, mixed blit/compute operations and a larger
copy spanning two dispatch chunks. Small readbacks are preserved exactly. The
large case asserts every byte in each executable and records a checksum plus
three boundary windows; those summaries do not constitute a raw native comparison
of its entire large buffer. The original queue-manager caller now uses 257-byte
copies on all three original queues (v3), without changing original C++ objects.

Native WARP and hosted Metal observations agree exactly on clean `57e6321` in
[CI 42367694508](https://github.com/daeken/Alkyhol/actions/runs/42367694508), which
passed all nineteen jobs. The separate physical M4 run, with Metal API validation,
matches too. The native observation/source identity is frozen at
`cpp/reference/byte-copies-windows-v1.json`; future live comparisons must continue
matching it. Full original application initialization and rendering remain gated.
