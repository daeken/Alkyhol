# Graphics subset: offscreen clear, triangle and resource copy

`alkyhol-d3d12` is a separate Rust crate using pinned wgpu 27.0.1 and pollster
0.4.0, with the CI-resolved transitive dependency lock checked in. It translates a restricted D3D12-shaped command sequence into real GPU
commands. macOS selects Metal, Windows selects DX12, Linux selects Vulkan. There
is no noop backend. A software adapter must be explicitly requested and is labeled
in the sample output; software Vulkan results are not Metal evidence.

```sh
cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
cargo run --manifest-path graphics/Cargo.toml --locked --example copy_resource
# Linux CI without a physical GPU:
cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle -- --allow-software
# Optional RGB image output:
ALKYHOL_PPM=triangle.ppm cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
```

The Rust facade exposes `D3D12CreateDevice`, `CreateCommittedResource`,
`CreateGraphicsPipelineState`, `CreateCommandList`, `ExecuteCommandLists`, and
`ReadbackRGBA8`. Command lists record transitions, ClearRenderTargetView,
DrawInstanced and CopyResource. This is a source API adapted to Rust; it does not export D3D12 COM
vtables or accept original D3D12 descriptors. Resource and pipeline tokens belong
to their device and remain allocated until the device drops. Future work should
add explicit destruction and native C/COM facades after contracts settle.

The supported resources are single-mip, single-sample, RGBA8 UNORM 2D render
targets. There is one direct queue. States advance at submission after validating
the entire list, including before-states and resource/pipeline ownership.
wgpu inserts physical barriers. The command-state validator and copy-footprint
calculation have dependency-free tests in the root crate.

Whole-resource copying supports distinct same-device RGBA8 2D textures with
identical width/height, one mip and one sample. Source must be CopySource and
destination CopyDest, with explicit transitions before later render/readback use.
Callers initialize source pixels through clear/draw/copy before observing them;
newly allocated texture contents have no promised value in this contract.
The copy preserves states and source pixels. Unsupported type/format reinterpretation,
resizing, partial/subresource copies, buffers, multi-adapter sharing and compressed,
depth or multisampled resources are not implemented. Resources are created with
real COPY_SRC/COPY_DST usage and the backend records a wgpu texture-to-texture copy;
there is no CPU copy substitute. All identity/state/extent checks run before any
list command is encoded or submitted, so a late invalid copy cannot execute earlier
clears or commit their logical transitions. These are the Rust facade's deliberate
rejection semantics; D3D12's void COM method and debug-layer errors are not emulated.
The contract follows the restricted valid subset of Microsoft's
[CopyResource documentation](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12graphicscommandlist-copyresource).

The sample first clears a 65×65 target blue and reads back **every** pixel. It
then draws a solid red triangle, checks center/corners and the allowed color set,
replays the list, and checks that rejected commands have no effect. Invalid shader
source must fail pipeline creation. Readback strips 256-byte row padding. Pixel
checks use exact endpoint colors away from edges, avoiding a global tolerance.

The `copy_resource` example separately renders a patterned 65×65 red triangle on
blue, captures its bytes, copies to a second texture, and compares every source
and destination byte. Real-device tests additionally chain two copies across
padded-width resources and reject a dimension mismatch after an earlier recorded
clear, verifying both actual pixels and destination state remain unchanged.
This sample is original project code guided by API documentation. It is not a
port of Microsoft's multi-adapter sample and has no native D3D12 comparison yet.

Shaders use explicit original WGSL with `vs_main` and `fs_main`, no vertex buffers
or bindings. Only three vertices and one instance are currently accepted. This
does not translate HLSL/DXIL. Raster tests establish the sample's behavior only;
there is no native D3D12 differential graphics reference yet. The graphics workflow
must actually build and execute before any backend is described as validated.

The new [native D3D12 reference scenario](graphics-reference.md) separately defines
a full-pixel checkerboard render/copy comparison against Windows WARP, Metal and
software Vulkan. Its executed results are recorded in the validation log. It does
not make the existing triangle example a native D3D12 differential reference.

Still missing: swapchains/presentation, windows, buffers/upload heaps, descriptors,
root signatures, fences, multiple queues, general shaders, compute, indirect draws,
resource lifetime APIs, residency, DXR and vendor features. No D3D feature level is
advertised. Backend allocation/device-loss errors can terminate a sample; they do
not become a successful compatibility result.
