# Executed validation

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
