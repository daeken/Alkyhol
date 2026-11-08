# Static function records throughout isolated DLL lifecycle

`unsafe RegisteredLibrary::load(bytes, rebase, &process)` maps a trusted isolated
DLL, registers its validated static function table, then invokes dynamic
PROCESS_ATTACH. Its `library()` view exposes existing base, export name/ordinal
and relocation queries. Drop invokes PROCESS_DETACH while records remain visible,
removes the registry entry, then unmaps. A rejected initializer receives exactly
one cleanup DETACH before removal/unmapping. Registration-capacity failure occurs
before any guest initializer; mapping/metadata/import/forwarder failures do not
claim a successful load. DLLs without an entry point or static records also work.

The owner borrows ProcessContext through cleanup, enforced by positive/negative
compiler checks. The older LoadedLibrary, LinkedLibrary, LibrarySet and LinkedImage
startup gates remain unchanged. This API supports builtin imports/forwarders only;
it adds no filesystem discovery, guest LoadLibrary/FreeLibrary, loader lock,
reference counting, dependency graph registration, TLS, threads or SEH dispatch.
The [static metadata validation](static-functions.md) and bounded manual unwind
contracts still apply. Raw exported pointers require their owner to remain alive.

```sh
python3 tools/build_pe_registered_dll.py
python3 tools/run_pe_registered_dll.py --directory local/pe/registered-dll --output local/pe/registered-dll/observation.json
```

Two new owned MIT no-CRT DLLs contain real linker-generated static records for
their assembly DllMain and exported Probe. The C lifecycle helper queries the
original record pointers/base for both functions, attempts a rejected static
deletion, verifies lookup remains, and writes raw records during both ATTACH and
DETACH. One variant accepts attach and its Probe returns `0x42a11c0d` through a
real Win64 call; name and biased ordinal 17 select the same pointer. The other
variant rejects attach and records its cleanup DETACH. These source files are
original test code, not Microsoft or game-derived samples.

Windows loads/frees the identical checked DLL snapshots using its actual loader;
the Mac observer uses RegisteredLibrary. All 40 raw DWORDs across both DLLs,
exported result/alias and success/rejection predicates are compared exactly,
alongside both DLL/source hashes, clean revision and normal/forced-rebase host
observations. Failed-load last-error values, OS loader internals and post-unmap
native lookups are outside this observer. Downloaded clean native Windows/hosted Mac reports and clean physical M4 runs
at `331beeb` agree on every raw trace word, call/status result and exact DLL/source
identity. The actual native record at `pe/reference/registered-dll-windows-v1.json`
is frozen and required by default. See
[validation](validation.md) for actual results and tested revisions.

Host units separately check live static visibility and protected deletion during
attach/detach, absence after drop, failed-attach cleanup, capacity rollback before
initializer execution, ordinary/no-entry DLLs, export views and preflight failures.
Portable tests check DLL/EXE kind, export validation, supported static recipes and
every existing auxiliary startup gate. Frozen EXE, dynamic-table, manual-unwind
and older DLL observers remain unchanged. This is finite isolated DLL evidence,
not general Windows loading, exception dispatch or game compatibility.

Primary contracts: [DllMain](https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain),
[FreeLibrary](https://learn.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-freelibrary)
and [AMD64 static function records](https://learn.microsoft.com/en-us/cpp/build/exception-handling-x64).
