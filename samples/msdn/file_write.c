/* Adapted from MicrosoftDocs/win32: Opening a File for Reading or Writing.
 * Copyright (c) Microsoft Corporation. MIT: see LICENSE-CODE and README.md.
 * This ports the synchronous WRITE example, adding a synchronous readback oracle.
 * The upstream asynchronous ReadFileEx example is not implemented here. */
#ifdef _WIN32
#include <windows.h>
#else
#include "alkyhol.h"
#endif
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "line %d: %s (error %lu)\n", __LINE__, #x, (unsigned long)GetLastError()); return 1; } } while (0)
int main(void) {
    const WCHAR path[] = {'m','s','d','n','-','w','r','i','t','e','.','b','i','n',0};
    const char payload[] = "This is some test data to write to the file.";
    const DWORD length = (DWORD)(sizeof(payload) - 1);
    const DWORD share = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    DWORD transferred = 0;
    HANDLE file = CreateFileW(path, GENERIC_WRITE, share, NULL,
                             CREATE_NEW, FILE_ATTRIBUTE_NORMAL, 0);
    CHECK(file != INVALID_HANDLE_VALUE);
    CHECK(WriteFile(file, payload, length, &transferred, NULL));
    CHECK(transferred == length);
    CHECK(CloseHandle(file));
    file = CreateFileW(path, GENERIC_READ, share, NULL,
                       OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, 0);
    CHECK(file != INVALID_HANDLE_VALUE);
    char bytes[sizeof(payload)] = {0};
    CHECK(ReadFile(file, bytes, (DWORD)sizeof(bytes), &transferred, NULL));
    CHECK(transferred == length && memcmp(bytes, payload, length) == 0);
    CHECK(ReadFile(file, bytes, 1, &transferred, NULL) && transferred == 0);
    CHECK(CloseHandle(file));
    CHECK(remove("msdn-write.bin") == 0);
    puts("{\"write_count\":true,\"readback\":true,\"eof\":true}");
    return 0;
}
