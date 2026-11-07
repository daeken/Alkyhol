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
cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle
```

Create `local/` before requesting a report there. Graphics commands need access to
the crates in their lockfile and a usable backend. Linux without hardware must add
`-- --allow-software` and provide Vulkan/llvmpipe. A missing backend or timeout fails.
