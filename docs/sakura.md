# Sakura Forest: faithful Rust port over direct Metal

The concrete application target is Microsoft's
[D3D12RaytracingSakuraForestSER](https://github.com/microsoft/DirectX-Graphics-Samples/tree/68a2b1e97f71642924e74c7757ca21756ad221b5/Samples/Desktop/D3D12Raytracing/src/D3D12RaytracingSakuraForestSER),
pinned to upstream revision `68a2b1e97f71642924e74c7757ca21756ad221b5`.
The user accepted a faithful Rust port of the scene/rendering logic, then explicitly
selected direct Metal from Rust instead of wgpu. The upstream checkout is ignored
under `local/upstream/DirectX-Graphics-Samples`.

The application must use the actual three OBJ assets and two JPEG textures, not a
lookalike forest. The original instance groups are 441 floor cubes, 961 reflective
cubes, 961 trunks, 961 leaf meshes and 10,201 bushes: 13,525 instances total. Mesh
positions are scaled by 0.007, with the upstream placement transforms and MT19937
seed 42/draw order. Repeated `o tree` objects in trunk.obj combine as the original
loader does. Expanded face vertices retain original primitive order and normal/UV
attributes. The unused rotation matrix in BuildTreeGeometry is not applied.

The first complete target is the existing SORTMODE_OFF path: ray generation,
closest triangle intersection, floor reflections, trunk triplanar texturing,
three petal colors, textured bushes, reflective cubes and the 200×50 Star Nest
sky. Native Metal triangle BLAS/TLAS and intersectors will use the M4's ray-tracing
hardware. Explicit DirectX HitObject/MaybeReorderThread scheduling is a separate
capability, not automatically supplied by a successful ray intersection. Report
SER support honestly and do not present an ignored hint as supported hardware SER.

Two observed source quirks need deliberate handling and documentation: the trunk
shader reads IndicesLeaves before indexing VerticesTrunk, and the reflective cube
shader uses cube indices with VerticesTrunk normals. Do not silently replace these
with matching mesh buffers and call the result exact upstream behavior. Native DXR
rendering of this full sample has not been measured here.

Implementation sequence:

1. Replace the graphics crate's wgpu dependency with direct Metal Rust bindings;
   keep existing command/state tests and the unchanged native D3D12 pixel oracle.
2. Add pinned asset intake, OBJ/MT19937/instance/camera tests and source attribution.
3. Add reusable native Metal triangle acceleration structures and verify real ray
   hits/misses/instance transforms on the physical M4.
4. Port the upstream shading to MSL and render the full instance set offscreen.
5. Provide a live window, camera movement, frame statistics and reproducible launch
   and capture commands. Check the actual running UI and saved output.

The original camera is left-handed, vertical FOV 45°, near/far 1/125, initial eye
(0,2.2,-2) rotated 45° around Y, target (1,2.5,-6), up (0,1,0). Light position
(0,15.8,233) rotates around Y once every eight seconds; diffuse is (1.2,1,1).
The recursion limit is two. Keep the actual sky/material math and document any
render-resolution/performance controls. A screenshot alone does not establish
all shader, camera or input behavior.

Upstream code is MIT licensed by Microsoft. Star.hlsli additionally credits Pablo
Roman Andrioli under MIT. Preserve those notices in adapted shader code and keep
pinned asset/source hashes with the intake manifest. Public sample assets are
distinct from proprietary game assets; no game binaries/data are needed here.

## Running the port

Requires macOS, Rust and a ray-tracing-capable Metal device. The physical M4 has
been exercised. The hosted CI Mac's paravirtual GPU has no RT support and is not a
Sakura rendering result.

From the repository root:

```sh
python3 tools/fetch_sakura.py
cargo run --manifest-path sakura/Cargo.toml --locked --release
```

The intake fetches only the five public sample assets at the pinned commit,
verifies every SHA-256 and keeps them in ignored `local/sakura/assets`. Hashes of
adapted source files are also in `sakura/upstream.json`; attribution is in
`sakura/licenses/`. No game assets or binaries are involved.

WASD moves; Q/E moves up/down; arrows look; R resets the camera; Space pauses the
moving light; Escape closes the app. The window title reports measured overall
frame rate, render resolution, native Metal RT and SER off. Resizing preserves
camera aspect ratio at the selected render width. The presentation image scales
to the window. Rendering currently waits for each submitted frame to complete.

```sh
cargo run --manifest-path sakura/Cargo.toml --locked --release -- \
  --headless --width 960 --height 540 --time 0 --capture local/sakura/frame.png
cargo run --manifest-path sakura/Cargo.toml --locked --release -- \
  --frames 3 --capture local/sakura/live-frame.png
python3 tools/build_sakura_app.py
```

The last command builds `local/sakura/Sakura Forest.app`, which can be opened
normally. It uses this checkout's asset location; this is a local development app,
not a relocatable distribution. `--assets PATH` accepts another asset directory.
`--width`/`--height` select resolution without changing geometry, sky iterations or
recursion depth. The default is 960×540. The sky always performs all 200×50 steps;
no lower-quality preview substitutes for it.

## Tests and fidelity boundary

```sh
cargo test --manifest-path sakura/Cargo.toml --locked -- --include-ignored --nocapture
cargo test --manifest-path graphics/Cargo.toml --locked -- --include-ignored --nocapture
```

CPU tests cover face/primitive ordering, repeated OBJ sections, attributes,
malformed indices, original triangle counts, MT19937 outputs, all instance groups,
paired leaves/trunks, camera basis/motion/clamping and CLI validation. The physical
GPU test renders the full scene, checks every primary ray is accounted for,
observes floor and cube reflection rays/hits/misses, compares repeated full PNG
pixels, changes light/camera, exercises padded readback and checks resize failures.
Asset and physical GPU tests are explicitly ignored by default; run the command
above to execute them. CI explicitly runs the asset tests, but its hosted Mac
cannot run the RT test. Missing capability is an error when launching Sakura.

CI also compiles an independent C++ placement adaptation on native MSVC and
compares all 13,525 records (material IDs and every scale/position float bit) with
Rust. This tests CPU placement, not native DXR rendering. The standard distribution
implementation can change across MSVC versions: this port targets current MSVC's
24-bit `generate_canonical<float>` conversion and seed/draw order.

The trunk JPEG has EXIF ColorSpace=1, so the upstream WIC default creates an sRGB
texture; the bush JPEG has no sRGB marker and remains linear UNORM. The port uses
those respective Metal formats. See Microsoft's
[WIC loader documentation](https://github.com/microsoft/DirectXTK12/wiki/WICTextureLoader).
JPEG decoder rounding may differ between WIC and the Rust decoder; full native DXR
pixel equivalence has not been measured.

The trunk index-read quirk is preserved using explicit zero for indices beyond
the leaf index SRV's declared bounds. Upstream binds these buffers through a
descriptor table, not root SRVs. Direct3D documents
[zero for out-of-bounds resource reads](https://learn.microsoft.com/en-us/windows/win32/direct3d11/direct3d-11-advanced-stages-cs-access).
Reflective cubes retain the source's trunk normals and 0.5 reflection origin offset.
Interpolated normals are not silently normalized. The explicit two-level MSL loop
preserves the source's recursion limit, including floor alpha and cube attenuation.

This is a running source-level rendering port. It does not run the original Windows
executable, implement DXR shader binaries or provide DirectX SER scheduling modes.
Its real-device assertions and live UI observations do not establish game compatibility.
