# Original MiniEngine HLSL: first runtime migration

The `original/` files are byte-for-byte Microsoft sources at revision
`68a2b1e97f71642924e74c7757ca21756ad221b5`; `upstream.json` binds each upstream
path, length and SHA-256. Microsoft MIT notice: `../../licenses/Microsoft-MIT.txt`.
The header filename is cased `ModelViewerRaytracing.h` to match the original
HLSL includes on case-sensitive hosts; its bytes are unchanged.

`bary.hlsl` is an explicit Alkyhol **source compatibility adapter**, compiling
unchanged `RayGen`, `Hit` and `Miss` bodies into a compute shader. Dispatch ID and
hit-distance builtins are supplied as per-invocation state; actual native Metal
triangle traversal comes from HLSL RayQuery → DXIL → SPIR-V → MSL. The fixed scene
uses opaque triangles, one hit group and miss shader. This adapter does not
implement arbitrary shader tables, recursion or full DXR library translation.

`camera.hlsl` calls unchanged `RayGen` to enqueue its primary ray. This is a
specialized split of that particular entry (which does not read the returned
payload); the next pass performs actual traversal. All Sponza modes consume the
original HLSL-generated primary rays. The shaded modes still use manual MSL for
materials and shadow/reflection hit logic, while view 2 uses original hit/miss.

Tone mapping, SDR display conversion, fullscreen vertices, diagnostic copies and
shaded live upsampling now compile the unmodified `ToneMapCS.hlsl`,
`PresentSDRPS.hlsl`, `ScreenQuadPresentVS.hlsl`, `BufferCopyPS.hlsl` and
`BilinearUpsamplePS.hlsl`, together with their full original include closure.
The runtime explicitly replaces their root signatures with reflected bindings,
supplies the original linear-clamp sampler, exposure 2 and zero bloom strength,
and selects the typed-UAV-load path. Adaptive exposure and bloom remain absent.
Linear HDR/SDR uses RGBA32Float instead of the original packed RGB framebuffer;
RGB fragment outputs are explicitly widened for Metal RGBA attachments and the
host preserves an opaque alpha. No edits to source or generated MSL implement this.

The generated JSON files contain compiler-emitted MSL and authoritative binding
metadata. They are checked in so running an app does not need a compiler download.
No edits to generated MSL are allowed. Rebuild or verify with the pinned compiler:

```sh
cargo run --manifest-path shader/Cargo.toml --locked --example translate_samples -- "$DXC"
cargo run --manifest-path shader/Cargo.toml --locked --example translate_samples -- "$DXC" --check
```

CI checks regeneration, and Python verifies all original source hashes. The
physical M4 additionally ran a full-scene A/B comparison at 129×73 for each of
five camera presets: exact RGBA8 pixels, per-mesh nearest-hit counts and misses
agree with the manual intersector reference using the same original camera rays.
This is internal Metal agreement, not a native Windows DXR frame comparison.
An additional 13×7 real-GPU compute/raster test compares tone-mapped/display RGB
to an independent scalar curve (one UNORM8 code for quantization) and the original
log-luminance output (absolute tolerance 2e-6). The physical M4 passed all 15
MiniEngine and 14 shader tests. Hosted Metal does not supply RT evidence.
