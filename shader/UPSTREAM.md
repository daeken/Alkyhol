# Shader dependencies and source provenance

The Alkyhol API, resource mapping, CLI and tests are Rust. The compiler cores are
C++, exposed by Rust bindings. No wgpu or Vulkan runtime is used.

| Component | Pin | License / source |
| --- | --- | --- |
| Microsoft DXC | v1.8.2505.1, `b106a961d09221b3c5bdb37be45b679257da08b8` | LLVM release license / MIT components; [upstream notices](https://github.com/microsoft/DirectXShaderCompiler/blob/b106a961d09221b3c5bdb37be45b679257da08b8/LICENSE.TXT) |
| dxil-spirv Rust bindings | 0.1.0+dxil-spirv.2.72.1 | MIT OR Apache-2.0; vendored [dxil-spirv](https://github.com/HansKristian-Work/dxil-spirv), MIT |
| spirv-cross2 Rust bindings | 0.7.2 | MIT OR Apache-2.0; vendored [SPIRV-Cross](https://github.com/KhronosGroup/SPIRV-Cross), Apache-2.0 |

`Cargo.lock` pins the native bindings/transitive source packages and checksums.
`tools/build_dxc.py` builds the exact DXC source revision into ignored `local/`.
The macOS build disables the new Clang `invalid-specialization` diagnostic for
older LLVM code; it makes no edits to the dependency. DXC's existing upstream
submodule revisions remain pinned by that commit.

`tests/fixtures/microsoft-hello-triangle.hlsl` is an **unmodified byte-for-byte**
copy of Microsoft's MIT-licensed
[HelloTriangle/shaders.hlsl](https://github.com/microsoft/DirectX-Graphics-Samples/blob/68a2b1e97f71642924e74c7757ca21756ad221b5/Samples/Desktop/D3D12HelloWorld/src/HelloTriangle/shaders.hlsl).
SHA-256: `c300fc6586b8e0410ea3df4c00d080efeeeeac7e0e2485a84864fae118f6597e`.
Its Microsoft copyright/license header and original CRLF bytes are preserved.
The full MIT notice is in `licenses/Microsoft-MIT.txt`.

Other fixtures are original Alkyhol MIT-licensed contract probes. `checker.hlsl`
implements the existing independent native D3D12 checker scenario; the native
reference and frozen pixel oracle remain unchanged. Public generated DXIL may be
used for tests; proprietary game shaders/captures stay local.

The [MiniEngine shader package](samples/miniengine/README.md) also records the
complete unmodified source/include closure for its migrated ray-generation,
hit/miss, tone, display and fullscreen shaders. Its `upstream.json` binds every
original file to the same pinned Microsoft source revision and exact SHA-256.

The [Sakura package](samples/sakura/README.md) preserves full original HLSL/header
files and extracts exact function bodies for a bounded two-level SER-off adapter.
Microsoft MIT and Pablo Roman Andrioli Star Nest notices accompany the source.
This specialization uses open inline-query compilation, not Apple's proprietary
converter or an ignored SER hint.
