# Bounded COM compute dispatch

The restricted Rust device now executes real compute work through original
D3D12 command-list calls: SetPipelineState, SetComputeRootSignature, the three
root buffer descriptor setters, Dispatch, ResourceBarrier, ExecuteCommandLists
and fence-synchronized readback. It uses the real open-translated Metal PSOs;
no diagnostic dispatch API or manual substitute shader drives this workload.
Production D3D12CreateDevice and original application rendering remain gated.

## Contract

DIRECT and COMPUTE lists support compute PSOs, including the initial PSO argument
on CreateCommandList/Reset. Reset clears compute roots and arguments. Setting the
same root object preserves arguments; a different root identity clears them.
Changing a compute PSO preserves arguments. Dispatch requires the exact supported
owned layout, an ALL-visible root CBV/SRV/UAV descriptor for every reflected
buffer, aligned non-null GPU addresses wholly inside registered same-device
allocations, and all required arguments initialized. CBVs require 256-byte and
SRVs/UAVs 4-byte address alignment. Readback heaps cannot be shader inputs; UAVs
require DEFAULT buffers with ALLOW_UNORDERED_ACCESS. Each group count is bounded
by 65535; a zero dimension is a checked no-work operation.

Each nonzero dispatch owns an immutable snapshot of the actual pipeline, native
buffer allocations, root offsets, reflection and group counts. Later setters and
COM shell release cannot rewrite that snapshot. Extra retained native allocation
lifetimes are a tested local extension, not a measured native resource-lifetime
claim. Invalid known recording operations make Close fail and preserve the first
error. COPY-list shader operations and the existing unknown void methods still
abort explicitly. Root descriptor tables, root constants, graphics bindings,
textures, indirect dispatch and embedded-root DXIL remain unavailable.

DEFAULT committed buffers now accept ALLOW_UNORDERED_ACCESS alone (flag 4).
UAV state 8 requires that flag; UPLOAD/READBACK remain fixed-state Shared buffers.
Private shader reads/writes join the existing transactional whole-batch state
validation, queue ownership, rollback and completion model. CBV/SRV require their
read states; UAV access requires UAV state. COMPUTE-list transitions accept the
bounded compute read/COPY/UAV subset. Per-resource UAV barriers retain the
resource and create an encoder boundary; global/null, split and alias barriers
remain unavailable. Only the active SDK barrier union member is read.

Classic MTLCommandQueue encoders bind tracked buffers directly. Encoder boundaries
preserve order across compute, blits and the unaligned GPU copy utility. This uses
[Apple's documented automatic resource synchronization](https://developer.apple.com/documentation/metal/resource-synchronization),
not Metal 4's untracked execution model. State promotion/decay and buffer rules
follow [Microsoft's resource-barrier documentation](https://learn.microsoft.com/en-us/windows/win32/direct3d12/using-resource-barriers-to-synchronize-resource-states-in-direct3d-12).
These references define the intended contract; the finite execution evidence is
listed below, not inferred from the documentation alone.

## Executed evidence

On physical M4/x86-64 with Metal API validation, the shared C++ probe performs
six readbacks across DIRECT/COMPUTE queues, initial PSOs and reset PSOs, root
rebinding snapshots, repeated identical root binding, zero dispatch after a real
dispatch, per-resource UAV barriers and interleaved aligned/one-byte copies.
Root CBV/SRV/SRV/UAV offsets are 256/16/8/16 bytes. Every full 416-byte output is
checked independently in C++ and retained as 104 words: 76 shader data words and
28 prefix/tail/suffix guards. All 624 observed words pass. No geometry, shader
quality or original asset is changed.

The shader is the unchanged established 4932-byte compute fixture, SHA-256
`e3084e25dfa23ef53e98eb1dd6eac386c3695718ac03d6631904b6711cc11d4e`.
Unlike the earlier PSO unit, this workload submits through the actual COM command
list/queue and reads back the real Private UAV buffers. It is an owned test shader,
not original MiniEngine HLSL or an original Sponza frame. Native Windows SDK/WARP
and hosted Rust/Metal comparisons are newly added to CI and remain pending.
Windows debug-layer validation is not claimed by this probe.

Seven new host/runtime units pass (65 total), covering identity/reset/snapshot
semantics, typed/missing/foreign/out-of-bounds bindings, zero and excessive group
counts, UAV flags/heaps, active-union reads, unsupported compute transitions,
whole-batch shader-state rejection and uncommitted rollback. Those host assertions
are separate from GPU/native observations. Existing frozen Shared/Private/byte
copy and CBV-copy probes still match their native payloads locally. Clippy and
formatting, all 118 Python tests and the unchanged original subsystem v6 caller
pass. The complete original application relinks and still exits 78 at production
device creation, before original initialization. All original source/shader
headers, five original helper objects, working Rust samples and assets remain
unchanged. Full original rendering and game compatibility are not established.

```sh
cargo test --manifest-path cpp/runtime/Cargo.toml --locked
python3 tools/run_cpp_compute_dispatch.py --output local/compute-dispatch.json
python3 tools/run_original_command_manager.py --output local/original-subsystems.json
```
