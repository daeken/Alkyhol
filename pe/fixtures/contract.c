/* Alkyhol MIT fixture: an actual no-CRT Windows AMD64 executable, not a native
 * recompilation. The identical PE payload runs on Windows and under Rosetta. */
typedef unsigned int U32;
typedef unsigned long long U64;
typedef long long I64;
typedef void *HANDLE;
typedef unsigned short WCHAR;
#define DLL __declspec(dllimport)
DLL U32 GetLastError(void);
DLL void SetLastError(U32);
DLL int CloseHandle(HANDLE);
DLL HANDLE CreateFileW(const WCHAR *, U32, U32, void *, U32, U32, HANDLE);
DLL int ReadFile(HANDLE, void *, U32, U32 *, void *);
DLL int WriteFile(HANDLE, const void *, U32, U32 *, void *);
DLL int GetFileSizeEx(HANDLE, I64 *);
DLL int SetFilePointerEx(HANDLE, I64, I64 *, U32);
DLL int SetEndOfFile(HANDLE);
DLL int FlushFileBuffers(HANDLE);
DLL HANDLE CreateEventW(void *, int, int, const WCHAR *);
DLL int SetEvent(HANDLE);
DLL int ResetEvent(HANDLE);
DLL HANDLE CreateSemaphoreW(void *, int, int, const WCHAR *);
DLL int ReleaseSemaphore(HANDLE, int, int *);
DLL U32 WaitForSingleObject(HANDLE, U32);
DLL int QueryPerformanceCounter(I64 *);
DLL int QueryPerformanceFrequency(I64 *);
DLL void Sleep(U32);
DLL U32 TlsAlloc(void);
DLL int TlsFree(U32);
DLL void *TlsGetValue(U32);
DLL int TlsSetValue(U32, void *);
DLL __declspec(noreturn) void ExitProcess(U32);
extern U32 abi_canary(void);

static volatile U32 zero_fill[256];
/* Force an actual base relocation in writable data, then dereference it. */
static volatile U32 *volatile relocated_pointer = zero_fill;
static const WCHAR file_name[] = {'p','e','-',0x03bb,'-',0xd83c,0xdf38,'.','b','i','n',0};
static const WCHAR report_name[] = {'p','e','-','o','b','s','e','r','v','a','t','i','o','n','.','b','i','n',0};
static U32 records[64];
static U32 used;
#define CHECK(c, id) do { if (!(c)) ExitProcess(id); } while (0)
#define RECORD(v) do { records[used++] = (U32)(v); } while (0)

void entry(void) {
    U32 n = 0, read = 0, slot, i;
    U32 input[4] = {0x12345678, 0x90abcdef, 0x31415926, 0xdeadbeef};
    U32 output[5] = {0,0,0,0,0xfeedface};
    I64 size = -1, pos = -1, before, after, frequency;
    HANDLE file, event, automatic, semaphore;
    int previous = -1;
    CHECK(abi_canary() == 0, 10);
    RECORD(0x41424931); /* ABI1 */
    CHECK(relocated_pointer == zero_fill && relocated_pointer[17] == 0 && zero_fill[255] == 0, 11);
    relocated_pointer[17] = 0xcafebabe;
    RECORD(zero_fill[17]);
    for (i=0; i<10000; ++i) { SetLastError(0x98760000u + i); CHECK(GetLastError() == 0x98760000u + i, 12); }
    RECORD(GetLastError());
    slot = TlsAlloc();
    CHECK(slot != 0xffffffffu && TlsGetValue(slot) == (void *)0, 13);
    for (i=1; i<10000; ++i) { CHECK(TlsSetValue(slot, (void *)(U64)i), 14); CHECK(TlsGetValue(slot) == (void *)(U64)i, 15); }
    SetLastError(0x1234);
    CHECK(TlsGetValue(slot) == (void *)(U64)9999 && GetLastError() == 0, 16);
    CHECK(TlsFree(slot), 17);
    RECORD(9999);
    event = CreateEventW(0, 1, 0, 0);
    automatic = CreateEventW(0, 0, 1, 0);
    CHECK(event && automatic, 18);
    RECORD(WaitForSingleObject(event, 0));
    CHECK(records[used-1] == 258, 19);
    CHECK(SetEvent(event), 20);
    CHECK(WaitForSingleObject(event, 0) == 0 && WaitForSingleObject(event, 0) == 0, 21);
    CHECK(ResetEvent(event) && WaitForSingleObject(event, 0) == 258, 22);
    CHECK(WaitForSingleObject(automatic, 0) == 0 && WaitForSingleObject(automatic, 0) == 258, 23);
    CHECK(CloseHandle(event) && CloseHandle(automatic), 24);
    RECORD(0x45564e54);
    semaphore = CreateSemaphoreW(0, 1, 2, 0);
    CHECK(semaphore && WaitForSingleObject(semaphore, 0) == 0 && WaitForSingleObject(semaphore, 0) == 258, 25);
    CHECK(ReleaseSemaphore(semaphore, 2, &previous) && previous == 0, 26);
    RECORD(previous);
    CHECK(WaitForSingleObject(semaphore, 0) == 0 && WaitForSingleObject(semaphore, 0) == 0 && WaitForSingleObject(semaphore, 0) == 258, 27);
    CHECK(CloseHandle(semaphore), 28);
    CHECK(QueryPerformanceFrequency(&frequency) && frequency > 0 && QueryPerformanceCounter(&before), 29);
    Sleep(1);
    CHECK(QueryPerformanceCounter(&after) && after >= before, 30);
    RECORD(1); /* Predicate, never pretend raw timing/frequency agrees. */
    file = CreateFileW(file_name, 0xc0000000u, 7, 0, 2, 128, 0);
    CHECK(file != (HANDLE)(U64)-1, 31);
    CHECK(WriteFile(file, input, 16, &n, 0) && n == 16, 32);
    CHECK(GetFileSizeEx(file, &size) && size == 16, 33);
    RECORD(size);
    CHECK(SetFilePointerEx(file, -8, &pos, 2) && pos == 8, 34);
    CHECK(ReadFile(file, output, 16, &read, 0) && read == 8 && output[0] == input[2] && output[1] == input[3] && output[2] == 0 && output[4] == 0xfeedface, 35);
    RECORD(read); RECORD(output[0]); RECORD(output[1]); RECORD(output[4]);
    CHECK(SetFilePointerEx(file, 12, &pos, 0) && pos == 12 && SetEndOfFile(file), 36);
    CHECK(GetFileSizeEx(file, &size) && size == 12 && FlushFileBuffers(file) && CloseHandle(file), 37);
    RECORD(size);
    /* Explicitly invalid handle must fail, not become a successful stub. */
    SetLastError(0);
    CHECK(!CloseHandle((HANDLE)(U64)-1) && GetLastError() == 6, 38);
    RECORD(GetLastError());
    file = CreateFileW(report_name, 0x40000000u, 7, 0, 2, 128, 0);
    CHECK(file != (HANDLE)(U64)-1 && WriteFile(file, records, used * 4, &n, 0) && n == used * 4, 39);
    CHECK(CloseHandle(file), 40);
    ExitProcess(0);
}
