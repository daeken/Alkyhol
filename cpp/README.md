# Original MiniEngine C++ build

The target is Microsoft's complete MiniEngine Sponza sample, not the Rust scene
port. Upstream C++ and HLSL stay verbatim at
`68a2b1e97f71642924e74c7757ca21756ad221b5`. Build tools verify the dependency
revision and reject tracked edits; generated files stay outside the checkout.

Current evidence: all **70 original C++ translation units** and **184 original
project shader entries** compile, including the seven complete DXR libraries.
The complete original application links as an x86-64 Mach-O executable against
Rust's `libalkyhol.a`. Original Core/Model libraries use ordinary static archive
extraction, as in the upstream project. No sample source edits, removed project
units, placeholder shader bytecode or unresolved-symbol linker exceptions are used.

This establishes compilation and linking, **not original application rendering**.
D3D12/DXGI device creation, window/input services and root-signature serialization
remain explicit unavailable exports. The small host entry bridge checks device
creation first and exits 78 with its failing HRESULT; upstream Release assertions
otherwise discard failures and would dereference missing interfaces. It does not
launch the Rust reference port or claim an original frame. See
[runtime status](runtime-status.md) for the current boundary.

C++ virtual-call/WRL probes execute the real Rust ID3DBlob implementation,
including canonical IUnknown identity, failed QueryInterface output clearing,
copy/move/attach/detach lifetimes and synchronized shared-MTA references. Uncoordinated
atomic reference stress is a separate Rust extension: the native SDK experiment
crashed during that phase even with common-MTA initialization. Additional executed
probes cover apartment nesting, header/task contracts, actual file I/O/metadata,
UTF-32 text bridges, wide CRT conventions, compiler extensions and failure outputs.
Native Windows SDK/MSVC, hosted macOS and the physical M4 agree on the shared
synchronized COM probe output and every normalized DirectInput descriptor field,
from identical actual probe source bytes. All twelve jobs passed on `42fff60` in
[CI 42355345904](https://github.com/daeken/Alkyhol/actions/runs/42355345904).
The Rust-only atomic extension and compiler-extension fixtures pass locally;
that does not turn the failed native uncoordinated stress into positive evidence.

The Rust ID3D12Object CPU metadata base also matches the separately frozen 34-record
native WARP metadata contract through actual C++ virtual calls. Its explicit
UTF-32 diagnostic constructor is not D3D12CreateDevice and represents no GPU/fence.
Private-data snapshots preserve foreign COM lifetimes and release outside locks.
The native reference and its acceptance data remain unchanged.

```sh
python3 tools/fetch_cpp_deps.py
python3 tools/build_cpp_clang.py
python3 tools/build_original_miniengine.py --shaders-only
python3 tools/build_original_miniengine.py --cpp-only
python3 tools/build_original_miniengine.py --link-only
python3 tools/run_cpp_probes.py --compiler-tests --output local/cpp-observation.json
```

The compiler bootstrap currently requires installed open LLVM **21.1.6**, including
its development headers/static libraries. It rebuilds four Sema components and
one parser component, the stock driver and required builtin objects, then links
static Clang with LLVM. The checked-in patches supply bounded MSVC record-temporary
reinterpretation, non-const record-reference binding, legacy `typedef public` and
header explicit-instantiation declarations whose body is provided by another TU.
The compiler does not synthesize a missing template body: a negative link fixture
requires the provider. Ordinary Clang modes retain rejection diagnostics.
Temporary storage/lifetime and base adjustments use Clang's AST lowering. Tests
execute copied bits, destruction and non-primary-base adjustment. These are
compiler compatibility extensions, not edits or rewrites of sample code.
`EngineTuning.cpp` additionally receives a forced header after its original PCH to
select its existing modern-CRT branch, avoiding pre-VS2013 math replacements.

Initial application objects target x86-64 and use Rosetta on this ARM Mac to retain
upstream SSE behavior. Rust's `x86_64-apple-darwin` standard library is required
for the linked COM probe. This is native-host source ABI compatibility, not PE
loading. Native `wchar_t` remains 32 bits as in Microsoft's Linux header adapter;
Win32 wide-path bridges must perform explicit conversion to the core UTF-16 APIs.
Do not force short wchar onto the macOS C++ standard library.

`cpp-build.json`, `shader-build.json`, `link-build.json`, compiler commands, hashes, objects and
per-unit diagnostics live under ignored `local/original-miniengine/`. A failed
unit makes the overall build fail. A declaration or a successful object compile
is not an implemented API or a working application. Unsupported runtime behavior
must fail honestly; rendering evidence remains separate from build evidence.
The existing Rust Sakura/Sponza apps remain working reference ports.

## Header origins

`upstream.json` pins every public dependency. `include/{basetsd,rpc,rpcndr,unknwn,
unknwnbase,winapifamily,oaidl,ocidl}.h` adapt Microsoft's MIT Linux header support
from DirectX-Headers. Original copyright notices and the MIT license accompany
them. Changes normalize whitespace and text line endings, make HWND/HDC pointer-sized and BOOL
signed, expose LARGE_INTEGER's normal anonymous fields and set a numeric RPC
version. Other headers/build glue and Rust COM code are Alkyhol MIT code.
DirectX-Headers, DirectXMath, DirectXTex and DirectXMesh are Microsoft MIT projects;
MinGW header notices remain with the fetched originals. Clang source and patches
are Apache-2.0 WITH LLVM-exception. No proprietary converter is used.
