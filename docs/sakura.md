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
