# MiniEngine Sponza: geometry debugging milestone

The next application target is Microsoft's
[D3D12RaytracingMiniEngineSample](https://github.com/microsoft/DirectX-Graphics-Samples/tree/68a2b1e97f71642924e74c7757ca21756ad221b5/Samples/Desktop/D3D12Raytracing/src/D3D12RaytracingMiniEngineSample),
pinned to the same upstream commit as Sakura. The user selected it as a less
cluttered debugging scene, then requested textured shading, shadows and reflections.
The first running milestone is complete Sponza geometry with the original primary
Bary Rays path and additional explicitly labeled geometry/normal/depth diagnostics.
The other original rendering modes and lit textures are not yet implemented.

```sh
python3 tools/fetch_miniengine.py
cargo run --manifest-path miniengine/Cargo.toml --locked --release
python3 tools/build_miniengine_app.py
```

The bundle is `local/miniengine/MiniEngine Sponza.app`. Controls:

- **1/G**: neutral geometry (added diagnostic headlight, not original raster mode).
- **2**: original primary Bary Rays shading `(barycentric.x,barycentric.y,1,1)`.
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
The lit path must deliberately address invalid tangent frames before claiming
normal-map support. Position, UV and vertex-normal nonfinite values are rejected.

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

Next: the actual material and normal textures, directional lighting, native shadow
rays, reflection rays and the sample's postprocessing paths. Keep diagnostic views
available and distinguish original sample modes from added debugging views.
