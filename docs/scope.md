# Scope and evidence

## Inventory boundary

Run the PE scanner against a locally supplied executable and, separately, its
DLLs. Keep derived reports under ignored `local/`. Import presence does not
establish reachability, describe arguments, or cover dynamically loaded methods.
Graphics interposers and vendor libraries need separate investigation before
assuming that exported D3D12/DXGI symbols represent the complete graphics surface.

## Implementation sequence

1. Dependency-free kernel subset: opaque lifetime-checked handles, thread-local
   last-error, synchronous file I/O, unnamed manual/auto-reset events and counting
   semaphores, dynamic TLS slots, native host thread callbacks/waits/exit status, monotonic
   performance counters and Sleep. Portable C samples use the same source with
   Windows headers on Windows and Alkyhol's native header on macOS/Linux.
2. D3D12-shaped device, resources, command recording, submission and readback on
   direct Metal from Rust. Initial samples cover offscreen clear/copy and triangle.
   Native MSL shaders are explicit ports, not a DXIL translator. Validate pixels and lifecycle failures.
3. Faithful Sakura Forest Rust/Metal port with native triangle BLAS/TLAS, ray
   intersectors, actual upstream assets/materials and a live window. Add device and
   resource capability queries. No window API should fake a successful result.
4. Expand from sample evidence: memory mappings, synchronization, allocation,
   threads/TLS, overlapped files, timing, input/XInput, sockets, COM and audio.
5. D3D12 descriptor/root-signature translation, barriers, fences, indirect draws,
   compute, shader translation and explicit unsupported-capability reporting.

D3D12's entire feature surface cannot be assumed to map directly to Metal. DXR, vendor
extensions and game shaders require investigation. Keep feature support separate
from symbol export coverage. Media, physics, platform and vendor libraries in the
import inventory are dependencies, not automatically Windows API work.

## Validation policy

- Prefer original, tiny sample sources exercising documented Microsoft contracts.
  Cite upstream sample inspiration; do not import unreviewed code or its license.
- Compile the same C source on Windows against kernel32 and on native hosts against
  Alkyhol. Compare deterministic JSON observations, including errors and memory.
- Keep timing values out of deterministic comparisons; test monotonicity/bounds.
- Graphics samples read back pixels and fail on absent GPU/unsupported features.
  Never count a skipped/noop/software mock as a Metal test.
- Unknown/unsupported operations report an error. Broader semantics are never
  implied by success on a restricted subset. Keep the subset documented.

## References

- https://learn.microsoft.com/en-us/windows/win32/fileio/opening-a-file-for-reading-or-writing
- https://learn.microsoft.com/en-us/windows/win32/sync/using-event-objects
- https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilew
- https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createeventw
- https://learn.microsoft.com/en-us/samples/microsoft/directx-graphics-samples/d3d12-hello-world-samples-win32/
- https://github.com/microsoft/DirectX-Graphics-Samples (MIT; inspiration, no copied implementation)
- https://docs.rs/metal/0.32.0/metal/
