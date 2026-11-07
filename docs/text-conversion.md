# Direct PE UTF conversion boundary observer

The owned MIT no-CRT PE directly calls kernel32 MultiByteToWideChar and
WideCharToMultiByte using real six/eight-argument Win64 signatures. It complements
the released peports corpus, which only exercises valid UTF-8/UTF-16. Both
functions now implement default U+FFFD replacement for malformed Unicode, while
MB_ERR_INVALID_CHARS/WC_ERR_INVALID_CHARS reject it with error 1113. CP_UTF8 remains
the only supported code page; other code pages/flags fail honestly.

```sh
python3 tools/build_pe_text_conversion.py
python3 tools/run_pe_text_conversion.py --fixture local/pe/text-conversion/text-conversion.exe --output local/pe/text-conversion/observation.json
```

The C source defines twenty-eight UTF-8 and ten UTF-16 inputs, each with default and
strict flags: 76 sizing-query cases, 50 exact-size writes for successful queries,
and two separate insufficient-buffer calls. Inputs include empty strings,
explicit embedded NULs, NUL-terminated ASCII/supplementary characters, U+10FFFF,
illegal bytes, truncated sequences, UTF-8-encoded surrogates, overlong/out-of-range
UTF-8, lone/repeated UTF-16 surrogates, and invalid zero input lengths.

Each query records its return value and actual last error after an explicit
sentinel. A successful query is followed by an exact-size write, recording its
return/error and all sixteen destination units/bytes, including the untouched
sentinel tail. Strict-invalid and zero-length cases only make a sizing query;
they deliberately do not make a write call after that failed query. No conclusions
about failed conversion write buffers follow from these records. Separate short-
buffer calls compare return/error only, because their destination contents are
not part of this observer's contract. The C probe does not record or normalize
those contents. All 1,832 emitted DWORDs are compared exactly.

The first native comparison exposed a difference from Rust's lossy decoder:
Windows consumes an invalid leading pair for UTF-8-encoded surrogates and scalars
above U+10FFFF, producing two replacements for `ED A0 80` and three for
`F4 90 80 80`. The corrected runtime matches every word of that original probe on
the unchanged first PE. The expanded matrix adds valid neighboring boundaries,
overlong lower prefixes, illegal leaders, invalid continuations and valid suffixes
to check the streaming adapter. Native Windows/hosted Mac/physical M4 agree on every word of the expanded matrix
at `c9a974c`, including those added boundary hypotheses. This upgrades those
finite cases to independently measured observations.

The producer validates every call identity/order, valid-query success, strict/
zero-length errors, query/write count agreement, output-unit width, preserved
tails, full raw-byte decoding and complete record count. Windows receives the
identical PE/source manifest, with no native rebuild. Native and Alkyhol outputs,
input hashes and clean revisions must agree, including normal/forced-rebase host
execution; the actual Windows record is frozen at `pe/reference/text-conversion-windows-v1.json`
and required by default, including exact input identity and all 1,832 words.
The source is original test code, not copied Microsoft sample code or game data.
This finite corpus does not establish every Windows code page or string behavior.

Source-byte spans/scans remain capped at one MiB. Positive input lengths preserve
embedded NULs; -1 includes the first NUL. Queries return UTF-16 units or UTF-8 bytes
as appropriate. Existing source compatibility uses 32-bit wchar_t and remains
separate. ACP/legacy code pages, best-fit/default-character machinery and other
flags are unsupported. Negative/oversized/unaligned/null and reserved-pointer
boundaries have host assertions unless explicitly included above; failed-write
buffer contents remain unmeasured by this comparison. Existing released programs
and frozen native inputs are preserved.

Executed evidence, counts and precise revisions are in [validation](validation.md).
Primary API contracts: [MultiByteToWideChar](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-multibytetowidechar)
and [WideCharToMultiByte](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-widechartomultibyte).
