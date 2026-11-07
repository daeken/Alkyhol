# Alkyhol

A Windows compatibility layer in Rust, with a D3D12-shaped graphics subset using
direct Metal from Rust. Source compatibility supports portable Win32 programs and
recompiled/decompiled application code. A first [binary execution path](docs/pe.md)
also runs unchanged Windows PE64 programs through Win64 ABI bridges in an
x86-64 Mach-O host under Rosetta.

This is an early implementation, not a Wine replacement or a claim that Cyberpunk
2077 runs. Source callers use the host ABI; PE callers use explicit binary ABI
bridges and checked image mappings. General Windows process startup is incomplete.

The first [off-the-shelf Windows program](docs/u-config.md) is the released
u-config/pkg-config executable from w64devkit. Its original machine code handles
package queries, dependency flags, directory listing and quoted Unicode arguments
through Alkyhol under Rosetta.

## Evidence first

The PE scanner inventories a locally supplied executable without executing it.
Executable bytes and game-derived inventories stay in ignored `local/` storage.
Regenerate locally with:

```sh
mkdir -p local
python3 tools/pe_imports.py /path/to/Cyberpunk2077.exe --output local/imports.json
```

The inventory covers normal and delay imports. Dynamic lookup, COM methods and
transitive DLL dependencies need separate evidence. See [scope](docs/scope.md).

## Development

Automatic shader translation now has a first tested milestone:
**HLSL → DXIL → SPIR-V → MSL**, with direct compiled-DXIL input, Rust binding
metadata and real Metal compute/raster tests. See [shader translation](docs/shaders.md)
for commands, supported contracts and current limits. Both Sakura and MiniEngine
Sponza now use original HLSL function bodies for their default ray-traced rendering,
including material shading, shadows/reflections and Sakura's sky. MiniEngine's
tone/display/fullscreen passes use unchanged original HLSL too. Explicit bounded
source adapters supply the native Metal ray-query and resource interfaces; this
is not general DXR library translation. All translation components are open source.

The core is dependency-free so file/event conformance work can run without a GPU
or a network connection. Graphics has its own manifest and dependency resolution.

```sh
cargo test --offline
cargo build --offline
python3 -m unittest discover -s tests -v
```

The [kernel contracts and C sample instructions](docs/contracts.md) describe the
implemented exports. `python3 tools/run_samples.py` builds and checks the portable
C probe against the native shared library. CI also compiles that same source
against the Windows SDK and compares the observations. Passing portable
samples demonstrates only those tested contracts, not game compatibility. Native
Windows comparison results must be labeled separately from host-only assertions.

The runner also executes five [Microsoft sample adaptations](samples/msdn/README.md):
synchronous file writing with readback and four readers gated by a manual-reset
event, twelve workers sharing a ten-permit semaphore, and per-thread data retrieved
through a shared TLS index, plus parameterized CreateThread callbacks and exit-code
queries. Their upstream revision,
license and deliberate scope changes are recorded.
See the [test map](docs/testing.md) for function and logical-unit coverage.

## Graphics

[MiniEngine Sponza](docs/miniengine.md) now renders original material/normal maps,
native shadow rays and floor reflections, with Bary Rays and geometry/normal/depth
diagnostics available alongside the shaded views.


[Sakura Forest](docs/sakura.md) is the concrete application target: a faithful
Rust/Metal port of Microsoft’s ray-traced sample using its original scene and
assets. The graphics crate no longer depends on wgpu.

```sh
python3 tools/fetch_sakura.py
cargo run --manifest-path sakura/Cargo.toml --locked --release
```

The port requires a Metal ray-tracing device. It runs the complete SER-off scene;
explicit DirectX SER scheduling is separate. See the Sakura instructions for
controls, captures, the local app bundle and physical-device tests.

The [offscreen graphics milestone](docs/graphics.md) provides a restricted
D3D12-shaped Rust API over direct Metal with a clear/triangle/readback sample:

```sh
cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
cargo test --manifest-path graphics/Cargo.toml --locked -- --nocapture
cargo run --manifest-path graphics/Cargo.toml --locked --example copy_resource
```

The dependency-free root tests validate command lifecycles, state transitions,
rejected submissions and copy footprints. GPU execution is checked separately;
a missing device, missing dependencies or a skipped run never counts as a passing
GPU result.

See [executed validation](docs/validation.md) for host/backend identities, actual
CI results and the limits of the current sample evidence.
The [native D3D12 scenario](docs/graphics-reference.md) defines an independent
Windows WARP render/copy reference and exact full-pixel comparison with Alkyhol.
