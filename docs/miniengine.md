# MiniEngine Sponza

The next application target is Microsoft's
[D3D12RaytracingMiniEngineSample](https://github.com/microsoft/DirectX-Graphics-Samples/tree/68a2b1e97f71642924e74c7757ca21756ad221b5/Samples/Desktop/D3D12Raytracing/src/D3D12RaytracingMiniEngineSample),
pinned to the same upstream commit as Sakura. The user selected it as a less
cluttered debugging scene, then requested textured shading, shadows and reflections.
The first running milestone is complete Sponza geometry with the original primary
Bary Rays path and additional explicitly labeled geometry/normal/depth diagnostics.
Textured material lighting, native shadow rays and floor reflections are now implemented; the original raster/shadow-map and temporal postprocessing paths remain separate.

```sh
python3 tools/fetch_miniengine.py
cargo run --manifest-path miniengine/Cargo.toml --locked --release
python3 tools/build_miniengine_app.py
```

The bundle is `local/miniengine/MiniEngine Sponza.app`. Controls:

- **1**: textured lighting and shadow rays; **G**: neutral geometry diagnostic.
- **2**: original primary Bary Rays shading `(barycentric.x,barycentric.y,1,1)`.
- **3**: reflected hit barycentrics on reflective floor pixels.
- **4**: binary shadow visibility; **5**: textured lighting without shadows.
- **6**: diffuse with shadow rays (upstream ray-hit lighting model).
- **7**: textured lighting and real floor reflection rays (default).
- **N/Z**: added interpolated normal and depth diagnostics.
- WASD move; Q/E down/up; arrows or left-drag look; PageUp/PageDown cycle the five
  original camera presets; R reset; Escape close. The title names the active view
  and the shortcuts. Source preset literals are retained exactly.

```sh
cargo run --manifest-path miniengine/Cargo.toml --locked --release -- \
  --headless --mode geometry --camera 2 --capture local/miniengine/geometry.png
cargo test --manifest-path miniengine/Cargo.toml --locked -- --include-ignored --nocapture
```

The loader reads the original `Sponza/sponza.h3d`: 34 meshes, 27 materials and
278,919 triangles. It retains original triangle order, local uint16 index semantics,
positions, normals, UVs and tangent-frame bits. Serialized SIMD padding is explicit:
Header 64, Mesh 336, Material 992 bytes, full vertex stride 56. Bounds, lengths,
layouts, strings, local indices and geometry values are checked before GPU access.
Asset/source SHA-256 identities are in `miniengine/upstream.json`; public assets
remain in ignored `local/`. Sponza credits Frank Meinl/Crytek, Marko Dabrovic,
Morgan McGuire and PBR texture author Alexandre Pestana; upstream attribution is
preserved under `miniengine/licenses/`. No game data is used.

Some original tangent/bitangent attributes contain NaNs. Those raw bits are
preserved, not silently replaced; the current geometry/Bary paths do not read them.
The lit path repairs invalid/zero tangent frames from the original triangle's
position/UV derivatives. Degenerate UVs use a defined orthonormal frame instead.
Valid source frames remain intact; repairs are counted. This is an explicit port
policy, not a measured native Windows repair. Position, UV and vertex-normal
nonfinite values are rejected.

The reusable Metal scene builder now allows each instance to disable culling or
honor ray culling. Sakura retains disabled culling. MiniEngine enables ray-controlled
culling and explicitly selects backface culling, clockwise front winding and
closest-hit traversal, matching the primary source path's intent. Physical tests
cover two meshes with far-first/near-second instances, closest hits from both sides
and culling with/without the instance override.

Local tests cover parser corruption/truncation, SIMD layout, original mesh counts,
preserved NaN bits, camera conventions, CLI rejection of unimplemented modes,
repeatable actual GPU captures, view-independent hit counts and resizing. Physical
RT and pinned-asset tests are ignored by default and must be explicitly invoked.
CI runs parser/CLI/asset tests on hosted Mac; that GPU lacks RT. The independent
native MSVC ABI/geometry reader and Rust reader export every expanded indexed
vertex attribute byte for hash comparison. This checks model intake and ABI,
not original native DXR rendering or full MiniEngine behavior.

## Textured lighting, shadows and reflections

The asset manifest now includes 50 original diffuse/normal/specular DDS files.
BC1 blocks and every source mip level upload unchanged. Diffuse/specular BC1 sRGB
and normal-map linear formats are retained. The exact ModelH3D missing-slot
choices are recorded: white diffuse, flat normal, black specular. A manifest-listed
missing/corrupt texture fails intake; it never silently becomes a placeholder.
The generated HLSL material pipeline binds three fixed arrays of 27 textures
through reflected Metal slots, without an atlas or texture resampling. The prior
manual-MSL reference retains its Tier 2 argument buffers.

Lighting ports the directional sun, ambient intensity, normal mapping, derivative
texture sampling, Schlick modulation and specular model. Mode 6 retains the source
ray-hit diffuse model, including zero direct specular mask; the shaded material
and reflection-base modes also use original specular textures. Primary rays use
closest hit/backface culling; shadow rays use native any-hit, no culling and TMin
0.1. Floor reflection rays use the actual normal-mapped normal, the source's 0.1
ray-origin offset and a single secondary hit. The floor mask is explicitly material
10 with its specular texture strength. This replaces dependence on the source
hybrid pass's G-buffer alpha; it does not assert that unmeasured alpha semantics
were reproduced. Misses retain the base color, and nested mirror recursion is not
silently added. Each secondary reflected surface receives a real shadow query.

The source Stanard tone curve and sRGB output transfer are used at fixed exposure
2. Adaptive exposure, bloom, SSAO, temporal antialiasing, motion blur, local light
grids, the raster path and shadow maps are not implemented. These are not silently
approximated or claimed as supported. Mode 4 is a clean visibility diagnostic,
not a reproduction of the source README's known buggy shadow-ray-generation pass.
The original derivative heuristic is retained for secondary surfaces too; it is
not a claim that the sample's known mip/LOD limitation has been corrected.

Every primary ray is counted by mesh/miss; shadow/reflection queries and hits,
tangent repairs and short self-reflections are separate counters. Nonfinite
shaded surfaces/colors fail the frame rather than count as a successful render.
Tests verify all source mips with actual GPU samples, analytic tangent-frame repair
including mirrored/degenerate UVs, source lighting endpoints and tone mapping.
Image tests prove shadows change pixels, reflection changes stay within eligible
floor pixels, diagnostic modes preserve geometry and all five original camera
presets render without nonfinite output or immediate self-reflection hits.

The default rendering path now uses the unchanged Microsoft primary `RayGen`,
Bary `Hit`/`Miss`, `DiffuseHit` helpers/`Hit`, and SSR `RayGen` bodies through the
open DXC/DXIL/SPIR-V/MSL chain. Explicit source adapters queue primary/secondary
hits, supply real native shadow/reflection queries and restore the sample's
finite reverse-Z projection/depth reconstruction. Mesh attribute routes split
expanded vertices into uint16-addressable chunks without dropping triangles or
changing the acceleration structures. Three texture descriptor arrays retain
all 81 material slots and their original mip/color-space choices.

The material adapter retains the port's explicit invalid-tangent repair and
primary specular/view extensions described above. It intercepts specific calls
in the pinned hit body and counts an unexpected call sequence as a frame error;
these extensions are not claimed to be byte-for-byte native DXR shading behavior.
Neutral SSAO input is 1. The named no-shadows mode uses a cleared reverse-Z depth
input with a comparison sampler; it does not implement shadow-map rendering.
Diagnostics without upstream counterparts are Alkyhol-owned HLSL. Prior hand-MSL
shading remains available only through an explicit reference toggle, never as an
automatic fallback. Controls and the full scene are preserved.

Tone mapping, SDR display, fullscreen vertices/copies and shaded live upsampling
use unchanged original HLSL and its complete include closure. The intermediate
is RGBA32Float instead of packed R11G11B10; fixed exposure 2 and zero bloom
strength preserve the current effect scope. No proprietary shader converter is
used. [Shader intake](../shader/samples/miniengine/README.md) records source
hashes, generated artifacts and the adapters' fixed contracts.

Physical M4 tests cover all five source camera presets, cross-view hit counts,
shadow/reflection effects, repeatable captures, resize, explicit reference toggle
and exact restoration, reverse-Z endpoints/unprojection, uint16 chunk routing,
rejected binding metadata, and independent original tone/display/luminance GPU
checks at 13×7. Hosted Metal checks compute/texture interfaces but lacks RT.
Native MSVC geometry-byte agreement remains separate from these physical render
assertions. No native Windows full-frame DXR comparison has been measured.

SSAO, bloom, adaptive exposure, temporal antialiasing/motion blur, local light
grids, original raster/shadow-map paths and generic DXR shader-table/library
translation remain future work.
