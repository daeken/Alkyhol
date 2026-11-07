# Released Windows peports

The unchanged public-domain `peports.exe` by Chris Wellons from w64devkit 2.11.0
x64 runs through Alkyhol/Rosetta. Its original Windows machine code reads PE files
and prints import/export tables and machine types. It supplies a third released
application alongside u-config and make2compdb, and a separate PE-table parser
running against the exact older frozen owned EXE/DLL inputs.

```sh
python3 tools/build_pe_fixture.py
python3 tools/build_pe_export_fixture.py
python3 tools/fetch_peports_binary.py
python3 tools/run_peports_binary.py --binary local/pe/peports/peports.exe --output local/pe/peports/observation.json
```

## Provenance

The official [w64devkit 2.11.0 release](https://github.com/skeeto/w64devkit/releases/tag/v2.11.0)
archive `w64devkit-x64-2.11.0.7z.exe` is SHA-256
`54c2bb9962e8a30c8270e34c5efc3d5fee579f53087fa7d435b95fbba2af7b2b`.
The fetcher validates the archive before extracting two explicitly named members
with libarchive; it never executes the SFX wrapper.

| Member | SHA-256 |
| --- | --- |
| `w64devkit/bin/peports.exe` (29,848 bytes) | `1a57a02c27cbdd69e1c49ee59a2e84e46684163750700b56d03fad4108c71c3d` |
| `w64devkit/src/peports.c` | `8635550fa472b8393122174427bae61de0630a938ecd09c2718440978a72d670` |

The source agrees byte-for-byte with the [tagged original](https://github.com/skeeto/w64devkit/blob/v2.11.0/src/peports.c).
The [distribution build recipe](https://github.com/skeeto/w64devkit/blob/v2.11.0/Dockerfile)
uses GCC with `-nostdlib`, `-fno-asynchronous-unwind-tables` and kernel32/shell32/
memory support. The tagged [Unlicense](https://github.com/skeeto/w64devkit/blob/v2.11.0/UNLICENSE),
SHA-256 `7e12e5df4bae12cb21581ba157ced20e1986a0508dd10d0e8a4ab9a4cf94e85c`,
accompanies local downloads and CI transfers. No binary/source rebuild or patch is
performed; the complete signed release bytes remain the embedded payload.

## Runtime additions and bounds

The original program reserves a 512-MiB static arena, making SizeOfImage
`0x20008000` (512 MiB plus 32 KiB). The loader's previous 512-MiB policy rejected
it. The bounded image limit is now explicitly **one GiB**. Mappings remain sparse
anonymous demand-zero reservations; no image-sized Vec or eager zeroing is added.
Portable parser tests cover the exact one-GiB boundary, excessive image/section
extents and base overflow. A physical host unit maps the original program's image
size using a synthetic independent layout, executes RX code, reads zeroes and
writes the last BSS page. Other startup/section/W^X/relocation gates remain enforced.
The separate VirtualAlloc bound stays 512 MiB.

Two new kernel32 Win64 imports bring that whitelist to 47:

| Import | Supported contract |
| --- | --- |
| MultiByteToWideChar | CP_UTF8 only; flags zero for valid input, or MB_ERR_INVALID_CHARS. Positive byte counts preserve embedded NULs; -1 scans through a terminating NUL. Output/query counts are 16-bit UTF-16 units, including surrogate pairs and any converted NUL. |
| WideCharToMultiByte | CP_UTF8 only; flags zero for valid input, or WC_ERR_INVALID_CHARS. Positive counts are UTF-16 units; -1 includes the first NUL. Output/query counts are UTF-8 bytes. Default-character and used-default pointers must be NULL. All eight real Win64 arguments cross the bridge. |

Input spans/scans are bounded to one MiB of source bytes. Queries accept zero
output capacity, insufficient buffers fail, and successful conversions copy the
exact required bytes/units. Strict malformed Unicode returns error 1113. Lossy
replacement for malformed input with flags zero, other code pages and other flags
fail explicitly with error 50; there is no silent ACP assumption or replacement
success. Native application comparisons cover valid flag-zero inputs; strict/
malformed/unsupported/buffer boundaries have host assertions only. The source ABI's
32-bit wide-character converters remain separate.

## Corpus and evidence

Sixteen cases include import/export/machine modes, original names/ordinals/data/
forwarder text from the frozen DLL, Unicode/spaced filenames and directories,
stdin PE input, option delimiters and multiple files. Malformed/truncated input,
missing/empty paths and unknown options retain exact original errors/statuses.
A missing-file-then-valid-PE case verifies that the original program recovers and
continues processing the next file. Its GCC builtin setjmp/longjmp executes in
those error paths; this does not implement Windows SEH or exception dispatch.

Owned input EXE/DLL hashes, malformed bytes, corpus, release binary/source/license,
clean revision and exact raw status/stdout/stderr are checked. No path, whitespace,
ordering, symbol or error normalization occurs; only one exact verified loader
prefix is removed. Normal and forced-rebase host execution must agree. Windows
runs the same transferred release PE over the same transferred owned inputs;
actual observations and frozen reference are recorded in [validation](validation.md).
The program's own upstream warning that it is not a security tool remains relevant:
this does not establish equivalence between its PE interpretation and Windows'
loader on arbitrary crafted inputs. No graphics-binary or game compatibility is
claimed.

Primary API contracts: [MultiByteToWideChar](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-multibytetowidechar)
and [WideCharToMultiByte](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-widechartomultibyte).

All 16 cases agree on native Windows, hosted Mac and physical M4 at `64f37ae`.
The actual native record at `pe/reference/peports-windows-v1.json` is frozen and
required by default, including release/source/license/corpus and original owned
PE input identities. This covers the program's valid flag-zero conversion calls;
host-only conversion boundaries remain explicitly separate.

Strict-reference revision `04fb895` passes all 27 CI jobs. Downloaded clean native/
hosted records and fresh clean physical M4 execution at that same revision agree
on all 16 cases and frozen identities. See [executed validation](validation.md)
for CI links, exact test counts and the distinct host/native evidence scopes.
