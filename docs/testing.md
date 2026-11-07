# Function and logical-unit test map

Tests assert the implemented contracts, including explicit unsupported behavior.
This map is not a coverage percentage or evidence for untested Win32/game behavior.
Private helpers are exercised through their public callers.

| Unit/functions | Tests and observations |
| --- | --- |
| GetLastError, SetLastError; thread-local storage | `tests/ffi.rs`: roundtrip, success preservation, error propagation; `tests/kernel.rs`: thread isolation |
| CreateFileW; UTF-16 decoder; result-to-handle helper | `tests/ffi.rs`: valid path, null/invalid/unbounded UTF-16, DOS/UNC/backslash rejection, security/template rejection; `tests/kernel.rs`: creation dispositions, flags, rights, last-error |
| ReadFile, WriteFile; boolean helper | `tests/ffi.rs`: byte counts, short read, untouched tail, EOF, zero-length null buffer, missing outputs, OVERLAPPED rejection, invalid handles, last-error preservation; core rights tests |
| GetFileSizeEx, SetFilePointerEx | `tests/ffi.rs`: size, begin/end seeks, optional output, null size, failed output preservation; core tests: current-relative/negative/overflow seeks preserve position |
| SetEndOfFile, FlushFileBuffers | Facade lifecycle, invalid handles; core truncation and flush, read-only denial |
| CloseHandle; Kernel insert/lookup; global token allocation | Facade close/stale token and last-error; core stale/cross-kernel handles, wrong object types, all file methods rejecting events |
| CreateEventW, SetEvent, ResetEvent, WaitForSingleObject | Facade creation/unsupported options/nonzero BOOLs, initial state, finite/zero/infinite waits on signaled events, stale handles; core saturation, manual persistence/reset, concurrent single/multiple readers |
| CreateSemaphoreW, ReleaseSemaphore, semaphore waits | `tests/semaphore.rs`: invalid signed count boundaries, exact permit consumption, zero/finite/infinite waits, maximum overflow without mutation, previous counts/optional output, stale/cross-kernel/wrong-type handles, unsupported names/security, concurrent batch release and serialized previous counts; Microsoft worker sample compares deterministic checks against Windows |
| TlsAlloc/TlsFree/TlsGetValue/TlsSetValue; allocation table/generation tags | `tests/tls.rs`: opaque/null pointer roundtrip, last-error behavior, slot/thread isolation, full-pool exhaustion and forced reuse with zero initialization in an already-existing thread, invalid/unallocated indices. Exhaustion/forced-reuse tests own this test binary's pool. Microsoft sample separately compares four workers' actual pointer/content/null observations against Windows. |
| CreateThread/GetExitCodeThread/GetCurrentThreadId; completion monitor/thread waits | `tests/thread.rs`: callback/parameter and DWORD bits, optional/null outputs, pending/finite/infinite/persistent waits, returned 259, stable and distinct IDs including foreign host threads, full host TLS destructor completion, closing active handles without termination, wrong/cross-kernel/stale handles, unsupported options and explicit Rust panic failure. Microsoft CreateThread port separately compares parameters, IDs, pending/completion and exit codes against Windows. |
| QueryPerformanceCounter, QueryPerformanceFrequency, Sleep | Facade nonnegative monotonic counters, frequency, null outputs, zero/positive sleep and last-error preservation; no scheduler-precision assertion |
| WinError I/O mapping | `tests/kernel.rs`: common error kinds and unknown-error failure |
| GraphicsCommandList new/device/record/close/reset/commands | `tests/graphics.rs`: lifecycle errors, reset discards commands, channel-by-channel invalid colors rejected without appending |
| validate_submission, require_target | Closed-list/device/token/state checks; sequential transitions, unsupported draws, no partial state commit |
| copy_footprint, GraphicsError display | Zero dimensions, exact alignment boundaries, u32 addition/multiplication rejection, u64 large products; error display |
| D3D12CreateDevice, adapter_info | Every `graphics/tests/device.rs` test requires a real backend and prints its identity; absence fails, software only via explicit opt-in |
| CreateCommittedResource, CreateCommandList, ExecuteCommandLists | GPU extent checks, open/foreign lists, resource and pipeline ownership, rejected list preserves actual pixels and states |
| CreateGraphicsPipelineState, backend error conversion | Invalid WGSL produces a backend error; valid pipeline can subsequently be created; foreign pipeline rejected |
| ReadbackRGBA8 | GPU wrong-state rejection, padded 65-pixel rows match every expected pixel; triangle sample checks draw pixels and replay |
| PE construction/take/unpack/offset/string/thunks/imports/inventory/CLI | `tests/test_pe_imports.py`: 32/64-bit and normal/delay/legacy import fixtures, ordinal/named imports, fallback lookup, bounds, malformed data, hash identity and CLI success/failure |
| Sample compile/run/observation/reporting | `tests/test_run_samples.py`: strict booleans/key sets, duplicate/malformed/multiple JSON rejection, actual process failure and timeout, path arguments; actual six-sample runs check compiler/link/library/report integration |

Run root tests with `cargo test --offline`, Python tests with
`python3 -m unittest discover -s tests -v`, and sample integration with
`cargo build --offline && python3 tools/run_samples.py`.
Run GPU tests with `cargo test --manifest-path graphics/Cargo.toml --locked -- --nocapture`
and the drawing sample with `cargo run --manifest-path graphics/Cargo.toml --locked --example hello_triangle`.
Linux software Vulkan requires `ALKYHOL_ALLOW_SOFTWARE=1` for tests and
`-- --allow-software` for the sample, plus a usable Vulkan adapter.

Sample reports use schema 2, retain the original kernel `observation` field and
add all six results under `samples`. CI compares the full `samples` mapping.
Reports bind source, runner, build inputs, executed library and executable hashes,
plus compiler/host/revision identities. These describe the files that actually ran;
they do not prove universal correctness or that a library was freshly rebuilt.
