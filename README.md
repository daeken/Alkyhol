# Alkyhol

A Windows **source compatibility** layer in Rust, with a D3D12-shaped graphics
subset targeting wgpu/Metal. The first consumers are small, portable Win32 sample
programs, eventually followed by recompiled/decompiled application code.

This is an early implementation, not a PE loader, Wine replacement or a claim
that Cyberpunk 2077 runs. Host calling conventions are intentional. Guest addresses
must be translated before entering these APIs; they are never native pointers.

## Evidence first

The PE scanner inventories a locally supplied executable without executing it.
Executable bytes and game-derived inventories stay in ignored `local/` storage.
Regenerate locally with:

```sh
mkdir -p local
python3 tools/pe_imports.py /path/to/Cyberpunk2077.exe --output local/imports.json
```

The inventory covers normal and delay imports. Dynamic lookup, COM methods and
transitive DLL dependencies need separate evidence. See [scope](docs/scope.md).

## Development

The core is dependency-free so file/event conformance work can run without a GPU
or a network connection. Graphics has its own manifest and dependency resolution.

```sh
cargo test --offline
cargo build --offline
python3 -m unittest discover -s tests -v
```

The [kernel contracts and C sample instructions](docs/contracts.md) describe the
implemented exports. `python3 tools/run_samples.py` builds and checks the portable
C probe against the native shared library. CI also compiles that same source
against the Windows SDK and compares the observations. Passing portable
samples demonstrates only those tested contracts, not game compatibility. Native
Windows comparison results must be labeled separately from host-only assertions.
