# Original Sakura shader bodies over open ray queries

`original/` contains byte-for-byte Microsoft/Pablo Roman Andrioli source at
`68a2b1e97f71642924e74c7757ca21756ad221b5`. `upstream.json` binds all paths,
lengths and SHA-256 hashes. Notices are in `../../licenses/Microsoft-MIT.txt` and
`../../licenses/Star-Nest-MIT.txt`.

`tools/prepare_sakura_shaders.py` extracts the original camera/attribute/Fresnel,
five closest-hit, radiance and miss function definitions into generated include
files. Bodies, including the original material/index/normal quirks, are unchanged.
The complete original Star Nest include keeps its 200 sample steps and 50 inner
iterations. Tests reject source-hash changes, unexpected overload counts,
unbalanced definitions and generated-file differences.

`ser_off.hlsl` is an explicit **fixed source specialization**, not a generic DXR
library compiler. Its compute entry implements the original SER-off raygen flow,
calls the original camera helper and dispatches original hit/miss bodies after
actual HLSL inline queries. A preprocessor namespace creates depth-one/two copies
of the original functions. The original `TraceRadianceRay` guard returns black
at depth two; reaching a third trace instead raises a counted fault and NaN
payload. The renderer rejects faults/nonfinite pixels. SER modes fail explicitly;
HitObject/MaybeReorderThread support is not claimed.

Context is saved/restored across the nested trace so a caller's ray builtins still
refer to its own ray. The five original local-root CBs all have white albedo;
that constant is preserved. Their material IDs are only consumed by excluded SER
hints. Dispatch picks materials from the original fixed instance-group order.
The payload's compute representation carries the original colour/depth fields;
DXR payload access qualifiers are not a compute ABI. Upstream Star Nest returns
a replicated float3 but its miss shader assigns it to float; the adapter makes
that original implicit `.x` selection explicit without editing either body.

Runtime buffers use the original 36-byte Vertex layout and indexed 24-vertex
cube. The trunk shader still reads **leaf indices followed by trunk normals**.
The unused tail of the physical leaf-index buffer is zero padded through the
maximum original trunk read, reproducing D3D out-of-view index reads as vertex
index zero. This is a proved fixed-source access range; no general Metal OOB
robustness is implied. No geometry is added/dropped and the original OBJ quad
loader behavior remains unchanged.

Generate/verify using the pinned open DXC and translation stack:

```sh
python3 tools/prepare_sakura_shaders.py --check
cargo run --manifest-path shader/Cargo.toml --locked --example translate_samples -- "$DXC"
cargo run --manifest-path shader/Cargo.toml --locked --example translate_samples -- "$DXC" --check
```

The checked-in JSON contains generated MSL and authoritative binding metadata;
running the app does not download a compiler. The old manual MSL remains an
explicit reference toggle, never an automatic fallback after shader failure.

On the physical M4, all ten Sakura tests and sixteen shader tests passed, including
the full 13,525-instance scene, repeatability/light/camera/resize/reflection checks,
buffer packing/padding and original helper/budget-fault GPU tests. The default
129×73 original/manual observations have identical material/reflection hit counts
but 642 different RGBA8 pixels: arithmetic/source-camera differences are observed,
not hidden behind a wider equality tolerance. No native Windows full DXR frame
comparison has been measured. The helper-only compute fixture strips unreachable
queries and can run on hosted Metal without RT; that is separate evidence.
