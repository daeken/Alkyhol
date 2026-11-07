# Original MiniEngine C++ build

The target is Microsoft's complete MiniEngine Sponza sample, not the Rust scene
port. Upstream C++ and HLSL stay verbatim at
`68a2b1e97f71642924e74c7757ca21756ad221b5`. Build tools verify the dependency
revision and reject tracked edits; generated files stay outside the checkout.

Current evidence: all 184 original project shader entries compiled using open
DXC, including the seven complete original DXR libraries. The actual unmodified
`ModelViewer.cpp` compiled to an x86-64 Mach-O object. The first complete project
scan compiled 42/70 C++ translation units; remaining Windows/CRT header gaps are
being implemented. Complete application linking and rendering are not established.
A C++ virtual-call/WRL probe executes the real Rust `ID3DBlob` implementation,
including canonical IUnknown identity, QueryInterface failure outputs, copy/move/
attach/detach lifetimes and concurrent reference ownership. Separate Rust tests
cover allocation failure, null outputs and the local zero-length contract.
Native Windows edge-case return codes have not been measured for this new export.

```sh
python3 tools/fetch_cpp_deps.py
python3 tools/build_cpp_clang.py
python3 tools/build_original_miniengine.py --shaders-only
python3 tools/build_original_miniengine.py --cpp-only
```

The compiler bootstrap currently requires installed open LLVM **21.1.6**, including
its development headers/static libraries. It rebuilds only three Sema components,
the stock driver and required builtin objects, then links static Clang with LLVM.
The checked-in patches supply bounded MSVC record-temporary reinterpretation and
non-const record-reference binding. The ordinary Clang modes retain their standard
rejection paths. Temporary storage/lifetime and base adjustments use Clang's own
AST lowering. This is a compiler compatibility extension, not an edit or source
rewrite of the sample. A local executable tests copied bits and destruction.

Initial application objects target x86-64 and use Rosetta on this ARM Mac to retain
upstream SSE behavior. Rust's `x86_64-apple-darwin` standard library is required
for the linked COM probe. This is native-host source ABI compatibility, not PE
loading. Native `wchar_t` remains 32 bits as in Microsoft's Linux header adapter;
Win32 wide-path bridges must perform explicit conversion to the core UTF-16 APIs.
Do not force short wchar onto the macOS C++ standard library.

`cpp-build.json`, `shader-build.json`, compiler commands, hashes, objects and
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
