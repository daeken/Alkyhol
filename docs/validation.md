# Executed validation

Current graphics development uses direct Metal from Rust. The historical records
below describe their exact revisions, including the retired wgpu/Vulkan backend.
See the latest direct-Metal record for current implementation evidence.

## First complete milestone

[Conformance run 42326374803](https://github.com/daeken/Alkyhol/actions/runs/42326374803)
on revision `347e88767ffd114dfb6ac53e971d4603d00bf6f7` completed with all six jobs
passing. These are executed results, not expectations inferred from workflow YAML.

| Check | Actual host/backend | Result |
| --- | --- | --- |
| Core Rust tests | macOS, Ubuntu Linux, Windows | 16 passed on each host (11 kernel, 5 command-state/footprint) |
| PE parser tests | Python on all three hosts | 7 passed on each host; fixtures cover PE32/PE32+, normal/delay and legacy-VA imports, named/ordinal entries and malformed input |
| Native C contract probe | Alkyhol shared library on macOS and Linux | Passed |
| Same C source, Windows SDK | Native Windows runner, MSVC/kernel32 | Passed |
| Differential observation comparison | Separate Ubuntu job downloading all three observations | Exact JSON agreement |
| Offscreen rendering | Metal; `Apple Paravirtual device`, IntegratedGpu, hosted macOS | Clear/copy, padded readback, triangle pixels, replay, invalid states and invalid shader all passed |
| Offscreen rendering | Vulkan; llvmpipe LLVM 21.1.8, Cpu, hosted Linux | Same checks passed; explicitly requested software adapter |
| Style/static checks | Root crate on all three hosts; graphics on macOS/Linux | rustfmt and Clippy with warnings denied passed |

Local development also executed the core tests, parser tests and native C sample
on macOS 27.0.1 ARM64 with Rust 1.99.1 and Apple Clang 21.0.0. Local GPU compilation
was blocked by unavailable cached crates and terminal network restrictions; GPU
execution above happened on the recorded CI hosts, not this developer machine.

The kernel sample checks file contents/counts/EOF, creation error codes, truncation,
closed handles, event saturation/reset/timeout and clock monotonicity. It prints
only deterministic booleans after its assertions; it does not compare absolute
clock epochs, scheduler precision or unspecified last-error values. Sample runner reports from [run 42326549684](https://github.com/daeken/Alkyhol/actions/runs/42326549684)
on revision `1f2135c647b5617b6213a97c3257930e0a017ee6` additionally bind compiler identity, revision, source and
executed library hashes. Artifacts belong to their exact source revision.

## Limits

This is sample-level evidence for the restricted contracts in
[contracts.md](contracts.md) and [graphics.md](graphics.md). The raster sample has
no native D3D12 differential reference. Hosted Metal uses Apple's paravirtual GPU,
not an identified physical Apple Silicon GPU. Software Vulkan evidence is labeled
separately. No game execution, exhaustive API coverage, binary ABI compatibility,
COM compatibility, HLSL/DXIL translation or game equivalence has been established.

## Expanded tests and Microsoft sample ports

On macOS 27.0.1 ARM64, Rust 1.99.1, Apple Clang 21.0.0 and a physical Apple M4
Metal adapter, local execution passed 32 root Rust tests, 18 Python tests, four
real-device integration tests, all three portable C/C++ samples and the original
clear/triangle/readback example. Root and graphics Clippy checks with warnings
denied passed. The [test map](testing.md) records what each function/unit exercises.

The new [Microsoft ports](../samples/msdn/README.md) cover synchronous file writing
with a readback check and event-gated readers. They include pinned provenance and
MIT attribution. Deliberate API substitutions are documented; the separate upstream
asynchronous file read example is not ported.

[Conformance run 42329817238](https://github.com/daeken/Alkyhol/actions/runs/42329817238)
on revision `20f8785587b05e8330634fdf3d87f2657729b6d8` completed with all six jobs
passing. Downloaded sample reports bind that revision and clean tracked trees.

| Executed check | Host/backend | Result |
| --- | --- | --- |
| 32 root Rust and 18 Python tests; style/static checks | macOS 26.6.2 ARM64; Linux x86_64/glibc 2.43; Windows Server 2025 AMD64, build 26100 | Passed on each host |
| Three portable C/C++ samples | Windows SDK/kernel32; Alkyhol native C ABI on macOS and Linux | Each executed; full `samples` mapping agreed exactly in the comparison job |
| Four GPU integration tests plus clear/triangle/readback sample | Hosted Metal, `Apple Paravirtual device`, IntegratedGpu | Passed |
| Same four GPU integration tests and drawing sample | Explicit software Vulkan, llvmpipe LLVM 21.1.8, Mesa 26.0.8, Cpu | Passed |

The physical Apple M4 result above is separate local evidence. Hosted Metal and
software Vulkan results are labeled as such. No native D3D12 graphics comparison
or additional game compatibility is established by this run.

## Unnamed counting semaphore milestone

[Conformance run 42330264837](https://github.com/daeken/Alkyhol/actions/runs/42330264837)
on revision `4da84e503cba3407b7264ce9f4ad77c7947ec19a` completed with all six jobs
passing. This adds CreateSemaphoreW, ReleaseSemaphore and semaphore waits to the
restricted kernel contract, plus a pinned adaptation of Microsoft's semaphore
worker example. All four portable C/C++ samples executed on Windows SDK/kernel32,
native macOS Alkyhol and native Linux Alkyhol; the complete observations agreed
exactly. Downloaded reports bind the same revision with clean tracked trees.

The semaphore sample checks ten permits shared by twelve workers, zero-timeout
acquisition, returned permits, previous counts, invalid creation/release counts
and maximum-count overflow errors. It asserts an upper bound on active workers,
not a minimum degree of concurrency, waiter order or fairness. C++ threads/join
remain harness substitutions for unsupported Win32 thread APIs.

There are now 39 root Rust tests (seven new semaphore tests), 18 Python tests and
four real-device integration tests. The CI root tests passed on all three hosts;
GPU tests and the drawing sample passed on hosted Metal and explicit software
Vulkan. Local macOS 27.0.1 ARM64/Rust 1.99.1 also passed those tests and all four
portable samples, with GPU execution on physical Apple M4 Metal. Root Clippy with
warnings denied passed locally; CI root and graphics static checks passed.

Named objects, security descriptors, handle inheritance, cross-process semaphores,
access masks and closing a semaphore while another thread waits remain unsupported.
The corpus and these samples do not establish game compatibility.

## Dynamic thread-local storage milestone

[Conformance run 42330786584](https://github.com/daeken/Alkyhol/actions/runs/42330786584)
on revision `e231f9e77f7336d1612b3c71400d07ef76a259dc` completed with all six jobs
passing. The four new exports are TlsAlloc, TlsFree, TlsSetValue and TlsGetValue.
The Microsoft TLS adaptation verifies four workers' separate 256-byte payloads
through a common retrieval function, null initialization, valid null values,
TlsGetValue clearing last-error, main-thread isolation and out-of-range errors.

All five portable samples executed against Windows SDK/kernel32 on Windows Server
2025 build 26100 AMD64, Alkyhol on macOS 26.7.2 ARM64, and Alkyhol on Linux
x86_64/glibc 2.43. Their complete observations agreed exactly. Downloaded reports
bind the same source revision and clean tracked trees. A clean local report on
macOS 27.0.1 ARM64 also matched those observations.

There are now 43 root Rust tests, 18 Python tests and four GPU integration tests.
Root tests and style/static checks passed on all three CI hosts. The GPU tests
and drawing sample passed on hosted Metal and explicit software Vulkan, and
locally on physical Apple M4 Metal. Local Rust was 1.99.1 and C/C++ used Apple
Clang 21.0.0. The four new host assertion tests additionally cover pool exhaustion,
forced slot reuse in an existing thread, opaque address bits and stricter rejection
of unallocated indices. These are separate from native Windows sample evidence:
the portable sample does not compare exhaustion capacity/error, allocation order,
in-range unallocated reads, or forced index reuse.

Dynamic TLS does not implement PE static TLS, DLL initialization/teardown, FLS,
thread creation, memory allocation or pointer ownership. Stored addresses remain
caller-owned host values. Full slot-lifetime rules and deliberate limits are in
[the kernel contracts](contracts.md). No game compatibility is established.

## Native thread callback milestone

[Conformance run 42331502760](https://github.com/daeken/Alkyhol/actions/runs/42331502760)
on revision `925f8f914bd73e5b2390a56ddfbf51fad8120a70` completed with all six jobs
passing. CreateThread now starts a native host callback, GetCurrentThreadId returns
its opaque Alkyhol identity, GetExitCodeThread queries pending/returned status, and
WaitForSingleObject accepts thread handles. Completion remains signaled, including
when a completed callback returns the numeric STILL_ACTIVE value (259).

The pinned Microsoft Creating Threads port executes three actual CreateThread
callbacks with separate parameter records, an event gate, individual finite waits
and explicit handle cleanup. It checks parameter writes, callback IDs against the
creation outputs, pending status, returned DWORD values and repeated completion
waits. All six portable samples executed against Windows SDK/kernel32 on Windows
Server 2025 build 26100 AMD64, Alkyhol on macOS 26.6.2 ARM64, and Alkyhol on Linux
x86_64/glibc 2.43. Their complete observations agreed exactly. Downloaded reports
bind the same revision and clean tracked trees; a clean local macOS report agreed.

All 52 root Rust tests and 18 Python tests passed on each CI host. Nine new host
thread tests include pending/completed/error paths, optional/null outputs, early
handle close, distinct foreign/created thread IDs, returned 259, and a deliberately
blocked Rust TLS destructor proving callback return alone does not publish
completion. A safe Rust worker panic must produce an explicit host error. These
host tests are separate from the Windows SDK sample; the panic path does not
establish native Windows exception handling or DLL callback semantics.

Four GPU integration tests and the drawing example passed on hosted Metal
(`Apple Paravirtual device`) and explicit CPU Vulkan (llvmpipe LLVM 21.1.8).
The same checks passed locally on physical Apple M4 Metal, macOS 27.0.1 ARM64 and
Rust 1.99.1; all six C/C++ samples also passed with Apple Clang 21.0.0. Root local
Clippy and CI root/graphics static checks passed with warnings denied.

The initial thread backend uses a detached host monitor per callback to join the
worker, then publish completion after host TLS destructors. This adds resources
and does not emulate Windows stack reserve/commit, system-wide IDs, DLL lifecycle,
security, inheritance, suspended creation, priority or abnormal termination. Existing
event/semaphore/TLS ports still use their documented C++ thread substitutions;
the new Creating Threads port provides the direct thread-API comparison. No game
compatibility or universal API equivalence is established.

## Whole-resource GPU copy milestone

[Conformance run 42332065590](https://github.com/daeken/Alkyhol/actions/runs/42332065590)
on revision `6093e2d74ea5d2287e61e7d7f56115d3f6570a48` completed with all six jobs
passing. The Rust graphics facade now records CopyResource for distinct matching
RGBA8 2D textures, with explicit CopySource/CopyDest states. It issues an actual
wgpu texture-to-texture GPU copy. Identity, state and both dimension checks finish
before any list command is encoded or submitted.

Three new root tests exercise copy state/token failures, sequential copy chains
and rejection after earlier logical transitions. Two real-device tests compare
every pixel after chained 65×3 copies and reject width/height mismatches after an
earlier recorded clear. They verify source pixels and destination state are still
unchanged, including a subsequent valid transition from the original state.
The new `copy_resource` example first renders and captures a patterned 65×65
triangle, then compares the entire copied destination and original source bytes.
It is original project code guided by the API docs, not a Microsoft sample port.

All six GPU tests and both graphics examples passed locally on physical Apple M4
Metal (macOS 27.0.1 ARM64, Rust 1.99.1), on hosted Metal (`Apple Paravirtual device`),
and on explicit software Vulkan (CPU llvmpipe LLVM 21.1.8, Mesa 26.0.8).
These are separate backend results. There is no native D3D12 copy/raster reference
in this milestone, no generalized format/subresource support and no game evidence.

The 55 root Rust tests and 18 Python tests passed on Windows, macOS and Linux CI,
with style/static checks passing for both crates. All six existing kernel C/C++
samples still executed on Windows SDK/kernel32 and native macOS/Linux Alkyhol;
their complete observations agreed. Downloaded reports bind this exact revision
and clean tracked trees, as does a matching local C/C++ report. Those kernel
comparisons do not validate the new GPU copy operation against D3D12.

## First native D3D12 pixel comparison

[Conformance run 42332990445](https://github.com/daeken/Alkyhol/actions/runs/42332990445)
on revision `ac4dfd463c7572912fdf31d5f407b10c52163e83` completed with all eight jobs
passing, including the new native Windows reference and graphics comparison jobs.
The independent Windows SDK program used D3D12 COM calls with explicit WARP,
compiled original HLSL through D3DCompile, rendered a 65×4 multichannel checkerboard,
called CopyResource and read both surfaces after a queue fence. It did not link
Alkyhol. Every one of the 1,040 source and 1,040 destination RGBA8 bytes agreed with
Alkyhol Metal and explicit software Vulkan, as well as the fixed scenario contract.

| Execution | Observed identity |
| --- | --- |
| Native Windows D3D12 | Windows Server 2025 build 26100; WARP vendor 5140, device 140, software flags 2; MSVC 19.51.36260 x64; Windows SDK 10.0.26100.0 |
| Hosted Metal | macOS 26.7.2 ARM64; Apple Paravirtual device, IntegratedGpu; Rust 1.99.1 |
| Explicit software Vulkan | Linux x86_64/glibc 2.43; llvmpipe LLVM 21.1.8, Mesa 26.0.8, Cpu; Rust 1.99.1 |
| Local physical Metal | macOS 27.0.1 ARM64; Apple M4, IntegratedGpu; Rust 1.99.1 |

Downloaded reports bind the exact revision, clean tracked trees, compiler/SDK and
adapter context, source hashes and executed-program hashes; the clean local
report agreed too. The actual full observations were compared, without reducing
pixels to checksums. All 55 root Rust tests, 26 Python tests and six GPU integration
tests passed on their applicable CI hosts; the two earlier graphics examples and
six existing kernel samples also passed. Eight new Python tests include exhaustive
single-channel perturbation of these two finite surfaces and synthetic process
faults; those tests are not GPU evidence.

This establishes agreement for the fixed [checkerboard render/copy scenario](graphics-reference.md),
using different handwritten HLSL and WGSL frontends and the labeled backends. It
does not establish HLSL/DXIL translation, general raster/copy equivalence, COM facade
compatibility, native invalid-command behavior or game compatibility. WARP and
llvmpipe are software renderers; runtime DLLs and driver binaries are not pinned.

## Reproduce

```sh
cargo fmt --check
cargo test --offline
cargo clippy --offline --all-targets -- -D warnings
cargo build --offline
python3 -m unittest discover -s tests -v
python3 tools/run_samples.py --output local/kernel-observation.json
cargo fmt --manifest-path graphics/Cargo.toml --check
cargo clippy --manifest-path graphics/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path graphics/Cargo.toml --locked -- --nocapture
cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
```

Create `local/` before requesting a report there. Graphics commands need access to
the crates in their lockfile and a usable backend. Linux without hardware must add
`-- --allow-software` and provide Vulkan/llvmpipe. A missing backend or timeout fails.

## Direct Metal migration, 2026-11-07

Revision `818cfe8` retired wgpu, Naga and the Vulkan graphics path. Local physical
Apple M4/macOS 27.0.1 ARM64/Rust 1.99.1 passed 55 root tests, 26 Python tests,
seven GPU tests, graphics Clippy and the unchanged v1 full-pixel contract. CI run
[42334642328](https://github.com/daeken/Alkyhol/actions/runs/42334642328)
passed all seven jobs, including exact agreement of all 1040 source and 1040 copied
RGBA8 bytes between independent native Windows SDK D3D12 WARP and hosted direct
Metal. The hosted Apple Paravirtual device reports ray tracing **false**; it does
not validate Sakura ray tracing. Physical M4 ray-tracing tests are separate and
require an explicit `--include-ignored` invocation; absence is not a passing ray test.

## Running Sakura Forest on physical Metal, 2026-11-07

Code revision `b72e0790ee8b785d0399ad044cf820d62b8901eb` runs the complete
SORTMODE_OFF rendering port in Rust/MSL. All five public sample assets are pinned
and hash-verified; four BLAS contain 33,983 original triangles, instanced 13,525
times through a native Metal TLAS. The code retains original scene, texture color
spaces, material quirks, two-level reflections and 200×50 fractal sky iterations.

On physical Apple M4, macOS 27.0.1 ARM64 and Rust 1.99.1:

- Eight graphics integration tests passed with `--include-ignored`, including real
  native ray hits/misses, scaled/translated instances, barycentrics and masks.
- Seven Sakura integration tests passed with `--include-ignored`: five CPU/asset
  tests, CLI rejection/help and the actual full-scene rendering test. Repeated PNG
  pixels matched exactly at fixed camera/light; moving the light changed pixels;
  the close camera observed 1,359 primary reflective-cube hits and reflection rays.
- The 129×73 default test observed 687 floor reflection rays, 663 secondary hits and
  24 secondary misses. All primary rays were accounted for and floor output alpha
  stayed opaque; 257×257 close-camera testing exercised cube attenuation/recursion.
- A release 960×540/time-zero capture observed primary counts
  `[151825, 0, 153240, 135415, 60987, 16933]` for floor, reflective cubes, trunks,
  leaves, bushes and sky; 37,272 floor reflections led to 35,837 secondary hits and
  1,435 misses. The initial view has no primary reflective-cube hits; the dedicated
  camera test covers them instead. PNG SHA-256:
  `118e5c5ba39d3488bd0810b71716db06842244bd0ce4c17e7990e7db6b1e225d`.
- The native app window visibly displayed the forest. Held Right/W keys changed
  the camera view. A final release build with correct trunk sRGB handling was
  observed at about 16 FPS at 960×540. This is an observed UI rate under current
  machine load, not a controlled performance benchmark.

Raw logs, source/asset/executable hashes and the capture stay in ignored
`local/sakura/physical-metal-report.json` and `local/sakura/metal-sakura.png`.
Use [the launch/test instructions](sakura.md#running-the-port) to reproduce.

CI run [42335656952](https://github.com/daeken/Alkyhol/actions/runs/42335656952)
passed all eight jobs on that code revision. Native MSVC and hosted macOS Rust
produced identical 662,725-byte placement observations: every material/mesh ID and
scale/position float bit across all 13,525 instances. Both artifact SHA-256 values
are `0e9cfe7ea4a45fd374fb20ef10ba3df8b6e83f365e9beafe06170e009a03a6a3`.
The same run also passed all three hosts' root/kernel sample comparisons, 28 Python
tests and exact native D3D12 WARP/direct Metal checker-copy pixels. Hosted Metal
CPU/asset tests passed; its RT integration test was explicitly ignored because
that device reports no ray-tracing support. This is not hosted Sakura GPU evidence.

No full Sakura native DXR capture has been measured, so there is no claim of
Windows/Metal full-frame pixel equivalence. WIC/Rust JPEG rounding and cross-GPU
floating-point behavior remain comparison work. DirectX SER scheduling modes are
not implemented; the UI says SER off. This source-level application port does not
run original Windows/game binaries and is not evidence of game compatibility.

## Sakura geometry audit and shader units, 2026-11-07

On the same physical M4/macOS 27.0.1/Rust 1.99.1, the full-scene intersection audit
compared 58 Metal nearest-hit queries with an independent CPU slab/Möller–Trumbore
triangle implementation. Queries cover a pixel grid and directions toward nearby
trunks, considering all 13,525 instances as potential occluders. All 58 agreed on
nearest distance; the reported GPU triangle was independently intersected too,
allowing coincident triangles at the same depth. Material counts were
`[11, 1, 32, 13, 1, 0]`. This establishes the queried geometry only, not every view,
all trunks or native Windows/DXR equivalence. It did not reproduce a depth error.

The pinned trunk OBJ contains 17,970 triangle faces and 479 quad faces. Upstream's
ObjModelLoader reads only the first three corners of each face; the port retains
that behavior. This drops the second triangle of each quad. This is a possible
source of visible holes, not a demonstrated explanation for every reported missing
trunk. No default asset/geometry repair was made.

Shader arithmetic helpers now have real Metal compute checks for all five normal
buffer paths, the leaf-index bounds behavior, unnormalized interpolation, original
cube/trunk index mixing, Schlick endpoints/interior/clamping, Euclidean triplanar
weights (including zero), and all three petal colors. These analytic fixtures are
shader-unit assertions. They require Metal compute, not RT hardware. Full scene
rendering after extraction produced the same 960×540/time-zero PNG SHA-256 as the
running milestone: `118e5c5ba39d3488bd0810b71716db06842244bd0ce4c17e7990e7db6b1e225d`.
All nine Sakura integration tests passed locally with `--include-ignored`, plus
Sakura Clippy. Raw output is ignored under
`local/sakura/shader-and-geometry-audit-tests.log`.

A separate exploratory full-fractal scalar-host comparison **did not agree** at an
attempted 0.001 absolute tolerance. For directions +X/+Y/−Z, GPU densities were
`[0.8959779, 0.91883063, 0.63184154]`; independent scalar Rust f32 observations were
`[0.936103, 0.936103, 0.7602065]`. The discrepancy is unresolved, and no larger
acceptance tolerance was adopted. The shader test records these observations and
asserts only the finite density range for this part; it does not count them as
host/GPU equality or native DXR evidence. Original sky math, loop counts and the
working full-frame output are retained. The user clarified that the purple/noisy
sky matches the sample screenshot; the remaining visual concern is trunk coverage.

## MiniEngine Sponza geometry diagnostics, 2026-11-07

Physical M4/macOS 27.0.1/Rust 1.99.1 rendered the original pinned Sponza H3D with
34 meshes, 27 materials and 278,919 triangles. Six MiniEngine integration tests
passed explicitly including pinned assets and actual GPU rendering: three parser/
asset tests, camera, CLI and full-scene view/capture/resize checks. All four views
accounted for the same primary rays and repeated full pixels matched at fixed
view. Original Bary Rays was observed in a live window around 59 FPS at 960×540;
UI switching from 2 to 1 visibly returned to Geometry diagnostic after the controls
fix. The neutral diagnostic is not the original raster/material shading mode.

Two physical graphics ray tests passed, including the new two-mesh far-first /
near-second instance scene and both directions with ray backface culling enabled
or disabled via instance override. Existing Sakura instances retain their prior
culling behavior. The original Sakura geometry/shader audit code revision
`fa757f6` passed all eight existing CI jobs in run
[42336806679](https://github.com/daeken/Alkyhol/actions/runs/42336806679).

The Rust H3D stream contains 46,858,664 bytes, including material/count headers and
every expanded original vertex attribute bit (including unused NaN tangents).
Local SHA-256: `0d937fea9ee97a5f38a8dcec326ae0ce48b154d36613a70358cf1c136478f51e`.
The original H3D asset SHA-256 is
`c52e851f67a37892acf229df7cce8e2b8aff997a087bc41fa8c48beb73b45747`.
A native MSVC mirror of the pinned serialized structs asserts the SIMD record
sizes and independently expands the uint16 index stream. Its CI comparison is
newly defined; no native result is claimed until that job actually completes.
There is no full MiniEngine native DXR pixel comparison yet. Textured lighting,
shadow/reflection modes and postprocessing remain subsequent work requested by
the user; diagnostic rendering does not count as those effects being implemented.
