# Graphics subset over direct Metal

The graphics crate is Rust over Apple's Metal API using pinned metal-rs 0.32.0.
wgpu, Naga, WGSL and the Vulkan backend are no longer dependencies. Historical
wgpu/Metal/Vulkan measurements remain bound to their original Git revisions in
the validation log. Current graphics execution requires macOS with a real Metal
device; Windows SDK reference programs remain independent.

    cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
    cargo run --manifest-path graphics/Cargo.toml --locked --example copy_resource
    cargo test --manifest-path graphics/Cargo.toml --locked -- --nocapture
    python3 tools/run_graphics_contract.py --output local/graphics-contract.json

The Rust source facade retains D3D12CreateDevice, CreateCommittedResource,
CreateGraphicsPipelineState, CreateCommandList, ExecuteCommandLists and
ReadbackRGBA8. Shader source is now native MSL with vertex vs_main and fragment
fs_main. No COM descriptors, guest calling conventions or HLSL/DXIL translator
are implied. The compatibility constructor retains its old boolean argument but
rejects software-backend requests. Native Metal device/queue accessors support
compute, presentation and hardware ray-tracing extensions for the Sakura target.

The supported source-facade resources are single-mip, single-sample RGBA8 UNORM 2D
textures, conservatively capped at 16,384 pixels per axis and the device's maximum
buffer length. Resource/pipeline tokens belong to one device and stay alive until
it drops. States are explicit RenderTarget, CopySource and CopyDest. The root
validator checks whole lists, IDs, before-states and supported draw shape before
any GPU work is encoded. Every copy extent is also checked in advance. A late
invalid command cannot run earlier clears or commit logical transitions.

Metal render encoders implement clear and triangle draw; blit encoders implement
whole-resource texture copies and padded readback to a shared buffer. There is no
CPU pixel-copy substitute. Metal tracks physical resource hazards. The initial
source-facade submission blocks for actual GPU completion, checks command-buffer
status, then commits logical states. This conservative behavior has extra latency;
it does not emulate D3D12 asynchronous queues/fences. Native backend access allows
the dedicated ray-tracing renderer to manage its own asynchronous frame commands.

CopyResource accepts distinct, same-device textures of matching width/height.
Source must be CopySource and destination CopyDest; source pixels and both states
are preserved by the copy. Callers initialize source pixels before observing them;
newly allocated pixels have no promised value. Type/format reinterpretation,
resizing, partial/subresource copies, buffers, multisampling, compressed/depth
formats and cross-adapter sharing are unsupported. The deliberate Rust error
results are distinct from D3D12's void COM methods and debug-layer errors.

The unchanged finite [native D3D12 pixel contract](graphics-reference.md) compares
actual Windows WARP with direct Metal. Each surface contributes all 1,040 RGBA8
bytes of a 65x4 multichannel pattern, without tolerance or checksums. Separate GPU
tests cover ownership, rejected lists, width/height mismatch and chained copies.
The earlier triangle and copy_resource examples still have their own host
assertions; the checkerboard comparison does not certify every raster scenario.

[Sakura Forest](sakura.md) is the next complete application target: faithful
upstream meshes, textures, instances and shading, native Metal acceleration
structures and ray intersectors, then an interactive window. Explicit SER support
is a separate capability. No D3D feature level, general DXR compatibility or game
compatibility is advertised. Allocation/device/command failures remain failures.
