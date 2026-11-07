# Original MiniEngine HLSL runtime

The `original/` files are byte-for-byte Microsoft sources at revision
`68a2b1e97f71642924e74c7757ca21756ad221b5`; `upstream.json` binds every upstream
path, length and SHA-256. Microsoft MIT notice: `../../licenses/Microsoft-MIT.txt`.
The header filename is cased `ModelViewerRaytracing.h` to match the original
HLSL includes on case-sensitive hosts; its bytes are unchanged.

These are explicit Alkyhol **source compatibility adapters**, compiled through
open DXC → DXIL → dxil-spirv → SPIRV-Cross → MSL. They specialize the pinned
sample's opaque triangles, fixed materials and primary/single-reflection passes;
they do not implement arbitrary shader tables, recursion or DXR library binaries.

- `bary.hlsl` calls unchanged primary `RayGen` and Bary `Hit`/`Miss` bodies,
  supplies actual native triangle queries, and queues hit records, reverse-Z depth
  and material normal output. `camera.hlsl` is the ray-only reference variant.
- `material.hlsl` includes exact extracted `DiffuseHitShaderLib.hlsl` helper and
  `Hit` bodies. An explicit adapter supplies builtins and actual nested shadow
  queries, routes expanded attributes in uint16-addressable chunks, and binds
  diffuse/normal/specular arrays of 27 original textures each. No geometry is
  dropped. `tools/prepare_sakura_shaders.py` verifies/extracts both sample packages.
- Existing port extensions remain explicit: invalid interpolated tangent frames
  are repaired from triangle positions/UVs; primary specular modes supply the
  specular texture mask and raster-style view direction. The adapter intercepts
  the six pinned `normalize` calls in `Hit`; a changed sequence fails a counted
  contract check. Diffuse-only and secondary hit shading retain the source view
  and direct specular mask. This is not a claim of full original raster parity.
- `reflection.hlsl` includes original SSR ray generation unchanged. It consumes
  actual projected depth and normal/reflectivity output, executes a native query
  and queues the secondary hit. The original entry does not consume its returned
  payload; original material or Bary shading executes in the following pass.
- `diagnostic.hlsl` and `diagnostic_present.hlsl` are Alkyhol-owned HLSL for the
  additional geometry/normal/depth views and diagnostic live scaling.

The source SSAO input is explicitly neutral (1), not an SSAO implementation.
The no-shadows mode supplies a real cleared reverse-Z depth comparison input;
there is no original shadow-map rendering path. The floor mask remains material
10 with original specular strength. Nonfinite output, unsupported nested payloads
and adapter-contract faults reject the frame. Prior manual MSL is an explicit
reference toggle only; it is never an automatic successful fallback.

Tone mapping, SDR display conversion, fullscreen vertices, copies and shaded live
upsampling compile unmodified `ToneMapCS.hlsl`, `PresentSDRPS.hlsl`,
`ScreenQuadPresentVS.hlsl`, `BufferCopyPS.hlsl`, `BilinearUpsamplePS.hlsl` and their
full include closure. The runtime replaces root signatures with reflected
bindings, supplies the original linear-clamp sampler, exposure 2 and zero bloom
strength, and selects typed UAV loads. Adaptive exposure and bloom remain absent.
Linear HDR/SDR uses RGBA32Float instead of packed RGB; RGB fragment outputs are
explicitly padded for Metal RGBA attachments and capture alpha remains opaque.

Generated JSON contains compiler-emitted MSL and authoritative binding metadata.
It is checked in so running an app needs no compiler download. Never edit generated
MSL. Rebuild or verify using the pinned compiler:

```sh
python3 tools/prepare_sakura_shaders.py --check
cargo run --manifest-path shader/Cargo.toml --locked --example translate_samples -- "$DXC"
cargo run --manifest-path shader/Cargo.toml --locked --example translate_samples -- "$DXC" --check
```

CI checks regeneration and every original source hash. On physical M4, full-scene
tests cover all five camera presets, geometry counts, shadow/reflection pixel
changes, finite output, repeat captures, resize and exact restoration after the
explicit reference toggle. Bary original/manual pixels and hit counts agree
exactly across all five presets. A separate 13×7 GPU test checks original
Stanard/sRGB output against independent scalar math (one UNORM8 code) and original
log-luminance (absolute tolerance 2e-6). Fixed texture-array selection has a real
nonuniform per-lane GPU test; the complete 81-texture material interface compiles
on physical RT Metal. These are Metal assertions, not native Windows DXR frames.
