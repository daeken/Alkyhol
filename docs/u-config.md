# Released Windows u-config executable

The unchanged Windows `pkg-config.exe` shipped in **w64devkit 2.11.0 x64** now runs
through Alkyhol's PE64/Win64 path on physical M4 under Rosetta. It is **u-config
0.35.0**, Chris Wellons' public-domain pkg-config implementation. This is a useful
off-the-shelf executable downloaded from its official distribution, not a local
rebuild, source port, patched binary or substitute implementation of pkg-config.
The Rust bridges provide process services; all package parsing and dependency
resolution execute the original Windows machine code.

## Run it

```sh
python3 tools/fetch_uconfig_binary.py
python3 tools/translate_pe.py local/pe/u-config/pkg-config.exe --output local/pe/u-config/pkg-config-macos
local/pe/u-config/pkg-config-macos -- --version
PKG_CONFIG_LIBDIR=pe/fixtures/u-config/pc local/pe/u-config/pkg-config-macos -- --dont-define-prefix --cflags --libs alkyhol-demo
python3 tools/run_uconfig_binary.py --binary local/pe/u-config/pkg-config.exe --output local/pe/u-config/observation.json
```

The version is `0.35.0`. The example package query emits compiler and linker flags
for the owned demo/base dependency fixtures. `--` separates host loader options
from guest arguments; `--rebase --` additionally exercises a different image base.
Normal execution preserves guest stdout/stderr without host chatter. Tests opt
into `--loader-info` to validate the exact mapping/entry diagnostic prefix.
The host creates a UTF-16 Windows command line, with distinct argv[0] and regular
argument quoting, rather than handing guest code a Unix argv pointer. It snapshots
the host environment, maps standard streams to real duplicated descriptors, and
provides the executable's host path through GetModuleFileNameW. This is a bounded
process view, not a Windows PEB/TEB or general CRT/DLL startup implementation.

## Exact upstream provenance

Official distribution:
[release v2.11.0](https://github.com/skeeto/w64devkit/releases/tag/v2.11.0), asset
`w64devkit-x64-2.11.0.7z.exe`, archive SHA-256
`54c2bb9962e8a30c8270e34c5efc3d5fee579f53087fa7d435b95fbba2af7b2b`.
The verified archive is read as data using libarchive; its SFX entry is never run.
Only specifically named members are extracted to controlled local paths:

| Member | SHA-256 |
| --- | --- |
| `w64devkit/bin/pkg-config.exe` (46,232 bytes) | `390f6c01b87cd8c71d3b5aa6cabd1cb63539ba6204972d920347f2376ef4debc` |
| `w64devkit/src/pkg-config.c` | `b1d5362c0c84753bc3a22831866cb92ada5b467da6fda406a33162e35e201210` |

The source member agrees byte-for-byte with the upstream [u-config v0.35.0
amalgamation](https://github.com/skeeto/u-config/releases/tag/v0.35.0). The
[w64devkit build recipe](https://github.com/skeeto/w64devkit/blob/v2.11.0/Dockerfile)
pins 0.35.0 and builds this file with `-nostdlib` and
`-fno-asynchronous-unwind-tables`. The released executable has 15 named kernel32
imports, eleven DIR64 fixups and no static TLS, exception or load-config directory.
No existing unsupported-startup gate is removed to accept it. The complete input
file, including its certificate bytes, remains unchanged in the read-only Mach-O
payload, whose extracted hash is checked against the release. The pinned
[upstream UNLICENSE](https://github.com/skeeto/u-config/blob/v0.35.0/UNLICENSE), SHA-256
`7e12e5df4bae12cb21581ba157ced20e1986a0508dd10d0e8a4ab9a4cf94e85c`, accompanies
local downloads and CI transfers. No game files or game-derived data are used.

## Process-service contract

Ten additional named Win64 bridges support this executable, bringing the explicit
kernel32 whitelist to 34 names. Existing file/event/semaphore/TLS bridges continue
to implement their [bounded kernel contracts](contracts.md).

| API | Implemented behavior / explicit bounds |
| --- | --- |
| GetCommandLineW | Stable, NUL-terminated UTF-16 snapshot; quoted Unicode argv; Windows 32767-unit command limit. |
| GetEnvironmentVariableW | Snapshotted environment, ASCII-insensitive key identity, exact Unicode values, missing/empty distinction and required-buffer sizing. Non-ASCII key queries fail explicitly. |
| GetModuleFileNameW | Current executable only (`hModule=0`), host path, UTF-16 length, truncation and NUL termination. Other modules fail. |
| GetStdHandle | Inherited stdin/stdout/stderr, each backed by a real owned duplicated host descriptor. Invalid selector fails. |
| GetConsoleMode | Real terminal detection. Redirected streams fail with invalid-handle; terminal input/output expose the bounded processed/wrap defaults. No mode-setting API is advertised. |
| WriteConsoleW | Valid UTF-16 to UTF-8 terminal output, counts UTF-16 units; redirected handles, wrong direction, reserved data and invalid input fail. |
| VirtualAlloc | NULL address, combined reserve+commit, PAGE_READWRITE, up to 512 MiB per allocation. Real zeroed, 64-KiB-aligned anonymous mappings retained until context destruction; other modes/addresses fail. No VirtualFree import is offered. |
| FindFirstFileW / FindNextFileW / FindClose | Owned stable-directory search, 592-byte SDK layout, real names/sizes/timestamps/directory and read-only metadata, correct exhaustion/closed-handle errors. Supports exact names, `*`, `*.*` and `*.extension`, ASCII-insensitive matching and relative backslash separators. Non-ASCII literal patterns, other wildcard forms, wildcard directories, drive/UNC paths, symlink/special entries and oversized names fail; ASCII `*.pc` still returns Unicode filenames. Alternate names and Windows archive-bit tracking are absent. |

ReadFile/WriteFile now route inherited stream handles to real I/O with direction,
error and byte-count checks; ordinary files still use the Rust kernel. File-path
bridges normalize relative backslash separators while rejecting drive/UNC paths.
Kernel and companion objects reserve tokens from one allocator, preventing alias
with a live file, stream, event, semaphore or search; tokens remain positive and
32-bit representable. Closing a search requires FindClose, not CloseHandle.

The released program declares CreateFileW's template parameter as int32, leaving
the upper half of its Win64 stack slot unspecified. Kernel-handle arguments are
therefore canonicalized from their low 32 bits, as described in Microsoft's
[handle interoperability contract](https://learn.microsoft.com/en-us/windows/win32/winprog64/interprocess-communication).
This rule applies to kernel handles, not memory pointers, module addresses or
arbitrary opaque search handles. The original source-ABI kernel contract remains
strict; its file-sharing/security/template limitations are not silently relaxed.

## Execution evidence

Eleven corpus cases exercise version output, dependency flags, private static
libraries, package versions, quoted Unicode/trailing-backslash and empty argument
values, successful/failed version requirements, missing-package diagnostics,
directory listing and validation. The runner creates three owned `.pc` files in
a directory containing spaces and a surrogate-pair Unicode name; no installed
packages or inherited PKG_CONFIG settings influence the cases. All status/stdout/
stderr bytes are checked. Only listing order is normalized (whole lines, preserving
duplicates), because filesystem enumeration order is unspecified; raw bytes are
retained in the report. Loader diagnostics are removed only as one exact checked
entry prefix, never by discarding arbitrary stderr.

All eleven cases pass on physical M4 in normal and forced-rebase execution. Native
Windows and hosted Mac agree on all eleven cases in CI 42386144339 at `1ccfd87`;
a fresh clean physical M4 run of the transferred executable agrees too. The
full native record is frozen at `pe/reference/u-config-windows-v1.json`, and
future pairs must match its exact inputs, binary and observations. Raw stream
records are checked against the compared outputs, including duplicate detection.
All 27 jobs pass at `1ccfd87`; strict-reference PE run 42386424722 then passes
all three jobs at `9645f17`, with another clean matching physical M4 run.
Twenty-three PE Rust tests pass,
including real PTY Unicode output, mapped memory, environment/buffer boundaries,
search layout/exhaustion, file separators and nonzero upper handle bits. Ten
portable tests pass on native arm64. Root tests (78), Python tests (134), formatting
and PE Clippy pass. The original frozen 24-import/19-word PE observer still passes
unchanged. Direct unit assertions, PTY output, released-binary behavior and native
Windows observations are separate evidence; see [validation](validation.md).

This demonstrates this program's tested paths. It does not add general DLL loading,
Windows unwinding, static TLS, guest threads, binary COM/graphics or game support.
Those remain separate milestones; working original-source and Rust graphics
samples are preserved.
