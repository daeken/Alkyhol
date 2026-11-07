# Direct3D shader translation to Metal

Project constraint: the shader translation components must be open source and
freely available without an account-gated proprietary compiler. Apple's Metal
Shader Converter is deliberately excluded. Native Metal remains the host API.

The `alkyhol-shader` Rust crate and CLI take **compiled DXIL containers** to native
Metal Shading Language. HLSL source uses Microsoft's DXC frontend first:

```text
HLSL → DXC → DXIL → dxil-spirv → SPIR-V → SPIRV-Cross → MSL → Metal
              ↑
       existing DXIL input
```

SPIR-V is an intermediate representation, with no Vulkan runtime or wgpu backend.
The Rust API owns validation, compiler invocation, resource mapping and returned
runtime metadata. Compiler cores currently use C++ through Rust bindings; this is
not a pure Rust compiler rewrite. [Pins, licenses and sample provenance](../shader/UPSTREAM.md)
are recorded separately.

## Running

Build prerequisites are Rust, CMake, Ninja and Clang/libclang. On macOS, Xcode's
Metal compiler/runtime is needed for real GPU tests. Build pinned DXC locally:

```sh
python3 tools/build_dxc.py
export DXC="$PWD/local/dxc-build/bin/dxc"
cargo test --manifest-path shader/Cargo.toml --locked -- --nocapture
cargo run --manifest-path shader/Cargo.toml --locked -- \
  hlsl "$DXC" shader/tests/fixtures/compute.hlsl CSMain cs
cargo run --manifest-path shader/Cargo.toml --locked -- \
  dxil /path/to/public-shader.dxil CSMain
```

Both CLI paths print JSON containing generated MSL, its actual entry point, stage,
fixed compute thread-group dimensions and resource binding metadata. DXC is an
explicit executable path; an installed compiler may be passed instead of the
pinned bootstrap, but results then belong to that compiler. No source rewriting,
manually written MSL fallback or DXC direct-to-SPIR-V bypass is used.

## First contract

Vertex, pixel and compute shaders use SM6.0 profiles through the HLSL frontend.
`compile_hlsl_with_options` explicitly selects SM6.5, preprocessor definitions,
or root-signature stripping for runtimes that supply reflected discrete bindings.
Inline ray queries now translate acceleration-structure resources into Metal's
buffer namespace; `requires_ray_query` reports that capability in the output.
The binary API requires a bounded, structurally checked DXBC envelope containing
one DXIL program. Legacy SHDR/SHEX shader instructions, raw LLVM bitcode, embedded
root signatures, DXR/library/geometry/tessellation/mesh stages, buffer/sampler
descriptor arrays, unbounded heaps, UAV counters, typed buffers and required auxiliary Metal buffers
are explicitly rejected in this milestone. A compile error remains an error.

Constant buffers use uniform storage. Raw/structured buffers use storage buffers,
with four-byte alignment and no descriptor suballocation/offset emulation.
Textures and samplers retain their native Metal resource kinds. Each D3D binding
maps to SPIR-V set `space * 4 + class` (`b=0,t=1,u=2,s=3`) and binding `register`.
This keeps `b0`, `t0`, `u0`, `s0` and other register spaces distinct. Sorted D3D
bindings receive dense indices independently in Metal's buffer, texture and
sampler namespaces. The returned metadata is authoritative; callers must not use
the original register number as the Metal slot. Limits are 24 buffers (leaving
slots for vertex inputs/future helpers), 128 textures and 16 samplers.

Vertex metadata preserves original semantic/index, signature row and row count.
The caller supplies a matching Metal vertex descriptor. This first milestone
exercises compatible vertex/pixel signatures; it does not implement a general
semantic linker or D3D12 root-signature/descriptor-heap runtime. Successful
translation is not proof of arbitrary shader semantics, DXR support or game
compatibility. Fixed texture descriptor arrays now report their element count
and reserve contiguous Metal texture slots; overlapping D3D ranges and exhausted
slot budgets fail. Repeated remapper callbacks must agree on kind, namespace,
array count and alignment; conflicting declarations at the same register fail.
Once a remapper fault occurs, later callbacks cannot add accepted bindings or
replace the original fault. Nonuniform per-lane selection is tested on actual Metal.

Sakura uses original HLSL function bodies through an explicit
[bounded SER-off adapter](../shader/samples/sakura/README.md). Sponza's default
path uses original primary/hit/miss/material/SSR bodies through explicit
[source adapters](../shader/samples/miniengine/README.md), plus original
tone/display/fullscreen shaders and their include closure. Port-specific tangent
repair, primary specular/view extensions and diagnostic HLSL are identified
separately. No full DXR binary translation is claimed. Native Metal compilation
and runtime remain supplied by the host platform.

`translate_dxil_with_options` additionally accepts explicit fragment-output
component counts. This is needed when an original RGB shader targets a Metal
RGBA attachment. Generated metadata records the requested attachment components;
invalid stages/counts/locations/duplicates fail. Shader source remains unchanged.

## Executed local validation

On physical Apple M4, all 24 shader tests pass: five host remapper unit tests
and 19 integration tests, including real compute/raster
execution, fixed texture arrays with nonuniform per-lane indexing, the original
81-texture material interface, original Sakura helper/budget contracts, original
HelloTriangle interpolation, exact checker/copy pixels, malformed/unsupported
DXIL, reflection, attachment contracts and CLI failures. All 40 Python tests
pass, including original-source hashes and exact extraction. Formatting, Clippy
and checked-in artifact regeneration also pass. Physical full-scene tests are
recorded separately in [executed validation](validation.md).

The checker expectation and independent Windows SDK D3D12 reference are unchanged.
Colour interpolation permits one UNORM8 code; compute words, sampled texture-array
colours, checker bytes and sentinel tails are exact. Hosted Metal repeats compute/
raster tests but lacks RT; physical RT interface tests are explicitly ignored
there. Native Windows DXIL compute has the finite same-container evidence below;
full shaded DXR frame comparison remains unmeasured. Existing portable Win32/native
SDK, MSVC geometry/placement and checker comparisons continue through the
conformance workflow.

## Identical-DXIL native compute reference

The finite `dxil-compute-v1` probe executes the same compiled container on Metal
and on an independent Windows SDK/D3D12 WARP program. It reuses the existing
Alkyhol compute fixture unchanged: overlapping register numbers, a separate
register space, a 64-byte cbuffer/row-major matrix, group-shared reversal/barrier,
integer wraparound, a padded final group and an untouched output tail. A separate
Python scalar contract checks every word; the comparison also requires matching
DXIL SHA-256 and a clean matching source revision. Compiler/executable/source
hashes and actual adapter identity accompany each observation.

```sh
# macOS: build the public fixture using the explicitly selected frontend, then
# translate and execute its DXIL. The generated DXIL is the Windows input too.
python3 tools/run_shader_compute.py --dxc "$DXC" \
  --dxil local/compute.dxil --output local/shader-compute-Metal.json
# Windows Developer Command Prompt: no DXC download or Alkyhol linkage needed.
python tools/run_shader_compute.py --dxil compute.dxil --output shader-compute.json
```

The conformance workflow now transfers that DXIL from the shader job to a native
Windows job, then compares all 96 words, including 20 sentinel words. Physical M4
passed the probe locally. Native Windows WARP and hosted Metal
then agreed on all 96 words from identical DXIL in
[CI 42347852341](https://github.com/daeken/Alkyhol/actions/runs/42347852341)
on `3d866ad`; all twelve jobs passed. Full shaded DXR frame comparison is outside
this finite compute contract. Synthetic validator/process-failure tests do not
count as GPU evidence.

Host remapper tests separately cover adjacent and overlapping descriptor intervals
in both insertion orders, independent register spaces/classes, idempotent and
conflicting duplicate callbacks, maximum representable registers/spaces, range
wraparound, zero/unbounded/oversized ranges, unsupported arrays/counters/kinds,
and preservation of the first fault. These synthetic callback tests exercise Rust
validation; they are not additional native shader/GPU observations. Checked-in
Sakura and MiniEngine shader regeneration remains byte-for-byte unchanged.

A real DXC/DXIL translation test additionally accepts an array using exactly 128
texture slots with a separate buffer UAV, then rejects the same 128 SRVs plus a
texture UAV because both consume Metal's shared texture namespace. This checks
the translator's allocation contract; the 128-texture boundary is not a new
native GPU execution observation.
