/* Adapted from MicrosoftDocs/win32: Creating Threads.
 * Copyright (c) Microsoft Corporation. MIT: see LICENSE-CODE and README.md.
 * Three parameterized callbacks use real CreateThread and individual waits.
 * Worker callbacks use no CRT functions; the parent owns data until all terminate. */
#ifdef _WIN32
#include <windows.h>
#define THREAD_CALL WINAPI
#else
#include "alkyhol.h"
#define THREAD_CALL
#endif
#include <stdio.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "line %d: %s (error %lu)\n", __LINE__, #x, (unsigned long)GetLastError()); return 1; } } while (0)

typedef struct MyData {
    DWORD val1, val2, sum, observed_id, exit_code;
    HANDLE gate;
} MYDATA;

static DWORD THREAD_CALL MyThreadFunction(void* parameter) {
    MYDATA* data = (MYDATA*)parameter;
    if (WaitForSingleObject(data->gate, 2000) != WAIT_OBJECT_0) return 0xfffffffe;
    data->sum = data->val1 + data->val2;
    data->observed_id = GetCurrentThreadId();
    return data->exit_code;
}

int main(void) {
    const DWORD parent = GetCurrentThreadId();
    CHECK(parent != 0 && GetCurrentThreadId() == parent);
    const HANDLE gate = CreateEventW(NULL, TRUE, FALSE, NULL);
    CHECK(gate);
    MYDATA data[3] = {{0}};
    HANDLE threads[3];
    DWORD ids[3];
    for (DWORD i = 0; i < 3; ++i) {
        data[i].val1 = i;
        data[i].val2 = i + 100;
        data[i].exit_code = i == 2 ? STILL_ACTIVE : i + 100;
        data[i].gate = gate;
        threads[i] = CreateThread(NULL, 0, MyThreadFunction, &data[i], 0, &ids[i]);
        CHECK(threads[i]);
        CHECK(ids[i] != 0 && ids[i] != parent);
        for (DWORD previous = 0; previous < i; ++previous) CHECK(ids[i] != ids[previous]);
        DWORD code = 0;
        CHECK(GetExitCodeThread(threads[i], &code) && code == STILL_ACTIVE);
        CHECK(WaitForSingleObject(threads[i], 0) == WAIT_TIMEOUT);
    }
    CHECK(SetEvent(gate));
    for (DWORD i = 0; i < 3; ++i) {
        CHECK(WaitForSingleObject(threads[i], 5000) == WAIT_OBJECT_0);
        CHECK(WaitForSingleObject(threads[i], 0) == WAIT_OBJECT_0);
        DWORD code = 0;
        CHECK(GetExitCodeThread(threads[i], &code) && code == data[i].exit_code);
        CHECK(data[i].sum == 2 * i + 100 && data[i].observed_id == ids[i]);
        CHECK(CloseHandle(threads[i]));
    }
    CHECK(CloseHandle(gate));
    puts("{\"parameters\":true,\"thread_ids\":true,\"pending\":true,\"completion\":true,\"exit_codes\":true,\"persistent_signal\":true}");
    return 0;
}
