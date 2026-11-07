#ifndef ALKYHOL_H
#define ALKYHOL_H
/* Native host ABI only. All pointers must designate valid, aligned host memory.
 * UTF-16 strings are NUL terminated. Guest addresses need an adapter first. */
#include <stdint.h>
#include <stddef.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef uintptr_t HANDLE;
typedef int32_t BOOL;
typedef uint32_t DWORD;
typedef int32_t LONG;
typedef uint16_t WCHAR;
typedef union { int64_t QuadPart; } LARGE_INTEGER;
#define TRUE 1
#define FALSE 0
#define INVALID_HANDLE_VALUE ((HANDLE)~(uintptr_t)0)
#define GENERIC_READ UINT32_C(0x80000000)
#define GENERIC_WRITE UINT32_C(0x40000000)
#define FILE_SHARE_READ 1
#define FILE_SHARE_WRITE 2
#define FILE_SHARE_DELETE 4
#define FILE_ATTRIBUTE_NORMAL 0x80
#define CREATE_NEW 1
#define CREATE_ALWAYS 2
#define OPEN_EXISTING 3
#define OPEN_ALWAYS 4
#define TRUNCATE_EXISTING 5
#define FILE_BEGIN 0
#define FILE_CURRENT 1
#define FILE_END 2
#define WAIT_OBJECT_0 0
#define WAIT_TIMEOUT 258
#define WAIT_FAILED UINT32_MAX
#define INFINITE UINT32_MAX
#define ERROR_INVALID_HANDLE 6
#define ERROR_FILE_EXISTS 80
#define ERROR_ALREADY_EXISTS 183
#define ERROR_NEGATIVE_SEEK 131
#define ERROR_INVALID_PARAMETER 87
#define ERROR_TOO_MANY_POSTS 298
DWORD GetLastError(void);
void SetLastError(DWORD);
BOOL CloseHandle(HANDLE);
HANDLE CreateFileW(const WCHAR*, DWORD, DWORD, const void*, DWORD, DWORD, HANDLE);
BOOL ReadFile(HANDLE, void*, DWORD, DWORD*, void*);
BOOL WriteFile(HANDLE, const void*, DWORD, DWORD*, void*);
BOOL GetFileSizeEx(HANDLE, LARGE_INTEGER*);
BOOL SetFilePointerEx(HANDLE, LARGE_INTEGER, LARGE_INTEGER*, DWORD);
BOOL SetEndOfFile(HANDLE);
BOOL FlushFileBuffers(HANDLE);
HANDLE CreateEventW(const void*, BOOL, BOOL, const WCHAR*);
BOOL SetEvent(HANDLE);
BOOL ResetEvent(HANDLE);
HANDLE CreateSemaphoreW(const void*, LONG, LONG, const WCHAR*);
BOOL ReleaseSemaphore(HANDLE, LONG, LONG*);
DWORD WaitForSingleObject(HANDLE, DWORD);
BOOL QueryPerformanceCounter(LARGE_INTEGER*);
BOOL QueryPerformanceFrequency(LARGE_INTEGER*);
void Sleep(DWORD);
#ifdef __cplusplus
}
#endif
#endif
