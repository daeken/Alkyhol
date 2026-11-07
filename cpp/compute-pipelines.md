# Bounded compute PSO creation

The restricted Rust Metal device now implements real
`ID3D12Device::CreateComputePipelineState`. It translates actual DXIL through the
existing open dxil-spirv/SPIRV-Cross pipeline and compiles the returned MSL with
Metal. The object retains a real native compute pipeline, reflected resource
metadata, copied DXIL, copied validated root layout and parent device. No manual
MSL shader, empty pipeline or successful fallback substitutes for conversion.

## Exact boundary

The full typed ID3D12PipelineState vtable has nine slots. Canonical identity,
Object/DeviceChild/Pageable interfaces, naming, private metadata and parent
ownership are implemented. Atomic references are independently exercised with a
retained coordinator reference. GetCachedBlob explicitly returns E_NOTIMPL and
clears its output: Windows cache blobs are not fabricated. CachedPSO inputs,
nonzero pipeline flags and node masks beyond 0/1 are unavailable.

A non-null same-device root object and one executable DXIL compute entry are
required. The translator selects the declared entry name; it never guesses
`main`. Graphics programs, libraries, ambiguous entries, legacy DXBC instructions
and embedded RTS0 root signatures fail. The executable shader must have a fixed
positive thread-group size fitting the native pipeline, at most 1024 threads.
MSL compilation disables fast math and uses Metal language version 3.1.

This first PSO contract accepts single buffer resources only, without ray queries
or auxiliary GetDimensions metadata. Reflected register class, register space
and register must be covered by ALL-visible root buffer descriptors or finite
descriptor-table ranges. Referenced CBVs mapped to root constants remain
unavailable until their complete size/packing contract is implemented. Resource
arrays, texture/sampler bindings and other unsupported lowering fail instead of
silently losing bindings. Unused root parameters remain owned metadata.

Command-list SetPipelineState, shader root bindings, initial PSOs on list
creation/reset, and D3D12 Dispatch still fail explicitly. This milestone does not
advertise a D3D12 feature level, enable production D3D12CreateDevice, implement
root tables at submission time or add graphics PSOs. The complete original
application remains compiled/linked and exits 78 before original initialization.

## Fixture and executed evidence

The committed 4932-byte `cpp/tests/compute_probe.dxil` is regenerated with pinned
open DXC from the existing MIT `shader/tests/fixtures/compute.hlsl`. Its SHA-256
is `e3084e25dfa23ef53e98eb1dd6eac386c3695718ac03d6631904b6711cc11d4e`,
the identical container used by the earlier native WARP/Metal compute comparison.
The paired C++ header is checked byte-for-byte against that binary. No original
sample shader header is changed; no proprietary/gated converter is added.

Six new runtime units pass (58 total): source layout/bounds, reflected
class/space/visibility coverage and explicit unsupported bindings, real pipeline
ownership and metadata, foreign-device rejection/failure outputs/capability
checks, atomic ownership, and actual execution of the COM-created native pipeline.
The GPU unit submits direct Metal commands with reflected buffer indices and
checks all 76 data words plus 20 untouched tail words against the established
independent scalar contract. This is physical M4 Metal execution of the actual
compiled object; it is not D3D12 command-list dispatch. The earlier native WARP
96-word observation is separate evidence and remains unchanged.

Two translator units exercise automatic entry selection (CSMain rather than
main) and graphics/corrupt-container rejection. All 114 Python tests, runtime and
shader Clippy/formatting pass. A shared C++ source executes construction,
null-output capability probes, two real PSO objects, canonical/Pageable identity,
metadata after discarding input bytecode and retained parent lifetime on physical
M4/x86-64. It compares those common construction/COM assertions against native
Windows SDK/WARP. All twenty-three jobs passed on `ded5bbd` in
[42375900712](https://github.com/daeken/Alkyhol/actions/runs/42375900712). Clean
downloaded native/hosted records and physical M4 agree on every common observation
and shared source/fixture identity. The report is frozen at
`cpp/reference/compute-pipelines-windows-v1.json`; future pairs also match it.
The first run failed MSVC compilation on an int-to-byte fill in the observer; the
fix types the fill value and leaves all assertions/runtime semantics intact.
The freeze adds an immutable native/source fixture test (115 Python tests pass);
a fresh complete strict-reference CI result remains pending. This C++ probe does
not execute shader results after discarding inputs. GetCachedBlob is tested
as unavailable only on Rust; this is not full native pipeline API parity.

The original subsystem caller now links five verified unchanged original objects,
adding `MiniEngine/Core/PipelineState.cpp`. Its ComputePSO::Finalize and original
cache path execute with a separately supplied established compute fixture. Two
original helper wrappers reuse one real pipeline object; the v6 report also
retains every previous queue/copy/allocator/heap/root check. Four root helper
wrappers are exercised in total. The original PipelineState.o SHA-256 is
`9e5388385a6529918d91463bd67d1dd318be46f45c49803818d7c7a009b4770a`.
This is unchanged original C++ helper execution with an owned test shader, not
original MiniEngine HLSL execution through COM. The original compiled shaders
contain embedded root signatures and are still outside this PSO subset. Their
source, generated headers, all 70 original objects and working Rust apps remain
unchanged. General embedded-root lowering and D3D12 bindings/dispatch are later
milestones; original rendering and game compatibility are not established.

```sh
cargo test --manifest-path cpp/runtime/Cargo.toml --locked
python3 tools/build_cpp_compute_probe.py --check
python3 tools/run_cpp_compute_pipelines.py --output local/compute-pipelines.json
python3 tools/run_original_command_manager.py --output local/original-subsystems.json
```
