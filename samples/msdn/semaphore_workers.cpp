// Adapted from MicrosoftDocs/win32: Using Semaphore Objects.
// Copyright (c) Microsoft Corporation. MIT: see LICENSE-CODE and README.md.
// Keeps twelve workers and ten permits; standard C++ threads replace Win32
// thread creation/join, and retries have a finite deadline instead of spinning forever.
#ifdef _WIN32
#include <windows.h>
#else
#include "alkyhol.h"
#endif
#include <array>
#include <atomic>
#include <chrono>
#include <cstdio>
#include <thread>
#define CHECK(x) do { if (!(x)) { std::fprintf(stderr, "line %d: %s (error %lu)\n", __LINE__, #x, (unsigned long)GetLastError()); return 1; } } while (0)

int main() {
    constexpr LONG capacity = 10;
    const HANDLE semaphore = CreateSemaphoreW(NULL, capacity, capacity, NULL);
    CHECK(semaphore);
    CHECK(!ReleaseSemaphore(semaphore, 1, NULL) && GetLastError() == ERROR_TOO_MANY_POSTS);
    CHECK(!ReleaseSemaphore(semaphore, 0, NULL) && GetLastError() == ERROR_INVALID_PARAMETER);
    // Exactly ten waits can succeed; failed releases above must not change count.
    for (LONG i = 0; i < capacity; ++i) CHECK(WaitForSingleObject(semaphore, 0) == WAIT_OBJECT_0);
    CHECK(WaitForSingleObject(semaphore, 0) == WAIT_TIMEOUT);
    LONG previous = -1;
    CHECK(ReleaseSemaphore(semaphore, capacity, &previous) && previous == 0);
    CHECK(CreateSemaphoreW(NULL, -1, capacity, NULL) == 0 && GetLastError() == ERROR_INVALID_PARAMETER);
    std::array<std::thread, 12> workers;
    std::array<bool, 12> completed = {};
    std::atomic<int> active{0};
    std::atomic<bool> exceeded{false};
    for (unsigned i = 0; i < workers.size(); ++i) {
        workers[i] = std::thread([&, i] {
            const auto deadline = std::chrono::steady_clock::now() + std::chrono::seconds(2);
            while (std::chrono::steady_clock::now() < deadline) {
                const DWORD result = WaitForSingleObject(semaphore, 0);
                if (result == WAIT_OBJECT_0) {
                    const int simultaneous = active.fetch_add(1) + 1;
                    if (simultaneous > capacity) exceeded.store(true);
                    Sleep(5);
                    active.fetch_sub(1);
                    completed[i] = ReleaseSemaphore(semaphore, 1, NULL) != 0;
                    return;
                }
                if (result != WAIT_TIMEOUT) return;
                std::this_thread::yield();
            }
        });
    }
    for (auto& worker : workers) worker.join();
    for (bool success : completed) CHECK(success);
    CHECK(!exceeded.load() && active.load() == 0);
    // All workers must have returned their permits, with no duplicate/lost release.
    for (LONG i = 0; i < capacity; ++i) CHECK(WaitForSingleObject(semaphore, 0) == WAIT_OBJECT_0);
    CHECK(WaitForSingleObject(semaphore, 0) == WAIT_TIMEOUT);
    CHECK(ReleaseSemaphore(semaphore, 2, &previous) && previous == 0);
    CHECK(ReleaseSemaphore(semaphore, 1, &previous) && previous == 2);
    CHECK(CloseHandle(semaphore));
    std::puts("{\"permit_capacity\":true,\"all_workers\":true,\"bounded_active\":true,\"previous_count\":true,\"errors\":true}");
    return 0;
}
