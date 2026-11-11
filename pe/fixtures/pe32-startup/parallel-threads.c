/* Original MIT witness for concurrent PE32 instructions and serialized host calls. */
typedef unsigned int DWORD;
typedef void *HANDLE;
typedef DWORD (__stdcall *ThreadProc)(void *);

__declspec(dllimport) HANDLE __stdcall CreateThread(void *, DWORD, ThreadProc, void *, DWORD, DWORD *);
__declspec(dllimport) DWORD __stdcall WaitForSingleObject(HANDLE, DWORD);
__declspec(dllimport) int __stdcall GetExitCodeThread(HANDLE, DWORD *);
__declspec(dllimport) int __stdcall CloseHandle(HANDLE);
__declspec(dllimport) DWORD __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall SetLastError(DWORD);
__declspec(dllimport) DWORD __stdcall InterlockedExchangeAdd(DWORD *, DWORD);
__declspec(dllimport) HANDLE __stdcall GetProcessHeap(void);
__declspec(dllimport) void *__stdcall HeapAlloc(HANDLE, DWORD, DWORD);
__declspec(dllimport) int __stdcall HeapFree(HANDLE, DWORD, void *);
__declspec(dllimport) HANDLE __stdcall GetStdHandle(DWORD);
__declspec(dllimport) int __stdcall WriteFile(HANDLE, const void *, DWORD, DWORD *, void *);

static DWORD arrived, host_adds;
static DWORD teb[3];

static DWORD current_teb(void) {
    DWORD address;
    __asm__ volatile("movl %%fs:0x18,%0" : "=r"(address));
    return address;
}

static DWORD __stdcall participant(void *parameter) {
    const DWORD index = (DWORD)parameter;
    const DWORD marker = 0xabcd3210 + index;
    teb[index] = current_teb();
    SetLastError(marker);
    __atomic_fetch_add(&arrived, 1, __ATOMIC_ACQ_REL);

    // Reaching three requires guest instruction execution without any host API
    // call or scheduling yield. The runner bounds the entire process externally.
    while (__atomic_load_n(&arrived, __ATOMIC_ACQUIRE) != 3)
        __asm__ volatile("pause");

    for (DWORD i = 0; i < 2048; i++) {
        InterlockedExchangeAdd(&host_adds, 1);
        if (GetLastError() != marker || current_teb() != teb[index])
            return 0;
    }
    HANDLE heap = GetProcessHeap();
    for (DWORD i = 0; i < 128; i++) {
        volatile DWORD *block = HeapAlloc(heap, 0, 16);
        if (!block)
            return 0;
        *block = marker ^ i;
        if (*block != (marker ^ i) || !HeapFree(heap, 0, (void *)block))
            return 0;
    }
    return marker;
}

DWORD entry(void) {
    HANDLE first = CreateThread(0, 0, participant, (void *)1, 0, 0);
    HANDLE second = CreateThread(0, 0, participant, (void *)2, 0, 0);
    if (!first || !second)
        return 2;
    DWORD main = participant(0);
    if (WaitForSingleObject(first, 10000) || WaitForSingleObject(second, 10000))
        return 3;
    DWORD a = 0, b = 0;
    if (!GetExitCodeThread(first, &a) || !GetExitCodeThread(second, &b)
        || !CloseHandle(first) || !CloseHandle(second))
        return 4;
    const DWORD out[] = {
        0x4c524150, arrived, host_adds, main, a, b,
        teb[0] && teb[1] && teb[2] && teb[0] != teb[1] && teb[0] != teb[2] && teb[1] != teb[2],
    };
    DWORD bytes = 0;
    return WriteFile(GetStdHandle((DWORD)-11), out, sizeof(out), &bytes, 0)
        && bytes == sizeof(out) ? 0 : 5;
}
