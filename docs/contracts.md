# Implemented kernel subset

The exported API uses native `extern "C"` conventions, pointer-width integer
handles and Windows-width integer/UTF-16 data. It is for recompiled source.
Pointers must designate valid aligned **host** storage; foreign pointers are not
made safe by null checks. Output parameters must not alias input buffers.

| API | Supported contract | Explicit limits |
| --- | --- | --- |
| GetLastError / SetLastError | Per-thread DWORD state | Success normally preserves prior value |
| CreateFileW | Synchronous regular files; read/write rights; five creation dispositions | Host POSIX paths, valid UTF-16, normal attributes, share mask 7 only, null security/template; no metadata-only handles |
| ReadFile / WriteFile | Byte counts, current position, short reads/writes, EOF | Non-null byte-count output, null OVERLAPPED; no pipes/devices |
| GetFileSizeEx | Current size | Regular files only |
| SetFilePointerEx | Begin/current/end; negative target fails without moving | Nonnegative signed 64-bit target; huge unsigned FILE_BEGIN offsets unsupported |
| SetEndOfFile | Truncate/extend at current offset | Write access required |
| FlushFileBuffers | Host fsync | Write access required; host storage guarantees apply |
| CloseHandle | Release token, reject stale or wrong tokens | Closing during a wait is undefined on Windows; not supported here |
| CreateEventW | Unnamed manual/auto reset, initial state | Null security and name; no cross-process namespace |
| SetEvent / ResetEvent | Saturating signaled bit; notify one/all | No waiter-order guarantee |
| WaitForSingleObject | Event waits, zero/finite/infinite timeout | Other waitable object types unsupported |
| QueryPerformanceCounter/Frequency | Monotonic nanosecond counter, fixed 1GHz frequency | Epoch unrelated to Windows boot time; no exact scheduling guarantee |
| Sleep | Host sleep/yield | No Windows scheduler/alertable behavior emulation |

Unsupported flags/options return `ERROR_NOT_SUPPORTED`; unknown handles fail.
The safe Rust methods return `WinError`; the C facade sets last-error on failure.
CreateFileW sets last-error to 0/ERROR_ALREADY_EXISTS for OPEN_ALWAYS/CREATE_ALWAYS.
The core deliberately accepts only fully unrestricted sharing; it does not claim
to enforce Windows share-denial or cross-process locks. Paths are case-sensitive
according to the host filesystem; DOS drives, UNC paths and backslashes fail.
Read-only creation dispositions beyond OPEN_EXISTING are not implemented.
Host I/O errors are mapped for common cases; ACLs, path-not-found distinctions,
reparse points, device namespaces and exhaustive Windows error mapping remain work.

## Run the portable sample

```sh
cargo build --offline
python3 tools/run_samples.py
```

On Windows use a Developer Command Prompt (MSVC):

```bat
cl /nologo /W4 /WX samples\kernel_contract.c /Fe:kernel_contract.exe
kernel_contract.exe
```

The sample creates its file exclusively; run in an empty scratch directory. The
runner uses an isolated temporary directory and checks the exact JSON observation.
The C source is original project code informed by the linked Microsoft docs, not
a copied upstream program. Sample success covers only its assertions. CI compares
native Windows and native Alkyhol observations when those jobs actually execute.
