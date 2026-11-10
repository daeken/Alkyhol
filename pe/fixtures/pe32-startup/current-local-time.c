/* Original MIT microtest for DAO's reached GetLocalTime call. This is a host
 * clock/ABI assertion, not a frozen cross-machine wall-clock observation. */
typedef unsigned int U;
typedef unsigned short W;
typedef void *H;
struct FT { U low, high; };
struct ST { W fields[8]; };
_Static_assert(sizeof(H) == 4 && sizeof(struct ST) == 16, "Win32 widths");
__declspec(dllimport) H __stdcall GetStdHandle(U);
__declspec(dllimport) int __stdcall WriteFile(H, const void *, U, U *, H);
__declspec(dllimport) void __stdcall SetLastError(U);
__declspec(dllimport) U __stdcall GetLastError(void);
__declspec(dllimport) void __stdcall GetSystemTimeAsFileTime(struct FT *);
__declspec(dllimport) void __stdcall GetLocalTime(struct ST *);
static void emit(const void *p, U n) {
    U done = 0;
    if (!WriteFile(GetStdHandle((U)-11), p, n, &done, 0) || done != n)
        __builtin_trap();
}
U entry(void) {
    U head[2] = {0x4c543332, MODE};
    emit(head, sizeof(head));
#if MODE == 0
    for (U i = 0; i < 24; ++i) {
        struct FT before, after;
        /* Deliberately word aligned, not DWORD aligned. */
        _Alignas(4) struct { W before; struct ST value; W after; } box;
        _Static_assert(sizeof(box) == 20, "SYSTEMTIME output guards");
        box.before = 0x1a2b; box.after = 0x3c4d;
        for (U j = 0; j < 8; ++j) box.value.fields[j] = 0xa5a5;
        GetSystemTimeAsFileTime(&before);
        SetLastError(0xaabbeeff);
        GetLocalTime(&box.value);
        U error = GetLastError();
        GetSystemTimeAsFileTime(&after);
        emit(&before, sizeof(before)); emit(&after, sizeof(after));
        emit(&box, sizeof(box)); emit(&error, sizeof(error));
    }
#elif MODE == 1
    GetLocalTime((struct ST *)0);
#elif MODE == 2
    /* A readable, non-writable image section. */
    static const W readonly[8] = {1,2,3,4,5,6,7,8};
    GetLocalTime((struct ST *)(void *)readonly);
#elif MODE == 3
    W storage[9];
    GetLocalTime((struct ST *)((char *)storage + 1));
#else
#error Unknown test mode
#endif
    return 0;
}
