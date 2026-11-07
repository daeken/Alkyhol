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
buffer shapes fail; other resource types/flags/heaps/alignment are unsupported.

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
