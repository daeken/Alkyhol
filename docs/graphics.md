# Graphics milestone: offscreen clear and triangle

`alkyhol-d3d12` is a separate Rust crate using pinned wgpu 27.0.1 and pollster
0.4.0, with the CI-resolved transitive dependency lock checked in. It translates a restricted D3D12-shaped command sequence into real GPU
commands. macOS selects Metal, Windows selects DX12, Linux selects Vulkan. There
is no noop backend. A software adapter must be explicitly requested and is labeled
in the sample output; software Vulkan results are not Metal evidence.

```sh
cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
# Linux CI without a physical GPU:
cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle -- --allow-software
# Optional RGB image output:
ALKYHOL_PPM=triangle.ppm cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
```

The Rust facade exposes `D3D12CreateDevice`, `CreateCommittedResource`,
`CreateGraphicsPipelineState`, `CreateCommandList`, `ExecuteCommandLists`, and
`ReadbackRGBA8`. Command lists record transitions, ClearRenderTargetView and
DrawInstanced. This is a source API adapted to Rust; it does not export D3D12 COM
vtables or accept original D3D12 descriptors. Resource and pipeline tokens belong
to their device and remain allocated until the device drops. Future work should
add explicit destruction and native C/COM facades after contracts settle.

The supported resources are single-mip, single-sample, RGBA8 UNORM 2D render
targets. There is one direct queue. States advance at submission after validating
the entire list, including before-states and resource/pipeline ownership.
wgpu inserts physical barriers. The command-state validator and copy-footprint
calculation have dependency-free tests in the root crate.

The sample first clears a 65×65 target blue and reads back **every** pixel. It
then draws a solid red triangle, checks center/corners and the allowed color set,
replays the list, and checks that rejected commands have no effect. Invalid shader
source must fail pipeline creation. Readback strips 256-byte row padding. Pixel
checks use exact endpoint colors away from edges, avoiding a global tolerance.

Shaders use explicit original WGSL with `vs_main` and `fs_main`, no vertex buffers
or bindings. Only three vertices and one instance are currently accepted. This
does not translate HLSL/DXIL. Raster tests establish the sample's behavior only;
there is no native D3D12 differential graphics reference yet. The graphics workflow
must actually build and execute before any backend is described as validated.

Still missing: swapchains/presentation, windows, buffers/upload heaps, descriptors,
root signatures, fences, multiple queues, general shaders, compute, indirect draws,
resource lifetime APIs, residency, DXR and vendor features. No D3D feature level is
advertised. Backend allocation/device-loss errors can terminate a sample; they do
not become a successful compatibility result.
