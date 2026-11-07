/* Original portable contract probe, inspired by Microsoft's file/event samples.
 * This identical source also builds against the real Windows SDK. */
#ifdef _WIN32
#include <windows.h>
#else
#include "alkyhol.h"
#endif
#include <stdio.h>
#include <string.h>
#define CHECK(x) do { if (!(x)) { fprintf(stderr, "line %d: %s (error %lu)\n", __LINE__, #x, (unsigned long)GetLastError()); return 1; } } while (0)
int main(void) {
    const WCHAR path[] = {'a','l','k','y','h','o','l','-','p','r','o','b','e','.','b','i','n',0};
    DWORD share = FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE;
    HANDLE file = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, share, NULL, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, 0);
    CHECK(file != INVALID_HANDLE_VALUE);
    HANDLE duplicate = CreateFileW(path, GENERIC_WRITE, share, NULL, CREATE_NEW, FILE_ATTRIBUTE_NORMAL, 0);
    CHECK(duplicate == INVALID_HANDLE_VALUE && GetLastError() == ERROR_FILE_EXISTS);
    DWORD written = 99, read = 99;
    CHECK(WriteFile(file, "hello", 5, &written, NULL) && written == 5);
    LARGE_INTEGER size, pos, distance;
    CHECK(GetFileSizeEx(file, &size) && size.QuadPart == 5);
    distance.QuadPart = 0;
    CHECK(SetFilePointerEx(file, distance, &pos, FILE_BEGIN) && pos.QuadPart == 0);
    char bytes[8] = {0};
    CHECK(ReadFile(file, bytes, 8, &read, NULL) && read == 5 && memcmp(bytes, "hello", 5) == 0);
    CHECK(ReadFile(file, bytes, 8, &read, NULL) && read == 0);
    distance.QuadPart = -9;
    CHECK(!SetFilePointerEx(file, distance, &pos, FILE_CURRENT) && GetLastError() == ERROR_NEGATIVE_SEEK);
    distance.QuadPart = 2;
    CHECK(SetFilePointerEx(file, distance, NULL, FILE_BEGIN) && SetEndOfFile(file));
    CHECK(GetFileSizeEx(file, &size) && size.QuadPart == 2 && FlushFileBuffers(file));
    CHECK(CloseHandle(file));
    CHECK(!CloseHandle(file) && GetLastError() == ERROR_INVALID_HANDLE);
    file = CreateFileW(path, GENERIC_READ | GENERIC_WRITE, share, NULL, OPEN_ALWAYS, FILE_ATTRIBUTE_NORMAL, 0);
    CHECK(file != INVALID_HANDLE_VALUE && GetLastError() == ERROR_ALREADY_EXISTS);
    CHECK(CloseHandle(file));
    CHECK(remove("alkyhol-probe.bin") == 0);
    HANDLE automatic = CreateEventW(NULL, FALSE, FALSE, NULL);
    CHECK(automatic != 0 && WaitForSingleObject(automatic, 0) == WAIT_TIMEOUT);
    CHECK(SetEvent(automatic) && SetEvent(automatic)); /* one bit, not a semaphore */
    CHECK(WaitForSingleObject(automatic, 0) == WAIT_OBJECT_0);
    CHECK(WaitForSingleObject(automatic, 0) == WAIT_TIMEOUT);
    HANDLE manual = CreateEventW(NULL, TRUE, TRUE, NULL);
    CHECK(manual != 0 && WaitForSingleObject(manual, 0) == WAIT_OBJECT_0);
    CHECK(WaitForSingleObject(manual, 0) == WAIT_OBJECT_0);
    CHECK(ResetEvent(manual) && WaitForSingleObject(manual, 1) == WAIT_TIMEOUT);
    CHECK(CloseHandle(automatic) && CloseHandle(manual));
    LARGE_INTEGER a, b, frequency;
    CHECK(QueryPerformanceFrequency(&frequency) && frequency.QuadPart > 0);
    CHECK(QueryPerformanceCounter(&a)); Sleep(1); CHECK(QueryPerformanceCounter(&b));
    CHECK(b.QuadPart >= a.QuadPart);
    puts("{\"file_roundtrip\":true,\"eof\":true,\"truncate\":true,\"errors\":true,\"auto_reset\":true,\"manual_reset\":true,\"clock_monotonic\":true}");
    return 0;
}
