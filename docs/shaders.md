# Direct3D shader translation to Metal

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
The binary API requires a bounded, structurally checked DXBC envelope containing
one DXIL program. Legacy SHDR/SHEX shader instructions, raw LLVM bitcode, embedded
root signatures, DXR/library/geometry/tessellation/mesh stages, descriptor arrays,
unbounded heaps, UAV counters, typed buffers and required auxiliary Metal buffers
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
compatibility. The existing Sakura and Sponza apps still use their manual MSL
ports; no automatic conversion of those full renderers is claimed.

## Executed local validation

On the physical Apple M4, all eleven shader integration tests pass:

- Four real Metal tests: exact compute words for overlapping register namespaces,
  separate spaces, a 64-byte cbuffer including a row-major matrix, shared-memory
  reversal/barrier, padded dispatch, integer wraparound and untouched tail;
  2×2 RGBA8 texture/sampler reads; exact 65×4 checker pixels; and interpolated
  vertex colours from Microsoft's byte-for-byte original HelloTriangle shaders.
- Five translation tests: reflection/workgroup metadata, original Microsoft
  vertex/pixel intake, every truncation of a real pixel DXIL container, corrupt
  headers, compiler errors/parameters and actual unsupported arrays/counters/
  geometry shader rejection.
- Two CLI tests: real HLSL JSON output and unsuccessful commands producing errors
  without success output.

The checker expectation and independent Windows SDK D3D12 reference are unchanged.
The colour interpolation check allows one UNORM8 code for quantization; compute
words, sampled colours, checker bytes and sentinel tail checks are exact. A
separate Python test verifies the original Microsoft shader SHA-256. All 30
Python tests and shader rustfmt/Clippy checks passed locally.

The CI shader job builds the pinned frontend and repeats these tests on hosted
Metal, recording that adapter separately. Its configuration alone is not an
executed result. Native Windows DXIL compute/full shaded DXR frame comparisons
remain unmeasured; existing portable Win32/native SDK and checker comparisons
continue through the conformance workflow.
