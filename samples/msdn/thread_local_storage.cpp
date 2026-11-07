// Adapted from MicrosoftDocs/win32: Using Thread Local Storage.
// Copyright (c) Microsoft Corporation. MIT: see LICENSE-CODE and README.md.
// Standard C++ threads and owned 256-byte arrays replace CreateThread and LocalAlloc.
#ifdef _WIN32
#include <windows.h>
#else
#include "alkyhol.h"
#endif
#include <array>
#include <atomic>
#include <chrono>
#include <cstdio>
#include <memory>
#include <thread>
#define CHECK(x) do { if (!(x)) { std::fprintf(stderr, "line %d: %s (error %lu)\n", __LINE__, #x, (unsigned long)GetLastError()); return 1; } } while (0)

static bool CommonFunc(DWORD index, unsigned char seed) {
    const auto data = static_cast<unsigned char*>(TlsGetValue(index));
    if (!data || GetLastError() != ERROR_SUCCESS) return false;
    for (unsigned i = 0; i < 256; ++i) {
        if (data[i] != static_cast<unsigned char>(i ^ seed)) return false;
    }
    return true;
}

int main() {
    const DWORD index = TlsAlloc();
    CHECK(index != TLS_OUT_OF_INDEXES);
    SetLastError(123);
    CHECK(TlsGetValue(index) == NULL && GetLastError() == ERROR_SUCCESS);
    int main_value = 42;
    CHECK(TlsSetValue(index, &main_value));
    std::array<std::thread, 4> workers;
    std::array<bool, 4> results = {};
    std::atomic<unsigned> ready{0};
    for (unsigned i = 0; i < workers.size(); ++i) {
        workers[i] = std::thread([&, i] {
            SetLastError(123);
            if (TlsGetValue(index) != NULL || GetLastError() != ERROR_SUCCESS) return;
            std::unique_ptr<std::array<unsigned char, 256>> data(new std::array<unsigned char, 256>{});
            const auto seed = static_cast<unsigned char>(i + 1);
            for (unsigned offset = 0; offset < data->size(); ++offset) {
                (*data)[offset] = static_cast<unsigned char>(offset ^ seed);
            }
            if (!TlsSetValue(index, data->data())) return;
            ready.fetch_add(1);
            const auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(2);
            while (ready.load() != workers.size()) {
                if (std::chrono::steady_clock::now() >= deadline) return;
                std::this_thread::yield();
            }
            const bool content = CommonFunc(index, seed);
            const bool pointer = TlsGetValue(index) == data->data();
            const bool cleared = TlsSetValue(index, NULL) != 0;
            SetLastError(123);
            results[i] = content && pointer && cleared && TlsGetValue(index) == NULL && GetLastError() == ERROR_SUCCESS;
            // The array is freed by its owner, not TlsFree or thread-slot cleanup.
        });
    }
    for (auto& worker : workers) worker.join();
    for (bool success : results) CHECK(success);
    CHECK(TlsGetValue(index) == &main_value && main_value == 42);
    CHECK(TlsSetValue(index, NULL));
    CHECK(TlsFree(index));
    CHECK(TlsGetValue(TLS_OUT_OF_INDEXES) == NULL && GetLastError() == ERROR_INVALID_PARAMETER);
    CHECK(!TlsSetValue(TLS_OUT_OF_INDEXES, NULL) && GetLastError() == ERROR_INVALID_PARAMETER);
    CHECK(!TlsFree(TLS_OUT_OF_INDEXES) && GetLastError() == ERROR_INVALID_PARAMETER);
    std::puts("{\"initial_null\":true,\"thread_isolation\":true,\"common_payload\":true,\"null_clears_error\":true,\"invalid_index\":true}");
    return 0;
}
