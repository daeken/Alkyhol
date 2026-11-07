# Native COM reference contracts

`object-private-data-windows-v1.json` preserves the actual 34-record observation
from CI 42356150734 on `e3218ee`, Windows SDK 10.0.26100.0, MSVC 19.51.36260,
using a real D3D12 WARP fence. It contains source/executable hashes, command,
compiler/host/adapter identity and exact output bytes. The custom IUnknown tracks
its own real AddRef/Release/destruction; it is not an Alkyhol object.

Important measured distinctions for the upcoming Rust object implementation:

- Missing byte data returns `887a0002`, sets size zero and leaves output untouched.
- Insufficient space returns `887a0003`, reports required size and copies no bytes.
- A null output is a successful size query without acquiring an interface reference.
- Removing existing bytes returns S_OK; removing absent bytes returns S_FALSE.
- Zero-length non-null data remains a present zero-length entry.
- Null interface assignment remains a present pointer-sized entry, not removal.
- Getting a stored interface acquires a reference; replacement and object destruction
  release retained references. Same-pointer assignment still calls AddRef/Release.
- SetName stores UTF-16 debug-name bytes including NUL, even for an empty name.

This is finite evidence on this SDK/runtime, not all COM behavior. Callback
reentrancy, concurrent setters/getters, allocation failures, invalid pointers,
DXGI object metadata and arbitrary object types remain separate questions.
There is no Metal object implementation, GPU command completion, original C++
initialization or rendering evidence in this reference.

The current observer must still match these exact fields and the original source
hash. Change a contract only with separately measured evidence and an explicit
version/review; do not change this reference to make a Rust candidate pass.
