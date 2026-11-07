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
