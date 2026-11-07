# Native D3D12 render/copy scenario

`rgba8-checker-copy-v1` compares actual Windows SDK D3D12 execution on explicitly
selected WARP with Alkyhol on Metal and explicit software Vulkan. This is a finite
offscreen pixel scenario, not a feature-level claim or general D3D12 equivalence.
Executed results belong in [validation.md](validation.md); adding a workflow or
reference program alone is not a passing result.

The independent [Windows reference](../graphics/reference/d3d12_copy_contract.cpp)
links d3d12/dxgi/d3dcompiler and kernel32, without linking Alkyhol or its command
validator. It selects EnumWarpAdapter and checks the software flag, creates a
D3D12 device/direct queue, compiles original HLSL with D3DCompile, creates a binding-
free root signature and pipeline, renders, copies and reads back through native
COM methods. Explicit barriers and a queue fence/event guard CPU mapping. The
[Alkyhol probe](../graphics/examples/copy_contract.rs) uses its existing Rust
facade and original WGSL. These are separate handwritten shader frontends, not
HLSL/DXIL translation. The programs are original project code guided by Microsoft
API documentation, not an unmodified upstream sample.

The fixed v1 sequence uses one 65×4 RGBA8 UNORM, one-mip, one-sample source and a
matching destination. It clears the source green, then draws an oversized triangle
covering every pixel, with pixel coordinates generating exact endpoint channels:

| Channel | Value at integer pixel (x,y) |
| --- | --- |
| R | 255 × (x mod 2) |
| G | 255 × (y mod 2) |
| B | 255 × ((floor(x/2) + y) mod 2) |
| A | 255 |

After RenderTarget→CopySource, CopyResource copies the entire source to the
CopyDest destination, which transitions to CopySource. Both surfaces are copied
to readback buffers, completed before CPU access, and serialized without row
padding. Each contributes all 1,040 observed bytes; neither is reduced to a hash
or a few chosen pixels. Width 65 requires padded rows, and separate x/y channel
patterns expose row/column mistakes. Only exact endpoint values are used, without
a global floating-point or pixel tolerance.

The observation must contain exactly `scenario`, `width`, `height`, `source_rgba8`
and `destination_rgba8`, with the fixed string/integer types and lowercase full
hex data. [The v1 contract](../tools/graphics_contract.py) independently generates
the expected bytes. The runner rejects bad pixels, malformed/duplicate/extra JSON,
wrong types/dimensions, nonzero exits, timeout, missing/ambiguous backend identity,
and an unrequested software class. Unit tests perturb every individual channel on
both surfaces; synthetic process tests exercise failures without claiming GPU
evidence.

Run with the Windows SDK Developer Command Prompt:

```bat
python tools\run_graphics_contract.py --output local\graphics-contract.json
```

On macOS with Rust available:

```sh
python3 tools/run_graphics_contract.py --output local/graphics-contract.json
```

On Linux with explicitly requested software Vulkan:

```sh
XDG_RUNTIME_DIR=/tmp python3 tools/run_graphics_contract.py --allow-software --output local/graphics-contract.json
```

Create `local/` before writing reports there. The runner builds the selected
program, records host/compiler/SDK/adapter/revision information and source and
executed-program hashes, verifies input hashes before/after execution, and emits
the actual full observation. It never substitutes a successful mock for an absent
GPU or Windows reference. Runtime DLL and driver binaries are not pinned by these
reports; their OS/adapter context matters. This is not a hermetic runtime proof.

CI has distinct Windows WARP reference and Alkyhol Metal/Vulkan jobs, followed by
an exact observation comparison requiring all three reports from the same clean
source revision. WARP and llvmpipe are labeled CPU/software evidence. No swapchain,
window, shaders from games, COM facade compatibility, error-path equivalence,
multisampling, format reinterpretation or universal raster behavior is established.
The negative GPU state/extent tests remain separate Alkyhol assertions; the native
reference does not issue undefined invalid D3D12 command lists.

API references: [EnumWarpAdapter](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_4/nf-dxgi1_4-idxgifactory4-enumwarpadapter),
[D3DCompile](https://learn.microsoft.com/en-us/windows/win32/api/d3dcompiler/nf-d3dcompiler-d3dcompile),
[CopyResource](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12graphicscommandlist-copyresource),
[GetCopyableFootprints](https://learn.microsoft.com/en-us/windows/win32/api/d3d12/nf-d3d12-id3d12device-getcopyablefootprints).
