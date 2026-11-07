# Bounded original C++ root-signature support

The original MiniEngine Sponza C++ application still compiles and links as a
whole. Its production device preflight exits 78 before original initialization.
This milestone implements real owned CPU layout data and source-ABI COM objects;
it does not add GPU root binding, pipeline creation, dispatch or original frames.

## Contract

`D3D12SerializeRootSignature` and `D3D12SerializeVersionedRootSignature` support
version 1.0. They deep-copy SDK pointer arrays, validate the bounded layout and
return real owned ID3DBlob objects with complete standalone RTS0/DXBC containers
and a retail container checksum. Failures clear the output and, when requested,
return an owned NUL-terminated diagnostic blob. Diagnostic text is ours; native
compiler diagnostic text is not claimed equivalent.

The finite contract accepts descriptor tables (finite SRV/UAV/CBV or sampler
ranges), 32-bit constants, root CBV/SRV/UAV descriptors and static samplers. Spaces,
shader visibility, explicit offsets, APPEND values and sampler float bits survive
serialization. Register overlaps are checked within class/space/intersecting
visibility; table offsets cannot overlap. The global root budget is 64 DWORDs.
Current bounds are 64 parameters, 64 ranges per table, 16 static samplers,
one-million-slot table extents, visibility ALL through PIXEL, and ordinary root
flags through ALLOW_STREAM_OUTPUT (0x7f). Reserved register spaces, unbounded
ranges, local signatures, heap indexing, newer visibilities, non-finite LODs and
versions 1.1/1.2 fail explicitly. This is a finite implementation, not the full SDK.

The restricted Metal device's `CreateRootSignature` accepts checked canonical
standalone version-1.0 root containers through 1 MiB, node mask 0/1, and supplies
all eight ID3D12RootSignature/IUnknown/Object/DeviceChild vtable slots. It owns the
parsed layout and retained parent, metadata and atomic interface references.
Valid null-output capability checks return S_FALSE without retaining anything.
Embedded/multipart shader containers and noncanonical root encodings are not yet
accepted; the existing compiled sample DXIL remains untouched. Binding commands
and compute/graphics PSO construction continue to fail rather than consume no-op
objects. D3D12CreateDevice remains unavailable.

## Open provenance

The Rust serialization layout and checksum are ports of Microsoft's open
DirectXShaderCompiler revision `b106a961d09221b3c5bdb37be45b679257da08b8`
(v1.8.2505.1), already pinned by `tools/build_dxc.py`. No external serializer or
proprietary/gated converter is added. Adapted portions retain Microsoft's
copyright and the University of Illinois/NCSA license at `cpp/licenses/DXC-NCSA.txt`.

| Pinned source | SHA-256 |
|---|---|
| `lib/DxilHash/DxilHash.cpp` | `d66b631ea28d4c90a8217fac004f9290eaac07f79115451e0c4e954c0b0c94af` |
| `include/dxc/DxilRootSignature/DxilRootSignature.h` | `2628008530957af53d9ac6356040b2822e60e9f28db4d0b3b14255dad7e041c1` |
| `lib/DxilRootSignature/DxilRootSignatureSerializer.cpp` | `277e6f88d3d46901f5b4beec23f13b75bf84f0040b97ae34893dcf26e52e2281` |
| `unittests/DxilHash/DxilHashTest.cpp` | `6a6a37b660abaa5ff7707a7c09006b4e0dde9592096cf80440e72a3b8ffa32e1` |

Primary source: [pinned open DXC](https://github.com/microsoft/DirectXShaderCompiler/tree/b106a961d09221b3c5bdb37be45b679257da08b8).
Retail checksum test vectors come from that pinned unit; this checksum is not a
security or cryptographic-integrity claim. Serialization uses explicit little
endian words; no host pointers or SDK padding enter the byte stream.

## Executed evidence

Nine new core units pass (77 total core tests), covering pinned hash vectors,
source ABI, deep snapshots, cost and namespace checks, sampler/unsupported bounds,
all truncations and repaired corrupt offsets, and both real blob exports/error
paths. Three runtime units pass (52 total), including owned input independence,
all inherited interfaces/metadata, parent lifetime, failure outputs and atomic
ownership with a retained coordinator reference. All 108 Python tests pass.

The shared C++ probe runs locally on physical M4/x86-64 and records all 980 bytes
of six root containers, including empty, 64-DWORD constants, seven descriptors
(the two-block checksum-padding branch), offset/APPEND tables, static samplers
and stage-disjoint registers. Legacy/versioned serialization must be identical;
real COM identity/metadata/parent lifetime assertions run per case. These are CPU
layout/COM observations on a Metal host, not GPU execution of the layouts.
Native Windows SDK/WARP and hosted Rust/Metal execution in CI 42373677759
on `5473a58` agree with clean physical M4 execution on all 980 bytes and COM
assertions. The native report is frozen at `cpp/reference/root-signatures-windows-v1.json`;
future live pairs must also match it. Its full canonical observation SHA-256 is
`f0c59ed82431c950da3780d21b233c267d5b204cfc641ed5a67d7cc46fcdc3fe`.
The initial overall run caught an older host ABI check that still expected root
serialization to be unavailable. That check now requires real success blobs,
owned diagnostics and honest version-1.1 rejection; all five C++ ABI probes pass
locally. The strict-reference run
[42373991380](https://github.com/daeken/Alkyhol/actions/runs/42373991380) passed all
twenty-two jobs on `28cf869`, retaining every previous SDK/shader/graphics and
resource comparison. Downloaded clean reports also match fresh physical M4
execution and the frozen payload on all 980 bytes and COM assertions. The original
subsystem v5 caller passes cleanly at the same revision. The extra frozen
source/byte/joint-drift checks bring the Python suite to 110 passing tests.

The separate original-subsystem caller now links the unchanged original
`RootSignature.cpp` object alongside the three prior queue/allocator/heap objects.
The new v5 observation executes real original Finalize, static-sampler conversion,
repeat-finalization and cache reuse; the original mixed root also supplies a
retained parent interface. The verified original RootSignature.o SHA-256 is
`d87b0cb74eb6306bd6e1d6362123baf3ac1f85e9fb2755a35f95e3a5d397f0a2`.
Its source, all 70 original objects, original shaders/assets and working Rust apps
are preserved. The caller uses the already pinned build compiler/headers and
checks their current provenance; it is not the complete original application.

```sh
cargo test --offline
cargo test --manifest-path cpp/runtime/Cargo.toml --locked
python3 tools/run_cpp_root_signatures.py --output local/cpp-root-signatures.json
python3 tools/run_original_command_manager.py --output local/original-subsystems.json
python3 tools/build_original_miniengine.py --link-only
```
