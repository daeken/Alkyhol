// Adapted from MicrosoftDocs/win32: Using Event Objects (Synchronization).
// Copyright (c) Microsoft Corporation. MIT: see LICENSE-CODE and README.md.
// Standard C++ threads replace CreateThread/WaitForMultipleObjects; the unnamed
// Win32 manual-reset event remains the writer-to-reader publication boundary.
#ifdef _WIN32
#include <windows.h>
#else
#include "alkyhol.h"
#endif
#include <array>
#include <atomic>
#include <cstdio>
#include <thread>

int main() {
    const HANDLE written = CreateEventW(NULL, TRUE, FALSE, NULL);
    if (!written) return 1;
    std::array<int, 4> buffer = {};
    std::array<bool, 4> results = {};
    std::array<std::thread, 4> readers;
    std::atomic<unsigned> ready{0};
    for (unsigned i = 0; i < readers.size(); ++i) {
        readers[i] = std::thread([&, i] {
            ready.fetch_add(1, std::memory_order_relaxed);
            // A finite timeout makes a broken implementation fail the sample.
            if (WaitForSingleObject(written, 2000) == WAIT_OBJECT_0) {
                results[i] = buffer == std::array<int, 4>{{11, 22, 33, 44}};
            }
        });
    }
    while (ready.load(std::memory_order_relaxed) != readers.size()) {
        std::this_thread::yield();
    }
    buffer = {{11, 22, 33, 44}};
    const bool signaled = SetEvent(written) != 0;
    for (auto& reader : readers) reader.join();
    bool passed = signaled;
    for (bool read : results) passed = passed && read;
    passed = passed && WaitForSingleObject(written, 0) == WAIT_OBJECT_0;
    passed = passed && ResetEvent(written) != 0;
    passed = passed && WaitForSingleObject(written, 0) == WAIT_TIMEOUT;
    const bool closed = CloseHandle(written) != 0;
    if (!passed || !closed) { std::fprintf(stderr, "event publication failed\n"); return 1; }
    std::puts("{\"all_readers\":true,\"manual_stays_signaled\":true,\"reset\":true}");
    return 0;
}
