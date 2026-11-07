# Microsoft sample adaptations

Code provenance: [MicrosoftDocs/win32 revision
51c9b7cc54ef393b6406c8320a7840fdd88209d5](https://github.com/MicrosoftDocs/win32/tree/51c9b7cc54ef393b6406c8320a7840fdd88209d5).
Microsoft's code is MIT licensed; the original notice is in [LICENSE-CODE](LICENSE-CODE).
These are adapted subsets, not unmodified upstream conformance programs.

| Port | Pinned source | Deliberate adaptations and limits |
| --- | --- | --- |
| `file_write.c` | [Opening a File for Reading or Writing](https://github.com/MicrosoftDocs/win32/blob/51c9b7cc54ef393b6406c8320a7840fdd88209d5/desktop-src/FileIO/opening-a-file-for-reading-or-writing.md) | Ports the synchronous **write** example. Fixed UTF-16 scratch filename; unrestricted sharing replaces upstream exclusive sharing; diagnostics use stderr instead of FormatMessage/LocalAlloc; checks exact byte count, closes, then adds synchronous readback and EOF checks. Does not port the separate asynchronous ReadFileEx/SleepEx example and does not test exclusive sharing. |
| `event_readers.cpp` | [Using Event Objects](https://github.com/MicrosoftDocs/win32/blob/51c9b7cc54ef393b6406c8320a7840fdd88209d5/desktop-src/Sync/using-event-objects.md) | Preserves four readers gated by a manual-reset event after the writer publishes a shared buffer; fills the upstream buffer TODOs. Uses an unnamed event and `std::thread`/join instead of CreateThread/WaitForMultipleObjects. Finite waits replace infinite waits; adds content, persistent-signaled and reset assertions. Thread creation, multiple-object waits, named objects and precise scheduling are not covered. |
| `semaphore_workers.cpp` | [Using Semaphore Objects](https://github.com/MicrosoftDocs/win32/blob/51c9b7cc54ef393b6406c8320a7840fdd88209d5/desktop-src/Sync/using-semaphore-objects.md) | Preserves twelve workers, ten permits, zero-timeout acquisition retries, five-millisecond sample work, and one-permit release. Uses `std::thread`/join instead of Win32 thread APIs. A steady-clock retry deadline and yields replace the unbounded busy loop. Adds deterministic capacity/previous-count/error checks, active-worker bounds and final permit accounting. No assertion of fairness, waiter order, peak occupancy or scheduler precision. |
| `thread_local_storage.cpp` | [Using Thread Local Storage](https://github.com/MicrosoftDocs/win32/blob/51c9b7cc54ef393b6406c8320a7840fdd88209d5/desktop-src/ProcThread/using-thread-local-storage.md) | Preserves a process-wide TLS index, four workers owning 256-byte allocations, and a common function retrieving each worker's data. Uses C++ threads/join and unique_ptr instead of CreateThread/LocalAlloc/LocalFree. Asserts all payload bytes and pointer identity instead of printing variable addresses; replaces the five-second sleep with a finite barrier that lets workers store values concurrently. Adds zero/null/last-error, main-thread isolation and out-of-range index checks. No PE static TLS, DLL callbacks, FLS or allocator/thread API compatibility is tested. |

`python3 tools/run_samples.py` compiles and executes all four ports plus the original
kernel probe. Each runs in its own empty temporary directory. The same files use
kernel32 via the Windows SDK on Windows and Alkyhol's C header/library elsewhere.
The runner rejects nonzero exits, timeouts, malformed or unexpected observations.
CI compares every sample's exact deterministic observation on all three hosts.

The relaxed `ready` counter in the event port establishes that reader threads
have started; it does not publish the buffer. The Win32 event is the publication
boundary. Readers write distinct `std::array<bool>` elements; joining them makes
their results visible to the main thread. No claim is made that all readers are
already blocked inside the wait when the writer signals.
